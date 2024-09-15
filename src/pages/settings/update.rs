use iced::Task;
use rfd::FileDialog;

use crate::config::AppConfig;
use crate::{app::Message, Page};

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
        SettingsPageMessage::SetDefaultPageOnOpen(page_str) => {
            app_config.default_page_on_open = match page_str {
                "Settings" => Page::Settings,
                "Notes" => Page::Notes,
                "Tasks" => Page::Tasks,
                "File Manager" => Page::FileManager,
                "Gallery" => Page::Gallery,
                "Passwords" => Page::Passwords,
                _ => Page::Notes,
            };
            return Task::done(Message::SaveConfig);
        }
        SettingsPageMessage::NotesSetDefaultFolder => {
            let selected_folder = FileDialog::new().pick_folder();
            app_config.notes_config.default_folder = selected_folder;
            return Task::done(Message::SaveConfig);
        }
        SettingsPageMessage::NotesSetShowSidebarOnStart(b) => {
            app_config.notes_config.show_sidebar_on_start = b;
            return Task::done(Message::SaveConfig);
        }
        SettingsPageMessage::NotesSetShowEditorOnStart(b) => {
            app_config.notes_config.show_editor_on_start = b;
            return Task::done(Message::SaveConfig);
        }
        SettingsPageMessage::NotesSetShowMarkdownOnStart(b) => {
            app_config.notes_config.show_markdown_on_start = b;
            return Task::done(Message::SaveConfig);
        }
        SettingsPageMessage::PasswordsSetDefaultDatabase => {
            let selected_file = FileDialog::new()
                .add_filter("keepass", &["kdbx"])
                .pick_file();
            app_config.passwords_config.default_database = selected_file;
            return Task::done(Message::SaveConfig);
        }
        SettingsPageMessage::PasswordsSetShowSidebarOnStart(b) => {
            app_config.passwords_config.show_sidebar_on_start = b;
            return Task::done(Message::SaveConfig);
        }
    }
    Task::none()
}
