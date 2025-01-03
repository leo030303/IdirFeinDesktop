use iced::{
    futures::{SinkExt, Stream},
    stream::try_channel,
    Error,
};
use rust_faces::Nms;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use rust_faces::{
    BlazeFaceParams, Face as DetectedFace, FaceDetection, FaceDetectorBuilder, InferParams,
    Provider, ToArray3,
};

use crate::pages::gallery::page::FACE_DATA_FOLDER_NAME;

use super::page::FACE_DATA_FILE_NAME;

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
    pub thumbnail_path: PathBuf,

    /// Image cropped from bounds returned by face detection algorithm
    pub bounds_path: PathBuf,

    /// Bounds of detected face.
    pub bounds: Rect,

    /// Confidence (0.0 to 1.0) that the detected face is actually a face.
    pub confidence: f32,

    face_features: Option<Vec<(f32, f32)>>,
}

impl FaceData {
    fn get_face_feature(&self, index: usize) -> Option<(f32, f32)> {
        self.face_features
            .as_ref()
            .filter(|x| x.len() == 5)
            .map(|x| (x[index].0, x[index].1))
    }

    pub fn right_eye(&self) -> Option<(f32, f32)> {
        self.get_face_feature(0)
    }

    pub fn left_eye(&self) -> Option<(f32, f32)> {
        self.get_face_feature(1)
    }

    pub fn nose(&self) -> Option<(f32, f32)> {
        self.get_face_feature(2)
    }

    pub fn right_mouth_corner(&self) -> Option<(f32, f32)> {
        self.get_face_feature(3)
    }

    pub fn left_mouth_corner(&self) -> Option<(f32, f32)> {
        self.get_face_feature(4)
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
            big_face_extractor: big_face_extractor.unwrap(),
            medium_face_extractor: medium_face_extractor.unwrap(),
            small_face_extractor: small_face_extractor.unwrap(),
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
                face1.rect.y.partial_cmp(&face2.rect.y).unwrap()
            } else {
                face1.rect.x.partial_cmp(&face2.rect.x).unwrap()
            }
        });

        faces
            .into_iter()
            .enumerate()
            .map(|(image_index, detected_face)| {
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
                    picture_path.file_stem().unwrap().to_str().unwrap(),
                    image_index
                ));
                let _ = thumbnail.save(&thumbnail_path);

                let bounds = Rect {
                    x: detected_face.rect.x,
                    y: detected_face.rect.y,
                    width: detected_face.rect.width,
                    height: detected_face.rect.height,
                };

                let bounds_img = original_image.crop_imm(
                    bounds.x as u32,
                    bounds.y as u32,
                    bounds.width as u32,
                    bounds.height as u32,
                );

                let bounds_path = face_data_folder.join(format!(
                    "{}_original_{}.png",
                    picture_path.file_stem().unwrap().to_str().unwrap(),
                    image_index
                ));
                let _ = bounds_img.save(&bounds_path);

                FaceData {
                    thumbnail_path,
                    bounds_path,
                    bounds,
                    confidence: detected_face.confidence,
                    face_features: detected_face.landmarks,
                    name_of_person: None,
                    is_ignored: false,
                    original_filename: picture_path.file_name().unwrap().to_owned(),
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
) -> impl Stream<Item = Result<f32, Error>> {
    try_channel(1, move |mut progress_percentage_output| async move {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        tokio::spawn(async move {
            tx.send(0.0).await.unwrap();
            let face_extractor = FaceExtractor::build().unwrap();
            let mut face_data_vecs_map: HashMap<PathBuf, Vec<(OsString, Option<FaceData>)>> =
                HashMap::new();

            let total_number_of_images = image_paths_to_extract.len();
            for (image_index, image_path) in image_paths_to_extract.into_iter().enumerate() {
                let parent_path = image_path.parent().unwrap();
                let face_data_vec_option = face_data_vecs_map.get_mut(parent_path);
                let face_data_file = image_path
                    .parent()
                    .unwrap()
                    .join(FACE_DATA_FOLDER_NAME)
                    .join(FACE_DATA_FILE_NAME);
                match face_data_vec_option {
                    Some(face_data_vec) => {
                        if !face_data_vec
                            .iter()
                            .any(|face_data| face_data.0 == image_path.file_name().unwrap())
                        {
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
                            fs::write(face_data_file, serialised).unwrap();
                        }
                    }
                    None => {
                        let mut face_data_vec: Vec<(OsString, Option<FaceData>)> =
                            if face_data_file.exists() {
                                if let Ok(face_data_json) = fs::read_to_string(&face_data_file) {
                                    serde_json::from_str(&face_data_json).unwrap_or_default()
                                } else {
                                    vec![]
                                }
                            } else {
                                vec![]
                            };
                        let parent_pathbuf = parent_path.to_path_buf();
                        let new_face_data_vec = face_extractor.extract_faces(&image_path);
                        if new_face_data_vec.is_empty() {
                            face_data_vec.push((image_path.file_name().unwrap().to_owned(), None));
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
                            let _ = fs::create_dir_all(face_data_file.parent().unwrap());
                        }
                        fs::write(face_data_file, serialised).unwrap();
                        face_data_vecs_map.insert(parent_pathbuf, face_data_vec);
                    }
                };
                if image_index % 10 == 0 {
                    tx.send(image_index as f32 / total_number_of_images as f32)
                        .await
                        .unwrap();
                }
            }
            tx.send(1.0).await.unwrap();
        });

        while let Some(received) = rx.recv().await {
            let _ = progress_percentage_output.send(received * 100.0).await;
            if received >= 1.0 {
                break;
            }
        }

        Ok(())
    })
}

pub fn get_detected_faces_for_image(image_path: &Path) -> Vec<FaceData> {
    let face_data_file = image_path
        .parent()
        .unwrap()
        .join(FACE_DATA_FOLDER_NAME)
        .join(FACE_DATA_FILE_NAME);
    if face_data_file.exists() {
        if let Ok(face_data_json) = fs::read_to_string(&face_data_file) {
            let face_data_vec: Vec<(OsString, Option<FaceData>)> =
                serde_json::from_str(&face_data_json).unwrap_or_default();
            face_data_vec
                .into_iter()
                .filter(|face_data| face_data.0 == image_path.file_name().unwrap())
                .filter_map(|(_file_name, face_data_option)| face_data_option)
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}
