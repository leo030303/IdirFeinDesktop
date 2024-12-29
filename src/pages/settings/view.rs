use iced::{
    border,
    widget::{
        button, column, container, pick_list, row, scrollable, svg, text, text_input, toggler, Svg,
        Tooltip,
    },
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
    column![
        if state.is_connected_to_server {
            row![
                text("Connected to server").style(text::success),
                Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/connection.svg"
                )))
            ]
        } else {
            row![
                text("Disconnected from server").style(text::danger),
                Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/no_connection.svg"
                )))
            ]
        },
        row![
            text_input("Server Url", &state.server_url_editor_text)
                .width(Length::Fixed(200.0))
                .on_input(|s| Message::Settings(SettingsPageMessage::SyncUpdateServerUrl(s)))
                .on_submit(Message::Settings(SettingsPageMessage::SyncSetServerUrl)),
            button("Set Url").on_press(Message::Settings(SettingsPageMessage::SyncSetServerUrl))
        ],
        row![
            row![
                text_input("New ignore list entry", &state.ignore_list_editor_text)
                    .width(Length::Fixed(200.0))
                    .on_input(|s| Message::Settings(
                        SettingsPageMessage::SyncUpdateIgnoreListEditor(s)
                    ))
                    .on_submit(Message::Settings(SettingsPageMessage::SyncAddToIgnoreList)),
                button("Add").on_press(Message::Settings(SettingsPageMessage::SyncAddToIgnoreList))
            ],
            column(
                app_config
                    .sync_config
                    .ignore_string_list
                    .iter()
                    .enumerate()
                    .map(|(index, ignore_list_item)| {
                        row![
                            text(ignore_list_item),
                            Tooltip::new(
                                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                                    "../../../icons/delete.svg"
                                ))))
                                .on_press(Message::Settings(
                                    SettingsPageMessage::SyncDeleteFromIgnoreList(index)
                                ))
                                .style(button::danger)
                                .width(Length::Fixed(50.0))
                                .height(Length::Fixed(30.0)),
                                "Remove",
                                iced::widget::tooltip::Position::Bottom
                            ),
                        ]
                        .into()
                    })
            )
        ],
        row![
            row![
                button("Add folder to sync list").on_press(Message::Settings(
                    SettingsPageMessage::SyncPickNewSyncListFolder
                ))
            ],
            column(
                app_config
                    .sync_config
                    .folders_to_sync
                    .iter()
                    .enumerate()
                    .map(|(index, (_folder_id, folder_path))| {
                        row![
                            text(folder_path.to_str().unwrap_or("Error reading folder path")),
                            Tooltip::new(
                                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                                    "../../../icons/delete.svg"
                                ))))
                                .on_press(Message::Settings(
                                    SettingsPageMessage::SyncDeleteFromFolderList(index)
                                ))
                                .style(button::danger)
                                .width(Length::Fixed(50.0))
                                .height(Length::Fixed(30.0)),
                                "Remove",
                                iced::widget::tooltip::Position::Bottom
                            ),
                        ]
                        .into()
                    })
            )
        ],
        button("Test sync (this button will be removed)")
            .on_press(Message::SendServerMessage(String::from("Test")))
    ]
    .into()
}

fn tasks_tab<'a>(_state: &'a SettingsPage, app_config: &'a AppConfig) -> Element<'a, Message> {
    scrollable(
        container(
            column![
                row![
                    text(
                        app_config
                            .tasks_config
                            .default_folder
                            .as_ref()
                            .map(|value| format!("Default Task Projects Folder: {value:?}"))
                            .unwrap_or(String::from("No Default Task Projects Folder Selected"))
                    )
                    .align_x(Alignment::Center)
                    .width(Length::Fill),
                    button(
                        text("Select Default Task Projects Folder")
                            .width(Length::Fill)
                            .align_x(Alignment::Center)
                    )
                    .on_press(Message::Settings(
                        SettingsPageMessage::TasksPickDefaultProjectFolder
                    ))
                ]
                .width(Length::Fill),
                row![
                    text(
                        app_config
                            .tasks_config
                            .default_project_file
                            .as_ref()
                            .map(|value| format!("Default Tasks Project File: {value:?}"))
                            .unwrap_or(String::from("No Default Tasks Project File Selected"))
                    )
                    .align_x(Alignment::Center)
                    .width(Length::Fill),
                    button(
                        text("Select Default Tasks Project File")
                            .width(Length::Fill)
                            .align_x(Alignment::Center)
                    )
                    .on_press(Message::Settings(
                        SettingsPageMessage::TasksPickDefaultProjectFile
                    ))
                ]
                .width(Length::Fill),
                toggler(app_config.tasks_config.kanban_task_view_is_default)
                    .label("Kanban task view as default")
                    .on_toggle(|b| Message::Settings(
                        SettingsPageMessage::TasksSetKanbanTaskViewIsDefault(b)
                    )),
                toggler(app_config.tasks_config.show_sidebar_on_start)
                    .label("Show sidebar on start")
                    .on_toggle(|b| Message::Settings(
                        SettingsPageMessage::TasksSetShowSidebarOnStart(b)
                    )),
                toggler(app_config.tasks_config.confirm_before_delete)
                    .label("Confirm before deleting a task")
                    .on_toggle(|b| Message::Settings(
                        SettingsPageMessage::TasksSetConfirmBeforeDelete(b)
                    )),
                toggler(app_config.tasks_config.show_task_completion_toolbar)
                    .label("Show task completion toolbar on each task")
                    .on_toggle(|b| Message::Settings(
                        SettingsPageMessage::TasksSetShowTaskCompletionToolbar(b)
                    )),
                toggler(app_config.tasks_config.right_click_to_edit_task)
                    .label("Right clicking on a task should open it for editing")
                    .on_toggle(|b| Message::Settings(
                        SettingsPageMessage::TasksSetRightClickToEditTask(b)
                    )),
            ]
            .padding(20)
            .spacing(30),
        )
        .style(container::bordered_box)
        .width(Length::Fill),
    )
    .into()
}

