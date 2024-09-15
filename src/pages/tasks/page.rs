use iced::{Element, Task};
use serde::{Deserialize, Serialize};

use crate::app::Message;

use super::update::update;
use super::view::{main_view, tool_view};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskPageConfig {}

pub struct TasksPage {}

#[derive(Debug, Clone)]
pub enum TasksPageMessage {}

impl TasksPage {
    pub fn new(_config: &TaskPageConfig) -> Self {
        Self {}
    }

    pub fn opening_task() -> Task<Message> {
        Task::none()
    }

    pub fn closing_task(&mut self) -> Task<Message> {
        println!("Closing task from tasks");
        Task::none()
    }

    pub fn update(&mut self, message: TasksPageMessage) -> Task<Message> {
        update(self, message)
    }

    pub fn view(&self) -> Element<Message> {
        main_view(self)
    }

    pub fn tool_view(&self) -> Element<Message> {
        tool_view(self)
    }
}
