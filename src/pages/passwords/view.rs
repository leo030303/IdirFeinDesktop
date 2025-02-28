use fluent_templates::Loader;
use iced::Alignment::Center;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{
    button, column, container, row, svg, text, text_input, Scrollable, Space, Svg, Tooltip,
};
use iced::Element;
use iced::{Font, Length};

use crate::app::Message;
use crate::LOCALES;

use super::page::{PasswordsPage, PasswordsPageMessage};

pub fn main_view(state: &PasswordsPage) -> Element<Message> {
    if state.is_creating_new_keepass_file {
        // Creating new database
        if state.selected_keepass_file.is_none() {
            creating_new_database_choose_path_view(state)
        } else {
            new_database_set_password_view(state)
        }
    } else if state.is_unlocked {
        // Unlocked
        row![
            if state.show_sidebar {
                sidebar_view(state)
            } else {
                column![].into()
            },
            entry_edit_view(state)
        ]
        .into()
    } else {
        // Locked
        if state.selected_keepass_file.is_some() {
            existing_database_selected_and_locked_view(state)
        } else {
            no_database_selected_view(state)
        }
    }
}

fn new_database_set_password_view(state: &PasswordsPage) -> Element<Message> {
    column![
        text(LOCALES.lookup(&state.locale, "set-password-and-or-keyfile"))
            .size(24)
            .width(Length::Fill)
            .align_x(Horizontal::Center),
        text(LOCALES.lookup(&state.locale, "enter-the-master-password"))
            .width(Length::Fill)
            .align_x(Horizontal::Center),
        row![
            text_input(
                &LOCALES.lookup(&state.locale, "master-password"),
                &state.master_password_field_text
            )
            .secure(state.hide_master_password_entry)
            .on_input(|s| Message::Passwords(PasswordsPageMessage::UpdateMasterPasswordField(s)))
            .width(Length::FillPortion(9)),
            Tooltip::new(
                button(if state.hide_master_password_entry {
                    Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/eye.svg"
                    )))
                    .height(Length::Fill)
                } else {
                    Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/eye-blocked.svg"
                    )))
                    .height(Length::Fill)
                })
                .on_press(Message::Passwords(
                    PasswordsPageMessage::ToggleHideMasterPassword
                ))
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .style(if state.hide_master_password_entry {
                    button::primary
                } else {
                    button::secondary
                }),
                if state.hide_master_password_entry {
                    text(LOCALES.lookup(&state.locale, "show-password"))
                } else {
                    text(LOCALES.lookup(&state.locale, "hide-password"))
                },
                iced::widget::tooltip::Position::Bottom,
            )
        ]
        .height(Length::Shrink),
        text(LOCALES.lookup(&state.locale, "re-enter-the-master-password"))
            .width(Length::Fill)
            .align_x(Horizontal::Center),
        row![
            text_input(
                &LOCALES.lookup(&state.locale, "re-enter-the-master-password"),
                &state.master_password_reentry_field_text
            )
            .secure(state.hide_master_password_reentry_entry)
            .on_input(|s| Message::Passwords(
                PasswordsPageMessage::UpdateMasterPasswordReentryField(s)
            ))
            .width(Length::FillPortion(9)),
            Tooltip::new(
                button(if state.hide_master_password_reentry_entry {
                    Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/eye.svg"
                    )))
                    .height(Length::Fill)
                } else {
                    Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/eye-blocked.svg"
                    )))
                    .height(Length::Fill)
                })
                .on_press(Message::Passwords(
                    PasswordsPageMessage::ToggleHideMasterPasswordReentry
                ))
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .style(if state.hide_master_password_reentry_entry {
                    button::primary
                } else {
                    button::secondary
                }),
                if state.hide_master_password_reentry_entry {
                    text(LOCALES.lookup(&state.locale, "show-password"))
                } else {
                    text(LOCALES.lookup(&state.locale, "hide-password"))
                },
                iced::widget::tooltip::Position::Bottom,
            )
        ]
        .height(Length::Shrink),
        text(if state.passwords_dont_match {
            LOCALES.lookup(&state.locale, "passwords-dont-match")
        } else {
            String::new()
        })
        .width(Length::Fill)
        .align_x(Horizontal::Center),
        container(
            button(text(if let Some(keyfile) = &state.selected_key_file {
                format!(
                    "{} {}",
                    LOCALES.lookup(&state.locale, "selected-keyfile"),
                    keyfile.as_path().to_string_lossy()
                )
            } else {
                LOCALES.lookup(&state.locale, "select-keyfile")
            }))
            .on_press(Message::Passwords(PasswordsPageMessage::PickKeyFile))
        )
        .width(Length::Fill)
        .align_x(Horizontal::Center),
        Space::with_height(20),
        container(
            button(text(LOCALES.lookup(&state.locale, "create-database")))
                .on_press(Message::Passwords(PasswordsPageMessage::CreateDatabase))
                .style(button::success)
        )
        .width(Length::Fill)
        .align_x(Horizontal::Center),
    ]
    .spacing(10)
    .padding(10)
    .into()
}

