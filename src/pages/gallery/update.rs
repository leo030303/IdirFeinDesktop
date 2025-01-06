use arboard::Clipboard;
use std::{
    fs::{self},
    os::linux::fs::MetadataExt,
    path::PathBuf,
};
use walkdir::WalkDir;

use iced::{
    advanced::graphics::image::image_rs,
    futures::future,
    widget::{image::Handle, scrollable},
    Task,
};
use rfd::FileDialog;

use crate::{app::Message, pages::gallery::page::IMAGE_HEIGHT};

use super::page::{
    GalleryPage, GalleryPageMessage, ImageRow, ARROW_KEY_SCROLL_AMOUNT, FACE_DATA_FOLDER_NAME,
    NUM_IMAGES_IN_ROW, PAGE_KEY_SCROLL_AMOUNT, ROW_BATCH_SIZE, SCROLLABLE_ID,
    THUMBNAIL_FOLDER_NAME, THUMBNAIL_SIZE,
};
use super::{
    gallery_utils::{
        self, get_all_photos_by_name, get_detected_faces_for_image, get_named_people_for_display,
        update_face_data, PhotoProcessingProgress,
    },
    page::PersonToView,
};

pub fn update(state: &mut GalleryPage, message: GalleryPageMessage) -> Task<Message> {
    match message {
        GalleryPageMessage::PickGalleryFolder => {
            return Task::perform(
                async {
                    FileDialog::new()
                        .set_directory("/")
                        .set_can_create_directories(true)
                        .pick_folder()
                },
                |selected_folder| {
                    Message::Gallery(GalleryPageMessage::SetGalleryFolder(selected_folder))
                },
            );
        }
        GalleryPageMessage::SetGalleryFolder(selected_folder) => {
            state.selected_folder = selected_folder;
            return Task::done(Message::Gallery(GalleryPageMessage::LoadGalleryFolder));
        }
        GalleryPageMessage::LoadGalleryFolder => {
            let selected_folder = state.selected_folder.clone();
            let mut loading_task = Task::perform(
                async {
                    let gallery_files_list: Vec<Vec<PathBuf>> =
                        if let Some(selected_folder) = selected_folder {
                            let directory_iterator = WalkDir::new(selected_folder)
                                .into_iter()
                                .filter_entry(|entry| {
                                    !entry.path().ends_with(THUMBNAIL_FOLDER_NAME)
                                        && !entry.path().ends_with(FACE_DATA_FOLDER_NAME)
                                });
                            let mut all_image_files = directory_iterator
                                .filter_map(|read_dir_object| read_dir_object.ok())
                                .map(|read_dir_object| read_dir_object.into_path())
                                .filter(|path| {
                                    path.extension().is_some_and(|extension_os_str| {
                                        extension_os_str.to_str().is_some_and(|extension| {
                                            extension == "jpg"
                                                || extension == "jpeg"
                                                || extension == "png"
                                        })
                                    })
                                })
                                .collect::<Vec<PathBuf>>();
                            all_image_files.sort_unstable_by(|file_path1, file_path2| {
                                file_path1
                                    .metadata()
                                    .unwrap()
                                    .st_mtime()
                                    .cmp(&file_path2.metadata().unwrap().st_mtime())
                                    .reverse()
                            });
                            all_image_files
                                .chunks(NUM_IMAGES_IN_ROW)
                                .map(|item| item.to_vec())
                                .collect()
                        } else {
                            vec![]
                        };
                    gallery_files_list
                },
                |gallery_files_list| {
                    Message::Gallery(GalleryPageMessage::SetGalleryFilesList(gallery_files_list))
                },
            );
            if state.run_thumbnail_generation_on_start {
                loading_task = loading_task.chain(Task::done(Message::Gallery(
                    GalleryPageMessage::GenerateAllThumbnails,
                )));
            } else if state.run_face_extraction_on_start {
                loading_task = loading_task.chain(Task::done(Message::Gallery(
                    GalleryPageMessage::ExtractAllFaces,
                )));
            } else if state.run_face_recognition_on_start {
                loading_task = loading_task.chain(Task::done(Message::Gallery(
                    GalleryPageMessage::RunFaceRecognition,
                )));
            }
            return loading_task;
        }
        GalleryPageMessage::SetGalleryFilesList(gallery_files_list) => {
            state.gallery_paths_list = gallery_files_list.clone().into_iter().flatten().collect();
            state.gallery_row_list = gallery_files_list
                .clone()
                .into_iter()
                .enumerate()
                .map(|(index, photo_vec)| ImageRow {
                    loaded: false,
                    index,
                    images_data: photo_vec.into_iter().map(|file| (file, None)).collect(),
                })
                .collect();
            let image_paths_to_process = state.gallery_paths_list.clone();
            return Task::done(Message::Gallery(GalleryPageMessage::LoadImageRows(
                state
                    .gallery_row_list
                    .iter()
                    .take(ROW_BATCH_SIZE)
                    .cloned()
                    .collect(),
            )))
            .chain(Task::perform(
                async move { get_named_people_for_display(&image_paths_to_process) },
                |list_of_people| {
                    Message::Gallery(GalleryPageMessage::SetPeopleList(list_of_people))
                },
            ));
        }
        GalleryPageMessage::LoadImageRows(mut image_rows_to_load_list) => {
            return Task::perform(
                async move {
                    let loaded_image_data_list =
                        future::join_all(image_rows_to_load_list.clone().into_iter().map(
                            |image_row| {
                                future::join_all(image_row.images_data.into_iter().map(
                                    |(image_path, _)| async move {
                                        let file_name = image_path.file_name().unwrap();
                                        let mut thumbnail_path =
                                            image_path.parent().unwrap().to_path_buf();
                                        thumbnail_path.push(THUMBNAIL_FOLDER_NAME);
                                        if !thumbnail_path.exists() {
                                            fs::create_dir_all(&thumbnail_path).unwrap();
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
                                                    y_val = (original_height / 2)
                                                        - (original_width / 2);
                                                } else {
                                                    new_width = original_height;
                                                    new_height = original_height;
                                                    x_val = (original_width / 2)
                                                        - (original_height / 2);
                                                    y_val = 0;
                                                }
                                                let cropped = img
                                                    .crop_imm(x_val, y_val, new_width, new_height);
                                                let resized = cropped.resize(
                                                    THUMBNAIL_SIZE,
                                                    THUMBNAIL_SIZE,
                                                    image_rs::imageops::FilterType::Nearest,
                                                );
                                                resized.save(&thumbnail_path).unwrap();
                                            };
                                        }
                                        let handle = Handle::from_path(thumbnail_path);
                                        (image_path, handle)
                                    },
                                ))
                            },
                        ))
                        .await
                        .concat();
                    image_rows_to_load_list.iter_mut().for_each(|image_row| {
                        image_row.loaded = true;
                        image_row.images_data.iter_mut().for_each(
                            |(unloaded_path, unloaded_image_data)| {
                                *unloaded_image_data = loaded_image_data_list
                                    .iter()
                                    .find(|(loaded_path, _loaded_image_data)| {
                                        loaded_path == unloaded_path
                                    })
                                    .map(|val| val.1.clone());
                            },
                        )
                    });
                    image_rows_to_load_list
                },
                |image_handles_list| {
                    Message::Gallery(GalleryPageMessage::SetImageRows(image_handles_list))
                },
            );
        }
        GalleryPageMessage::UnloadImageRows(mut image_rows_to_unload_list) => {
            return Task::perform(
                async move {
                    image_rows_to_unload_list.iter_mut().for_each(|image_row| {
                        image_row.loaded = false;
                        image_row
                            .images_data
                            .iter_mut()
                            .for_each(|(_, image_data)| {
                                *image_data = None;
                            })
                    });
                    image_rows_to_unload_list
                },
                |image_handles_list| {
                    Message::Gallery(GalleryPageMessage::SetImageRows(image_handles_list))
                },
            );
        }
        GalleryPageMessage::SetImageRows(loaded_images_list) => {
            if let Some(person_to_view) = state.person_to_view.as_mut() {
                loaded_images_list.into_iter().for_each(|image_row| {
                    let image_row_index = image_row.index;
                    *person_to_view
                        .list_of_rows
                        .get_mut(image_row_index)
                        .expect("Shouldn't fail") = image_row;
                });
            } else {
                loaded_images_list.into_iter().for_each(|image_row| {
                    let image_row_index = image_row.index;
                    *state
                        .gallery_row_list
                        .get_mut(image_row_index)
                        .expect("Shouldn't fail") = image_row;
                });
            }
        }
        GalleryPageMessage::GalleryScrolled(viewport) => {
            state.scrollable_viewport_option = Some(viewport);
            let view_height = viewport.bounds().height;
            let displayed_images = ((view_height / IMAGE_HEIGHT).ceil() + 1.0) as usize;
            let images_scrolled_passed =
                (viewport.absolute_offset().y / IMAGE_HEIGHT).floor() as usize;

            if state.first_loaded_row_index != images_scrolled_passed {
                let mut images_to_unload_list: Vec<ImageRow> =
                    if let Some(person_to_view) = state.person_to_view.as_ref() {
                        person_to_view
                            .list_of_rows
                            .iter()
                            .skip(state.first_loaded_row_index)
                            .take(displayed_images)
                            .cloned()
                            .collect()
                    } else {
                        state
                            .gallery_row_list
                            .iter()
                            .skip(state.first_loaded_row_index)
                            .take(displayed_images)
                            .cloned()
                            .collect()
                    };
                state.first_loaded_row_index = images_scrolled_passed;
                let images_to_load_list: Vec<ImageRow> =
                    if let Some(person_to_view) = state.person_to_view.as_ref() {
                        person_to_view
                            .list_of_rows
                            .iter()
                            .skip(images_scrolled_passed)
                            .take(displayed_images)
                            .cloned()
                            .collect()
                    } else {
                        state
                            .gallery_row_list
                            .iter()
                            .skip(images_scrolled_passed)
                            .take(displayed_images)
                            .cloned()
                            .collect()
                    };
                images_to_unload_list.retain(|image_row| !images_to_load_list.contains(image_row));

                return Task::done(Message::Gallery(GalleryPageMessage::LoadImageRows(
                    images_to_load_list,
                )))
                .chain(Task::done(Message::Gallery(
                    GalleryPageMessage::UnloadImageRows(images_to_unload_list),
                )));
            }
        }
        GalleryPageMessage::ArrowDownKeyPressed => {
            if state.selected_image.is_none() && !state.show_people_view {
                if let Some(viewport) = state.scrollable_viewport_option {
                    let new_y = viewport.absolute_offset().y + ARROW_KEY_SCROLL_AMOUNT;
                    if new_y < viewport.content_bounds().height {
                        return scrollable::scroll_to(
                            SCROLLABLE_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: new_y,
                            },
                        );
                    }
                }
            }
        }
        GalleryPageMessage::ArrowUpKeyPressed => {
            if state.selected_image.is_none() && !state.show_people_view {
                if let Some(viewport) = state.scrollable_viewport_option {
                    let new_y = viewport.absolute_offset().y - ARROW_KEY_SCROLL_AMOUNT;
                    if new_y > 0.0 {
                        return scrollable::scroll_to(
                            SCROLLABLE_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: new_y,
                            },
                        );
                    }
                }
            }
        }
        GalleryPageMessage::PageDownKeyPressed => {
            if state.selected_image.is_none() && !state.show_people_view {
                if let Some(viewport) = state.scrollable_viewport_option {
                    let new_y = viewport.absolute_offset().y + PAGE_KEY_SCROLL_AMOUNT;
                    if new_y < viewport.content_bounds().height {
                        return scrollable::scroll_to(
                            SCROLLABLE_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: new_y,
                            },
                        );
                    }
                }
            }
        }
        GalleryPageMessage::PageUpKeyPressed => {
            if state.selected_image.is_none() && !state.show_people_view {
                if let Some(viewport) = state.scrollable_viewport_option {
                    let new_y = viewport.absolute_offset().y - PAGE_KEY_SCROLL_AMOUNT;
                    if new_y > 0.0 {
                        return scrollable::scroll_to(
                            SCROLLABLE_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: new_y,
                            },
                        );
                    }
                }
            }
        }
        GalleryPageMessage::SelectImageForBigView(image_path_option) => {
            state.person_to_manage = None;
            state.rename_person_editor_text = String::new();
            state.show_ignore_person_confirmation = false;
            state.show_rename_confirmation = false;
            if image_path_option.is_none() {
                state.selected_image = None;
            } else {
                let image_path = image_path_option.expect("Can't fail");
                let faces_vec = get_detected_faces_for_image(&image_path);
                state.selected_image = Some((image_path, faces_vec));
            }
            if let Some(viewport) = state.scrollable_viewport_option {
                return scrollable::scroll_to(
                    SCROLLABLE_ID.clone(),
                    scrollable::AbsoluteOffset {
                        x: viewport.absolute_offset().x,
                        y: viewport.absolute_offset().y,
                    },
                );
            }
        }
        GalleryPageMessage::EscapeKeyPressed => {
            if state.selected_image.is_some() {
                return Task::done(Message::Gallery(GalleryPageMessage::SelectImageForBigView(
                    None,
                )));
            }
        }
        GalleryPageMessage::SelectPreviousImage => {
            if let Some(selected_image) = &state.selected_image {
                if let Some(person_to_view) = state.person_to_view.as_ref() {
                    if let Some(current_index) = person_to_view
                        .list_of_image_paths
                        .iter()
                        .position(|current_path| *current_path == selected_image.0)
                    {
                        return Task::done(Message::Gallery(
                            GalleryPageMessage::SelectImageForBigView(
                                person_to_view
                                    .list_of_image_paths
                                    .get(current_index.saturating_sub(1))
                                    .cloned(),
                            ),
                        ));
                    }
                } else if let Some(current_index) = state
                    .gallery_paths_list
                    .iter()
                    .position(|current_path| *current_path == selected_image.0)
                {
                    return Task::done(Message::Gallery(
                        GalleryPageMessage::SelectImageForBigView(
                            state
                                .gallery_paths_list
                                .get(current_index.saturating_sub(1))
                                .cloned(),
                        ),
                    ));
                }
            }
        }
        GalleryPageMessage::SelectNextImage => {
            if let Some(selected_image) = &state.selected_image {
                if let Some(person_to_view) = state.person_to_view.as_ref() {
                    if let Some(current_index) = person_to_view
                        .list_of_image_paths
                        .iter()
                        .position(|current_path| *current_path == selected_image.0)
                    {
                        let new_index = current_index + 1;
                        if new_index < person_to_view.list_of_image_paths.len() {
                            return Task::done(Message::Gallery(
                                GalleryPageMessage::SelectImageForBigView(
                                    person_to_view.list_of_image_paths.get(new_index).cloned(),
                                ),
                            ));
                        }
                    }
                } else if let Some(current_index) = state
                    .gallery_paths_list
                    .iter()
                    .position(|current_path| *current_path == selected_image.0)
                {
                    let new_index = current_index + 1;
                    if new_index < state.gallery_paths_list.len() {
                        return Task::done(Message::Gallery(
                            GalleryPageMessage::SelectImageForBigView(
                                state.gallery_paths_list.get(new_index).cloned(),
                            ),
                        ));
                    }
                }
            }
        }
        GalleryPageMessage::ExtractAllFaces => {
            if matches!(state.photo_process_progress, PhotoProcessingProgress::None) {
                let image_paths = state.gallery_paths_list.clone();
                let (mut new_task, abort_handle) =
                    Task::run(gallery_utils::extract_all_faces(image_paths), |progress| {
                        Message::Gallery(GalleryPageMessage::SetPhotoProcessProgress(
                            progress.unwrap_or_default(),
                        ))
                    })
                    .abortable();
                state.photo_process_abort_handle = Some(abort_handle);
                if state.run_face_recognition_on_start {
                    new_task = new_task.chain(Task::done(Message::Gallery(
                        GalleryPageMessage::RunFaceRecognition,
                    )));
                }
                return new_task;
            } else {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("Can't run multiple photo processes at once"),
                ));
            }
        }
        GalleryPageMessage::GenerateAllThumbnails => {
            if matches!(state.photo_process_progress, PhotoProcessingProgress::None) {
                let image_paths = state.gallery_paths_list.clone();
                let (mut new_task, abort_handle) = Task::run(
                    gallery_utils::generate_thumbnails(image_paths),
                    |progress| {
                        Message::Gallery(GalleryPageMessage::SetPhotoProcessProgress(
                            progress.unwrap_or_default(),
                        ))
                    },
                )
                .abortable();
                state.photo_process_abort_handle = Some(abort_handle);
                if state.run_face_extraction_on_start {
                    new_task = new_task.chain(Task::done(Message::Gallery(
                        GalleryPageMessage::ExtractAllFaces,
                    )));
                } else if state.run_face_recognition_on_start {
                    new_task = new_task.chain(Task::done(Message::Gallery(
                        GalleryPageMessage::RunFaceRecognition,
                    )));
                }

                return new_task;
            } else {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("Can't run multiple photo processes at once"),
                ));
            }
        }
        GalleryPageMessage::SetPhotoProcessProgress(progress) => {
            if matches!(progress, PhotoProcessingProgress::None) {
                state.photo_process_abort_handle = None;
            }
            state.photo_process_progress = progress;
        }
        GalleryPageMessage::AbortProcess => {
            if let Some(abort_handle) = state.photo_process_abort_handle.as_ref() {
                abort_handle.abort();
                state.photo_process_progress = PhotoProcessingProgress::None;
            } else {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("No process handle found"),
                ));
            }
        }
        GalleryPageMessage::CopySelectedImagePath => {
            if let Some((image_path, _)) = state.selected_image.as_ref() {
                Clipboard::new()
                    .unwrap()
                    .set_text(image_path.as_path().to_str().unwrap_or_default())
                    .unwrap();
            }
        }
        GalleryPageMessage::RunFaceRecognition => {
            if matches!(state.photo_process_progress, PhotoProcessingProgress::None) {
                let image_paths = state.gallery_paths_list.clone();
                let (new_task, abort_handle) =
                    Task::run(gallery_utils::group_all_faces(image_paths), |progress| {
                        Message::Gallery(GalleryPageMessage::SetPhotoProcessProgress(
                            progress.unwrap_or_default(),
                        ))
                    })
                    .abortable();
                state.photo_process_abort_handle = Some(abort_handle);
                return new_task;
            } else {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("Can't run multiple photo processes at once"),
                ));
            }
        }
        GalleryPageMessage::CloseManagePersonView => {
            state.person_to_manage = None;
            state.rename_person_editor_text = String::new();
            state.show_ignore_person_confirmation = false;
            state.show_rename_confirmation = false;
        }
        GalleryPageMessage::OpenManagePersonView(original_image_path, face_data) => {
            state.person_to_manage = Some((original_image_path, face_data));
        }
        GalleryPageMessage::MaybeRenamePerson(name_option) => {
            if let Some(name) = name_option {
                state.rename_person_editor_text = name;
                state.show_rename_confirmation = true;
            } else if !state.rename_person_editor_text.is_empty() {
                state.show_rename_confirmation = true;
            }
        }
        GalleryPageMessage::ConfirmRenamePerson => {
            if let Some((image_path, mut face_data)) = state.person_to_manage.take() {
                face_data.name_of_person = Some(state.rename_person_editor_text.clone());
                let finishing_message = Task::done(Message::Gallery(
                    GalleryPageMessage::SelectImageForBigView(Some(image_path.clone())),
                ));
                let image_paths_to_process = state.gallery_paths_list.clone();
                let background_task = Task::perform(
                    async move {
                        update_face_data(image_path, face_data);
                    },
                    |_| Message::None,
                )
                .chain(finishing_message)
                .chain(Task::perform(
                    async move { get_named_people_for_display(&image_paths_to_process) },
                    |list_of_people| {
                        Message::Gallery(GalleryPageMessage::SetPeopleList(list_of_people))
                    },
                ));
                state.person_to_manage = None;
                state.rename_person_editor_text = String::new();
                state.show_ignore_person_confirmation = false;
                state.show_rename_confirmation = false;
                return background_task;
            }
        }
        GalleryPageMessage::CancelRenamePerson => {
            state.show_rename_confirmation = false;
            state.rename_person_editor_text = String::new();
        }
        GalleryPageMessage::UpdateRenamePersonEditor(s) => {
            state.rename_person_editor_text = s;
        }
        GalleryPageMessage::MaybeIgnorePerson => {
            state.show_ignore_person_confirmation = true;
            state.show_rename_confirmation = false;
        }
        GalleryPageMessage::ConfirmIgnorePerson => {
            if let Some((image_path, mut face_data)) = state.person_to_manage.take() {
                face_data.is_ignored = true;
                let finishing_message = Task::done(Message::Gallery(
                    GalleryPageMessage::SelectImageForBigView(Some(image_path.clone())),
                ));
                let background_task = Task::perform(
                    async move {
                        update_face_data(image_path, face_data.clone());
                    },
                    |_| Message::None,
                )
                .chain(finishing_message);
                state.person_to_manage = None;
                state.show_ignore_person_confirmation = false;
                state.rename_person_editor_text = String::new();
                state.show_rename_confirmation = false;
                return background_task;
            }
        }
        GalleryPageMessage::CancelIgnorePerson => {
            state.show_ignore_person_confirmation = false;
        }
        GalleryPageMessage::TogglePeopleView => {
            if state.person_to_view.is_some() {
                state.person_to_view = None;
                state.show_people_view = false;
                return scrollable::scroll_to(
                    SCROLLABLE_ID.clone(),
                    scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
                );
            } else {
                state.show_people_view = !state.show_people_view;
            }
        }
        GalleryPageMessage::SetPeopleList(people_list) => {
            state.people_list = people_list;
        }
        GalleryPageMessage::SetPersonToViewName(person_to_view_name_option) => {
            if let Some(person_to_view_name) = person_to_view_name_option {
                state.person_to_view = Some(PersonToView {
                    name: person_to_view_name.clone(),
                    list_of_image_paths: vec![],
                    list_of_rows: vec![],
                });
                let target_name = person_to_view_name;
                let image_paths_to_process = state.gallery_paths_list.clone();

                return Task::perform(
                    async move { get_all_photos_by_name(target_name, &image_paths_to_process) },
                    |list_of_paths| {
                        Message::Gallery(GalleryPageMessage::SetPersonToViewPaths(list_of_paths))
                    },
                );
            } else {
                state.person_to_view = None;
            }
        }
        GalleryPageMessage::SetPersonToViewPaths(mut list_of_paths) => {
            if let Some(person_to_view) = state.person_to_view.as_mut() {
                list_of_paths.sort_unstable_by(|file_path1, file_path2| {
                    file_path1
                        .metadata()
                        .unwrap()
                        .st_mtime()
                        .cmp(&file_path2.metadata().unwrap().st_mtime())
                        .reverse()
                });
                let grouped_list_of_paths: Vec<Vec<PathBuf>> = list_of_paths
                    .chunks(NUM_IMAGES_IN_ROW)
                    .map(|item| item.to_vec())
                    .collect();
                let list_of_rows = grouped_list_of_paths
                    .into_iter()
                    .enumerate()
                    .map(|(index, photo_vec)| ImageRow {
                        loaded: false,
                        index,
                        images_data: photo_vec.into_iter().map(|file| (file, None)).collect(),
                    })
                    .collect();
                person_to_view.list_of_rows = list_of_rows;
                person_to_view.list_of_image_paths = list_of_paths;
                state.first_loaded_row_index = 0;
                state.selected_image = None;
                state.person_to_manage = None;
                state.scrollable_viewport_option = None;
                state.show_people_view = false;
                return Task::done(Message::Gallery(GalleryPageMessage::LoadImageRows(
                    person_to_view
                        .list_of_rows
                        .iter()
                        .take(ROW_BATCH_SIZE)
                        .cloned()
                        .collect(),
                )));
            }
        }
    }
    Task::none()
}
