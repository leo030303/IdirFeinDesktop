use iced::{Element, Task};
use serde::{Deserialize, Serialize};

use crate::app::Message;

use super::update::update;
use super::view::{main_view, tool_view};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GalleryPageConfig {}

pub struct GalleryPage {}

#[derive(Debug, Clone)]
pub enum GalleryPageMessage {}

impl GalleryPage {
    pub fn new(_config: &GalleryPageConfig) -> Self {
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
