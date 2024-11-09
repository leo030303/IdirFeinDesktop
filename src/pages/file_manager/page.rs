use iced::{Element, Task};
use serde::{Deserialize, Serialize};

use crate::app::Message;

use super::update::update;
use super::view::{main_view, tool_view};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileManagerPageConfig {}

pub struct FileManagerPage {}

#[derive(Debug, Clone)]
pub enum FileManagerPageMessage {}

impl FileManagerPage {
    pub fn new(_config: &FileManagerPageConfig) -> Self {
        Self {}
    }

    pub fn opening_task() -> Task<Message> {
        Task::none()
    }

    pub fn closing_task(&mut self) -> Task<Message> {
        Task::none()
    }

    pub fn update(&mut self, message: FileManagerPageMessage) -> Task<Message> {
        update(self, message)
    }

    pub fn view(&self) -> Element<Message> {
        main_view(self)
    }

    pub fn tool_view(&self) -> Element<Message> {
        tool_view(self)
    }
}
