use std::path::PathBuf;

use iced::{Element, Task, Theme};

use crate::app::Message;
use crate::config::AppConfig;

use super::update::update;
use super::view::{main_view, tool_view};

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsTab {
    General,
    Sync,
    Gallery,
    Passwords,
    Notes,
    Tasks,
}

impl SettingsTab {
    pub fn get_all() -> [SettingsTab; 6] {
        [
            SettingsTab::General,
            SettingsTab::Sync,
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
            SettingsTab::Gallery => "Gallery",
            SettingsTab::Passwords => "Passwords",
            SettingsTab::Notes => "Notes",
            SettingsTab::Tasks => "Tasks",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SettingsPage {
    pub(crate) locale: fluent_templates::LanguageIdentifier,
    pub(crate) is_saving: bool,
    pub(crate) save_was_successful: bool,
    pub(crate) save_message: String,
    pub(crate) current_tab: SettingsTab,
    pub(crate) server_url_editor_text: String,
}

#[derive(Debug, Clone)]
pub enum SettingsPageMessage {
    SetTheme(Theme),
    ChangeTab(SettingsTab),
    StartSaving,
    ResultFromSave((bool, String)),
    SetDefaultPageOnOpen(&'static str),
    NotesPickDefaultFolder,
    NotesSetDefaultFolder(Option<PathBuf>),
    NotesSetShowSidebarOnStart(bool),
    NotesSetShowEditorOnStart(bool),
    NotesSetShowMarkdownOnStart(bool),
    NotesSetShowConfirmDelete(bool),
    NotesSetAutocompleteBrackets(bool),
    NotesSetAutocompleteLists(bool),
    PasswordsPickDefaultDatabase,
    PasswordsSetDefaultDatabase(Option<PathBuf>),
    PasswordsSetShowSidebarOnStart(bool),
    TasksPickDefaultProjectFolder,
    TasksSetDefaultProjectFolder(Option<PathBuf>),
    TasksPickDefaultProjectFile,
    TasksSetDefaultProjectFile(Option<PathBuf>),
    TasksSetKanbanTaskViewIsDefault(bool),
    TasksSetShowSidebarOnStart(bool),
    TasksSetConfirmBeforeDelete(bool),
    TasksSetRightClickToEditTask(bool),
    TasksSetShowTaskCompletionToolbar(bool),
    GalleryPickDefaultFolder,
    GallerySetDefaultFolder(Option<PathBuf>),
    GallerySetRunThumbnailGenerationOnStart(bool),
    GallerySetRunFaceExtractionOnStart(bool),
    GallerySetRunFaceRecognitionOnStart(bool),
    SyncUpdateServerUrl(String),
    SyncSetServerUrl,
    SyncPickDefaultFolder,
    SyncSetDefaultFolder(Option<PathBuf>),
    SyncSetShouldSync(bool),
}

impl SettingsPage {
    pub fn new(app_config: &AppConfig) -> Self {
        let locale: fluent_templates::LanguageIdentifier = current_locale::current_locale()
            .expect("Can't get locale")
            .parse()
            .expect("Failed to parse locale");
        Self {
            locale,
            is_saving: false,
            save_was_successful: true,
            current_tab: SettingsTab::General,
            save_message: String::from("Settings saved"),
            server_url_editor_text: app_config.sync_config.server_url.clone(),
        }
    }

    pub fn opening_task() -> Task<Message> {
        Task::none()
    }

    pub fn closing_task(&mut self) -> Task<Message> {
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
