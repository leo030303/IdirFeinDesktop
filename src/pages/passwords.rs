use arboard::Clipboard;
use rfd::FileDialog;
use std::path::PathBuf;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{button, column, container, row, text, text_input, Scrollable, Svg, Tooltip};
use iced::Element;
use iced::{Font, Length};

use crate::utils::passwords_handler::{get_passwords, save_database};
use crate::Message;

// TODO dialog on close of if you'd like to save the database if you haven't already
// TODO toast on database save/lock
// TODO allow edit of all database fields, database name, master password
// TODO support all entry fields, notes, expiry, tags
// TODO allow create new kdbx file
// TODO handle key files
// TODO handle groups
// TODO closed eye icon for password fields
// TODO code cleanup

#[derive(Debug, Clone)]
pub struct Password {
    pub id: uuid::Uuid,
    pub title: String,
    pub username: String,
    pub url: String,
    pub password: String,
}

pub struct PasswordsPage {
    is_unlocked: bool,
    incorrect_password_entered: bool,
    passwords_list: Vec<Password>,
    keepass_file_option: Option<PathBuf>,
    master_password_field_text: String,
    current_title_text: String,
    current_url_text: String,
    current_username_text: String,
    current_password_text: String,
    selected_password: Option<Password>,
    current_filter: String,
    is_dirty: bool,
    show_sidebar: bool,
    hide_master_password_entry: bool,
    hide_current_password_entry: bool,
}

#[derive(Debug, Clone)]
pub enum PasswordsPageMessage {
    UpdatePasswordEntry(),
    DeletePasswordEntry(uuid::Uuid),
    TryUnlock(String),
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
    OpenFilePicker,
}

