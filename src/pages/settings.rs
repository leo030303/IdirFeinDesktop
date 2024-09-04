use iced::widget::{row, text};
use iced::{Element, Length};

use crate::Message;

pub struct SettingsPage {}

#[derive(Debug, Clone)]
pub enum SettingsPageMessage {}

impl SettingsPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, message: SettingsPageMessage) {
        match message {}
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
