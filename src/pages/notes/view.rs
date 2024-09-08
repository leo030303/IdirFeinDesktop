use iced_aw::{badge, style};

use iced::alignment::Horizontal;
use iced::widget::{
    button, column, markdown, row, scrollable, text, text_editor, text_input, Scrollable, Svg,
    Tooltip,
};
use iced::{highlighter, Length};
use iced::{Element, Fill, Font};

use crate::Message;

use super::page::{NotesPage, NotesPageMessage};

pub fn main_view(state: &NotesPage) -> Element<Message> {
    row![
        if state.show_sidebar {
            if state.selected_folder.is_some() {
                sidebar_with_selected_folder(state)
            } else {
                sidebar_without_selected_folder(state)
            }
        } else {
            column![].into()
        },
        if state.show_editor {
            if state.is_loading_note {
                loading_editor_view(state)
            } else {
                editor_view(state)
            }
        } else {
            column![].into()
        },
        if state.show_markdown {
            if state.is_loading_note {
                loading_preview_view(state)
            } else {
                preview_view(state)
            }
        } else {
            column![].into()
        }
    ]
    .spacing(10)
    .padding(10)
    .into()
}

fn sidebar_with_selected_folder(state: &NotesPage) -> Element<Message> {
    column![
        text_input("Filter", &state.notes_list_filter)
            .on_input(|s| { Message::Notes(NotesPageMessage::FilterNotesList(s)) }),
        Scrollable::new(
            column(
                state
                    .notes_list
                    .iter()
                    .filter(|note| {
                        note.button_title
                            .to_lowercase()
                            .contains(&state.notes_list_filter.to_lowercase())
                    })
                    .map(|note| {
                        button(column![
                            text(note.button_title.clone())
                                .font(Font {
                                    weight: iced::font::Weight::Semibold,
                                    ..Default::default()
                                })
                                .width(Length::Fill)
                                .align_x(Horizontal::Center),
                            if let Some(category) = &note.category {
                                std::convert::Into::<Element<Message>>::into(
                                    badge(text(category)).style(style::badge::info),
                                )
                            } else {
                                column![].into()
                            }
                        ])
                        .on_press(Message::Notes(NotesPageMessage::OpenFile(
                            note.file_path.clone(),
                        )))
                        .style(if let Some(current_file) = &state.current_file {
                            if *current_file == note.file_path {
                                button::secondary
                            } else {
                                button::primary
                            }
                        } else {
                            button::primary
                        })
                        .width(Length::Fill)
                        .into()
                    }),
            )
            .spacing(5)
        )
    ]
    .into()
}

fn sidebar_without_selected_folder(_state: &NotesPage) -> Element<Message> {
    button(
        text("Select Notes Folder")
            .width(Length::Fill)
            .align_x(Horizontal::Center),
    )
    .on_press(Message::Notes(NotesPageMessage::OpenFilePicker))
    .width(Length::Fill)
    .into()
}

fn loading_editor_view(_state: &NotesPage) -> Element<Message> {
    column![text("Loading Note").size(24).width(Length::Fill),]
        .spacing(20)
        .height(Length::Shrink)
        .into()
}

fn editor_view(state: &NotesPage) -> Element<Message> {
    text_editor(&state.editor_content)
        .placeholder("Type your Markdown here...")
        .on_action(|action| Message::Notes(NotesPageMessage::Edit(action)))
        .height(Fill)
        .padding(10)
        .font(Font::MONOSPACE)
        .highlight("markdown", highlighter::Theme::Base16Ocean)
        .into()
}

fn loading_preview_view(_state: &NotesPage) -> Element<Message> {
    column![text("Loading Preview").size(24).width(Length::Fill),]
        .spacing(20)
        .height(Length::Shrink)
        .into()
}

fn preview_view(state: &NotesPage) -> Element<Message> {
    scrollable(
        markdown(
            &state.markdown_preview_items,
            markdown::Settings::default(),
            markdown::Style::from_palette(state.theme.palette()),
        )
        .map(|url| Message::Notes(NotesPageMessage::LinkClicked(url))),
    )
    .spacing(10)
    .height(Fill)
    .into()
}

pub fn tool_view(state: &NotesPage) -> Element<Message> {
    row![
        Tooltip::new(
            button(Svg::from_path("icons/toggle-sidebar.svg"))
                .on_press(Message::Notes(NotesPageMessage::ToggleSidebar))
                .style(if state.show_sidebar {
                    button::secondary
                } else {
                    button::primary
                }),
            "Toggle Sidebar",
            iced::widget::tooltip::Position::Bottom
        ),
        Tooltip::new(
            button(Svg::from_path("icons/markdown.svg"))
                .on_press(Message::Notes(NotesPageMessage::ToggleMarkdown))
                .style(if state.show_markdown {
                    button::secondary
                } else {
                    button::primary
                }),
            "Toggle Markdown Preview",
            iced::widget::tooltip::Position::Bottom
        ),
        Tooltip::new(
            button(Svg::from_path("icons/editor.svg"))
                .on_press(Message::Notes(NotesPageMessage::ToggleEditor))
                .style(if state.show_editor {
                    button::secondary
                } else {
                    button::primary
                }),
            "Toggle Editor",
            iced::widget::tooltip::Position::Bottom
        ),
        Tooltip::new(
            button(Svg::from_path("icons/add.svg"))
                .on_press(Message::Notes(NotesPageMessage::OpenFilePicker)),
            "New Note",
            iced::widget::tooltip::Position::Bottom
        ),
        Tooltip::new(
            button(Svg::from_path("icons/view-more.svg"))
                .on_press(Message::Notes(NotesPageMessage::OpenExtraToolsMenu)),
            "More Tools",
            iced::widget::tooltip::Position::Bottom
        ),
    ]
    .width(Length::FillPortion(1))
    .into()
}