fn creating_new_database_choose_path_view(state: &PasswordsPage) -> Element<Message> {
    column![
        text(LOCALES.lookup(&state.locale, "choose-file-path"))
            .size(24)
            .width(Length::Fill)
            .align_x(Horizontal::Center),
        container(
            button(text(LOCALES.lookup(&state.locale, "select-file-path"))).on_press(
                Message::Passwords(PasswordsPageMessage::PickNewDatabasePath)
            )
        )
        .width(Length::Fill)
        .align_x(Horizontal::Center),
    ]
    .spacing(10)
    .padding(10)
    .into()
}

fn sidebar_view(state: &PasswordsPage) -> Element<Message> {
    column![
        text_input(
            &LOCALES.lookup(&state.locale, "filter"),
            &state.current_passwords_list_filter
        )
        .on_input(|s| { Message::Passwords(PasswordsPageMessage::UpdatePasswordsFilter(s)) }),
        Scrollable::new(
            column(
                state
                    .passwords_list
                    .iter()
                    .filter(|password| password
                        .title
                        .to_lowercase()
                        .contains(&state.current_passwords_list_filter.to_lowercase()))
                    .map(|password| {
                        button(
                            text(if !password.title.is_empty() {
                                password.title.clone()
                            } else {
                                LOCALES.lookup(&state.locale, "no-title")
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
                        .style(
                            if let Some(selected_password) = &state.selected_password_entry {
                                if selected_password.id == password.id {
                                    button::secondary
                                } else {
                                    button::primary
                                }
                            } else {
                                button::primary
                            },
                        )
                        .width(Length::Fill)
                        .into()
                    })
            )
            .spacing(5)
        )
        .direction(Direction::Vertical(Scrollbar::new()))
        .height(Length::Fill)
        .width(Length::FillPortion(1)),
    ]
    .spacing(5)
    .into()
}

fn entry_edit_view(state: &PasswordsPage) -> Element<Message> {
    column![
        if state.selected_password_entry.is_none() {
            row![text(LOCALES.lookup(&state.locale, "new-entry"))
                .size(24)
                .align_x(Horizontal::Center)
                .width(Length::Fill),]
        } else {
            row![
                text(LOCALES.lookup(&state.locale, "edit-entry"))
                    .size(24)
                    .align_x(Horizontal::Center)
                    .width(Length::FillPortion(9)),
                button(
                    Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/delete.svg"
                    )))
                    .height(Length::Fill)
                )
                .style(button::danger)
                .height(Length::Fill)
                .width(Length::FillPortion(1))
                .on_press(Message::Passwords(
                    PasswordsPageMessage::DeletePasswordEntry(
                        state
                            .selected_password_entry
                            .clone()
                            .expect("Already checked")
                            .id
                    )
                ))
            ]
            .height(Length::Shrink)
        },
        text(LOCALES.lookup(&state.locale, "title")),
        row![
            text_input(
                &LOCALES.lookup(&state.locale, "edit-entry"),
                &state.current_title_text
            )
            .on_input(|s| { Message::Passwords(PasswordsPageMessage::UpdateCurrentTitleText(s)) })
            .width(Length::FillPortion(9)),
            Tooltip::new(
                button(
                    Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/copy.svg"
                    )))
                    .height(Length::Fill)
                )
                .on_press(Message::Passwords(
                    PasswordsPageMessage::CopyValueToClipboard(state.current_title_text.clone())
                ))
                .width(Length::FillPortion(1)),
                text(LOCALES.lookup(&state.locale, "copy")),
                iced::widget::tooltip::Position::Bottom,
            )
        ]
        .height(Length::Shrink),
        text(LOCALES.lookup(&state.locale, "url")),
        row![
            text_input(
                &LOCALES.lookup(&state.locale, "url"),
                &state.current_url_text
            )
            .on_input(|s| { Message::Passwords(PasswordsPageMessage::UpdateCurrentUrlText(s)) })
            .width(Length::FillPortion(9)),
            Tooltip::new(
                button(
                    Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/copy.svg"
                    )))
                    .height(Length::Fill)
                )
                .on_press(Message::Passwords(
                    PasswordsPageMessage::CopyValueToClipboard(state.current_url_text.clone())
                ))
                .width(Length::FillPortion(1)),
                text(LOCALES.lookup(&state.locale, "copy")),
                iced::widget::tooltip::Position::Bottom,
            )
        ]
        .height(Length::Shrink),
        text(LOCALES.lookup(&state.locale, "username")),
        row![
            text_input(
                &LOCALES.lookup(&state.locale, "username"),
                &state.current_username_text
            )
            .on_input(|s| {
                Message::Passwords(PasswordsPageMessage::UpdateCurrentUsernameText(s))
            })
            .width(Length::FillPortion(9)),
            Tooltip::new(
                button(
                    Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/copy.svg"
                    )))
                    .height(Length::Fill)
                )
                .on_press(Message::Passwords(
                    PasswordsPageMessage::CopyValueToClipboard(state.current_username_text.clone())
                ))
                .width(Length::FillPortion(1)),
                text(LOCALES.lookup(&state.locale, "copy")),
                iced::widget::tooltip::Position::Bottom,
            )
        ]
        .height(Length::Shrink),
        text(LOCALES.lookup(&state.locale, "password")),
        row![
            text_input(
                &LOCALES.lookup(&state.locale, "password"),
                &state.current_password_text
            )
            .on_input(|s| {
                Message::Passwords(PasswordsPageMessage::UpdateCurrentPasswordText(s))
            })
            .secure(state.hide_current_password_entry)
            .width(Length::FillPortion(8)),
            Tooltip::new(
                button(if state.hide_current_password_entry {
                    Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/eye.svg"
                    )))
                    .height(Length::Fill)
                } else {
                    Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/eye-blocked.svg"
                    )))
                    .height(Length::Fill)
                })
                .on_press(Message::Passwords(
                    PasswordsPageMessage::ToggleHideCurrentPassword
                ))
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .style(if state.hide_current_password_entry {
                    button::primary
                } else {
                    button::secondary
                }),
                if state.hide_current_password_entry {
                    text(LOCALES.lookup(&state.locale, "show-password"))
                } else {
                    text(LOCALES.lookup(&state.locale, "hide-password"))
                },
                iced::widget::tooltip::Position::Bottom,
            ),
            if state.selected_password_entry.is_none() {
                column![Tooltip::new(
                    button(
                        Svg::new(svg::Handle::from_memory(include_bytes!(
                            "../../../icons/generate-password.svg"
                        )))
                        .height(Length::Fill)
                    )
                    .on_press(Message::Passwords(PasswordsPageMessage::GeneratePassword))
                    .width(Length::FillPortion(1)),
                    text(LOCALES.lookup(&state.locale, "generate-password")),
                    iced::widget::tooltip::Position::Bottom,
                ),]
            } else {
                column![]
            },
            Tooltip::new(
                button(
                    Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/copy.svg"
                    )))
                    .height(Length::Fill)
                )
                .on_press(Message::Passwords(
                    PasswordsPageMessage::CopyValueToClipboard(state.current_password_text.clone())
                ))
                .width(Length::FillPortion(1)),
                text(LOCALES.lookup(&state.locale, "copy")),
                iced::widget::tooltip::Position::Bottom,
            )
        ]
        .height(Length::Shrink),
        button(text(if state.selected_password_entry.is_none() {
            LOCALES.lookup(&state.locale, "add-entry")
        } else {
            LOCALES.lookup(&state.locale, "update-entry")
        }))
        .on_press(Message::Passwords(
            PasswordsPageMessage::UpdatePasswordEntry
        ))
    ]
    .spacing(10)
    .padding(20)
    .height(Length::Fill)
    .width(Length::FillPortion(2))
    .into()
}

