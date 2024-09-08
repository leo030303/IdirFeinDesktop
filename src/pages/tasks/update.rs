use iced::Task;

use crate::Message;

use super::page::{TasksPage, TasksPageMessage};

pub fn update(_state: &mut TasksPage, message: TasksPageMessage) -> Task<Message> {
    match message {}
    Task::none()
}
