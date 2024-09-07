use iced::widget::{row, text};
use iced::{Element, Length, Task};

use crate::Message;

pub struct FileManagerPage {}

#[derive(Debug, Clone)]
pub enum FileManagerPageMessage {}

impl FileManagerPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, message: FileManagerPageMessage) -> Task<Message> {
        match message {}
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        text("File Manager Page").size(24).into()
    }

    pub fn tool_view(&self) -> Element<Message> {
        row![].width(Length::FillPortion(1)).into()
    }
}

impl Default for FileManagerPage {
    fn default() -> Self {
        Self::new()
    }
}
