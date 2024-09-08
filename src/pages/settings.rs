use iced::widget::{row, text};
use iced::{Element, Length, Task};

use crate::Message;

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
        match message {}
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        text("Settings Page").size(24).into()
    }

    pub fn tool_view(&self) -> Element<Message> {
        row![].width(Length::FillPortion(1)).into()
    }
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self::new()
    }
}
