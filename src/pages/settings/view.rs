use iced::{
    widget::{row, text},
    Element, Length,
};

use crate::Message;

use super::page::SettingsPage;

pub fn main_view(_state: &SettingsPage) -> Element<Message> {
    text("Settings Page").size(24).into()
}

pub fn tool_view(_state: &SettingsPage) -> Element<Message> {
    row![].width(Length::FillPortion(1)).into()
}