fn notes_tab<'a>(_state: &'a SettingsPage, app_config: &'a AppConfig) -> Element<'a, Message> {
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
                        SettingsPageMessage::NotesPickDefaultFolder
                    ))
                ]
                .width(Length::Fill),
                row![
                    text(
                        app_config
                            .notes_config
                            .website_folder
                            .as_ref()
                            .map(|value| format!("Website Folder: {value:?}"))
                            .unwrap_or(String::from("No Website Folder Selected"))
                    )
                    .align_x(Alignment::Center)
                    .width(Length::Fill),
                    button(
                        text("Select Website Folder")
                            .width(Length::Fill)
                            .align_x(Alignment::Center)
                    )
                    .on_press(Message::Settings(
                        SettingsPageMessage::NotesPickWebsiteFolder
                    ))
                ]
                .width(Length::Fill),
                toggler(app_config.notes_config.show_sidebar_on_start)
                    .label("Show sidebar on startup")
                    .on_toggle(|b| Message::Settings(
                        SettingsPageMessage::NotesSetShowSidebarOnStart(b)
                    )),
                toggler(app_config.notes_config.show_editor_on_start)
                    .label("Show editor on startup")
                    .on_toggle(|b| Message::Settings(
                        SettingsPageMessage::NotesSetShowEditorOnStart(b)
                    )),
                toggler(app_config.notes_config.show_markdown_on_start)
                    .label("Show markdown preview on startup")
                    .on_toggle(|b| Message::Settings(
                        SettingsPageMessage::NotesSetShowMarkdownOnStart(b)
                    )),
                toggler(app_config.notes_config.confirm_before_delete)
                    .label("Confirm before deleting a note")
                    .on_toggle(|b| Message::Settings(
                        SettingsPageMessage::NotesSetShowConfirmDelete(b)
                    )),
                toggler(app_config.notes_config.autocomplete_brackets_etc)
                    .label("Autocomplete brackets, quotes, etc")
                    .on_toggle(|b| Message::Settings(
                        SettingsPageMessage::NotesSetAutocompleteBrackets(b)
                    )),
                toggler(app_config.notes_config.autocomplete_lists)
                    .label("Autocomplete lists")
                    .on_toggle(|b| Message::Settings(
                        SettingsPageMessage::NotesSetAutocompleteLists(b)
                    )),
            ]
            .padding(20)
            .spacing(30),
        )
        .style(container::bordered_box)
        .width(Length::Fill),
    )
    .into()
}

fn passwords_tab<'a>(_state: &'a SettingsPage, app_config: &'a AppConfig) -> Element<'a, Message> {
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
                        SettingsPageMessage::PasswordsPickDefaultDatabase
                    ))
                ]
                .width(Length::Fill),
                toggler(app_config.passwords_config.show_sidebar_on_start)
                    .label("Show sidebar on startup")
                    .on_toggle(|b| Message::Settings(
                        SettingsPageMessage::PasswordsSetShowSidebarOnStart(b)
                    )),
            ]
            .padding(20)
            .spacing(30),
        )
        .style(container::bordered_box)
        .width(Length::Fill),
    )
    .into()
}

fn gallery_tab<'a>(_state: &'a SettingsPage, app_config: &'a AppConfig) -> Element<'a, Message> {
    scrollable(
        container(
            column![row![
                text(
                    app_config
                        .gallery_config
                        .default_folder
                        .as_ref()
                        .map(|value| format!("Default Folder: {value:?}"))
                        .unwrap_or(String::from("No Default Folder Selected"))
                )
                .align_x(Alignment::Center)
                .width(Length::Fill),
                button(
                    text("Select Default Folder")
                        .width(Length::Fill)
                        .align_x(Alignment::Center)
                )
                .on_press(Message::Settings(
                    SettingsPageMessage::GalleryPickDefaultFolder
                ))
            ]
            .width(Length::Fill),]
            .padding(20)
            .spacing(30),
        )
        .style(container::bordered_box)
        .width(Length::Fill),
    )
    .into()
}

fn file_manager_tab<'a>(
    _state: &'a SettingsPage,
    _app_config: &'a AppConfig,
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
