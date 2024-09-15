use std::path::PathBuf;

use iced::{Element, Task};
use serde::{Deserialize, Serialize};

use crate::app::Message;

use super::passwords_utils::save_database;
use super::update::update;
use super::view::{main_view, tool_view};

/**
CODE FINISHED, TODO:
Search "unwrap" and replace with error handling
Search "clone" and replace with reference where possible
Comment the code where necessary
Create tests for each update function and util function
*/

#[derive(Debug, Clone)]
pub struct Password {
    pub id: uuid::Uuid,
    pub title: String,
    pub username: String,
    pub url: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPageConfig {
    pub default_database: Option<PathBuf>,
    pub show_sidebar_on_start: bool,
}

impl Default for PasswordPageConfig {
    fn default() -> Self {
        Self {
            default_database: None,
            show_sidebar_on_start: true,
        }
    }
}

pub struct PasswordsPage {
    pub(super) is_unlocked: bool,
    pub(super) is_dirty: bool,
    pub(super) is_creating_new_keepass_file: bool,
    pub(super) show_sidebar: bool,
    pub(super) incorrect_password_entered: bool,
    pub(super) passwords_list: Vec<Password>,
    pub(super) selected_keepass_file: Option<PathBuf>,
    pub(super) selected_key_file: Option<PathBuf>,
    pub(super) master_password_field_text: String,
    pub(super) current_title_text: String,
    pub(super) current_url_text: String,
    pub(super) current_username_text: String,
    pub(super) current_password_text: String,
    pub(super) master_password_reentry_field_text: String,
    pub(super) current_passwords_list_filter: String,
    pub(super) selected_password_entry: Option<Password>,
    pub(super) hide_master_password_entry: bool,
    pub(super) hide_current_password_entry: bool,
    pub(super) hide_master_password_reentry_entry: bool,
    pub(super) passwords_dont_match: bool,
}

#[derive(Debug, Clone)]
pub enum PasswordsPageMessage {
    UpdatePasswordEntry,
    DeletePasswordEntry(uuid::Uuid),
    TryUnlock,
    Lock,
    RetrievedPasswordsList(Option<Vec<Password>>),
    SelectPassword(Option<Password>),
    UpdateMasterPasswordField(String),
    UpdateCurrentTitleText(String),
    UpdateCurrentUrlText(String),
    UpdateCurrentUsernameText(String),
    UpdateCurrentPasswordText(String),
    UpdateMasterPasswordReentryField(String),
    UpdatePasswordsFilter(String),
    SaveDatabaseToFile,
    ToggleShowSidebar,
    ToggleHideCurrentPassword,
    ToggleHideMasterPassword,
    ToggleHideMasterPasswordReentry,
    CopyValueToClipboard(String),
    PickDatabaseFile,
    StartCreatingNewKeepassFile,
    PickNewDatabasePath,
    CreateDatabase,
    CloseDatabase,
    PickKeyFile,
    LockAndDeselectDatabase,
    GeneratePassword,
}

impl PasswordsPage {
    pub fn new(config: &PasswordPageConfig) -> Self {
        Self {
            passwords_list: vec![],
            selected_keepass_file: config.default_database.clone(),
            is_unlocked: false,
            incorrect_password_entered: false,
            master_password_field_text: String::new(),
            selected_password_entry: None,
            current_title_text: String::new(),
            current_url_text: String::new(),
            current_username_text: String::new(),
            current_password_text: String::new(),
            current_passwords_list_filter: String::new(),
            is_dirty: false,
            show_sidebar: config.show_sidebar_on_start,
            hide_master_password_entry: true,
            hide_current_password_entry: true,
            is_creating_new_keepass_file: false,
            hide_master_password_reentry_entry: true,
            passwords_dont_match: false,
            master_password_reentry_field_text: String::new(),
            selected_key_file: None,
        }
    }

    pub fn closing_task(&mut self) -> Task<Message> {
        if self.is_dirty {
            let password = if self.master_password_field_text.is_empty() {
                None
            } else {
                Some(self.master_password_field_text.clone())
            };
            Task::perform(
                save_database(
                    self.selected_keepass_file.clone(),
                    password,
                    self.selected_key_file.clone(),
                    self.passwords_list.clone(),
                ),
                |_| Message::None,
            )
        } else {
            Task::none()
        }
    }

    pub fn update(&mut self, message: PasswordsPageMessage) -> Task<Message> {
        update(self, message)
    }

    pub fn view(&self) -> Element<Message> {
        main_view(self)
    }

    pub fn tool_view(&self) -> Element<Message> {
        tool_view(self)
    }
}
