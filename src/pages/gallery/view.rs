use iced::{
    widget::{row, text},
    Element, Length,
};

use crate::app::Message;

use super::page::GalleryPage;

pub fn main_view(_state: &GalleryPage) -> Element<Message> {
    text("Gallery Page").size(24).into()
}

pub fn tool_view(_state: &GalleryPage) -> Element<Message> {
    row![].width(Length::FillPortion(1)).into()
}
