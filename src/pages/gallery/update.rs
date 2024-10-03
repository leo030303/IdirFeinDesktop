use std::{
    fs::{self, File},
    io::Read,
    os::linux::fs::MetadataExt,
    path::PathBuf,
};

use iced::{
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
            let selected_folder = FileDialog::new()
                .set_directory("/")
                .set_can_create_directories(true)
                .pick_folder();
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
                .map(|photo_vec| photo_vec.into_iter().map(|file| (file, None)).collect())
                .collect();
            return Task::batch(gallery_files_list.into_iter().enumerate().take(3).map(
                |(index, photo_path_vec)| {
                    Task::done(Message::Gallery(GalleryPageMessage::LoadImageHandle(
                        index,
                        photo_path_vec
                            .into_iter()
                            .map(|photo_path| (photo_path, None))
                            .collect(),
                    )))
                },
            ));
        }
        GalleryPageMessage::LoadImageHandle(index, image_path_vec) => {
            return Task::batch(image_path_vec.into_iter().map(|(image_path, _)| {
                Task::perform(
                    async move {
                        let mut file = File::open(&image_path).unwrap();
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer).unwrap();
                        (image_path, Handle::from_bytes(buffer))
                    },
                    move |(image_path, image_handle)| {
                        Message::Gallery(GalleryPageMessage::SetImageHandle(
                            index,
                            image_path,
                            image_handle,
                        ))
                    },
                )
            }))
        }
        GalleryPageMessage::SetImageHandle(index, image_path, image_data) => {
            state
                .gallery_list
                .get_mut(index)
                .expect("Shouldn't fail")
                .iter_mut()
                .find(|(path, _)| *path == image_path)
                .expect("Shouldn't fail")
                .1 = Some(image_data);
        }
        GalleryPageMessage::UnloadImageHandle(index, image_path) => {
            println!("Unload image index: {index:?}");
            state
                .gallery_list
                .get_mut(index)
                .expect("Shouldn't fail")
                .iter_mut()
                .find(|(path, _)| *path == image_path)
                .expect("Shouldn't fail")
                .1 = None;
        }
        GalleryPageMessage::GalleryScrolled(viewport) => {
            let load_ahead_amount = 2;
            state.scrollable_viewport_option = Some(viewport);
            let images_scrolled_passed = viewport.absolute_offset().y as i64 / IMAGE_HEIGHT as i64;
            if state.last_images_scrolled_past_val != images_scrolled_passed {
                state.last_images_scrolled_past_val = images_scrolled_passed;
                let image_indexes_to_load: Vec<usize> = (images_scrolled_passed - load_ahead_amount
                    ..images_scrolled_passed + load_ahead_amount)
                    .filter(|value| *value >= 0)
                    .map(|val| val as usize)
                    .collect();
                state
                    .loaded_image_indexes
                    .iter()
                    .filter(|index| !image_indexes_to_load.contains(index))
                    .for_each(|index| {
                        if let Some(list_item) = state.gallery_list.get_mut(*index) {
                            list_item
                                .iter_mut()
                                .for_each(|(_path, image_handle)| *image_handle = None);
                        }
                    });
                let mut tasks_list = vec![];
                image_indexes_to_load
                    .iter()
                    .filter(|index| image_indexes_to_load.contains(index))
                    .for_each(|index| {
                        if let Some(list_item) = state.gallery_list.get(*index) {
                            state.loaded_image_indexes.push(*index);
                            tasks_list.push(Task::done(Message::Gallery(
                                GalleryPageMessage::LoadImageHandle(*index, list_item.clone()),
                            )));
                        }
                    });
                state.loaded_image_indexes = image_indexes_to_load;
                return tasks_list
                    .into_iter()
                    .fold(Task::none(), |accumulator, task_item| {
                        accumulator.chain(task_item)
                    });
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
