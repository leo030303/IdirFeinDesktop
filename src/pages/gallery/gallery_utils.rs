use iced::{
    advanced::graphics::image::image_rs::{self},
    futures::{SinkExt, Stream},
    stream::try_channel,
};
use image::DynamicImage;
use opencv::{
    core::{Mat, MatTraitConstManual},
    imgcodecs,
    objdetect::{
        FaceRecognizerSF, FaceRecognizerSFTrait, FaceRecognizerSFTraitConst,
        FaceRecognizerSF_DisType,
    },
};
use rust_faces::Nms;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

use rust_faces::{
    BlazeFaceParams, Face as DetectedFace, FaceDetection, FaceDetectorBuilder, InferParams,
    Provider, ToArray3,
};

use crate::pages::gallery::page::FACE_DATA_FOLDER_NAME;

use super::page::{
    FACE_DATA_FILE_NAME, PATH_TO_FACE_RECOGNITION_MODEL, THUMBNAIL_FOLDER_NAME, THUMBNAIL_SIZE,
    UNNAMED_STRING,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceData {
    /// The name of the file this was made from
    pub original_filename: OsString,

    /// The name of the detected person
    pub name_of_person: Option<String>,

    /// Whether the detected face should be ignored
    pub is_ignored: bool,

    /// Path to thumbnail generated from face bounds.
    /// Normalized to be square and expanded to capture the whole head.
    pub thumbnail_filename: OsString,

    /// The data required to match this face to other faces
    face_matrix_bytes: Vec<u8>,
}

impl FaceData {
    pub fn matrix(&self) -> Mat {
        Mat::from_bytes::<f32>(&self.face_matrix_bytes)
            .expect("Face data got corrupted")
            .clone_pointee()
    }

    pub fn get_matrix_bytes_from_features(
        face_features: Vec<(f32, f32)>,
        bounds: Rect,
        confidence: f32,
        original_image: &DynamicImage,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let initial_face_matrix = Mat::from_exact_iter(
            vec![
                0.0,
                0.0,
                bounds.width,
                bounds.height,
                face_features[0].0, // Right Eye
                face_features[0].1, // Right Eye
                face_features[1].0, // Left Eye
                face_features[1].1, // Left Eye
                face_features[2].0, // Nose
                face_features[2].1, // Nose
                face_features[3].0, // Right Mouth Corner
                face_features[3].1, // Right Mouth Corner
                face_features[4].0, // Left Mouth Corner
                face_features[4].1, // Left Mouth Corner
                confidence,
            ]
            .into_iter(),
        )?;
        let mut opencv_face_recognizer =
            FaceRecognizerSF::create_def(PATH_TO_FACE_RECOGNITION_MODEL, "")?;
        let bounds_img = original_image.crop_imm(
            bounds.x as u32,
            bounds.y as u32,
            bounds.width as u32,
            bounds.height as u32,
        );

        let bounds_tempfile_path = TempDir::new()?.into_path().join("tempfile.png");
        let _ = bounds_img.save(&bounds_tempfile_path);

        let face_img = imgcodecs::imread_def(&bounds_tempfile_path.to_string_lossy())?;

        let mut aligned_face = Mat::default();
        opencv_face_recognizer.align_crop(&face_img, &initial_face_matrix, &mut aligned_face)?;

        let mut face_features = Mat::default();
        opencv_face_recognizer.feature(&aligned_face, &mut face_features)?;

        Ok(face_features.data_bytes()?.to_owned())
    }
}

pub struct FaceExtractor {
    big_face_extractor: Box<dyn rust_faces::FaceDetector>,

    medium_face_extractor: Box<dyn rust_faces::FaceDetector>,

    small_face_extractor: Box<dyn rust_faces::FaceDetector>,
}

impl FaceExtractor {
    pub fn build() -> Option<FaceExtractor> {
        let big_face_params = BlazeFaceParams {
            score_threshold: 0.95,
            target_size: 160,
            ..BlazeFaceParams::default()
        };

        let big_face_extractor =
            FaceDetectorBuilder::new(FaceDetection::BlazeFace640(big_face_params))
                .download()
                .infer_params(InferParams {
                    provider: Provider::OrtCpu,
                    intra_threads: Some(5),
                    ..Default::default()
                })
                .build();

        let medium_face_params = BlazeFaceParams {
            score_threshold: 0.95,
            target_size: 640,
            ..BlazeFaceParams::default()
        };

        let medium_face_extractor =
            FaceDetectorBuilder::new(FaceDetection::BlazeFace640(medium_face_params))
                .download()
                .infer_params(InferParams {
                    provider: Provider::OrtCpu,
                    intra_threads: Some(5),
                    ..Default::default()
                })
                .build();

        let small_face_params = BlazeFaceParams {
            score_threshold: 0.95,
            target_size: 1280,
            ..BlazeFaceParams::default()
        };

        let small_face_extractor =
            FaceDetectorBuilder::new(FaceDetection::BlazeFace640(small_face_params))
                .download()
                .infer_params(InferParams {
                    provider: Provider::OrtCpu,
                    ..Default::default()
                })
                .build();

        if big_face_extractor.is_err() {
            println!("Big face extractor error");
            return None;
        } else if medium_face_extractor.is_err() {
            println!("Medium face extractor error");
            return None;
        } else if small_face_extractor.is_err() {
            println!("Small face extractor error");
            return None;
        }
        Some(FaceExtractor {
            big_face_extractor: big_face_extractor.expect("Prechecked this, can't fail"),
            medium_face_extractor: medium_face_extractor.expect("Prechecked this, can't fail"),
            small_face_extractor: small_face_extractor.expect("Prechecked this, can't fail"),
        })
    }
    /// Identify faces in a photo and return a vector of paths of extracted face images.
    pub fn extract_faces(&self, picture_path: &PathBuf) -> Vec<FaceData> {
        let original_image_option = image::open(picture_path);
        if original_image_option.is_err() {
            return vec![];
        }
        let original_image = original_image_option.expect("Can't fail");

        let image = original_image.clone().into_rgb8().into_array3();

        let mut faces: Vec<DetectedFace> = vec![];

        let result = self.medium_face_extractor.detect(image.view().into_dyn());
        if let Ok(detected_faces) = result {
            for f in detected_faces {
                faces.push(f);
            }
        } else {
            println!("Failed extracting faces with blaze_face_big: {:?}", result);
        }

        let result = self.small_face_extractor.detect(image.view().into_dyn());
        if let Ok(detected_faces) = result {
            for f in detected_faces {
                faces.push(f);
            }
        } else {
            println!(
                "Failed extracting faces with blaze_face_small: {:?}",
                result
            );
        }

        let result = self.big_face_extractor.detect(image.view().into_dyn());
        if let Ok(detected_faces) = result {
            for f in detected_faces {
                faces.push(f);
            }
        } else {
            println!("Failed extracting faces with blaze_face_huge: {:?}", result);
        }

        // Use "non-maxima suppression" to remove duplicate matches.
        let nms = Nms::default();
        let mut faces = nms.suppress_non_maxima(faces);

        println!("Picture {:?} has {} faces.", picture_path, faces.len());

        let mut face_data_folder = picture_path.clone();
        face_data_folder.pop();
        face_data_folder.push(FACE_DATA_FOLDER_NAME);
        if !face_data_folder.exists() {
            let _ = std::fs::create_dir_all(&face_data_folder);
        }

        faces.sort_unstable_by(|face1, face2| {
            if face1.rect.x == face2.rect.x {
                face1
                    .rect
                    .y
                    .partial_cmp(&face2.rect.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                face1
                    .rect
                    .x
                    .partial_cmp(&face2.rect.x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        faces
            .into_iter()
            .enumerate()
            .filter(|(_image_index, detected_face)| detected_face.landmarks.is_some())
            .filter_map(|(image_index, detected_face)| {
                // Extract face and save to thumbnail.
                // The bounding box is pretty tight, so make it a bit bigger.
                // Also, make the box a square.

                let longest: f32 = if detected_face.rect.width < detected_face.rect.height {
                    detected_face.rect.width
                } else {
                    detected_face.rect.height
                };

                let mut longest = longest * 1.6;
                let mut half_longest = longest / 2.0;

                let (centre_x, centre_y) = Self::centre(&detected_face);

                // Normalize thumbnail to be a square.
                if (original_image.width() as f32) < centre_x + half_longest {
                    half_longest = original_image.width() as f32 - centre_x;
                    longest = half_longest * 2.0;
                }
                if (original_image.height() as f32) < centre_y + half_longest {
                    half_longest = original_image.height() as f32 - centre_y;
                    longest = half_longest * 2.0;
                }

                if centre_x < half_longest {
                    half_longest = centre_x;
                    longest = half_longest * 2.0;
                }

                if centre_y < half_longest {
                    half_longest = centre_y;
                    longest = half_longest * 2.0;
                }

                // Don't panic when x or y would be < zero
                let mut x = centre_x - half_longest;
                if x < 0.0 {
                    x = 0.0;
                }
                let mut y = centre_y - half_longest;
                if y < 0.0 {
                    y = 0.0;
                }

                let thumbnail =
                    original_image.crop_imm(x as u32, y as u32, longest as u32, longest as u32);
                let thumbnail = thumbnail.thumbnail(200, 200);
                let thumbnail_path = face_data_folder.join(format!(
                    "{}_thumbnail_{}.png",
                    picture_path
                        .file_stem()
                        .and_then(|item| item.to_str())
                        .unwrap_or("error_reading_filename"),
                    image_index
                ));
                let _ = thumbnail.save(&thumbnail_path);

                let bounds = Rect {
                    x: detected_face.rect.x,
                    y: detected_face.rect.y,
                    width: detected_face.rect.width,
                    height: detected_face.rect.height,
                };

                // Make face landmarks relative to bounded image, not source image
                let face_features = detected_face
                    .landmarks
                    .expect("Can't fail")
                    .into_iter()
                    .map(|landmark_item| (landmark_item.0 - bounds.x, landmark_item.1 - bounds.y))
                    .collect();

                let face_matrix_bytes_option = FaceData::get_matrix_bytes_from_features(
                    face_features,
                    bounds,
                    detected_face.confidence,
                    &original_image,
                );
                if let Some(original_filename) = picture_path.file_name() {
                    if let Ok(face_matrix_bytes) = face_matrix_bytes_option {
                        Some(FaceData {
                            thumbnail_filename: format!(
                                "{}_thumbnail_{}.png",
                                picture_path
                                    .file_stem()
                                    .and_then(|item| item.to_str())
                                    .unwrap_or("error_reading_filename"),
                                image_index
                            )
                            .into(),
                            face_matrix_bytes,
                            name_of_person: None,
                            is_ignored: false,
                            original_filename: original_filename.to_owned(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    fn centre(f: &DetectedFace) -> (f32, f32) {
        if let Some(ref landmarks) = f.landmarks {
            // If we have landmarks, then the first two are the right and left eyes.
            // Use the midpoint between the eyes as the centre of the thumbnail.
            let x = (landmarks[0].0 + landmarks[1].0) / 2.0;
            let y = (landmarks[0].1 + landmarks[1].1) / 2.0;
            (x, y)
        } else {
            let x = f.rect.x + (f.rect.width / 2.0);
            let y = f.rect.y + (f.rect.height / 2.0);
            (x, y)
        }
    }
}

pub fn extract_all_faces(
    image_paths_to_extract: Vec<PathBuf>,
) -> impl Stream<Item = Result<PhotoProcessingProgress, iced::Error>> {
    try_channel(1, move |mut progress_percentage_output| async move {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        tokio::spawn(async move {
            if tx.send(0.0).await.is_err() {
                return;
            }
            let face_extractor = FaceExtractor::build().unwrap();
            let mut face_data_vecs_map: HashMap<PathBuf, Vec<(OsString, Option<FaceData>)>> =
                HashMap::new();

            let total_number_of_images = image_paths_to_extract.len();
            for (image_index, image_path) in image_paths_to_extract.into_iter().enumerate() {
                let parent_path = image_path.parent().unwrap_or(Path::new("/"));
                let face_data_vec_option = face_data_vecs_map.get_mut(parent_path);
                let face_data_file = image_path
                    .parent()
                    .unwrap_or(Path::new("/"))
                    .join(FACE_DATA_FOLDER_NAME)
                    .join(FACE_DATA_FILE_NAME);
                match face_data_vec_option {
                    Some(face_data_vec) => {
                        if !face_data_vec.iter().any(|face_data| {
                            image_path
                                .file_name()
                                .is_some_and(|file_name| face_data.0 == file_name)
                        }) {
                            let new_face_data_vec = face_extractor.extract_faces(&image_path);
                            if new_face_data_vec.is_empty() {
                                face_data_vec
                                    .push((image_path.file_name().unwrap().to_owned(), None));
                            } else {
                                new_face_data_vec.into_iter().for_each(|face_data| {
                                    face_data_vec.push((
                                        image_path.file_name().unwrap().to_owned(),
                                        Some(face_data),
                                    ))
                                });
                            }
                            let serialised = serde_json::to_string(&face_data_vec).unwrap();
                            let _ = fs::write(face_data_file, serialised);
                        }
                    }
                    None => {
                        let mut face_data_vec: Vec<(OsString, Option<FaceData>)> =
                            if face_data_file.exists() {
                                if let Ok(face_data_json) = fs::read_to_string(&face_data_file) {
                                    serde_json::from_str(&face_data_json).unwrap()
                                } else {
                                    vec![]
                                }
                            } else {
                                vec![]
                            };
                        if !face_data_vec.iter().any(|face_data| {
                            image_path
                                .file_name()
                                .is_some_and(|file_name| face_data.0 == file_name)
                        }) {
                            let new_face_data_vec = face_extractor.extract_faces(&image_path);
                            if new_face_data_vec.is_empty() {
                                face_data_vec
                                    .push((image_path.file_name().unwrap().to_owned(), None));
                            } else {
                                new_face_data_vec.into_iter().for_each(|face_data| {
                                    face_data_vec.push((
                                        image_path.file_name().unwrap().to_owned(),
                                        Some(face_data),
                                    ))
                                });
                            }
                            let serialised = serde_json::to_string(&face_data_vec).unwrap();
                            if !face_data_file.exists() {
                                let _ = fs::create_dir_all(
                                    face_data_file.parent().unwrap_or(Path::new("/")),
                                );
                            }
                            let _ = fs::write(face_data_file, serialised);
                        }
                        let parent_pathbuf = parent_path.to_path_buf();
                        face_data_vecs_map.insert(parent_pathbuf, face_data_vec);
                    }
                };
                if image_index % 5 == 0
                    && tx
                        .send(image_index as f32 / total_number_of_images as f32)
                        .await
                        .is_err()
                {
                    return;
                }
            }
            let _ = tx.send(1.0).await;
        });

        while let Some(received) = rx.recv().await {
            let _ = progress_percentage_output
                .send(PhotoProcessingProgress::FaceExtraction(received * 100.0))
                .await;
            if received >= 1.0 {
                let _ = progress_percentage_output
                    .send(PhotoProcessingProgress::None)
                    .await;
                break;
            }
        }
        let _ = progress_percentage_output
            .send(PhotoProcessingProgress::None)
            .await;

        Ok(())
    })
}

pub fn get_detected_faces_for_image(image_path: &Path) -> Vec<FaceData> {
    let face_data_file = image_path
        .parent()
        .unwrap_or(Path::new("/"))
        .join(FACE_DATA_FOLDER_NAME)
        .join(FACE_DATA_FILE_NAME);
    if face_data_file.exists() {
        if let Ok(face_data_json) = fs::read_to_string(&face_data_file) {
            let face_data_vec: Vec<(OsString, Option<FaceData>)> =
                serde_json::from_str(&face_data_json).unwrap();
            face_data_vec
                .into_iter()
                .filter(|face_data| {
                    image_path
                        .file_name()
                        .is_some_and(|file_name| face_data.0 == file_name)
                })
                .filter_map(|(_file_name, face_data_option)| face_data_option)
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

pub fn generate_thumbnails(
    image_paths_to_process: Vec<PathBuf>,
) -> impl Stream<Item = Result<PhotoProcessingProgress, iced::Error>> {
    try_channel(1, move |mut progress_percentage_output| async move {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        tokio::spawn(async move {
            if tx.send(0.0).await.is_err() {
                return;
            }
            let total_number_of_images = image_paths_to_process.len();
            for (image_index, image_path) in image_paths_to_process.into_iter().enumerate() {
                let file_name = image_path.file_name().unwrap();
                let mut thumbnail_path =
                    image_path.parent().unwrap_or(Path::new("/")).to_path_buf();
                thumbnail_path.push(THUMBNAIL_FOLDER_NAME);
                if !thumbnail_path.exists() {
                    let _ = fs::create_dir_all(&thumbnail_path);
                }
                thumbnail_path.push(file_name);
                if !thumbnail_path.exists() {
                    if let Ok(img) = image_rs::open(&image_path) {
                        let original_height = img.height();
                        let original_width = img.width();

                        let new_width;
                        let new_height;
                        let x_val;
                        let y_val;
                        if original_height > original_width {
                            new_width = original_width;
                            new_height = original_width;
                            x_val = 0;
                            y_val = (original_height / 2) - (original_width / 2);
                        } else {
                            new_width = original_height;
                            new_height = original_height;
                            x_val = (original_width / 2) - (original_height / 2);
                            y_val = 0;
                        }
                        let cropped = img.crop_imm(x_val, y_val, new_width, new_height);
                        let resized = cropped.resize(
                            THUMBNAIL_SIZE,
                            THUMBNAIL_SIZE,
                            image_rs::imageops::FilterType::Nearest,
                        );
                        let _ = resized.save(&thumbnail_path);
                    };
                }
                if image_index % 10 == 0
                    && tx
                        .send(image_index as f32 / total_number_of_images as f32)
                        .await
                        .is_err()
                {
                    return;
                }
            }
            let _ = tx.send(1.0).await;
        });

        while let Some(received) = rx.recv().await {
            let _ = progress_percentage_output
                .send(PhotoProcessingProgress::ThumbnailGeneration(
                    received * 100.0,
                ))
                .await;
            if received >= 1.0 {
                let _ = progress_percentage_output
                    .send(PhotoProcessingProgress::None)
                    .await;
                break;
            }
        }
        let _ = progress_percentage_output
            .send(PhotoProcessingProgress::None)
            .await;

        Ok(())
    })
}

#[derive(Debug, Clone)]
pub enum PhotoProcessingProgress {
    ThumbnailGeneration(f32),
    FaceExtraction(f32),
    FaceRecognition(f32),
    None,
}

impl Default for PhotoProcessingProgress {
    fn default() -> Self {
        Self::None
    }
}

pub fn match_face_to_person(
    unknown_face: &FaceData,
    named_people_list: Vec<(String, Mat)>,
) -> Option<String> {
    const L2NORM_SIMILAR_THRESH: f64 = 1.128;

    let best_person_and_score = named_people_list
        .iter()
        .map(|(person_name, named_person_face_features)| {
            let unknown_face_features = unknown_face.matrix();
            let opencv_face_recognizer =
                FaceRecognizerSF::create_def(PATH_TO_FACE_RECOGNITION_MODEL, "").unwrap();
            let l2_score = opencv_face_recognizer.match_(
                &named_person_face_features,
                &unknown_face_features,
                FaceRecognizerSF_DisType::FR_NORM_L2.into(),
            );
            println!("{person_name} score is {l2_score:?}");
            (
                person_name,
                l2_score.unwrap_or(L2NORM_SIMILAR_THRESH + 100.0),
            )
        })
        .min_by_key(|x| (x.1 * 10000.0) as i32);

    if let Some((person_name, l2_score)) = best_person_and_score {
        if l2_score <= L2NORM_SIMILAR_THRESH {
            return Some(person_name.clone());
        }
    }

    None
}

/// Can update any field other than thumbnail
pub fn update_face_data(image_path: PathBuf, new_face_data: FaceData) {
    let face_data_file = image_path
        .parent()
        .unwrap_or(Path::new("/"))
        .join(FACE_DATA_FOLDER_NAME)
        .join(FACE_DATA_FILE_NAME);
    if face_data_file.exists() {
        if let Ok(face_data_json) = fs::read_to_string(&face_data_file) {
            let mut face_data_vec: Vec<(OsString, Option<FaceData>)> =
                serde_json::from_str(&face_data_json).unwrap();
            let target_filename = image_path.file_name().unwrap_or_default().to_os_string();
            let target_index_option =
                face_data_vec
                    .iter()
                    .position(|(source_image_filename, face_data_option)| {
                        *source_image_filename == target_filename
                            && face_data_option.as_ref().is_some_and(|face_data| {
                                face_data.thumbnail_filename == new_face_data.thumbnail_filename
                            })
                    });
            if let Some(target_index) = target_index_option {
                face_data_vec[target_index] =
                    (face_data_vec[target_index].0.clone(), Some(new_face_data));
                let serialised = serde_json::to_string(&face_data_vec).unwrap();
                let _ = fs::write(face_data_file, serialised);
            }
        }
    }
}

pub fn get_parent_folders(image_paths_to_process: &[PathBuf]) -> Vec<PathBuf> {
    let mut parent_paths: Vec<PathBuf> = image_paths_to_process
        .iter()
        .filter_map(|image_path| {
            image_path
                .parent()
                .map(|path_reference| path_reference.to_path_buf())
        })
        .collect();
    parent_paths.sort_unstable();
    parent_paths.dedup();
    parent_paths
}

pub fn get_all_named_people(parent_folders: &[PathBuf]) -> Vec<(String, Mat)> {
    let mut all_named_people: Vec<(String, Mat)> = vec![];
    parent_folders.iter().for_each(|parent_path| {
        let face_data_file = parent_path
            .join(FACE_DATA_FOLDER_NAME)
            .join(FACE_DATA_FILE_NAME);
        if face_data_file.exists() {
            if let Ok(face_data_json) = fs::read_to_string(&face_data_file) {
                let face_data_vec: Vec<(OsString, Option<FaceData>)> =
                    serde_json::from_str(&face_data_json).unwrap();
                face_data_vec
                    .into_iter()
                    .filter_map(|(_original_image_name, face_data_option)| face_data_option)
                    .filter(|face_data| !face_data.is_ignored && face_data.name_of_person.is_some())
                    .for_each(|face_data| {
                        all_named_people.push((
                            face_data.name_of_person.clone().expect("Can't fail"),
                            face_data.matrix(),
                        ))
                    });
            }
        }
    });
    all_named_people.sort_unstable_by(|(name_1, _), (name_2, _)| name_1.cmp(name_2));
    all_named_people.dedup_by(|(name_1, _), (name_2, _)| name_1 == name_2);
    all_named_people
}

// TODO this is breaking and deleting faces somehow, make a backup of all faces and then triage this
pub fn group_all_faces(
    parent_folders: Vec<PathBuf>,
) -> impl Stream<Item = Result<PhotoProcessingProgress, iced::Error>> {
    try_channel(1, move |mut progress_percentage_output| async move {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        tokio::spawn(async move {
            if tx.send(0.0).await.is_err() {
                return;
            }
            let named_people = get_all_named_people(&parent_folders);

            let mut number_of_faces_processed_so_far = 0;
            let mut total_number_of_faces = 0;

            for parent_path in parent_folders.iter() {
                let face_data_file = parent_path
                    .join(FACE_DATA_FOLDER_NAME)
                    .join(FACE_DATA_FILE_NAME);
                if face_data_file.exists() {
                    if let Ok(face_data_json) = fs::read_to_string(&face_data_file) {
                        let face_data_vec: Vec<(OsString, Option<FaceData>)> =
                            serde_json::from_str(&face_data_json).unwrap();
                        total_number_of_faces += face_data_vec.len();
                    }
                }
            }

            for parent_path in parent_folders.iter() {
                let face_data_file = parent_path
                    .join(FACE_DATA_FOLDER_NAME)
                    .join(FACE_DATA_FILE_NAME);
                if face_data_file.exists() {
                    if let Ok(face_data_json) = fs::read_to_string(&face_data_file) {
                        let mut face_data_vec: Vec<(OsString, Option<FaceData>)> =
                            serde_json::from_str(&face_data_json).unwrap();
                        for (_original_image_name, face_data_option) in face_data_vec.iter_mut() {
                            if let Some(face_data) = face_data_option.as_mut() {
                                if !face_data.is_ignored && face_data.name_of_person.is_none() {
                                    let matched_name_option =
                                        match_face_to_person(face_data, named_people.clone());
                                    if let Some(matched_name) = matched_name_option {
                                        println!(
                                            "Recognised {:?} as {matched_name}",
                                            face_data.thumbnail_filename
                                        );
                                        face_data.name_of_person = Some(matched_name);
                                    }
                                }
                            }
                            number_of_faces_processed_so_far += 1;
                            if number_of_faces_processed_so_far % 5 == 0
                                && tx
                                    .send(
                                        number_of_faces_processed_so_far as f32
                                            / total_number_of_faces as f32,
                                    )
                                    .await
                                    .is_err()
                            {
                                return;
                            };
                        }
                        let serialised = serde_json::to_string(&face_data_vec).unwrap();
                        let _ = fs::write(face_data_file, serialised);
                    }
                }
            }
            let _ = tx.send(1.0).await;
        });

        while let Some(received) = rx.recv().await {
            let _ = progress_percentage_output
                .send(PhotoProcessingProgress::FaceRecognition(received * 100.0))
                .await;
            if received >= 1.0 {
                let _ = progress_percentage_output
                    .send(PhotoProcessingProgress::None)
                    .await;
                break;
            }
        }
        let _ = progress_percentage_output
            .send(PhotoProcessingProgress::None)
            .await;

        Ok(())
    })
}

pub fn get_named_people_for_display(parent_folders: &[PathBuf]) -> Vec<(String, PathBuf)> {
    let mut all_named_people: Vec<(String, PathBuf)> = vec![];
    parent_folders.iter().for_each(|parent_path| {
        let face_data_file = parent_path
            .join(FACE_DATA_FOLDER_NAME)
            .join(FACE_DATA_FILE_NAME);
        if face_data_file.exists() {
            if let Ok(face_data_json) = fs::read_to_string(&face_data_file) {
                let face_data_vec: Vec<(OsString, Option<FaceData>)> =
                    serde_json::from_str(&face_data_json).unwrap();
                face_data_vec
                    .into_iter()
                    .filter_map(|(_original_image_name, face_data_option)| face_data_option)
                    .filter(|face_data| !face_data.is_ignored)
                    .for_each(|face_data| {
                        all_named_people.push((
                            face_data
                                .name_of_person
                                .clone()
                                .unwrap_or(String::from(UNNAMED_STRING)),
                            parent_path
                                .join(FACE_DATA_FOLDER_NAME)
                                .join(face_data.thumbnail_filename)
                                .to_path_buf(),
                        ))
                    });
            }
        }
    });
    all_named_people.sort_unstable_by(|(name_1, _), (name_2, _)| name_1.cmp(name_2));
    all_named_people.dedup_by(|(name_1, _), (name_2, _)| name_1 == name_2);
    all_named_people
}

pub fn get_all_photos_by_name(target_name: String, parent_folders: &[PathBuf]) -> Vec<PathBuf> {
    let mut all_pictures_of_target_person: Vec<PathBuf> = vec![];
    parent_folders.iter().for_each(|parent_path| {
        let face_data_file = parent_path
            .join(FACE_DATA_FOLDER_NAME)
            .join(FACE_DATA_FILE_NAME);
        if face_data_file.exists() {
            if let Ok(face_data_json) = fs::read_to_string(&face_data_file) {
                let face_data_vec: Vec<(OsString, Option<FaceData>)> =
                    serde_json::from_str(&face_data_json).unwrap();
                face_data_vec
                    .into_iter()
                    .filter_map(|(_original_image_name, face_data_option)| face_data_option)
                    .filter(|face_data| {
                        !face_data.is_ignored
                            && (face_data
                                .name_of_person
                                .as_ref()
                                .is_some_and(|current_name| *current_name == target_name)
                                || (face_data.name_of_person.is_none()
                                    && target_name == UNNAMED_STRING))
                    })
                    .for_each(|face_data| {
                        all_pictures_of_target_person
                            .push(parent_path.join(face_data.original_filename).to_path_buf())
                    });
            }
        }
    });
    all_pictures_of_target_person.sort_unstable();
    all_pictures_of_target_person.dedup();
    all_pictures_of_target_person
}
