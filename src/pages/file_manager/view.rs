use iced::{
    widget::{row, text},
    Element, Length,
};

use crate::Message;

use super::page::FileManagerPage;

pub fn main_view(_state: &FileManagerPage) -> Element<Message> {
    text("File Manager Page").size(24).into()
}

pub fn tool_view(_state: &FileManagerPage) -> Element<Message> {
    row![].width(Length::FillPortion(1)).into()
}
