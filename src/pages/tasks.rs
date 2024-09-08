use iced::widget::{row, text};
use iced::{Element, Length, Task};

use crate::Message;

pub struct TasksPage {}

#[derive(Debug, Clone)]
pub enum TasksPageMessage {}

impl TasksPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn closing_task(&mut self) -> Task<Message> {
        println!("Closing task from tasks");
        Task::none()
    }

    pub fn update(&mut self, message: TasksPageMessage) -> Task<Message> {
        match message {}
        Task::none()
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
