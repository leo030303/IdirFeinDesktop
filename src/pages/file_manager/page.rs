use iced::{Element, Task};

use crate::app::Message;

use super::update::update;
use super::view::{main_view, tool_view};

pub struct FileManagerPage {}

#[derive(Debug, Clone)]
pub enum FileManagerPageMessage {}

impl FileManagerPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn closing_task(&mut self) -> Task<Message> {
        println!("Closing task from files");
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

impl Default for FileManagerPage {
    fn default() -> Self {
        Self::new()
    }
}
