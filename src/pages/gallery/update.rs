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
    GalleryPage, GalleryPageMessage, ARROW_KEY_SCROLL_AMOUNT, PAGE_KEY_SCROLL_AMOUNT, SCROLLABLE_ID,
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
                                        .chunks(4)
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
                .map(|photo_vec| {
                    (
                        false,
                        photo_vec.into_iter().map(|file| (file, None)).collect(),
                    )
                })
                .collect();
            state.top_offset = 0.0;
            state.bottom_offset = (state.gallery_list.len() - 3) as f32 * IMAGE_HEIGHT;
            return Task::done(Message::Gallery(GalleryPageMessage::LoadImageHandles(
                gallery_files_list
                    .into_iter()
                    .enumerate()
                    .take(3)
                    .map(|(index, photo_path_vec)| {
                        (
                            index,
                            (
                                true,
                                photo_path_vec
                                    .into_iter()
                                    .map(|photo_path| (photo_path, None))
                                    .collect(),
                            ),
                        )
                    })
                    .collect(),
            )));
        }
        GalleryPageMessage::LoadImageHandles(images_to_load_list) => {
            return Task::perform(
                async move {
                    future::join_all(images_to_load_list.into_iter().map(
                        |(index, row_images_list)| {
                            future::join_all(row_images_list.1.into_iter().map(
                                |(image_path, _)| async move {
                                    (index, image_path.clone(), Handle::from_path(image_path))
                                },
                            ))
                        },
                    ))
                    .await
                    .concat()
                },
                |image_handles_list| {
                    Message::Gallery(GalleryPageMessage::SetImageHandles(image_handles_list))
                },
            );
        }
        GalleryPageMessage::SetImageHandles(loaded_images_list) => {
            loaded_images_list
                .into_iter()
                .for_each(|(index, image_path, image_data)| {
                    state
                        .gallery_list
                        .get_mut(index)
                        .expect("Shouldn't fail")
                        .1
                        .iter_mut()
                        .find(|(path, _)| *path == image_path)
                        .expect("Shouldn't fail")
                        .1 = Some(image_data);
                    state.gallery_list.get_mut(index).expect("Shouldn't fail").0 = true;
                });
        }
        GalleryPageMessage::UnloadImageHandle(index, image_path) => {
            println!("Unload image index: {index:?}");
            state
                .gallery_list
                .get_mut(index)
                .expect("Shouldn't fail")
                .1
                .iter_mut()
                .find(|(path, _)| *path == image_path)
                .expect("Shouldn't fail")
                .1 = None;
            state.gallery_list.get_mut(index).expect("Shouldn't fail").0 = false;
        }
        GalleryPageMessage::GalleryScrolled(viewport) => {
            let load_ahead_amount = 2;
            state.scrollable_viewport_option = Some(viewport);
            let images_scrolled_passed = viewport.absolute_offset().y as i64 / IMAGE_HEIGHT as i64;
            if state.last_images_scrolled_past_val != images_scrolled_passed {
                state.last_images_scrolled_past_val = images_scrolled_passed;
                let image_indexes_to_load: Vec<usize> = (images_scrolled_passed - load_ahead_amount
                    ..images_scrolled_passed + load_ahead_amount)
                    .filter(|value| *value >= 0 && *value < state.gallery_list.len() as i64)
                    .map(|val| val as usize)
                    .collect();
                state
                    .loaded_image_indexes
                    .iter()
                    .filter(|index| !image_indexes_to_load.contains(index))
                    .for_each(|index| {
                        if let Some(list_item) = state.gallery_list.get_mut(*index) {
                            list_item
                                .1
                                .iter_mut()
                                .for_each(|(_path, image_handle)| *image_handle = None);
                            list_item.0 = false;
                        }
                    });
                let mut images_to_load_list = vec![];
                image_indexes_to_load
                    .iter()
                    .filter(|index| image_indexes_to_load.contains(index))
                    .for_each(|index| {
                        if let Some(list_item) = state.gallery_list.get(*index) {
                            state.loaded_image_indexes.push(*index);
                            images_to_load_list.push((*index, list_item.clone()));
                        }
                    });
                state.loaded_image_indexes = image_indexes_to_load;
                state.top_offset = images_scrolled_passed as f32 * IMAGE_HEIGHT;
                state.bottom_offset = (state.gallery_list.len() as f32
                    - ((load_ahead_amount * 2) + 1) as f32
                    - images_scrolled_passed as f32)
                    * IMAGE_HEIGHT;
                return Task::done(Message::Gallery(GalleryPageMessage::LoadImageHandles(
                    images_to_load_list,
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
