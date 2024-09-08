use iced::Task;

use crate::Message;

use super::page::{SettingsPage, SettingsPageMessage};

pub fn update(_state: &mut SettingsPage, message: SettingsPageMessage) -> Task<Message> {
    match message {}
    Task::none()
}
