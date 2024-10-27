use std::{
    fs::{self},
    os::linux::fs::MetadataExt,
    path::PathBuf,
};

use iced::{
    futures::future,
    widget::{image::Handle, scrollable},
    Task,
};
use rfd::FileDialog;

use crate::{app::Message, pages::gallery::page::IMAGE_HEIGHT};

use super::page::{
    GalleryPage, GalleryPageMessage, ImageRow, ARROW_KEY_SCROLL_AMOUNT, NUM_IMAGES_IN_ROW,
    PAGE_KEY_SCROLL_AMOUNT, ROW_BATCH_SIZE, SCROLLABLE_ID,
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
                    let gallery_files_list: Vec<Vec<PathBuf>> =
                        if let Some(selected_folder) = selected_folder {
                            match fs::read_dir(selected_folder) {
                                Ok(directory_iterator) => {
                                    let mut all_image_files = directory_iterator
                                        .filter_map(|read_dir_object| read_dir_object.ok())
                                        .map(|read_dir_object| read_dir_object.path())
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
                                }
                                Err(err) => {
                                    println!("Error reading directory: {err:?}");
                                    vec![]
                                }
                            }
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
            state.top_offset = 0.0;
            if !state.gallery_list.is_empty() {
                state.bottom_offset =
                    (state.gallery_list.len() - ROW_BATCH_SIZE) as f32 * IMAGE_HEIGHT;
            }
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
                                        let handle = Handle::from_path(&image_path);
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
            let images_scrolled_passed = viewport.absolute_offset().y as i64 / IMAGE_HEIGHT as i64;
            let batch_to_load = images_scrolled_passed as usize / ROW_BATCH_SIZE;

            if state.loaded_batch_index != batch_to_load {
                let images_to_unload_list = state
                    .gallery_list
                    .chunks(ROW_BATCH_SIZE)
                    .nth(state.loaded_batch_index)
                    .unwrap()
                    .to_vec();
                state.loaded_batch_index = batch_to_load;
                let images_to_load_list = state
                    .gallery_list
                    .chunks(ROW_BATCH_SIZE)
                    .nth(batch_to_load)
                    .unwrap()
                    .to_vec();
                state.top_offset = images_scrolled_passed as f32 * IMAGE_HEIGHT;
                state.bottom_offset = (state.gallery_list.len() as f32
                    - ROW_BATCH_SIZE as f32
                    - images_scrolled_passed as f32)
                    * IMAGE_HEIGHT;

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
            state.selected_image = image_path_option
        }
    }
    Task::none()
}
