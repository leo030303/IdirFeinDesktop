use crate::LOCALES;
use fluent_templates::Loader;
use iced::Alignment::Center;
use iced_aw::{drop_down, DropDown};

use iced::alignment::Horizontal;
use iced::widget::{
    button, column, markdown, row, scrollable, svg, text, text_editor, text_input, Scrollable,
    Space, Svg, Tooltip,
};
use iced::{highlighter, Length};
use iced::{Element, Fill, Font};

use crate::app::Message;
use crate::pages::notes::page::NEW_NOTE_TEXT_INPUT_ID;

use super::page::{Note, NotesPage, NotesPageMessage, RENAME_NOTE_TEXT_INPUT_ID, TEXT_EDITOR_ID};

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
            row![if state.show_document_statistics_view {
                if state.is_loading_note {
                    loading_note_view(state)
                } else {
                    document_statistics_view(state)
                }
            } else {
                column![].into()
            },]
            .spacing(10),
            if state.current_file.is_some() {
                row![
                    if state.show_spell_check_view {
                        if state.is_loading_note {
                            loading_note_view(state)
                        } else {
                            spell_check_view(state)
                        }
                    } else {
                        column![].into()
                    },
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
                        column![
                            text(LOCALES.lookup(&state.locale, "open-editor-or-preview"))
                                .size(24)
                                .width(Length::Fill)
                                .height(Length::Fill)
                        ]
                    } else {
                        column![]
                    }
                ]
                .spacing(10)
                .into()
            } else {
                markdown_guide_view(state)
            }
        ]
        .spacing(10)
        .width(Length::FillPortion(2))
    ]
    .spacing(10)
    .padding(10)
    .into()
}

fn new_note_button(state: &NotesPage) -> Element<Message> {
    if state.is_creating_new_note {
        row![
            text_input(
                &LOCALES.lookup(&state.locale, "new-note-title"),
                &state.new_note_title_entry_content
            )
            .width(Length::Fill)
            .on_input(|s| Message::Notes(NotesPageMessage::UpdateNewNoteTitleEntry(s)))
            .on_submit(Message::Notes(NotesPageMessage::CreateNewNote))
            .id(text_input::Id::new(NEW_NOTE_TEXT_INPUT_ID)),
            Tooltip::new(
                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/ok.svg"
                ))))
                .on_press(Message::Notes(NotesPageMessage::CreateNewNote))
                .style(button::success)
                .width(Length::Fixed(50.0))
                .height(Length::Fixed(30.0)),
                text(LOCALES.lookup(&state.locale, "create")),
                iced::widget::tooltip::Position::Bottom
            ),
            Tooltip::new(
                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/close.svg"
                ))))
                .on_press(Message::Notes(NotesPageMessage::CancelCreateNewNote))
                .style(button::danger)
                .width(Length::Fixed(50.0))
                .height(Length::Fixed(30.0)),
                text(LOCALES.lookup(&state.locale, "cancel")),
                iced::widget::tooltip::Position::Bottom
            ),
        ]
        .spacing(5)
    } else {
        row![
            button(
                text(if state.show_archived_notes {
                    LOCALES.lookup(&state.locale, "hide-archived")
                } else {
                    LOCALES.lookup(&state.locale, "show-archived")
                })
                .width(Length::Fill)
                .align_x(Center)
            )
            .width(Length::Fill)
            .on_press(Message::Notes(NotesPageMessage::ToggleShowArchivedNotes)),
            button(
                text(LOCALES.lookup(&state.locale, "new-note-shortcut"))
                    .width(Length::Fill)
                    .align_x(Center)
            )
            .width(Length::Fill)
            .style(button::success)
            .on_press(Message::Notes(NotesPageMessage::StartCreatingNewNote))
        ]
        .spacing(5)
    }
    .into()
}

