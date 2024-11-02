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
    GalleryPage, GalleryPageMessage, ImageRow, ARROW_KEY_SCROLL_AMOUNT, NUM_IMAGES_IN_ROW,
    PAGE_KEY_SCROLL_AMOUNT, ROW_BATCH_SIZE, SCROLLABLE_ID, THUMBNAIL_FOLDER_NAME, THUMBNAIL_SIZE,
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
            return Task::perform(
                async {
                    let gallery_files_list: Vec<Vec<PathBuf>> = if let Some(selected_folder) =
                        selected_folder
                    {
                        let directory_iterator = WalkDir::new(selected_folder)
                            .into_iter()
                            .filter_entry(|entry| !entry.path().ends_with(THUMBNAIL_FOLDER_NAME));
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
        }
        GalleryPageMessage::SetGalleryFilesList(gallery_files_list) => {
            state.gallery_list = gallery_files_list
                .clone()
                .into_iter()
                .enumerate()
                .map(|(index, photo_vec)| ImageRow {
                    loaded: false,
                    index,
                    images_data: photo_vec.into_iter().map(|file| (file, None)).collect(),
                })
                .collect();
            return Task::done(Message::Gallery(GalleryPageMessage::LoadImageRows(
                state
                    .gallery_list
                    .iter()
                    .take(ROW_BATCH_SIZE)
                    .cloned()
                    .collect(),
            )));
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
            loaded_images_list.into_iter().for_each(|image_row| {
                let image_row_index = image_row.index;
                *state
                    .gallery_list
                    .get_mut(image_row_index)
                    .expect("Shouldn't fail") = image_row;
            });
        }
        GalleryPageMessage::GalleryScrolled(viewport) => {
            state.scrollable_viewport_option = Some(viewport);
            let view_height = viewport.bounds().height;
            let displayed_images = ((view_height / IMAGE_HEIGHT).ceil() + 1.0) as usize;
            let images_scrolled_passed =
                (viewport.absolute_offset().y / IMAGE_HEIGHT).floor() as usize;

            if state.first_loaded_row_index != images_scrolled_passed {
                let mut images_to_unload_list: Vec<ImageRow> = state
                    .gallery_list
                    .iter()
                    .skip(state.first_loaded_row_index)
                    .take(displayed_images)
                    .cloned()
                    .collect();
                state.first_loaded_row_index = images_scrolled_passed;
                let images_to_load_list: Vec<ImageRow> = state
                    .gallery_list
                    .iter()
                    .skip(images_scrolled_passed)
                    .take(displayed_images)
                    .cloned()
                    .collect();
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
        GalleryPageMessage::ArrowUpKeyPressed => {
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
        GalleryPageMessage::PageDownKeyPressed => {
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
        GalleryPageMessage::PageUpKeyPressed => {
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
        GalleryPageMessage::SelectImageForBigView(image_path_option) => {
            state.selected_image = image_path_option;
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
    }
    Task::none()
}
