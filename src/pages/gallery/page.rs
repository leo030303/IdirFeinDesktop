use iced::{Element, Task};

use crate::Message;

use super::update::update;
use super::view::{main_view, tool_view};

pub struct GalleryPage {}

#[derive(Debug, Clone)]
pub enum GalleryPageMessage {}

impl GalleryPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn closing_task(&mut self) -> Task<Message> {
        println!("Closing task from gallery");
        Task::none()
    }

    pub fn update(&mut self, message: GalleryPageMessage) -> Task<Message> {
        update(self, message)
    }

    pub fn view(&self) -> Element<Message> {
        main_view(self)
    }

    pub fn tool_view(&self) -> Element<Message> {
        tool_view(self)
    }
}

impl Default for GalleryPage {
    fn default() -> Self {
        Self::new()
    }
}
