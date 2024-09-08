use iced::Task;

use crate::Message;

use super::page::{GalleryPage, GalleryPageMessage};

pub fn update(_state: &mut GalleryPage, message: GalleryPageMessage) -> Task<Message> {
    match message {}
    Task::none()
}
