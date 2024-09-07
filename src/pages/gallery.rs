use iced::widget::{row, text};
use iced::{Element, Length, Task};

use crate::Message;

pub struct GalleryPage {}

#[derive(Debug, Clone)]
pub enum GalleryPageMessage {}

impl GalleryPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, message: GalleryPageMessage) -> Task<Message> {
        match message {}
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        text("Gallery Page").size(24).into()
    }

    pub fn tool_view(&self) -> Element<Message> {
        row![].width(Length::FillPortion(1)).into()
    }
}

impl Default for GalleryPage {
    fn default() -> Self {
        Self::new()
    }
}