fn rename_note_view(state: &NotesPage) -> Element<Message> {
    row![
        text_input(
            &LOCALES.lookup(&state.locale, "rename-note"),
            &state.rename_note_entry_text
        )
        .width(Length::Fill)
        .on_input(|s| Message::Notes(NotesPageMessage::SetRenameNoteText(s)))
        .on_submit(Message::Notes(NotesPageMessage::RenameNote))
        .id(text_input::Id::new(RENAME_NOTE_TEXT_INPUT_ID)),
        Tooltip::new(
            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                "../../../icons/ok.svg"
            ))))
            .on_press(Message::Notes(NotesPageMessage::RenameNote))
            .style(button::success)
            .width(Length::Fixed(50.0))
            .height(Length::Fixed(30.0)),
            text(LOCALES.lookup(&state.locale, "rename")),
            iced::widget::tooltip::Position::Bottom
        ),
        Tooltip::new(
            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                "../../../icons/close.svg"
            ))))
            .on_press(Message::Notes(NotesPageMessage::ToggleRenameNoteView))
            .style(button::danger)
            .width(Length::Fixed(50.0))
            .height(Length::Fixed(30.0)),
            text(LOCALES.lookup(&state.locale, "cancel")),
            iced::widget::tooltip::Position::Bottom
        ),
    ]
    .spacing(5)
    .into()
}

fn confirm_delete_note_view(state: &NotesPage) -> Element<Message> {
    row![
        button(
            text(LOCALES.lookup(&state.locale, "delete"))
                .width(Length::Fill)
                .align_x(Center)
        )
        .style(button::danger)
        .width(Length::Fill)
        .on_press(Message::Notes(NotesPageMessage::DeleteNote)),
        button(
            text(LOCALES.lookup(&state.locale, "cancel"))
                .width(Length::Fill)
                .align_x(Center)
        )
        .width(Length::Fill)
        .on_press(Message::Notes(NotesPageMessage::ToggleDeleteNoteView)),
    ]
    .spacing(5)
    .into()
}

fn spell_check_view(state: &NotesPage) -> Element<Message> {
    column![
        button(
            text(LOCALES.lookup(&state.locale, "run-spell-check"))
                .width(Length::Fill)
                .align_x(Center)
        )
        .on_press(Message::Notes(
            NotesPageMessage::CalculateSpellingCorrectionsList
        )),
        if state.spelling_corrections_list.is_empty() {
            column![text(LOCALES.lookup(&state.locale, "no-errors"))
                .width(Length::Fill)
                .align_x(Center)]
        } else {
            column![Scrollable::new(
                column(
                    state
                        .spelling_corrections_list
                        .iter()
                        .map(|spelling_mistake_string| {
                            text(format!("\n{spelling_mistake_string}"))
                                .font(Font {
                                    style: iced::font::Style::Italic,
                                    ..Default::default()
                                })
                                .into()
                        },)
                )
                .width(Length::Fill)
            )]
        },
    ]
    .into()
}

fn manage_note_options_view(state: &NotesPage) -> Element<Message> {
    row![
        button(
            text(LOCALES.lookup(&state.locale, "rename"))
                .width(Length::Fill)
                .align_x(Center)
        )
        .width(Length::Fill)
        .on_press(Message::Notes(NotesPageMessage::ToggleRenameNoteView)),
        button(
            text(LOCALES.lookup(&state.locale, "archive"))
                .width(Length::Fill)
                .align_x(Center)
        )
        .width(Length::Fill)
        .on_press(Message::Notes(NotesPageMessage::ToggleArchiveNoteView)),
        button(
            text(LOCALES.lookup(&state.locale, "delete"))
                .width(Length::Fill)
                .align_x(Center)
        )
        .style(button::danger)
        .width(Length::Fill)
        .on_press(Message::Notes(if state.confirm_before_delete_note {
            NotesPageMessage::ToggleDeleteNoteView
        } else {
            NotesPageMessage::DeleteNote
        })),
        Tooltip::new(
            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                "../../../icons/close.svg"
            ))))
            .on_press(Message::Notes(NotesPageMessage::ShowMenuForNote(None)))
            .width(Length::Fixed(50.0))
            .height(Length::Fixed(30.0)),
            text(LOCALES.lookup(&state.locale, "close")),
            iced::widget::tooltip::Position::Right,
        )
    ]
    .spacing(5)
    .into()
}

fn sidebar_note_button<'a>(state: &'a NotesPage, note: &'a Note) -> Element<'a, Message> {
    row![
        button(
            row![text(note.button_title.clone())
                .font(Font {
                    weight: iced::font::Weight::Semibold,
                    ..Default::default()
                })
                .width(Length::Fill)
                .align_x(Horizontal::Center)]
            .align_y(Center)
        )
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
        .width(Length::Fill),
        Tooltip::new(
            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                "../../../icons/view-more.svg"
            ))))
            .on_press(Message::Notes(NotesPageMessage::ShowMenuForNote(Some(
                note.file_path.to_path_buf()
            ))))
            .height(Length::Fixed(30.0))
            .width(Length::Fixed(50.0)),
            text(LOCALES.lookup(&state.locale, "manage-details")),
            iced::widget::tooltip::Position::Right,
        )
    ]
    .spacing(5)
    .into()
}

