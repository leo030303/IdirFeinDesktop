use iced::Task;

use crate::app::Message;
use crate::config::AppConfig;

use super::page::{SettingsPage, SettingsPageMessage};

pub fn update(
    state: &mut SettingsPage,
    message: SettingsPageMessage,
    app_config: &mut AppConfig,
) -> Task<Message> {
    match message {
        SettingsPageMessage::ResultFromSave((is_success, message)) => {
            state.save_was_successful = is_success;
            state.save_message = message;
            state.is_saving = false;
        }
        SettingsPageMessage::StartSaving => state.is_saving = true,
        SettingsPageMessage::ChangeTab(tab) => state.current_tab = tab,
        SettingsPageMessage::SetTheme(theme) => {
            app_config.set_theme(theme);
            return Task::done(Message::SaveConfig);
        }
    }
    Task::none()
}
