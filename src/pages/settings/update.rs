use iced::Task;
use rfd::FileDialog;

use crate::config::AppConfig;
use crate::pages::gallery::page::GalleryPageMessage;
use crate::pages::notes::page::NotesPageMessage;
use crate::pages::passwords::page::PasswordsPageMessage;
use crate::pages::tasks::page::TasksPageMessage;
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
            return Task::none();
        }
        SettingsPageMessage::StartSaving => {
            state.is_saving = true;
            return Task::none();
        }
        SettingsPageMessage::ChangeTab(tab) => {
            state.current_tab = tab;
            return Task::none();
        }
        SettingsPageMessage::SetTheme(theme) => {
            app_config.set_theme(theme);
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
        }
        SettingsPageMessage::NotesPickDefaultFolder => {
            return Task::perform(
                async {
                    FileDialog::new()
                        .set_directory("/")
                        .set_can_create_directories(true)
                        .pick_folder()
                },
                |selected_folder| {
                    Message::Settings(SettingsPageMessage::NotesSetDefaultFolder(selected_folder))
                },
            );
        }
        SettingsPageMessage::NotesSetDefaultFolder(selected_folder) => {
            app_config.notes_config.default_folder = selected_folder.clone();
            return Task::done(Message::SaveConfig).chain(Task::done(Message::Notes(
                NotesPageMessage::SetNotesFolder(selected_folder),
            )));
        }
        SettingsPageMessage::NotesSetShowSidebarOnStart(b) => {
            app_config.notes_config.show_sidebar_on_start = b;
        }
        SettingsPageMessage::NotesSetShowEditorOnStart(b) => {
            app_config.notes_config.show_editor_on_start = b;
        }
        SettingsPageMessage::NotesSetShowMarkdownOnStart(b) => {
            app_config.notes_config.show_markdown_on_start = b;
        }
        SettingsPageMessage::NotesSetShowConfirmDelete(b) => {
            app_config.notes_config.confirm_before_delete = b;
            return Task::done(Message::SaveConfig).chain(Task::done(Message::Notes(
                NotesPageMessage::SetConfirmBeforeDelete(b),
            )));
        }
        SettingsPageMessage::NotesSetAutocompleteBrackets(b) => {
            app_config.notes_config.autocomplete_brackets_etc = b;
            return Task::done(Message::SaveConfig).chain(Task::done(Message::Notes(
                NotesPageMessage::SetAutocompleteBrackets(b),
            )));
        }
        SettingsPageMessage::NotesSetAutocompleteLists(b) => {
            app_config.notes_config.autocomplete_lists = b;
            return Task::done(Message::SaveConfig).chain(Task::done(Message::Notes(
                NotesPageMessage::SetAutoCompleteLists(b),
            )));
        }
        SettingsPageMessage::NotesPickWebsiteFolder => {
            return Task::perform(
                async {
                    FileDialog::new()
                        .set_directory("/")
                        .set_can_create_directories(true)
                        .pick_folder()
                },
                |selected_folder| {
                    Message::Settings(SettingsPageMessage::NotesSetWebsiteFolder(selected_folder))
                },
            );
        }
        SettingsPageMessage::NotesSetWebsiteFolder(selected_folder) => {
            app_config.notes_config.website_folder = selected_folder.clone();
            return Task::done(Message::SaveConfig).chain(Task::done(Message::Notes(
                NotesPageMessage::SetWebsiteFolder(selected_folder),
            )));
        }
        SettingsPageMessage::PasswordsPickDefaultDatabase => {
            return Task::perform(
                async {
                    FileDialog::new()
                        .add_filter("keepass", &["kdbx"])
                        .pick_file()
                },
                |selected_file| {
                    Message::Settings(SettingsPageMessage::PasswordsSetDefaultDatabase(
                        selected_file,
                    ))
                },
            );
        }
        SettingsPageMessage::PasswordsSetDefaultDatabase(selected_file) => {
            app_config.passwords_config.default_database = selected_file.clone();
            return Task::done(Message::SaveConfig).chain(Task::done(Message::Passwords(
                PasswordsPageMessage::SetDatabaseFile(selected_file),
            )));
        }
        SettingsPageMessage::PasswordsSetShowSidebarOnStart(b) => {
            app_config.passwords_config.show_sidebar_on_start = b;
        }
        SettingsPageMessage::TasksPickDefaultProjectFolder => {
            return Task::perform(
                async {
                    FileDialog::new()
                        .set_directory("/")
                        .set_can_create_directories(true)
                        .pick_folder()
                },
                |selected_folder| {
                    Message::Settings(SettingsPageMessage::TasksSetDefaultProjectFolder(
                        selected_folder,
                    ))
                },
            );
        }
        SettingsPageMessage::TasksSetDefaultProjectFolder(selected_folder) => {
            app_config.tasks_config.default_folder = selected_folder.clone();
            return Task::done(Message::SaveConfig).chain(Task::done(Message::Tasks(
                TasksPageMessage::SetProjectsFolder(selected_folder),
            )));
        }
        SettingsPageMessage::TasksSetKanbanTaskViewIsDefault(b) => {
            app_config.tasks_config.kanban_task_view_is_default = b
        }
        SettingsPageMessage::TasksSetShowSidebarOnStart(b) => {
            app_config.tasks_config.show_sidebar_on_start = b
        }
        SettingsPageMessage::TasksSetConfirmBeforeDelete(b) => {
            app_config.tasks_config.confirm_before_delete = b;
            return Task::done(Message::SaveConfig).chain(Task::done(Message::Tasks(
                TasksPageMessage::SetConfirmBeforeDelete(b),
            )));
        }
        SettingsPageMessage::TasksSetShowTaskCompletionToolbar(b) => {
            app_config.tasks_config.show_task_completion_toolbar = b;
            return Task::done(Message::SaveConfig).chain(Task::done(Message::Tasks(
                TasksPageMessage::SetShowTaskCompletionToolbar(b),
            )));
        }
        SettingsPageMessage::TasksSetRightClickToEditTask(b) => {
            app_config.tasks_config.right_click_to_edit_task = b;
            return Task::done(Message::SaveConfig).chain(Task::done(Message::Tasks(
                TasksPageMessage::SetRightClickToEditTask(b),
            )));
        }
        SettingsPageMessage::GalleryPickDefaultFolder => {
            return Task::perform(
                async {
                    FileDialog::new()
                        .set_directory("/")
                        .set_can_create_directories(true)
                        .pick_folder()
                },
                |selected_folder| {
                    Message::Settings(SettingsPageMessage::GallerySetDefaultFolder(
                        selected_folder,
                    ))
                },
            );
        }
        SettingsPageMessage::GallerySetDefaultFolder(selected_folder) => {
            app_config.gallery_config.default_folder = selected_folder.clone();
            return Task::done(Message::SaveConfig).chain(Task::done(Message::Gallery(
                GalleryPageMessage::SetGalleryFolder(selected_folder),
            )));
        }
    }
    Task::done(Message::SaveConfig)
}
