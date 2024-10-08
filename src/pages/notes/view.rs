use iced::Alignment::Center;
use iced_aw::{badge, drop_down, style, DropDown};

use iced::alignment::Horizontal;
use iced::widget::{
    button, column, markdown, row, scrollable, svg, text, text_editor, text_input, Scrollable, Svg,
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
                if state.show_confirm_delete_note_view {
                    if state.is_loading_note {
                        loading_note_view(state)
                    } else {
                        confirm_delete_view(state)
                    }
                } else {
                    column![].into()
                },
                if state.show_document_statistics_view {
                    if state.is_loading_note {
                        loading_note_view(state)
                    } else {
                        document_statistics_view(state)
                    }
                } else {
                    column![].into()
                },
                if state.show_edit_note_details_view || state.current_file.is_none() {
                    if state.is_loading_note {
                        loading_note_view(state)
                    } else {
                        edit_note_details_view(state)
                    }
                } else {
                    column![].into()
                },
                if state.show_manage_categories_view {
                    if state.is_loading_note {
                        loading_note_view(state)
                    } else {
                        manage_categories_view(state)
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
                },
                if !state.show_markdown && !state.show_editor {
                    column![text("Use the buttons in the top left of the screen to open the editor or preview").size(24).width(Length::Fill).height(Length::Fill)]
                } else {
                    column![]
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
                                column![badge(text(category)).style(style::badge::info)]
                            } else {
                                column![]
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
    column![
        if state.show_format_toolbar {
            row![Tooltip::new(
                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/header1.svg"
                ))))
                .on_press(Message::Notes(NotesPageMessage::InsertTitle)),
                "Insert Title",
                iced::widget::tooltip::Position::Bottom
            ),]
            .height(Length::Fixed(30.0))
        } else {
            row![]
        },
        text_editor(&state.editor_content)
            .placeholder("Type your Markdown here...")
            .on_action(|action| Message::Notes(NotesPageMessage::Edit(action)))
            .height(Fill)
            .padding(10)
            .font(Font::MONOSPACE)
            .highlight("markdown", highlighter::Theme::Base16Ocean)
    ]
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
                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/close.svg"
                ))))
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
            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                "../../../icons/refresh.svg"
            ))))
            .on_press(Message::Notes(NotesPageMessage::CalculateNoteStatistics))
            .width(Length::Fill),
            "Refresh Statistics",
            iced::widget::tooltip::Position::Bottom
        ),
    ]
    .into()
}

fn edit_note_details_view(state: &NotesPage) -> Element<Message> {
    column![
        row![
            text(if state.current_file.is_some() {
                "Rename Note"
            } else {
                "Set Note Title"
            })
            .width(Length::Fill)
            .size(24),
            if state.current_file.is_some() {
                column![Tooltip::new(
                    button(Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/close.svg"
                    ))))
                    .on_press(Message::Notes(NotesPageMessage::ToggleEditNoteDetailsView))
                    .width(Length::Fixed(50.0)),
                    "Close Rename Panel",
                    iced::widget::tooltip::Position::Bottom
                ),]
            } else {
                column![]
            }
        ],
        text_input(
            &format!(
                "New name for {}",
                state
                    .current_file
                    .as_ref()
                    .map(|note_path| note_path
                        .file_name()
                        .map(|os_str| os_str.to_str().unwrap_or("Unnamed"))
                        .unwrap_or("Unnamed"))
                    .unwrap_or("Unnamed")
            ),
            &state.current_rename_note_text
        )
        .on_input(|s| Message::Notes(NotesPageMessage::UpdateRenameNoteText(s)))
        .on_submit(Message::Notes(NotesPageMessage::RenameCurrentNote))
    ]
    .into()
}

