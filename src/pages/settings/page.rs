use iced::{Element, Task, Theme};

use crate::app::Message;
use crate::config::AppConfig;

use super::update::update;
use super::view::{main_view, tool_view};

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsTab {
    General,
    Sync,
    FileManager,
    Gallery,
    Passwords,
    Notes,
    Tasks,
}

impl SettingsTab {
    pub fn get_all() -> [SettingsTab; 7] {
        [
            SettingsTab::General,
            SettingsTab::Sync,
            SettingsTab::FileManager,
            SettingsTab::Gallery,
            SettingsTab::Passwords,
            SettingsTab::Notes,
            SettingsTab::Tasks,
        ]
    }
    pub fn name(&self) -> &'static str {
        match self {
            SettingsTab::General => "General",
            SettingsTab::Sync => "Sync",
            SettingsTab::FileManager => "File Manager",
            SettingsTab::Gallery => "Gallery",
            SettingsTab::Passwords => "Passwords",
            SettingsTab::Notes => "Notes",
            SettingsTab::Tasks => "Tasks",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SettingsPage {
    pub(crate) is_saving: bool,
    pub(crate) save_was_successful: bool,
    pub(crate) save_message: String,
    pub(crate) current_tab: SettingsTab,
}

#[derive(Debug, Clone)]
pub enum SettingsPageMessage {
    SetTheme(Theme),
    ChangeTab(SettingsTab),
    StartSaving,
    ResultFromSave((bool, String)),
    SetDefaultPageOnOpen(&'static str),
    NotesSetDefaultFolder,
    NotesSetShowSidebarOnStart(bool),
    NotesSetShowEditorOnStart(bool),
    NotesSetShowMarkdownOnStart(bool),
    PasswordsSetDefaultDatabase,
    PasswordsSetShowSidebarOnStart(bool),
}

impl SettingsPage {
    pub fn new() -> Self {
        Self {
            is_saving: false,
            save_was_successful: true,
            current_tab: SettingsTab::General,
            save_message: String::from("Settings saved"),
        }
    }

    pub fn closing_task(&mut self) -> Task<Message> {
        println!("Closing task from settings");
        Task::none()
    }

    pub fn update(
        &mut self,
        message: SettingsPageMessage,
        app_config: &mut AppConfig,
    ) -> Task<Message> {
        update(self, message, app_config)
    }

    pub fn view<'a>(&'a self, app_config: &'a AppConfig) -> Element<'a, Message> {
        main_view(self, app_config)
    }

    pub fn tool_view(&self) -> Element<Message> {
        tool_view(self)
    }
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self::new()
    }
}
