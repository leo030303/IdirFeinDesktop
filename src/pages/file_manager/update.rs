use iced::Task;

use crate::Message;

use super::page::{FileManagerPage, FileManagerPageMessage};

pub fn update(_state: &mut FileManagerPage, message: FileManagerPageMessage) -> Task<Message> {
    match message {}
    Task::none()
}