fn manage_categories_view(_state: &NotesPage) -> Element<Message> {
    column![row![
        text("Manage Categories").width(Length::Fill).size(24),
        Tooltip::new(
            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                "../../../icons/close.svg"
            ))))
            .on_press(Message::Notes(NotesPageMessage::ToggleManageCategoriesView))
            .width(Length::Fixed(50.0)),
            "Close Categories Manager",
            iced::widget::tooltip::Position::Bottom
        ),
    ],]
    .into()
}
fn confirm_delete_view(_state: &NotesPage) -> Element<Message> {
    column![
        text("Delete This Note").width(Length::Fill).size(24),
        row![
            button(text("Delete").align_x(Center).width(Length::Fill))
                .width(Length::Fill)
                .style(button::danger)
                .on_press(Message::Notes(NotesPageMessage::DeleteCurrentFile)),
            button(text("Cancel").align_x(Center).width(Length::Fill))
                .width(Length::Fill)
                .on_press(Message::Notes(NotesPageMessage::ToggleConfirmDeleteView))
        ]
        .spacing(20)
    ]
    .into()
}

pub fn tool_view(state: &NotesPage) -> Element<Message> {
    let underlay = Tooltip::new(
        button(Svg::new(svg::Handle::from_memory(include_bytes!(
            "../../../icons/view-more.svg"
        ))))
        .on_press(Message::Notes(NotesPageMessage::ToggleExtraToolsMenu)),
        "More Tools",
        iced::widget::tooltip::Position::Bottom,
    );
    let overlay = column![
        button(
            text("Select Notes Folder")
                .width(Length::Fill)
                .align_x(Center),
        )
        .on_press(Message::Notes(NotesPageMessage::OpenFilePicker)),
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
            text(if !state.show_manage_categories_view {
                "Manage Categories"
            } else {
                "Hide Categories Manager"
            })
            .width(Length::Fill)
            .align_x(Center)
        )
        .on_press(Message::Notes(NotesPageMessage::ToggleManageCategoriesView))
        .width(Length::Fill),
        button(
            text(if !state.show_edit_note_details_view {
                "Rename Note"
            } else {
                "Hide Rename Panel"
            })
            .width(Length::Fill)
            .align_x(Center)
        )
        .on_press(Message::Notes(NotesPageMessage::ToggleEditNoteDetailsView))
        .width(Length::Fill),
        button(text("Delete Note").width(Length::Fill).align_x(Center))
            .on_press(if state.confirm_before_delete_note {
                Message::Notes(NotesPageMessage::ToggleConfirmDeleteView)
            } else {
                Message::Notes(NotesPageMessage::DeleteCurrentFile)
            })
            .width(Length::Fill)
            .style(button::danger),
    ]
    .width(Length::Fixed(200.0));

    let drop_down = DropDown::new(underlay, overlay, state.show_extra_tools_menu)
        .on_dismiss(Message::Notes(NotesPageMessage::ToggleExtraToolsMenu))
        .width(Length::Fill)
        .alignment(drop_down::Alignment::Bottom);
    row![
        Tooltip::new(
            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                "../../../icons/toggle-sidebar.svg"
            ))))
            .on_press(Message::Notes(NotesPageMessage::ToggleSidebar))
            .style(if state.show_sidebar {
                button::secondary
            } else {
                button::primary
            }),
            "Toggle Sidebar (Ctrl+B)",
            iced::widget::tooltip::Position::Bottom
        ),
        Tooltip::new(
            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                "../../../icons/markdown.svg"
            ))))
            .on_press(Message::Notes(NotesPageMessage::ToggleMarkdown))
            .style(if state.show_markdown {
                button::secondary
            } else {
                button::primary
            }),
            "Toggle Markdown Preview (Ctrl+M)",
            iced::widget::tooltip::Position::Bottom
        ),
        Tooltip::new(
            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                "../../../icons/editor.svg"
            ))))
            .on_press(Message::Notes(NotesPageMessage::ToggleEditor))
            .style(if state.show_editor {
                button::secondary
            } else {
                button::primary
            }),
            "Toggle Editor (Ctrl+E)",
            iced::widget::tooltip::Position::Bottom
        ),
        Tooltip::new(
            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                "../../../icons/add.svg"
            ))))
            .on_press(Message::Notes(NotesPageMessage::NewNote)),
            "New Note (Ctrl+N)",
            iced::widget::tooltip::Position::Bottom
        ),
        drop_down
    ]
    .width(Length::FillPortion(1))
    .into()
}
