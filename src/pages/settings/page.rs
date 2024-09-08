use iced::{Element, Task};

use crate::Message;

use super::update::update;
use super::view::{main_view, tool_view};

pub struct SettingsPage {}

#[derive(Debug, Clone)]
pub enum SettingsPageMessage {}

impl SettingsPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn closing_task(&mut self) -> Task<Message> {
        println!("Closing task from settings");
        Task::none()
    }

    pub fn update(&mut self, message: SettingsPageMessage) -> Task<Message> {
        update(self, message)
    }

    pub fn view(&self) -> Element<Message> {
        main_view(self)
    }

    pub fn tool_view(&self) -> Element<Message> {
        tool_view(self)
    }
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self::new()
    }
}
