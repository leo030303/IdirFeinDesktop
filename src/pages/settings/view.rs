use iced::{
    border,
    widget::{button, column, container, pick_list, row, scrollable, text, toggler},
    Alignment, Background, Element, Length, Theme,
};
use iced_aw::Spinner;

use crate::config::AppConfig;
use crate::{app::Message, Page};

use super::page::{SettingsPage, SettingsPageMessage, SettingsTab};

pub fn main_view<'a>(state: &'a SettingsPage, app_config: &'a AppConfig) -> Element<'a, Message> {
    column![
        if state.is_saving {
            row![text("Settings are being saved"), Spinner::new()]
        } else if state.save_was_successful {
            row![text(&state.save_message).style(text::success)]
        } else {
            row![text(&state.save_message).style(text::danger)]
        },
        text(format!("{} Settings", state.current_tab.name())).size(24),
        row(SettingsTab::get_all()
            .into_iter()
            .map(|tab| button(text(tab.name()))
                .style(if state.current_tab == tab {
                    button::secondary
                } else {
                    button::primary
                })
                .on_press(Message::Settings(SettingsPageMessage::ChangeTab(tab)))
                .into()))
        .width(Length::Fill)
        .wrap(),
        match state.current_tab {
            SettingsTab::General => general_tab(state, app_config),
            SettingsTab::Sync => sync_tab(state, app_config),
            SettingsTab::FileManager => file_manager_tab(state, app_config),
            SettingsTab::Gallery => gallery_tab(state, app_config),
            SettingsTab::Passwords => passwords_tab(state, app_config),
            SettingsTab::Notes => notes_tab(state, app_config),
            SettingsTab::Tasks => tasks_tab(state, app_config),
        }
    ]
    .padding(20)
    .spacing(10)
    .into()
}

fn general_tab<'a>(state: &'a SettingsPage, app_config: &'a AppConfig) -> Element<'a, Message> {
    scrollable(
        column![
            theme_picker(state, app_config),
            default_page_picker(state, app_config)
        ]
        .spacing(30),
    )
    .into()
}

fn sync_tab<'a>(state: &'a SettingsPage, app_config: &'a AppConfig) -> Element<'a, Message> {
    text("sync tab").into()
}

fn tasks_tab<'a>(state: &'a SettingsPage, app_config: &'a AppConfig) -> Element<'a, Message> {
    text("tasks tab").into()
}

fn notes_tab<'a>(state: &'a SettingsPage, app_config: &'a AppConfig) -> Element<'a, Message> {
    scrollable(
        container(
            column![
                row![
                    text(
                        app_config
                            .notes_config
                            .default_folder
                            .as_ref()
                            .map(|value| format!("Default Notes Folder: {value:?}"))
                            .unwrap_or(String::from("No Default Notes Folder Selected"))
                    )
                    .align_x(Alignment::Center)
                    .width(Length::Fill),
                    button(
                        text("Select Default Notes Folder")
                            .width(Length::Fill)
                            .align_x(Alignment::Center)
                    )
                    .on_press(Message::Settings(
                        SettingsPageMessage::NotesSetDefaultFolder
                    ))
                ]
                .width(Length::Fill),
                toggler(
                    Some("Show sidebar on startup".to_string()),
                    app_config.notes_config.show_sidebar_on_start,
                    |b| Message::Settings(SettingsPageMessage::NotesSetShowSidebarOnStart(b))
                ),
                toggler(
                    Some("Show editor on startup".to_string()),
                    app_config.notes_config.show_editor_on_start,
                    |b| Message::Settings(SettingsPageMessage::NotesSetShowEditorOnStart(b))
                ),
                toggler(
                    Some("Show markdown preview on startup".to_string()),
                    app_config.notes_config.show_markdown_on_start,
                    |b| Message::Settings(SettingsPageMessage::NotesSetShowMarkdownOnStart(b))
                ),
                toggler(
                    Some("Confirm before deleting a note".to_string()),
                    app_config.notes_config.confirm_before_delete,
                    |b| Message::Settings(SettingsPageMessage::NotesSetShowConfirmDelete(b))
                ),
                toggler(
                    Some("Show formatting toolbar".to_string()),
                    app_config.notes_config.show_format_toolbar,
                    |b| Message::Settings(SettingsPageMessage::NotesSetShowFormatToolbar(b))
                ),
                toggler(
                    Some("Autocomplete lists".to_string()),
                    app_config.notes_config.autocomplete_lists,
                    |b| Message::Settings(SettingsPageMessage::NotesSetAutocompleteLists(b))
                ),
            ]
            .padding(20)
            .spacing(30),
        )
        .style(container::bordered_box)
        .width(Length::Fill),
    )
    .into()
}

