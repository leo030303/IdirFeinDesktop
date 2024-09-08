use iced::{
    widget::{row, text},
    Element, Length,
};

use crate::Message;

use super::page::TasksPage;

pub fn main_view(_state: &TasksPage) -> Element<Message> {
    text("Tasks Page").size(24).into()
}

pub fn tool_view(_state: &TasksPage) -> Element<Message> {
    row![].width(Length::FillPortion(1)).into()
}
