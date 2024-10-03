use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

use iced::{widget::image::Handle, Task};
use rfd::FileDialog;

use crate::{app::Message, pages::gallery::page::IMAGE_HEIGHT};

use super::page::{GalleryPage, GalleryPageMessage};

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
                    let gallery_files_list: Vec<PathBuf> =
                        if let Some(selected_folder) = selected_folder {
                            match fs::read_dir(selected_folder) {
                                Ok(directory_iterator) => directory_iterator
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
                                    .collect(),
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
                .map(|file| (file, None))
                .collect();
            return Task::batch(gallery_files_list.into_iter().take(6).map(|file_path| {
                Task::done(Message::Gallery(GalleryPageMessage::LoadImageHandle(
                    file_path,
                )))
            }));
        }
        GalleryPageMessage::LoadImageHandle(image_path) => {
            return Task::perform(
                async {
                    let mut file = File::open(&image_path).unwrap();
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer).unwrap();
                    (image_path, Handle::from_bytes(buffer))
                },
                |(image_path, image_handle)| {
                    Message::Gallery(GalleryPageMessage::SetImageHandle(image_path, image_handle))
                },
            );
        }
        GalleryPageMessage::SetImageHandle(image_path, image_data) => {
            state
                .gallery_list
                .get_mut(
                    state
                        .gallery_list
                        .clone()
                        .into_vec()
                        .iter()
                        .position(|(current_path, _)| *current_path == image_path)
                        .expect("Shouldn't fail"),
                )
                .expect("Shouldn't fail")
                .1 = Some(image_data);
        }
        GalleryPageMessage::UnloadImageHandle(image_path) => {
            state
                .gallery_list
                .get_mut(
                    state
                        .gallery_list
                        .clone()
                        .into_vec()
                        .iter()
                        .position(|(current_path, _)| *current_path == image_path)
                        .expect("Shouldn't fail"),
                )
                .expect("Shouldn't fail")
                .1 = None;
        }
        GalleryPageMessage::GalleryScrolled(viewport) => {
            let images_scrolled_passed = viewport.absolute_offset().y as i64 / IMAGE_HEIGHT as i64;
            if state.last_images_scrolled_past_val != images_scrolled_passed {
                state.last_images_scrolled_past_val = images_scrolled_passed;
                let image_indexes_to_load: Vec<usize> = (images_scrolled_passed - 3
                    ..images_scrolled_passed + 3)
                    .map(|val| val as usize)
                    .collect();
                state.loaded_image_indexes.iter().for_each(|index| {
                    if !image_indexes_to_load.contains(index) {
                        if let Some(list_item) = state.gallery_list.get_mut(*index) {
                            list_item.1 = None;
                        }
                    }
                });
                let mut tasks_list = vec![];
                image_indexes_to_load.iter().for_each(|index| {
                    if !state.loaded_image_indexes.contains(index) {
                        if let Some(list_item) = state.gallery_list.get(*index) {
                            state.loaded_image_indexes.push(*index);
                            tasks_list.push(Task::done(Message::Gallery(
                                GalleryPageMessage::LoadImageHandle(list_item.0.clone()),
                            )));
                        }
                    }
                });
                state.loaded_image_indexes = image_indexes_to_load;
                return Task::batch(tasks_list);
            }
        }
    }
    Task::none()
}