fn existing_database_selected_and_locked_view(state: &PasswordsPage) -> Element<Message> {
    column![
        text(LOCALES.lookup(
            &state.locale,
            "password-vault-locked-enter-password-or-keyfile"
        ))
        .size(24)
        .width(Length::Fill)
        .align_x(Horizontal::Center),
        container(
            row![
                button(text(format!(
                    "{} {}",
                    LOCALES.lookup(&state.locale, "selected-file"),
                    state
                        .selected_keepass_file
                        .as_ref()
                        .and_then(|selected| selected.to_str())
                        .unwrap_or_default()
                )))
                .on_press(Message::Passwords(PasswordsPageMessage::PickDatabaseFile)),
                button(text(if let Some(keyfile) = &state.selected_key_file {
                    format!(
                        "{} {}",
                        LOCALES.lookup(&state.locale, "selected-keyfile"),
                        keyfile.as_path().to_string_lossy()
                    )
                } else {
                    LOCALES.lookup(&state.locale, "select-keyfile")
                }))
                .on_press(Message::Passwords(PasswordsPageMessage::PickKeyFile))
            ]
            .spacing(20)
        )
        .width(Length::Fill)
        .align_x(Horizontal::Center),
        container(
            column![
                text(LOCALES.lookup(&state.locale, "enter-the-master-password"))
                    .width(Length::Fill)
                    .align_x(Horizontal::Left),
                row![
                    text_input(
                        &LOCALES.lookup(&state.locale, "master-password"),
                        &state.master_password_field_text
                    )
                    .secure(state.hide_master_password_entry)
                    .on_input(|s| Message::Passwords(
                        PasswordsPageMessage::UpdateMasterPasswordField(s)
                    ))
                    .on_submit(Message::Passwords(PasswordsPageMessage::TryUnlock))
                    .width(Length::FillPortion(9)),
                    Tooltip::new(
                        button(if state.hide_master_password_entry {
                            Svg::new(svg::Handle::from_memory(include_bytes!(
                                "../../../icons/eye.svg"
                            )))
                            .height(Length::Fill)
                        } else {
                            Svg::new(svg::Handle::from_memory(include_bytes!(
                                "../../../icons/eye-blocked.svg"
                            )))
                            .height(Length::Fill)
                        })
                        .on_press(Message::Passwords(
                            PasswordsPageMessage::ToggleHideMasterPassword
                        ))
                        .width(Length::FillPortion(1))
                        .height(Length::Fill)
                        .style(if state.hide_master_password_entry {
                            button::primary
                        } else {
                            button::secondary
                        }),
                        if state.hide_master_password_entry {
                            text(LOCALES.lookup(&state.locale, "show-password"))
                        } else {
                            text(LOCALES.lookup(&state.locale, "hide-password"))
                        },
                        iced::widget::tooltip::Position::Bottom,
                    )
                ]
                .height(Length::Shrink),
                text(if state.incorrect_password_entered {
                    LOCALES.lookup(&state.locale, "incorrect-master-password-try-again")
                } else {
                    String::new()
                })
                .width(Length::Fill)
                .align_x(Horizontal::Center)
            ]
            .max_width(500)
        )
        .width(Length::Fill)
        .align_x(Center),
        container(
            row![
                button(text(LOCALES.lookup(&state.locale, "close-database")))
                    .on_press(Message::Passwords(PasswordsPageMessage::CloseDatabase))
                    .style(button::danger),
                button(text(LOCALES.lookup(&state.locale, "open-database")))
                    .on_press(Message::Passwords(PasswordsPageMessage::TryUnlock))
                    .style(button::success)
            ]
            .spacing(20)
        )
        .width(Length::Fill)
        .align_x(Horizontal::Center),
    ]
    .spacing(10)
    .padding(10)
    .into()
}

