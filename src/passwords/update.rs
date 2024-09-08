use arboard::Clipboard;
use iced::Task;
use rand::{thread_rng, Rng};
use rfd::FileDialog;

use crate::Message;

use super::{
    page::{Password, PasswordsPage, PasswordsPageMessage},
    passwords_utils::{get_passwords, save_database},
};

pub fn update(state: &mut PasswordsPage, message: PasswordsPageMessage) -> Task<Message> {
    match message {
        PasswordsPageMessage::UpdatePasswordEntry() => {
            state.is_dirty = true;
            if let Some(selected_password) = &state.selected_password {
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
                state.selected_password = None;
            }
        }
        PasswordsPageMessage::TryUnlock => {
            if let Some(keepass_file_path) = state.keepass_file_option.clone() {
                let password = if state.master_password_field_text.is_empty() {
                    None
                } else {
                    Some(state.master_password_field_text.as_str())
                };
                if let Some(passwords_list) =
                    get_passwords(keepass_file_path, password, state.key_file_option.clone())
                {
                    state.is_unlocked = true;
                    state.passwords_list = passwords_list;
                    state.incorrect_password_entered = false;
                } else {
                    state.passwords_list = vec![];
                    state.is_unlocked = false;
                    state.incorrect_password_entered = true;
                }
            } else {
                state.passwords_list = vec![];
                state.is_unlocked = false;
            };
        }
        PasswordsPageMessage::UpdateMasterPasswordField(s) => state.master_password_field_text = s,
        PasswordsPageMessage::SelectPassword(password) => {
            state.selected_password = password.clone();
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
        PasswordsPageMessage::FilterPasswords(filter) => state.current_filter = filter,
        PasswordsPageMessage::SaveDatabase => {
            state.is_dirty = false;
            let password = if state.master_password_field_text.is_empty() {
                None
            } else {
                Some(state.master_password_field_text.clone())
            };
            return Task::perform(
                save_database(
                    state.keepass_file_option.clone(),
                    password,
                    state.key_file_option.clone(),
                    state.passwords_list.clone(),
                ),
                |_| Message::None,
            );
        }
        PasswordsPageMessage::Lock => {
            state.is_unlocked = false;
            if state.is_dirty {
                let master_password_field_text = state.master_password_field_text.clone();
                let key_file_option = state.key_file_option.clone();
                state.master_password_field_text = String::new();
                state.key_file_option = None;
                let password = if master_password_field_text.is_empty() {
                    None
                } else {
                    Some(master_password_field_text)
                };
                state.is_dirty = false;
                return Task::perform(
                    save_database(
                        state.keepass_file_option.clone(),
                        password,
                        key_file_option,
                        state.passwords_list.clone(),
                    ),
                    |_| Message::None,
                );
            }
        }
        PasswordsPageMessage::ToggleSidebar => state.show_sidebar = !state.show_sidebar,
        PasswordsPageMessage::ToggleHideMasterPassword => {
            state.hide_master_password_entry = !state.hide_master_password_entry
        }
        PasswordsPageMessage::ToggleHideCurrentPassword => {
            state.hide_current_password_entry = !state.hide_current_password_entry
        }
        PasswordsPageMessage::CopyValue(s) => Clipboard::new().unwrap().set_text(s).unwrap(),
        PasswordsPageMessage::PickDatabaseFile => {
            let selected_file = FileDialog::new()
                .add_filter("keepass", &["kdbx"])
                .pick_file();
            state.keepass_file_option = selected_file;
        }
        PasswordsPageMessage::StartCreatingNewKeepassFile => state.creating_new_keepass_file = true,
        PasswordsPageMessage::PickNewDatabasePath => {
            let selected_file = FileDialog::new()
                .add_filter("keepass", &["kdbx"])
                .save_file();
            state.keepass_file_option = selected_file;
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
                || state.key_file_option.is_some()
            {
                state.is_unlocked = true;
                state.passwords_dont_match = false;
                state.creating_new_keepass_file = false;
            } else if state.master_password_field_text != state.master_password_reentry_field_text {
                state.passwords_dont_match = true;
            }
        }
        PasswordsPageMessage::CloseDatabase => state.keepass_file_option = None,
        PasswordsPageMessage::PickKeyFile => {
            let selected_file = FileDialog::new().pick_file();
            state.key_file_option = selected_file;
        }
        PasswordsPageMessage::ResetView => {
            state.keepass_file_option = None;
            state.is_unlocked = false;
            state.creating_new_keepass_file = false;
        }
        PasswordsPageMessage::GeneratePassword => {
            let mut rng = thread_rng();
            state.current_password_text = (0..20)
                .map(|_| rng.gen_range(33u8..127u8) as char) // ASCII range for printable characters
                .collect();
        }
    }
    Task::none()
}
