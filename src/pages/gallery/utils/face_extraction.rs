use std::{
    collections::HashMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use iced::{
    futures::{SinkExt, Stream},
    stream::try_channel,
};
use rust_faces::{
    BlazeFaceParams, Face as DetectedFace, FaceDetection, FaceDetectorBuilder, InferParams, Nms,
    Provider, ToArray3,
};

use crate::{
    constants::APP_ID,
    pages::gallery::{
        page::{FACE_DATA_FILE_NAME, FACE_DATA_FOLDER_NAME, PATH_TO_FACE_EXTRACTION_MODEL},
        utils::common::Rect,
    },
};

use super::common::{FaceData, PhotoProcessingProgress};

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
        let idirfein_data_dir = dirs::data_dir()
            .expect("Can't find data dir")
            .as_path()
            .join(APP_ID);

        let big_face_extractor =
            FaceDetectorBuilder::new(FaceDetection::BlazeFace640(big_face_params))
                .from_file(
                    idirfein_data_dir
                        .join(PATH_TO_FACE_EXTRACTION_MODEL)
                        .to_string_lossy()
                        .into_owned(),
                )
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
                .from_file(
                    idirfein_data_dir
                        .join(PATH_TO_FACE_EXTRACTION_MODEL)
                        .to_string_lossy()
                        .into_owned(),
                )
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
                .from_file(
                    idirfein_data_dir
                        .join(PATH_TO_FACE_EXTRACTION_MODEL)
                        .to_string_lossy()
                        .into_owned(),
                )
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
                let longest: f32 = if detected_face.rect.width < detected_face.rect.height {
                    detected_face.rect.width
                } else {
                    detected_face.rect.height
                };

                let mut longest = longest * 1.6;
                let mut half_longest = longest / 2.0;

                let (centre_x, centre_y) = Self::centre(&detected_face);

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
                            checked_names: vec![],
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
