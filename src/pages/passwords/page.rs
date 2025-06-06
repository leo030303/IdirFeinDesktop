use std::path::PathBuf;

use iced::event::{self, Status};
use iced::keyboard::{Key, Modifiers};
use iced::{keyboard, Element, Event, Task};
use serde::{Deserialize, Serialize};

use crate::app::Message;

use super::passwords_utils::save_database;
use super::update::update;
use super::view::{main_view, tool_view};

#[derive(Debug, Clone)]
pub struct Password {
    /// Unique identifier for the password
    pub id: uuid::Uuid,
    pub title: String,
    pub username: String,
    /// URL the entry is for, if it's for a website
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
    /// The current language locale ID
    pub(crate) locale: fluent_templates::LanguageIdentifier,
    /// Whether the database is unlocked
    pub(super) is_unlocked: bool,
    /// Whether the database has been modified but not saved
    pub(super) is_dirty: bool,
    /// Whether the UI to create a new database should be shown
    pub(super) is_creating_new_keepass_file: bool,
    /// Whether to show the sidebar UI
    pub(super) show_sidebar: bool,
    /// Whether the incorrect password was entered
    pub(super) incorrect_password_entered: bool,
    /// The list of passwords from the database thats opened
    pub(super) passwords_list: Vec<Password>,
    /// The path to the database file currently open, if any
    pub(super) selected_keepass_file: Option<PathBuf>,
    /// The path to the file being use as a keyfile, if any
    pub(super) selected_key_file: Option<PathBuf>,
    /// The content of the Master Password text field
    pub(super) master_password_field_text: String,
    /// The content of the Title text field
    pub(super) current_title_text: String,
    /// The content of the URL text field
    pub(super) current_url_text: String,
    /// The content of the Username text field
    pub(super) current_username_text: String,
    /// The content of the Password text field
    pub(super) current_password_text: String,
    /// The content of the Master Password Reentry text field
    pub(super) master_password_reentry_field_text: String,
    /// The string to filter the titles of the passwords list by, if "" no filtering is done
    pub(super) current_passwords_list_filter: String,
    /// The entry selected to display/edit, if any
    pub(super) selected_password_entry: Option<Password>,
    /// Whether to hide the contents of the master password field
    pub(super) hide_master_password_entry: bool,
    /// Whether to hide the contents of the password editing field
    pub(super) hide_current_password_entry: bool,
    /// Whether to hide the contents of the master password reentry field
    pub(super) hide_master_password_reentry_entry: bool,
    /// Whether the master password fields text and its reentry fields text don't match
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
    PickDatabaseFile,
    SetDatabaseFile(Option<PathBuf>),
    StartCreatingNewKeepassFile,
    PickNewDatabasePath,
    CreateDatabase,
    CloseDatabase,
    PickKeyFile,
    SetKeyFile(Option<PathBuf>),
    LockAndDeselectDatabase,
    GeneratePassword,
}

impl PasswordsPage {
    pub fn new(config: &PasswordPageConfig) -> Self {
        let locale: fluent_templates::LanguageIdentifier = current_locale::current_locale()
            .expect("Can't get locale")
            .parse()
            .expect("Failed to parse locale");
        Self {
            locale,
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

    pub fn opening_task() -> Task<Message> {
        Task::none()
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

    pub fn subscription() -> iced::Subscription<Message> {
        // Keyboard shortcuts
        event::listen_with(|event, status, _id| match (event, status) {
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: Key::Character(pressed_char),
                    modifiers: Modifiers::CTRL,
                    ..
                }),
                Status::Ignored,
            ) => {
                if pressed_char.as_ref() == "n" || pressed_char.as_ref() == "N" {
                    Some(Message::Passwords(PasswordsPageMessage::SelectPassword(
                        None,
                    )))
                } else if pressed_char.as_ref() == "b" || pressed_char.as_ref() == "B" {
                    Some(Message::Passwords(PasswordsPageMessage::ToggleShowSidebar))
                } else if pressed_char.as_ref() == "l" || pressed_char.as_ref() == "L" {
                    Some(Message::Passwords(
                        PasswordsPageMessage::LockAndDeselectDatabase,
                    ))
                } else {
                    None
                }
            }
            _ => None,
        })
    }

    pub fn tool_view(&self) -> Element<Message> {
        tool_view(self)
    }
}
