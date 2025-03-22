use std::{
    ffi::OsStr,
    fs::{self},
    mem,
    os::linux::fs::MetadataExt,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use iced::{
    advanced::graphics::image::image_rs,
    futures::future,
    widget::{image::Handle, scrollable, text_input},
    Task,
};
use rfd::FileDialog;

use crate::{app::Message, pages::gallery::page::IMAGE_HEIGHT};

use super::{
    page::{
        GalleryPage, GalleryPageMessage, ImageRow, PersonToView, ARROW_KEY_SCROLL_AMOUNT,
        FACE_DATA_FOLDER_NAME, GALLERY_SCROLLABLE_ID, LIST_PEOPLE_SCROLL_ID, NUM_IMAGES_IN_ROW,
        PAGE_KEY_SCROLL_AMOUNT, RENAME_PERSON_INPUT_ID, ROW_BATCH_SIZE, THUMBNAIL_FOLDER_NAME,
        THUMBNAIL_SIZE,
    },
    utils::{
        common::{
            get_all_photos_by_name, get_capture_time_of_image, get_detected_faces_for_image,
            get_named_people_for_display, get_parent_folders, update_face_data,
            PhotoProcessingProgress,
        },
        face_extraction::extract_all_faces,
        face_recognition::group_all_faces,
        text_recognition::run_ocr,
        thumbnail_generation::generate_thumbnails,
    },
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
                    let gallery_files_list: Vec<PathBuf> =
                        if let Some(selected_folder) = selected_folder {
                            let directory_iterator = WalkDir::new(selected_folder)
                                .into_iter()
                                .filter_entry(|entry| {
                                    !entry.path().ends_with(THUMBNAIL_FOLDER_NAME)
                                        && !entry.path().ends_with(FACE_DATA_FOLDER_NAME)
                                        && entry.path().file_name().is_some_and(|basename| {
                                            !basename.to_string_lossy().starts_with(".")
                                        })
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
                                get_capture_time_of_image(file_path1)
                                    .cmp(&get_capture_time_of_image(file_path2))
                                    .reverse()
                            });
                            all_image_files
                        } else {
                            vec![]
                        };
                    let parent_paths = get_parent_folders(&gallery_files_list);
                    let chunked_gallery_files_list = gallery_files_list
                        .chunks(NUM_IMAGES_IN_ROW)
                        .map(|item| item.to_vec())
                        .collect();
                    (chunked_gallery_files_list, parent_paths)
                },
                |(chunked_gallery_files_list, parent_paths)| {
                    Message::Gallery(GalleryPageMessage::SetGalleryFilesList(
                        chunked_gallery_files_list,
                        parent_paths,
                    ))
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
        GalleryPageMessage::SetGalleryFilesList(gallery_files_list, parent_paths) => {
            state.gallery_paths_list = gallery_files_list.clone().into_iter().flatten().collect();
            state.gallery_parents_list = parent_paths;
            state.gallery_row_list = gallery_files_list
                .into_iter()
                .enumerate()
                .map(|(index, photo_vec)| ImageRow {
                    is_loaded: false,
                    index,
                    images_data: photo_vec.into_iter().map(|file| (file, None)).collect(),
                })
                .collect();
            let parent_folders = state.gallery_parents_list.clone();
            return Task::done(Message::Gallery(GalleryPageMessage::LoadImageRows(
                state
                    .gallery_row_list
                    .iter()
                    .take(ROW_BATCH_SIZE)
                    .cloned()
                    .collect(),
            )))
            .chain(Task::perform(
                async move { get_named_people_for_display(&parent_folders) },
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
                                        let file_name = image_path
                                            .file_name()
                                            .unwrap_or(OsStr::new("cant_read_filename"));
                                        let mut thumbnail_path = image_path
                                            .parent()
                                            .unwrap_or(Path::new("/"))
                                            .to_path_buf();
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
                                                let _ = resized.save(&thumbnail_path);
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
                        image_row.is_loaded = true;
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
                        image_row.is_loaded = false;
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
            state.gallery_scrollable_viewport_option = Some(viewport);
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
                if let Some(viewport) = state.gallery_scrollable_viewport_option {
                    let new_y = viewport.absolute_offset().y + ARROW_KEY_SCROLL_AMOUNT;
                    if new_y < viewport.content_bounds().height {
                        return scrollable::scroll_to(
                            GALLERY_SCROLLABLE_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: new_y,
                            },
                        );
                    } else {
                        return scrollable::scroll_to(
                            GALLERY_SCROLLABLE_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: viewport.content_bounds().height,
                            },
                        );
                    }
                }
            } else if state.show_people_view && state.person_to_view.is_none() {
                if let Some(viewport) = state.people_list_scrollable_viewport_option {
                    let new_y = viewport.absolute_offset().y + ARROW_KEY_SCROLL_AMOUNT;
                    if new_y < viewport.content_bounds().height {
                        return scrollable::scroll_to(
                            LIST_PEOPLE_SCROLL_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: new_y,
                            },
                        );
                    } else {
                        return scrollable::scroll_to(
                            LIST_PEOPLE_SCROLL_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: viewport.content_bounds().height,
                            },
                        );
                    }
                }
            }
        }
        GalleryPageMessage::ArrowUpKeyPressed => {
            if state.selected_image.is_none() && !state.show_people_view {
                if let Some(viewport) = state.gallery_scrollable_viewport_option {
                    let new_y = viewport.absolute_offset().y - ARROW_KEY_SCROLL_AMOUNT;
                    if new_y > 0.0 {
                        return scrollable::scroll_to(
                            GALLERY_SCROLLABLE_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: new_y,
                            },
                        );
                    } else {
                        return scrollable::scroll_to(
                            GALLERY_SCROLLABLE_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: 0.0,
                            },
                        );
                    }
                }
            } else if state.show_people_view && state.person_to_view.is_none() {
                if let Some(viewport) = state.people_list_scrollable_viewport_option {
                    let new_y = viewport.absolute_offset().y - ARROW_KEY_SCROLL_AMOUNT;
                    if new_y > 0.0 {
                        return scrollable::scroll_to(
                            LIST_PEOPLE_SCROLL_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: new_y,
                            },
                        );
                    } else {
                        return scrollable::scroll_to(
                            LIST_PEOPLE_SCROLL_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: 0.0,
                            },
                        );
                    }
                }
            }
        }
        GalleryPageMessage::PageDownKeyPressed => {
            if state.selected_image.is_none() && !state.show_people_view {
                if let Some(viewport) = state.gallery_scrollable_viewport_option {
                    let new_y = viewport.absolute_offset().y + PAGE_KEY_SCROLL_AMOUNT;
                    if new_y < viewport.content_bounds().height {
                        return scrollable::scroll_to(
                            GALLERY_SCROLLABLE_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: new_y,
                            },
                        );
                    } else {
                        return scrollable::scroll_to(
                            GALLERY_SCROLLABLE_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: viewport.content_bounds().height,
                            },
                        );
                    }
                }
            } else if state.show_people_view && state.person_to_view.is_none() {
                if let Some(viewport) = state.people_list_scrollable_viewport_option {
                    let new_y = viewport.absolute_offset().y + PAGE_KEY_SCROLL_AMOUNT;
                    if new_y < viewport.content_bounds().height {
                        return scrollable::scroll_to(
                            LIST_PEOPLE_SCROLL_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: new_y,
                            },
                        );
                    } else {
                        return scrollable::scroll_to(
                            LIST_PEOPLE_SCROLL_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: viewport.content_bounds().height,
                            },
                        );
                    }
                }
            }
        }
        GalleryPageMessage::PageUpKeyPressed => {
            if state.selected_image.is_none() && !state.show_people_view {
                if let Some(viewport) = state.gallery_scrollable_viewport_option {
                    let new_y = viewport.absolute_offset().y - PAGE_KEY_SCROLL_AMOUNT;
                    if new_y > 0.0 {
                        return scrollable::scroll_to(
                            GALLERY_SCROLLABLE_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: new_y,
                            },
                        );
                    } else {
                        return scrollable::scroll_to(
                            GALLERY_SCROLLABLE_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: 0.0,
                            },
                        );
                    }
                }
            } else if state.show_people_view && state.person_to_view.is_none() {
                if let Some(viewport) = state.people_list_scrollable_viewport_option {
                    let new_y = viewport.absolute_offset().y - PAGE_KEY_SCROLL_AMOUNT;
                    if new_y > 0.0 {
                        return scrollable::scroll_to(
                            LIST_PEOPLE_SCROLL_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: new_y,
                            },
                        );
                    } else {
                        return scrollable::scroll_to(
                            LIST_PEOPLE_SCROLL_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: viewport.absolute_offset().x,
                                y: 0.0,
                            },
                        );
                    }
                }
            }
        }
        GalleryPageMessage::SelectImageForBigView(image_path_option) => {
            state.person_to_manage = None;
            state.current_image_ocr_text = None;
            state.rename_person_field_text = String::new();
            state.show_ignore_person_confirmation = false;
            state.show_rename_confirmation = false;
            if image_path_option.is_none() {
                state.selected_image = None;
            } else {
                let image_path = image_path_option.expect("Can't fail");
                let faces_vec_result = get_detected_faces_for_image(&image_path);
                match faces_vec_result {
                    Ok(faces_vec) => {
                        state.selected_image = Some((image_path, faces_vec));
                    }
                    Err(err) => {
                        return Task::done(Message::ShowToast(
                            false,
                            format!("Couldn't load faces in image: {err}"),
                        ));
                    }
                }
            }
            if let Some(viewport) = state.gallery_scrollable_viewport_option {
                return scrollable::scroll_to(
                    GALLERY_SCROLLABLE_ID.clone(),
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
            } else if state.person_to_view.is_some() {
                state.person_to_view = None;
                state.show_people_view = true;
                return scrollable::scroll_to(
                    GALLERY_SCROLLABLE_ID.clone(),
                    scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
                )
                .chain(Task::done(Message::Gallery(
                    GalleryPageMessage::LoadImageRows(
                        state
                            .gallery_row_list
                            .iter()
                            .take(ROW_BATCH_SIZE)
                            .cloned()
                            .collect(),
                    ),
                )));
            } else if state.show_people_view {
                state.show_people_view = false;
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
                    Task::run(extract_all_faces(image_paths), |progress| {
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
                let (mut new_task, abort_handle) =
                    Task::run(generate_thumbnails(image_paths), |progress| {
                        Message::Gallery(GalleryPageMessage::SetPhotoProcessProgress(
                            progress.unwrap_or_default(),
                        ))
                    })
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
        GalleryPageMessage::RunFaceRecognition => {
            if matches!(state.photo_process_progress, PhotoProcessingProgress::None) {
                let parent_folders = state.gallery_parents_list.clone();
                let (new_task, abort_handle) =
                    Task::run(group_all_faces(parent_folders), |progress| {
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
            state.rename_person_field_text = String::new();
            state.show_ignore_person_confirmation = false;
            state.show_rename_confirmation = false;
        }
        GalleryPageMessage::OpenManagePersonView(original_image_path, face_data) => {
            state.person_to_manage = Some((original_image_path, face_data));
            return text_input::focus(text_input::Id::new(RENAME_PERSON_INPUT_ID));
        }
        GalleryPageMessage::MaybeRenamePerson(name_option) => {
            if let Some(name) = name_option {
                state.rename_person_field_text = name;
                state.show_rename_confirmation = true;
            } else if !state.rename_person_field_text.is_empty() {
                state.show_rename_confirmation = true;
            }
        }
        GalleryPageMessage::ConfirmRenamePerson => {
            if matches!(state.photo_process_progress, PhotoProcessingProgress::None) {
                if let Some((image_path, mut face_data)) = state.person_to_manage.take() {
                    face_data.name_of_person = Some(mem::take(&mut state.rename_person_field_text));
                    let finishing_message = Task::done(Message::Gallery(
                        GalleryPageMessage::SelectImageForBigView(Some(image_path.clone())),
                    ));
                    let parent_folders = state.gallery_parents_list.clone();
                    let background_task = Task::perform(
                        async move { update_face_data(image_path, face_data) },
                        |res| match res {
                            Ok(_) => Message::None,
                            Err(err) => Message::ShowToast(
                                false,
                                format!("Couldn't update face data: {err}"),
                            ),
                        },
                    )
                    .chain(finishing_message)
                    .chain(Task::perform(
                        async move { get_named_people_for_display(&parent_folders) },
                        |list_of_people| {
                            Message::Gallery(GalleryPageMessage::SetPeopleList(list_of_people))
                        },
                    ));
                    state.show_ignore_person_confirmation = false;
                    state.show_rename_confirmation = false;
                    return background_task;
                }
            } else {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("Can't manage people while background photo process is running"),
                ));
            }
        }
        GalleryPageMessage::CancelRenamePerson => {
            state.show_rename_confirmation = false;
            state.rename_person_field_text = String::new();
        }
        GalleryPageMessage::UpdateRenamePersonField(s) => {
            state.rename_person_field_text = s;
        }
        GalleryPageMessage::ShowIgnorePersonConfirmation => {
            state.show_ignore_person_confirmation = true;
            state.show_rename_confirmation = false;
        }
        GalleryPageMessage::ConfirmIgnorePerson => {
            if matches!(state.photo_process_progress, PhotoProcessingProgress::None) {
                if let Some((image_path, mut face_data)) = state.person_to_manage.take() {
                    face_data.is_ignored = true;
                    let finishing_message = Task::done(Message::Gallery(
                        GalleryPageMessage::SelectImageForBigView(Some(image_path.clone())),
                    ));
                    let background_task = Task::perform(
                        async move { update_face_data(image_path, face_data.clone()) },
                        |res| match res {
                            Ok(_) => Message::None,
                            Err(err) => Message::ShowToast(
                                false,
                                format!("Couldn't update face data: {err}"),
                            ),
                        },
                    )
                    .chain(finishing_message);
                    state.person_to_manage = None;
                    state.show_ignore_person_confirmation = false;
                    state.rename_person_field_text = String::new();
                    state.show_rename_confirmation = false;
                    return background_task;
                }
            } else {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("Can't manage people while background photo process is running"),
                ));
            }
        }
        GalleryPageMessage::CancelIgnorePerson => {
            state.show_ignore_person_confirmation = false;
        }
        GalleryPageMessage::TogglePeopleView => {
            if state.person_to_view.is_some() {
                state.show_people_view = true;
                state.person_to_view = None;
                return scrollable::scroll_to(
                    GALLERY_SCROLLABLE_ID.clone(),
                    scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
                )
                .chain(Task::done(Message::Gallery(
                    GalleryPageMessage::LoadImageRows(
                        state
                            .gallery_row_list
                            .iter()
                            .take(ROW_BATCH_SIZE)
                            .cloned()
                            .collect(),
                    ),
                )));
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
                let parent_folders = state.gallery_parents_list.clone();

                return Task::perform(
                    async move { get_all_photos_by_name(target_name, &parent_folders) },
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
                        .map(|metadata| metadata.st_mtime())
                        .unwrap_or_default()
                        .cmp(
                            &file_path2
                                .metadata()
                                .map(|metadata| metadata.st_mtime())
                                .unwrap_or_default(),
                        )
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
                        is_loaded: false,
                        index,
                        images_data: photo_vec.into_iter().map(|file| (file, None)).collect(),
                    })
                    .collect();
                person_to_view.list_of_rows = list_of_rows;
                person_to_view.list_of_image_paths = list_of_paths;
                state.first_loaded_row_index = 0;
                state.selected_image = None;
                state.person_to_manage = None;
                state.gallery_scrollable_viewport_option = None;
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
        GalleryPageMessage::PeopleListScrolled(viewport) => {
            state.people_list_scrollable_viewport_option = Some(viewport);
        }
        GalleryPageMessage::RunOcrOnSelectedImage => {
            if let Some((selected_image_path, _)) = state.selected_image.as_ref() {
                let ocr_image_path = selected_image_path.clone();
                return Task::perform(
                    async {
                        match run_ocr(ocr_image_path) {
                            Ok(ocr_text) => Some(ocr_text),
                            Err(err) => {
                                println!("Error with ocr: {err:?}");
                                None
                            }
                        }
                    },
                    |ocr_option| match ocr_option {
                        Some(ocr_text) => Message::Gallery(
                            GalleryPageMessage::SetCurrentImageOcrText(Some(ocr_text)),
                        ),
                        None => Message::ShowToast(
                            false,
                            String::from("Error running detecting text in image"),
                        ),
                    },
                );
            }
        }
        GalleryPageMessage::SetCurrentImageOcrText(ocr_text_option) => {
            state.current_image_ocr_text = ocr_text_option;
        }
    }
    Task::none()
}
