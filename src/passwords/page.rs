use std::path::PathBuf;

use iced::{Element, Task};

use crate::Message;

use super::passwords_utils::save_database;
use super::update::update;
use super::view::{tool_view, view};

#[derive(Debug, Clone)]
pub struct Password {
    pub id: uuid::Uuid,
    pub title: String,
    pub username: String,
    pub url: String,
    pub password: String,
}

pub struct PasswordsPage {
    pub(super) is_unlocked: bool,
    pub(super) incorrect_password_entered: bool,
    pub(super) passwords_list: Vec<Password>,
    pub(super) keepass_file_option: Option<PathBuf>,
    pub(super) master_password_field_text: String,
    pub(super) current_title_text: String,
    pub(super) current_url_text: String,
    pub(super) current_username_text: String,
    pub(super) current_password_text: String,
    pub(super) selected_password: Option<Password>,
    pub(super) current_filter: String,
    pub(super) is_dirty: bool,
    pub(super) show_sidebar: bool,
    pub(super) hide_master_password_entry: bool,
    pub(super) hide_current_password_entry: bool,
    pub(super) creating_new_keepass_file: bool,
    pub(super) hide_master_password_reentry_entry: bool,
    pub(super) passwords_dont_match: bool,
    pub(super) master_password_reentry_field_text: String,
    pub(super) key_file_option: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum PasswordsPageMessage {
    UpdatePasswordEntry(),
    DeletePasswordEntry(uuid::Uuid),
    TryUnlock,
    Lock,
    SelectPassword(Option<Password>),
    UpdateMasterPasswordField(String),
    UpdateCurrentTitleText(String),
    UpdateCurrentUrlText(String),
    UpdateCurrentUsernameText(String),
    UpdateCurrentPasswordText(String),
    FilterPasswords(String),
    SaveDatabase,
    ToggleSidebar,
    ToggleHideMasterPassword,
    ToggleHideCurrentPassword,
    CopyValue(String),
    PickDatabaseFile,
    StartCreatingNewKeepassFile,
    PickNewDatabasePath,
    UpdateMasterPasswordReentryField(String),
    ToggleHideMasterPasswordReentry,
    CreateDatabase,
    CloseDatabase,
    PickKeyFile,
    ResetView,
    GeneratePassword,
}

impl PasswordsPage {
    pub fn new() -> Self {
        Self {
            passwords_list: vec![],
            keepass_file_option: None,
            is_unlocked: false,
            incorrect_password_entered: false,
            master_password_field_text: String::new(),
            selected_password: None,
            current_title_text: String::new(),
            current_url_text: String::new(),
            current_username_text: String::new(),
            current_password_text: String::new(),
            current_filter: String::new(),
            is_dirty: false,
            show_sidebar: true,
            hide_master_password_entry: true,
            hide_current_password_entry: true,
            creating_new_keepass_file: false,
            hide_master_password_reentry_entry: true,
            passwords_dont_match: false,
            master_password_reentry_field_text: String::new(),
            key_file_option: None,
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
                    self.keepass_file_option.clone(),
                    password,
                    self.key_file_option.clone(),
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
        view(self)
    }

    pub fn tool_view(&self) -> Element<Message> {
        tool_view(self)
    }
}

impl Default for PasswordsPage {
    fn default() -> Self {
        Self::new()
    }
}
