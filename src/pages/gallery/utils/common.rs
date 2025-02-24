use image::DynamicImage;
use opencv::{
    core::{Mat, MatTraitConstManual},
    imgcodecs,
    objdetect::{FaceRecognizerSF, FaceRecognizerSFTrait, FaceRecognizerSFTraitConst},
};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

#[allow(unused)]
use rten_tensor::prelude::*;

use crate::{
    constants::APP_ID,
    pages::gallery::page::{
        FACE_DATA_FILE_NAME, FACE_DATA_FOLDER_NAME, PATH_TO_FACE_RECOGNITION_MODEL, UNNAMED_STRING,
    },
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
    pub thumbnail_filename: OsString,

    /// The data required to match this face to other faces
    pub face_matrix_bytes: Vec<u8>,

    /// The names that were already checked for recognition matches, to avoid repeated checking
    pub checked_names: Vec<String>,
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
        let idirfein_data_dir = dirs::data_dir()
            .expect("Can't find data dir")
            .as_path()
            .join(APP_ID);

        let mut opencv_face_recognizer = FaceRecognizerSF::create_def(
            &idirfein_data_dir
                .join(PATH_TO_FACE_RECOGNITION_MODEL)
                .to_string_lossy(),
            "",
        )?;
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
