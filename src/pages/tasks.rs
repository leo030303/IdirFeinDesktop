use iced::widget::{row, text};
use iced::{Element, Length};

use crate::Message;

pub struct TasksPage {}

#[derive(Debug, Clone)]
pub enum TasksPageMessage {}

impl TasksPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, message: TasksPageMessage) {
        match message {}
    }

    pub fn view(&self) -> Element<Message> {
        text("Tasks Page").size(24).into()
    }

    pub fn tool_view(&self) -> Element<Message> {
        row![].width(Length::FillPortion(1)).into()
    }
}

impl Default for TasksPage {
    fn default() -> Self {
        Self::new()
    }
}