fn sidebar_with_selected_folder(state: &NotesPage) -> Element<Message> {
    column![
        new_note_button(state),
        Space::with_height(20),
        text_input(
            &LOCALES.lookup(&state.locale, "filter"),
            &state.notes_list_filter
        )
        .on_input(|s| { Message::Notes(NotesPageMessage::FilterNotesList(s)) }),
        Scrollable::new(
            column(
                state
                    .notes_list
                    .iter()
                    .filter(|note| !state.archived_notes_list.contains(
                        &note
                            .file_path
                            .file_stem()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or(&LOCALES.lookup(&state.locale, "couldnt-read-filename"))
                            .to_lowercase()
                    ) ^ state.show_archived_notes)
                    .filter(|note| {
                        note.button_title
                            .to_lowercase()
                            .contains(&state.notes_list_filter.to_lowercase())
                    })
                    .map(|note| {
                        {
                            if state
                                .current_note_being_managed_path
                                .clone()
                                .is_some_and(|selected_note| selected_note == note.file_path)
                            {
                                if state.show_archived_notes {
                                    row![
                                        button(
                                            text(LOCALES.lookup(&state.locale, "unarchive"))
                                                .width(Length::Fill)
                                                .align_x(Center)
                                        )
                                        .width(Length::Fill)
                                        .on_press(Message::Notes(NotesPageMessage::UnarchiveNote)),
                                        Tooltip::new(
                                            button(Svg::new(svg::Handle::from_memory(
                                                include_bytes!("../../../icons/close.svg")
                                            )))
                                            .on_press(Message::Notes(
                                                NotesPageMessage::ShowMenuForNote(None)
                                            ))
                                            .width(Length::Fixed(50.0))
                                            .height(Length::Fixed(30.0)),
                                            text(LOCALES.lookup(&state.locale, "close")),
                                            iced::widget::tooltip::Position::Right,
                                        )
                                    ]
                                    .spacing(5)
                                    .into()
                                } else if state.display_archive_view {
                                    row![
                                        button(
                                            text(LOCALES.lookup(&state.locale, "archive"))
                                                .width(Length::Fill)
                                                .align_x(Center)
                                        )
                                        .style(button::danger)
                                        .width(Length::Fill)
                                        .on_press(Message::Notes(NotesPageMessage::ArchiveNote)),
                                        button(
                                            text(LOCALES.lookup(&state.locale, "cancel"))
                                                .width(Length::Fill)
                                                .align_x(Center)
                                        )
                                        .width(Length::Fill)
                                        .on_press(
                                            Message::Notes(NotesPageMessage::ToggleArchiveNoteView)
                                        ),
                                    ]
                                    .spacing(5)
                                    .into()
                                } else if state.display_rename_view {
                                    rename_note_view(state)
                                } else if state.display_delete_view {
                                    confirm_delete_note_view(state)
                                } else {
                                    manage_note_options_view(state)
                                }
                            } else {
                                sidebar_note_button(state, note)
                            }
                        }
                    }),
            )
            .spacing(5)
        )
        .spacing(5)
    ]
    .into()
}

fn sidebar_without_selected_folder(state: &NotesPage) -> Element<Message> {
    button(
        text(LOCALES.lookup(&state.locale, "select-notes-folder"))
            .width(Length::Fill)
            .align_x(Horizontal::Center),
    )
    .on_press(Message::Notes(NotesPageMessage::OpenFilePicker))
    .width(Length::Fill)
    .into()
}

fn loading_note_view(state: &NotesPage) -> Element<Message> {
    column![text(LOCALES.lookup(&state.locale, "loading-note"))
        .size(24)
        .width(Length::Fill),]
    .spacing(20)
    .height(Length::Shrink)
    .into()
}

fn editor_view(state: &NotesPage) -> Element<Message> {
    column![text_editor(&state.editor_content)
        .id(TEXT_EDITOR_ID)
        .placeholder(LOCALES.lookup(&state.locale, "type-your-markdown-here"))
        .on_action(|action| Message::Notes(NotesPageMessage::Edit(action)))
        .height(Fill)
        .padding(10)
        .font(Font::MONOSPACE)
        .highlight("markdown", highlighter::Theme::Base16Ocean)]
    .into()
}

