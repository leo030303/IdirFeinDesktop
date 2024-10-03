use arboard::Clipboard;
use iced::Task;
use rand::{thread_rng, Rng};
use rfd::FileDialog;

use crate::app::Message;

use super::{
    page::{Password, PasswordsPage, PasswordsPageMessage},
    passwords_utils::{get_passwords, save_database},
};

pub fn update(state: &mut PasswordsPage, message: PasswordsPageMessage) -> Task<Message> {
    match message {
        PasswordsPageMessage::UpdatePasswordEntry => {
            state.is_dirty = true;
            if let Some(selected_password) = &state.selected_password_entry {
                if let Some(password_index) = state
                    .passwords_list
                    .iter()
                    .position(|x| x.id == selected_password.id)
                {
                    state.passwords_list[password_index] = Password {
                        id: selected_password.id,
                        title: state.current_title_text.clone(),
                        username: state.current_username_text.clone(),
                        url: state.current_url_text.clone(),
                        password: state.current_password_text.clone(),
                    };
                }
            } else {
                state.passwords_list.push(Password {
                    id: uuid::Uuid::new_v4(),
                    title: state.current_title_text.clone(),
                    username: state.current_username_text.clone(),
                    url: state.current_url_text.clone(),
                    password: state.current_password_text.clone(),
                });
                state.current_title_text = String::new();
                state.current_url_text = String::new();
                state.current_username_text = String::new();
                state.current_password_text = String::new();
            }
        }
        PasswordsPageMessage::DeletePasswordEntry(id_to_delete) => {
            if let Some(password_index) = state
                .passwords_list
                .iter()
                .position(|x| x.id == id_to_delete)
            {
                state.passwords_list.remove(password_index);
                state.is_dirty = true;
                state.selected_password_entry = None;
            }
        }
        PasswordsPageMessage::TryUnlock => {
            if let Some(keepass_file_path) = state.selected_keepass_file.clone() {
                let password = if state.master_password_field_text.is_empty() {
                    None
                } else {
                    Some(state.master_password_field_text.clone())
                };
                return Task::perform(
                    get_passwords(keepass_file_path, password, state.selected_key_file.clone()),
                    |passwords_list_option| {
                        Message::Passwords(PasswordsPageMessage::RetrievedPasswordsList(
                            passwords_list_option,
                        ))
                    },
                );
            } else {
                state.passwords_list = vec![];
                state.is_unlocked = false;
            };
        }
        PasswordsPageMessage::RetrievedPasswordsList(passwords_list_option) => {
            if let Some(passwords_list) = passwords_list_option {
                state.is_unlocked = true;
                state.passwords_list = passwords_list;
                state.incorrect_password_entered = false;
            } else {
                state.passwords_list = vec![];
                state.is_unlocked = false;
                state.incorrect_password_entered = true;
            }
        }
        PasswordsPageMessage::UpdateMasterPasswordField(s) => state.master_password_field_text = s,
        PasswordsPageMessage::SelectPassword(password) => {
            state.selected_password_entry = password.clone();
            state.current_title_text = password
                .clone()
                .map_or(String::new(), |password| password.title);
            state.current_url_text = password
                .clone()
                .map_or(String::new(), |password| password.url);
            state.current_username_text = password
                .clone()
                .map_or(String::new(), |password| password.username);
            state.current_password_text =
                password.map_or(String::new(), |password| password.password);
        }
        PasswordsPageMessage::UpdateCurrentTitleText(s) => state.current_title_text = s,
        PasswordsPageMessage::UpdateCurrentUrlText(s) => state.current_url_text = s,
        PasswordsPageMessage::UpdateCurrentUsernameText(s) => state.current_username_text = s,
        PasswordsPageMessage::UpdateCurrentPasswordText(s) => state.current_password_text = s,
        PasswordsPageMessage::UpdatePasswordsFilter(filter) => {
            state.current_passwords_list_filter = filter
        }
        PasswordsPageMessage::SaveDatabaseToFile => {
            state.is_dirty = false;
            let password = if state.master_password_field_text.is_empty() {
                None
            } else {
                Some(state.master_password_field_text.clone())
            };
            return Task::perform(
                save_database(
                    state.selected_keepass_file.clone(),
                    password,
                    state.selected_key_file.clone(),
                    state.passwords_list.clone(),
                ),
                |(is_success, content)| Message::ShowToast(is_success, content),
            );
        }
        PasswordsPageMessage::Lock => {
            state.is_unlocked = false;
            if state.is_dirty {
                let master_password_field_text = state.master_password_field_text.clone();
                let key_file_option = state.selected_key_file.clone();
                state.master_password_field_text = String::new();
                state.selected_key_file = None;
                let password = if master_password_field_text.is_empty() {
                    None
                } else {
                    Some(master_password_field_text)
                };
                state.is_dirty = false;
                return Task::perform(
                    save_database(
                        state.selected_keepass_file.clone(),
                        password,
                        key_file_option,
                        state.passwords_list.clone(),
                    ),
                    |_| Message::None,
                );
            }
        }
        PasswordsPageMessage::ToggleShowSidebar => state.show_sidebar = !state.show_sidebar,
        PasswordsPageMessage::ToggleHideMasterPassword => {
            state.hide_master_password_entry = !state.hide_master_password_entry
        }
        PasswordsPageMessage::ToggleHideCurrentPassword => {
            state.hide_current_password_entry = !state.hide_current_password_entry
        }
        PasswordsPageMessage::CopyValueToClipboard(s) => {
            Clipboard::new().unwrap().set_text(s).unwrap()
        }
        PasswordsPageMessage::PickDatabaseFile => {
            return Task::perform(
                async {
                    FileDialog::new()
                        .add_filter("keepass", &["kdbx"])
                        .pick_file()
                },
                |selected_file| {
                    Message::Passwords(PasswordsPageMessage::SetDatabaseFile(selected_file))
                },
            );
        }
        PasswordsPageMessage::SetDatabaseFile(selected_file) => {
            state.selected_keepass_file = selected_file;
        }
        PasswordsPageMessage::StartCreatingNewKeepassFile => {
            state.is_creating_new_keepass_file = true
        }
        PasswordsPageMessage::PickNewDatabasePath => {
            return Task::perform(
                async {
                    FileDialog::new()
                        .add_filter("keepass", &["kdbx"])
                        .save_file()
                },
                |selected_file| {
                    Message::Passwords(PasswordsPageMessage::SetDatabaseFile(selected_file))
                },
            );
        }
        PasswordsPageMessage::UpdateMasterPasswordReentryField(s) => {
            state.master_password_reentry_field_text = s
        }
        PasswordsPageMessage::ToggleHideMasterPasswordReentry => {
            state.hide_master_password_reentry_entry = !state.hide_master_password_reentry_entry
        }
        PasswordsPageMessage::CreateDatabase => {
            if (!state.master_password_field_text.is_empty()
                && state.master_password_field_text == state.master_password_reentry_field_text)
                || state.selected_key_file.is_some()
            {
                state.is_unlocked = true;
                state.passwords_dont_match = false;
                state.is_creating_new_keepass_file = false;
            } else if state.master_password_field_text != state.master_password_reentry_field_text {
                state.passwords_dont_match = true;
            }
        }
        PasswordsPageMessage::CloseDatabase => state.selected_keepass_file = None,
        PasswordsPageMessage::PickKeyFile => {
            return Task::perform(async { FileDialog::new().pick_file() }, |selected_file| {
                Message::Passwords(PasswordsPageMessage::SetKeyFile(selected_file))
            });
        }
        PasswordsPageMessage::SetKeyFile(selected_file) => {
            state.selected_key_file = selected_file;
        }
        PasswordsPageMessage::LockAndDeselectDatabase => {
            state.selected_keepass_file = None;
            state.is_unlocked = false;
            state.is_creating_new_keepass_file = false;
        }
        PasswordsPageMessage::GeneratePassword => {
            let mut rng = thread_rng();
            state.current_password_text = (0..30)
                .map(|_| rng.gen_range(33u8..127u8) as char) // ASCII range for printable characters
                .collect();
        }
    }
    Task::none()
}