fn no_database_selected_view(state: &PasswordsPage) -> Element<Message> {
    column![container(
        column![
            button(
                text(LOCALES.lookup(&state.locale, "select-keepass-file"))
                    .width(Length::Fill)
                    .align_x(Center)
            )
            .on_press(Message::Passwords(PasswordsPageMessage::PickDatabaseFile))
            .width(Length::Fill),
            button(
                text(LOCALES.lookup(&state.locale, "create-new-keepass-file"))
                    .width(Length::Fill)
                    .align_x(Center)
            )
            .on_press(Message::Passwords(
                PasswordsPageMessage::StartCreatingNewKeepassFile
            ))
            .width(Length::Fill),
        ]
        .spacing(20)
        .width(Length::Fixed(250.0))
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

pub fn tool_view(state: &PasswordsPage) -> Element<Message> {
    if state.is_unlocked {
        row![
            Tooltip::new(
                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/toggle-sidebar.svg"
                ))))
                .on_press(Message::Passwords(PasswordsPageMessage::ToggleShowSidebar))
                .style(if state.show_sidebar {
                    button::secondary
                } else {
                    button::primary
                }),
                text(LOCALES.lookup(&state.locale, "toggle-sidebar-shortcut")),
                iced::widget::tooltip::Position::Bottom
            ),
            Tooltip::new(
                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/add.svg"
                ))))
                .on_press(Message::Passwords(
                    PasswordsPageMessage::SelectPassword(None)
                )),
                text(LOCALES.lookup(&state.locale, "add-entry-shortcut")),
                iced::widget::tooltip::Position::Bottom
            ),
            Tooltip::new(
                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/lock.svg"
                ))))
                .on_press(Message::Passwords(PasswordsPageMessage::Lock)),
                text(LOCALES.lookup(&state.locale, "lock-shortcut")),
                iced::widget::tooltip::Position::Bottom
            ),
            Tooltip::new(
                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/ok.svg"
                ))))
                .on_press(Message::Passwords(PasswordsPageMessage::SaveDatabaseToFile))
                .style(if state.is_dirty {
                    button::success
                } else {
                    button::secondary
                }),
                text(LOCALES.lookup(&state.locale, "save-changes")),
                iced::widget::tooltip::Position::Bottom,
            )
        ]
        .width(Length::FillPortion(1))
        .into()
    } else if state.is_creating_new_keepass_file {
        row![Tooltip::new(
            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                "../../../icons/back.svg"
            ))))
            .on_press(Message::Passwords(
                PasswordsPageMessage::LockAndDeselectDatabase
            )),
            text(LOCALES.lookup(&state.locale, "back")),
            iced::widget::tooltip::Position::Bottom,
        )]
        .width(Length::FillPortion(1))
        .into()
    } else {
        row![].width(Length::FillPortion(1)).into()
    }
}
