use iced::Alignment::Center;
use iced_aw::{badge, drop_down, style, DropDown};

use iced::alignment::Horizontal;
use iced::widget::{
    button, column, markdown, row, scrollable, text, text_editor, text_input, Scrollable, Svg,
    Tooltip,
};
use iced::{highlighter, Length};
use iced::{Element, Fill, Font};

use crate::app::Message;

use super::page::{NotesPage, NotesPageMessage};

pub fn main_view(state: &NotesPage) -> Element<Message> {
    row![
        column![if state.show_sidebar {
            column![if state.selected_folder.is_some() {
                sidebar_with_selected_folder(state)
            } else {
                sidebar_without_selected_folder(state)
            }]
            .width(Length::FillPortion(1))
        } else {
            column![]
        }],
        column![
            row![
                if state.show_document_statistics_view {
                    if state.is_loading_note {
                        loading_note_view(state)
                    } else {
                        document_statistics_view(state)
                    }
                } else {
                    column![].into()
                },
                if state.show_rename_note_view {
                    if state.is_loading_note {
                        loading_note_view(state)
                    } else {
                        rename_note_view(state)
                    }
                } else {
                    column![].into()
                },
            ]
            .spacing(10),
            row![
                if state.show_editor {
                    if state.is_loading_note {
                        loading_note_view(state)
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
        ]
        .spacing(10)
        .width(Length::FillPortion(2))
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

fn loading_note_view(_state: &NotesPage) -> Element<Message> {
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

fn document_statistics_view(state: &NotesPage) -> Element<Message> {
    column![
        row![
            text("Document Statistics").width(Length::Fill).size(24),
            Tooltip::new(
                button(Svg::from_path("icons/close.svg"))
                    .on_press(Message::Notes(
                        NotesPageMessage::ToggleDocumentStatisticsView
                    ))
                    .width(Length::Fixed(50.0)),
                "Close Statistics",
                iced::widget::tooltip::Position::Bottom
            ),
        ],
        text(format!(
            "Character Count: {}",
            state.current_note_statistics.char_count
        )),
        text(format!(
            "Word Count: {}",
            state.current_note_statistics.word_count
        )),
        text(format!(
            "Reading Time: {} minutes",
            state.current_note_statistics.reading_time_in_mins
        )),
        Tooltip::new(
            button(Svg::from_path("icons/refresh.svg"))
                .on_press(Message::Notes(NotesPageMessage::CalculateNoteStatistics))
                .width(Length::Fill),
            "Refresh Statistics",
            iced::widget::tooltip::Position::Bottom
        ),
    ]
    .into()
}

fn rename_note_view(_state: &NotesPage) -> Element<Message> {
    column![text("Rename Note").width(Length::Fill).size(24)].into()
}

pub fn tool_view(state: &NotesPage) -> Element<Message> {
    let underlay = Tooltip::new(
        button(Svg::from_path("icons/view-more.svg"))
            .on_press(Message::Notes(NotesPageMessage::ToggleExtraToolsMenu)),
        "More Tools",
        iced::widget::tooltip::Position::Bottom,
    );
    let overlay = column![
        button(text("Export PDF").width(Length::Fill).align_x(Center))
            .on_press(Message::Notes(NotesPageMessage::ExportPDF))
            .width(Length::Fill),
        button(text("Post to website").width(Length::Fill).align_x(Center))
            .on_press(Message::Notes(NotesPageMessage::ExportToWebsite))
            .width(Length::Fill),
        button(
            text(if !state.show_document_statistics_view {
                "Show statistics"
            } else {
                "Hide Statistics"
            })
            .width(Length::Fill)
            .align_x(Center)
        )
        .on_press(Message::Notes(
            NotesPageMessage::ToggleDocumentStatisticsView
        ))
        .width(Length::Fill),
        button(
            text(if !state.show_rename_note_view {
                "Rename Note"
            } else {
                "Hide Rename Panel"
            })
            .width(Length::Fill)
            .align_x(Center)
        )
        .on_press(Message::Notes(NotesPageMessage::ToggleRenameNoteView))
        .width(Length::Fill)
    ]
    .width(Length::Fixed(200.0));

    let drop_down = DropDown::new(underlay, overlay, state.show_extra_tools_menu)
        .on_dismiss(Message::Notes(NotesPageMessage::ToggleExtraToolsMenu))
        .width(Length::Fill)
        .alignment(drop_down::Alignment::Bottom);
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
        drop_down
    ]
    .width(Length::FillPortion(1))
    .into()
}