impl PasswordsPage {
    pub fn new() -> Self {
        // let keepass_file_option: Option<PathBuf> =
        //     Some(PathBuf::from_str("/home/leoring/Downloads/keepass_copy.kdbx").unwrap());

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
        }
    }

    pub fn update(&mut self, message: PasswordsPageMessage) {
        match message {
            PasswordsPageMessage::UpdatePasswordEntry() => {
                self.is_dirty = true;
                if let Some(selected_password) = &self.selected_password {
                    if let Some(password_index) = self
                        .passwords_list
                        .iter()
                        .position(|x| x.id == selected_password.id)
                    {
                        self.passwords_list[password_index] = Password {
                            id: selected_password.id,
                            title: self.current_title_text.clone(),
                            username: self.current_username_text.clone(),
                            url: self.current_url_text.clone(),
                            password: self.current_password_text.clone(),
                        };
                    }
                } else {
                    self.passwords_list.push(Password {
                        id: uuid::Uuid::new_v4(),
                        title: self.current_title_text.clone(),
                        username: self.current_username_text.clone(),
                        url: self.current_url_text.clone(),
                        password: self.current_password_text.clone(),
                    });
                    self.current_title_text = String::new();
                    self.current_url_text = String::new();
                    self.current_username_text = String::new();
                    self.current_password_text = String::new();
                }
            }
            PasswordsPageMessage::DeletePasswordEntry(id_to_delete) => {
                if let Some(password_index) = self
                    .passwords_list
                    .iter()
                    .position(|x| x.id == id_to_delete)
                {
                    self.passwords_list.remove(password_index);
                    self.is_dirty = true;
                    self.selected_password = None;
                }
            }
            PasswordsPageMessage::TryUnlock(password_attempt) => {
                if let Some(keepass_file_path) = self.keepass_file_option.clone() {
                    if let Some(passwords_list) =
                        get_passwords(keepass_file_path, &password_attempt)
                    {
                        self.is_unlocked = true;
                        self.passwords_list = passwords_list;
                        self.incorrect_password_entered = false;
                    } else {
                        self.passwords_list = vec![];
                        self.is_unlocked = false;
                        self.incorrect_password_entered = true;
                    }
                } else {
                    self.passwords_list = vec![];
                    self.is_unlocked = false;
                };
            }
            PasswordsPageMessage::UpdateMasterPasswordField(s) => {
                self.master_password_field_text = s
            }
            PasswordsPageMessage::SelectPassword(password) => {
                self.selected_password = password.clone();
                self.current_title_text = password
                    .clone()
                    .map_or(String::new(), |password| password.title);
                self.current_url_text = password
                    .clone()
                    .map_or(String::new(), |password| password.url);
                self.current_username_text = password
                    .clone()
                    .map_or(String::new(), |password| password.username);
                self.current_password_text =
                    password.map_or(String::new(), |password| password.password);
            }
            PasswordsPageMessage::UpdateCurrentTitleText(s) => self.current_title_text = s,
            PasswordsPageMessage::UpdateCurrentUrlText(s) => self.current_url_text = s,
            PasswordsPageMessage::UpdateCurrentUsernameText(s) => self.current_username_text = s,
            PasswordsPageMessage::UpdateCurrentPasswordText(s) => self.current_password_text = s,
            PasswordsPageMessage::FilterPasswords(filter) => self.current_filter = filter,
            PasswordsPageMessage::SaveDatabase => {
                self.is_dirty = false;
                save_database(
                    self.keepass_file_option.clone().unwrap(),
                    &self.master_password_field_text,
                    self.passwords_list.clone(),
                )
            }
            PasswordsPageMessage::Lock => {
                self.is_unlocked = false;
                self.master_password_field_text = String::new();
            }
            PasswordsPageMessage::ToggleSidebar => self.show_sidebar = !self.show_sidebar,
            PasswordsPageMessage::ToggleHideMasterPassword => {
                self.hide_master_password_entry = !self.hide_master_password_entry
            }
            PasswordsPageMessage::ToggleHideCurrentPassword => {
                self.hide_current_password_entry = !self.hide_current_password_entry
            }
            PasswordsPageMessage::CopyValue(s) => Clipboard::new().unwrap().set_text(s).unwrap(),
            PasswordsPageMessage::OpenFilePicker => {
                let selected_file = FileDialog::new()
                    .add_filter("keepass", &["kdbx"])
                    .set_directory("/")
                    .pick_file();
                self.keepass_file_option = selected_file;
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let locked_view = if self.keepass_file_option.is_some() {
            column![
                container(
                    button(text(format!(
                        "Selected file: {}",
                        self.keepass_file_option
                            .clone()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                    )))
                    .on_press(Message::Passwords(PasswordsPageMessage::OpenFilePicker))
                )
                .width(Length::Fill)
                .align_x(Horizontal::Center),
                text("Password Vault is Locked")
                    .size(24)
                    .width(Length::Fill)
                    .align_x(Horizontal::Center),
                text("Enter the master password:")
                    .width(Length::Fill)
                    .align_x(Horizontal::Center),
                row![
                    text_input("Master Password", &self.master_password_field_text)
                        .secure(self.hide_master_password_entry)
                        .on_input(|s| Message::Passwords(
                            PasswordsPageMessage::UpdateMasterPasswordField(s)
                        ))
                        .on_submit(Message::Passwords(PasswordsPageMessage::TryUnlock(
                            self.master_password_field_text.clone()
                        )))
                        .width(Length::FillPortion(9)),
                    Tooltip::new(
                        button(Svg::from_path("icons/eye.svg").height(Length::Fill))
                            .on_press(Message::Passwords(
                                PasswordsPageMessage::ToggleHideMasterPassword
                            ))
                            .width(Length::FillPortion(1))
                            .height(Length::Fill)
                            .style(if self.hide_master_password_entry {
                                button::primary
                            } else {
                                button::secondary
                            }),
                        "Show/Hide",
                        iced::widget::tooltip::Position::Bottom,
                    )
                ]
                .height(Length::Shrink),
                text(if self.incorrect_password_entered {
                    "Incorrect master password, try again."
                } else {
                    ""
                })
                .width(Length::Fill)
                .align_x(Horizontal::Center),
            ]
            .spacing(10)
            .padding(10)
        } else {
            column![container(
                button("Select Keepass File")
                    .on_press(Message::Passwords(PasswordsPageMessage::OpenFilePicker))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)]
            .width(Length::Fill)
            .height(Length::Fill)
        };

        let unlocked_view = row![
            if self.show_sidebar {
                column![
                    text_input("Filter", &self.current_filter).on_input(|s| {
                        Message::Passwords(PasswordsPageMessage::FilterPasswords(s))
                    }),
                    Scrollable::new(column(
                        self.passwords_list
                            .iter()
                            .filter(|password| password
                                .title
                                .to_lowercase()
                                .contains(&self.current_filter.to_lowercase()))
                            .map(|password| {
                                button(
                                    text(if !password.title.is_empty() {
                                        &password.title
                                    } else {
                                        "<No Title>"
                                    })
                                    .font(Font {
                                        weight: iced::font::Weight::Semibold,
                                        ..Default::default()
                                    })
                                    .width(Length::Fill)
                                    .align_x(Horizontal::Center),
                                )
                                .on_press(Message::Passwords(PasswordsPageMessage::SelectPassword(
                                    Some(password.clone()),
                                )))
                                .style(if let Some(selected_password) = &self.selected_password {
                                    if selected_password.id == password.id {
                                        button::secondary
                                    } else {
                                        button::primary
                                    }
                                } else {
                                    button::primary
                                })
                                .width(Length::Fill)
                                .into()
                            })
                    ))
                    .direction(Direction::Vertical(Scrollbar::new()))
                    .height(Length::Fill)
                    .width(Length::FillPortion(1)),
                ]
                .spacing(5)
            } else {
                column![]
            },
            column![
                if self.selected_password.is_none() {
                    row![text("New Entry")
                        .size(24)
                        .align_x(Horizontal::Center)
                        .width(Length::Fill),]
                } else {
                    row![
                        text("Edit Entry")
                            .size(24)
                            .align_x(Horizontal::Center)
                            .width(Length::FillPortion(9)),
                        button(Svg::from_path("icons/delete.svg").height(Length::Fill))
                            .style(button::danger)
                            .height(Length::Fill)
                            .width(Length::FillPortion(1))
                            .on_press(Message::Passwords(
                                PasswordsPageMessage::DeletePasswordEntry(
                                    self.selected_password.clone().unwrap().id
                                )
                            ))
                    ]
                    .height(Length::Shrink)
                },
                text("Title: "),
                row![
                    text_input("Title", &self.current_title_text)
                        .on_input(|s| {
                            Message::Passwords(PasswordsPageMessage::UpdateCurrentTitleText(s))
                        })
                        .width(Length::FillPortion(9)),
                    Tooltip::new(
                        button(Svg::from_path("icons/copy.svg").height(Length::Fill))
                            .on_press(Message::Passwords(PasswordsPageMessage::CopyValue(
                                self.current_title_text.clone()
                            )))
                            .width(Length::FillPortion(1)),
                        "Copy",
                        iced::widget::tooltip::Position::Bottom,
                    )
                ]
                .height(Length::Shrink),
                text("Url: "),
                row![
                    text_input("Url", &self.current_url_text)
                        .on_input(|s| {
                            Message::Passwords(PasswordsPageMessage::UpdateCurrentUrlText(s))
                        })
                        .width(Length::FillPortion(9)),
                    Tooltip::new(
                        button(Svg::from_path("icons/copy.svg").height(Length::Fill))
                            .on_press(Message::Passwords(PasswordsPageMessage::CopyValue(
                                self.current_url_text.clone()
                            )))
                            .width(Length::FillPortion(1)),
                        "Copy",
                        iced::widget::tooltip::Position::Bottom,
                    )
                ]
                .height(Length::Shrink),
                text("Username: "),
                row![
                    text_input("Username", &self.current_username_text)
                        .on_input(|s| {
                            Message::Passwords(PasswordsPageMessage::UpdateCurrentUsernameText(s))
                        })
                        .width(Length::FillPortion(9)),
                    Tooltip::new(
                        button(Svg::from_path("icons/copy.svg").height(Length::Fill))
                            .on_press(Message::Passwords(PasswordsPageMessage::CopyValue(
                                self.current_username_text.clone()
                            )))
                            .width(Length::FillPortion(1)),
                        "Copy",
                        iced::widget::tooltip::Position::Bottom,
                    )
                ]
                .height(Length::Shrink),
                text("Password: "),
                row![
                    text_input("Password", &self.current_password_text)
                        .on_input(|s| {
                            Message::Passwords(PasswordsPageMessage::UpdateCurrentPasswordText(s))
                        })
                        .secure(self.hide_current_password_entry)
                        .width(Length::FillPortion(8)),
                    Tooltip::new(
                        button(Svg::from_path("icons/eye.svg").height(Length::Fill))
                            .on_press(Message::Passwords(
                                PasswordsPageMessage::ToggleHideCurrentPassword
                            ))
                            .width(Length::FillPortion(1))
                            .height(Length::Fill)
                            .style(if self.hide_current_password_entry {
                                button::primary
                            } else {
                                button::secondary
                            }),
                        "Show/Hide",
                        iced::widget::tooltip::Position::Bottom,
                    ),
                    Tooltip::new(
                        button(Svg::from_path("icons/copy.svg").height(Length::Fill))
                            .on_press(Message::Passwords(PasswordsPageMessage::CopyValue(
                                self.current_password_text.clone()
                            )))
                            .width(Length::FillPortion(1)),
                        "Copy",
                        iced::widget::tooltip::Position::Bottom,
                    )
                ]
                .height(Length::Shrink),
                button(if self.selected_password.is_none() {
                    "Add Entry"
                } else {
                    "Update Entry"
                })
                .on_press(Message::Passwords(
                    PasswordsPageMessage::UpdatePasswordEntry()
                ))
            ]
            .spacing(10)
            .padding(20)
            .height(Length::Fill)
            .width(Length::FillPortion(2))
        ];

        if self.is_unlocked {
            unlocked_view.into()
        } else {
            locked_view.into()
        }
    }

    pub fn tool_view(&self) -> Element<Message> {
        if self.is_unlocked {
            row![
                Tooltip::new(
                    button(Svg::from_path("icons/toggle-sidebar.svg"))
                        .on_press(Message::Passwords(PasswordsPageMessage::ToggleSidebar))
                        .style(if self.show_sidebar {
                            button::secondary
                        } else {
                            button::primary
                        }),
                    "Toggle Sidebar",
                    iced::widget::tooltip::Position::Bottom
                ),
                Tooltip::new(
                    button(Svg::from_path("icons/add.svg")).on_press(Message::Passwords(
                        PasswordsPageMessage::SelectPassword(None)
                    )),
                    "Add Entry",
                    iced::widget::tooltip::Position::Bottom
                ),
                Tooltip::new(
                    button(Svg::from_path("icons/lock.svg"))
                        .on_press(Message::Passwords(PasswordsPageMessage::Lock)),
                    "Lock",
                    iced::widget::tooltip::Position::Bottom
                ),
                Tooltip::new(
                    button(Svg::from_path("icons/ok.svg"))
                        .on_press(Message::Passwords(PasswordsPageMessage::SaveDatabase))
                        .style(if self.is_dirty {
                            button::success
                        } else {
                            button::secondary
                        }),
                    "Save Changes",
                    iced::widget::tooltip::Position::Bottom,
                )
            ]
            .width(Length::FillPortion(1))
            .into()
        } else {
            row![].width(Length::FillPortion(1)).into()
        }
    }
}

impl Default for PasswordsPage {
    fn default() -> Self {
        Self::new()
    }
}