fn passwords_tab<'a>(state: &'a SettingsPage, app_config: &'a AppConfig) -> Element<'a, Message> {
    scrollable(
        container(
            column![
                row![
                    text(
                        app_config
                            .passwords_config
                            .default_database
                            .as_ref()
                            .map(|value| format!("Default Database: {value:?}"))
                            .unwrap_or(String::from("No Default Database Selected"))
                    )
                    .align_x(Alignment::Center)
                    .width(Length::Fill),
                    button(
                        text("Select Default Database")
                            .width(Length::Fill)
                            .align_x(Alignment::Center)
                    )
                    .on_press(Message::Settings(
                        SettingsPageMessage::PasswordsSetDefaultDatabase
                    ))
                ]
                .width(Length::Fill),
                toggler(
                    Some("Show sidebar on startup".to_string()),
                    app_config.passwords_config.show_sidebar_on_start,
                    |b| Message::Settings(SettingsPageMessage::PasswordsSetShowSidebarOnStart(b))
                ),
            ]
            .padding(20)
            .spacing(30),
        )
        .style(container::bordered_box)
        .width(Length::Fill),
    )
    .into()
}

fn gallery_tab<'a>(state: &'a SettingsPage, app_config: &'a AppConfig) -> Element<'a, Message> {
    text("gallery tab").into()
}

fn file_manager_tab<'a>(
    state: &'a SettingsPage,
    app_config: &'a AppConfig,
) -> Element<'a, Message> {
    text("file manager tab").into()
}

fn default_page_picker<'a>(
    _state: &'a SettingsPage,
    app_config: &'a AppConfig,
) -> Element<'a, Message> {
    container(
        column![
            text("Select the default page on startup").size(20),
            pick_list(
                Page::get_all()
                    .into_iter()
                    .map(|page| page.name())
                    .collect::<Vec<&str>>(),
                Some(app_config.default_page_on_open.name()),
                |page_str| Message::Settings(SettingsPageMessage::SetDefaultPageOnOpen(page_str)),
            )
        ]
        .spacing(10)
        .padding(20),
    )
    .style(container::bordered_box)
    .width(Length::Fill)
    .into()
}

fn theme_picker<'a>(_state: &'a SettingsPage, app_config: &'a AppConfig) -> Element<'a, Message> {
    container(
        column![
            text("Select a theme").size(20),
            text(format!(
                "Current theme: {:?}",
                app_config.get_theme().unwrap_or(Theme::Light)
            )),
            row(Theme::ALL.iter().map(|theme| {
                button(text(format!("{theme:?}")))
                    .on_press(Message::Settings(SettingsPageMessage::SetTheme(
                        theme.clone(),
                    )))
                    .style(|_, status| {
                        let palette = theme.extended_palette();
                        let base = button::Style {
                            background: Some(Background::Color(palette.primary.strong.color)),
                            text_color: palette.primary.strong.text,
                            border: border::rounded(2),
                            ..button::Style::default()
                        };

                        match status {
                            button::Status::Active | button::Status::Pressed => base,
                            button::Status::Hovered => button::Style {
                                background: Some(Background::Color(palette.primary.base.color)),
                                ..base
                            },
                            button::Status::Disabled => base,
                        }
                    })
                    .into()
            }))
            .spacing(20)
            .padding(20)
            .width(Length::Fill)
            .wrap()
        ]
        .spacing(10)
        .padding(20),
    )
    .style(container::bordered_box)
    .width(Length::Fill)
    .into()
}

pub fn tool_view(_state: &SettingsPage) -> Element<Message> {
    row![].width(Length::FillPortion(1)).into()
}