fn loading_preview_view(state: &NotesPage) -> Element<Message> {
    column![text(LOCALES.lookup(&state.locale, "loading-preview"))
        .size(24)
        .width(Length::Fill),]
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
            text(LOCALES.lookup(&state.locale, "document-statistics"))
                .width(Length::Fill)
                .size(24),
            Tooltip::new(
                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/close.svg"
                ))))
                .on_press(Message::Notes(
                    NotesPageMessage::ToggleDocumentStatisticsView
                ))
                .width(Length::Fixed(50.0)),
                text(LOCALES.lookup(&state.locale, "close-statistics")),
                iced::widget::tooltip::Position::Bottom
            ),
        ],
        text(format!(
            "{} {}",
            LOCALES.lookup(&state.locale, "character-count"),
            state.current_note_statistics.char_count
        )),
        text(format!(
            "{} {}",
            LOCALES.lookup(&state.locale, "word-count"),
            state.current_note_statistics.word_count
        )),
        text(format!(
            "{} {}",
            LOCALES.lookup(&state.locale, "reading-time"),
            state.current_note_statistics.reading_time_in_mins
        )),
        Tooltip::new(
            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                "../../../icons/refresh.svg"
            ))))
            .on_press(Message::Notes(NotesPageMessage::CalculateNoteStatistics))
            .width(Length::Fill),
            text(LOCALES.lookup(&state.locale, "refresh-statistics")),
            iced::widget::tooltip::Position::Bottom
        ),
    ]
    .into()
}

fn markdown_guide_view(state: &NotesPage) -> Element<Message> {
    scrollable(
        markdown(
            &state.markdown_guide_items,
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
    let underlay = Tooltip::new(
        button(Svg::new(svg::Handle::from_memory(include_bytes!(
            "../../../icons/view-more.svg"
        ))))
        .on_press(Message::Notes(NotesPageMessage::ToggleExtraToolsMenu)),
        text(LOCALES.lookup(&state.locale, "more-tools")),
        iced::widget::tooltip::Position::Bottom,
    );
    let overlay = column![
        button(
            text(LOCALES.lookup(&state.locale, "select-notes-folder"))
                .width(Length::Fill)
                .align_x(Center),
        )
        .on_press(Message::Notes(NotesPageMessage::OpenFilePicker)),
        button(
            text(LOCALES.lookup(&state.locale, "export-pdf"))
                .width(Length::Fill)
                .align_x(Center)
        )
        .on_press(Message::Notes(NotesPageMessage::ExportPDF))
        .width(Length::Fill),
        button(
            text(LOCALES.lookup(&state.locale, "post-to-website"))
                .width(Length::Fill)
                .align_x(Center)
        )
        .on_press(Message::Notes(NotesPageMessage::ExportToWebsite))
        .width(Length::Fill),
        button(
            text(LOCALES.lookup(&state.locale, "open-website-styles-file"))
                .width(Length::Fill)
                .align_x(Center)
        )
        .on_press(Message::Notes(NotesPageMessage::OpenWebsiteStylesFile))
        .width(Length::Fill),
        button(
            text(if !state.show_document_statistics_view {
                LOCALES.lookup(&state.locale, "show-statistics")
            } else {
                LOCALES.lookup(&state.locale, "hide-statistics")
            })
            .width(Length::Fill)
            .align_x(Center)
        )
        .on_press(Message::Notes(
            NotesPageMessage::ToggleDocumentStatisticsView
        ))
        .width(Length::Fill),
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
            text(LOCALES.lookup(&state.locale, "toggle-sidebar-shortcut")),
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
            text(LOCALES.lookup(&state.locale, "toggle-markdown-preview-shortcut")),
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
            text(LOCALES.lookup(&state.locale, "toggle-editor-shortcut")),
            iced::widget::tooltip::Position::Bottom
        ),
        Tooltip::new(
            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                "../../../icons/spell_check.svg"
            ))))
            .on_press(Message::Notes(NotesPageMessage::ToggleSpellCheckView))
            .style(if state.show_spell_check_view {
                button::secondary
            } else {
                button::primary
            }),
            text(LOCALES.lookup(&state.locale, "toggle-spell-check-view-shortcut")),
            iced::widget::tooltip::Position::Bottom
        ),
        drop_down
    ]
    .width(Length::FillPortion(1))
    .into()
}
