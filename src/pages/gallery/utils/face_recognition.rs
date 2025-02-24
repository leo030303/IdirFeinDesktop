use std::{ffi::OsString, fs, path::PathBuf};

use iced::{
    futures::{SinkExt, Stream},
    stream::try_channel,
};
use opencv::{
    core::Mat,
    objdetect::{FaceRecognizerSF, FaceRecognizerSF_DisType},
    prelude::FaceRecognizerSFTraitConst,
};

use crate::{
    constants::APP_ID,
    pages::gallery::page::{
        FACE_DATA_FILE_NAME, FACE_DATA_FOLDER_NAME, PATH_TO_FACE_RECOGNITION_MODEL,
    },
};

use super::common::{FaceData, PhotoProcessingProgress};

pub fn match_face_to_person(
    unknown_face: &mut FaceData,
    named_people_list: Vec<(String, Mat)>,
) -> Option<String> {
    const L2NORM_SIMILAR_THRESH: f64 = 1.128;

    let mut checked_names = vec![];
    let best_person_and_score = named_people_list
        .iter()
        .filter(|(person_name, _named_person_face_features)| {
            !unknown_face.checked_names.contains(person_name)
        })
        .map(|(person_name, named_person_face_features)| {
            let unknown_face_features = unknown_face.matrix();
            let idirfein_data_dir = dirs::data_dir()
                .expect("Can't find data dir")
                .as_path()
                .join(APP_ID);

            let opencv_face_recognizer = FaceRecognizerSF::create_def(
                &idirfein_data_dir
                    .join(PATH_TO_FACE_RECOGNITION_MODEL)
                    .to_string_lossy(),
                "",
            )
            .unwrap();
            let l2_score = opencv_face_recognizer.match_(
                &named_person_face_features,
                &unknown_face_features,
                FaceRecognizerSF_DisType::FR_NORM_L2.into(),
            );
            println!("{person_name} score is {l2_score:?}");
            checked_names.push(person_name.clone());
            (
                person_name,
                l2_score.unwrap_or(L2NORM_SIMILAR_THRESH + 100.0),
            )
        })
        .min_by_key(|x| (x.1 * 10000.0) as i32);
    unknown_face.checked_names.append(&mut checked_names);

    if let Some((person_name, l2_score)) = best_person_and_score {
        if l2_score <= L2NORM_SIMILAR_THRESH {
            return Some(person_name.clone());
        }
    }

    None
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
