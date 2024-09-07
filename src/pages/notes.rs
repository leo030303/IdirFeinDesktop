use iced_aw::{badge, style};
use std::fs;
use std::path::PathBuf;

use iced::alignment::Horizontal;
use iced::widget::{
    button, column, markdown, row, scrollable, text, text_editor, text_input, Scrollable, Svg,
    Tooltip,
};
use iced::{highlighter, Length, Task};
use iced::{Element, Fill, Font, Theme};
use rfd::FileDialog;

use crate::utils::notes_utils::{read_file_to_note, read_notes_from_folder};
use crate::Message;

// TODO Add ability to set category for note
// TODO Add ability to set category colours
// TODO Add category filter
// TODO Add word count
// TODO Sync scrolling between editor and preview
// TODO Autosave file
// TODO Rename file
// TODO Export as PDF
// TODO Lazy load notes list
// TODO Handle links correctly, websites open browser, within file jumps to that section, files that are markdown opens that file, images??

#[derive(Debug, Clone)]
pub struct Note {
    pub button_title: String,
    pub category: Option<String>,
    pub file_path: PathBuf,
    pub last_edited: u64,
}

pub struct NotesPage {
    editor_content: text_editor::Content,
    markdown_preview_items: Vec<markdown::Item>,
    theme: Theme,
    show_sidebar: bool,
    show_markdown: bool,
    show_editor: bool,
    notes_list: Vec<Note>,
    selected_folder: Option<PathBuf>,
    current_file: Option<PathBuf>,
    notes_list_filter: String,
    is_loading_note: bool,
}

#[derive(Debug, Clone)]
pub enum NotesPageMessage {
    Edit(text_editor::Action),
    LinkClicked(markdown::Url),
    ToggleSidebar,
    ToggleMarkdown,
    ToggleEditor,
    NewNote,
    SaveNote,
    OpenFilePicker,
    LoadFolderAsNotesList(Vec<Note>),
    SetTextEditorContent(String),
    OpenFile(PathBuf),
    FilterNotesList(String),
}

impl NotesPage {
    pub fn new() -> Self {
        const INITIAL_CONTENT: &str = include_str!("../../overview.md");

        let theme = Theme::TokyoNight;

        Self {
            editor_content: text_editor::Content::with_text(INITIAL_CONTENT),
            markdown_preview_items: markdown::parse(INITIAL_CONTENT).collect(),
            theme,
            show_sidebar: true,
            show_markdown: true,
            show_editor: true,
            selected_folder: None,
            current_file: None,
            notes_list: vec![],
            notes_list_filter: String::new(),
            is_loading_note: false,
        }
    }

    pub fn update(&mut self, message: NotesPageMessage) -> Task<Message> {
        match message {
            NotesPageMessage::Edit(action) => {
                let is_edit = action.is_edit();

                self.editor_content.perform(action);

                if is_edit {
                    self.markdown_preview_items =
                        markdown::parse(&self.editor_content.text()).collect();
                }
            }
            NotesPageMessage::LinkClicked(link) => {
                println!("{link}");
            }
            NotesPageMessage::ToggleSidebar => self.show_sidebar = !self.show_sidebar,
            NotesPageMessage::ToggleMarkdown => self.show_markdown = !self.show_markdown,
            NotesPageMessage::NewNote => {
                // Save current file content
                if let Some(current_file) = &self.current_file {
                    fs::write(current_file, self.editor_content.text()).unwrap();
                };
                self.current_file = None;
                self.editor_content = text_editor::Content::new();
                self.markdown_preview_items =
                    markdown::parse(&self.editor_content.text()).collect();
            }
            NotesPageMessage::SaveNote => todo!(),
            NotesPageMessage::OpenFilePicker => {
                let selected_folder = FileDialog::new().set_directory("/").pick_folder();
                self.selected_folder = selected_folder.clone();
                if let Some(selected_folder) = selected_folder {
                    return Task::perform(read_notes_from_folder(selected_folder), |notes_list| {
                        Message::Notes(NotesPageMessage::LoadFolderAsNotesList(notes_list))
                    });
                }
            }
            NotesPageMessage::OpenFile(new_filepath) => {
                self.is_loading_note = true;
                // Save current file content
                let old_filepath = self.current_file.take();
                self.current_file = Some(new_filepath.clone());
                return Task::perform(
                    read_file_to_note(new_filepath, old_filepath, self.editor_content.text()),
                    |new_content| {
                        Message::Notes(NotesPageMessage::SetTextEditorContent(new_content))
                    },
                );
            }
            NotesPageMessage::ToggleEditor => self.show_editor = !self.show_editor,
            NotesPageMessage::FilterNotesList(s) => self.notes_list_filter = s,
            NotesPageMessage::LoadFolderAsNotesList(notes_list) => {
                self.notes_list = notes_list;
                self.notes_list
                    .sort_unstable_by_key(|note| note.last_edited);
                self.notes_list.reverse();
            }
            NotesPageMessage::SetTextEditorContent(new_content) => {
                self.editor_content = text_editor::Content::with_text(&new_content);
                self.markdown_preview_items =
                    markdown::parse(&self.editor_content.text()).collect();
                self.is_loading_note = false;
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let editor = text_editor(&self.editor_content)
            .placeholder("Type your Markdown here...")
            .on_action(|action| Message::Notes(NotesPageMessage::Edit(action)))
            .height(Fill)
            .padding(10)
            .font(Font::MONOSPACE)
            .highlight("markdown", highlighter::Theme::Base16Ocean);

        let preview = markdown(
            &self.markdown_preview_items,
            markdown::Settings::default(),
            markdown::Style::from_palette(self.theme.palette()),
        )
        .map(|url| Message::Notes(NotesPageMessage::LinkClicked(url)));

        let sidebar: Element<Message> = if self.show_sidebar {
            if self.selected_folder.is_some() {
                column![
                    text_input("Filter", &self.notes_list_filter)
                        .on_input(|s| { Message::Notes(NotesPageMessage::FilterNotesList(s)) }),
                    Scrollable::new(
                        column(
                            self.notes_list
                                .iter()
                                .filter(|note| {
                                    note.button_title
                                        .to_lowercase()
                                        .contains(&self.notes_list_filter.to_lowercase())
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
                                    .style(if let Some(current_file) = &self.current_file {
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
            } else {
                button(
                    text("Select Notes Folder")
                        .width(Length::Fill)
                        .align_x(Horizontal::Center),
                )
                .on_press(Message::Notes(NotesPageMessage::OpenFilePicker))
                .width(Length::Fill)
                .into()
            }
        } else {
            column![].into()
        };
        row![
            sidebar,
            if self.show_editor {
                if self.is_loading_note {
                    column![text("Loading Note").size(24).width(Length::Fill),]
                        .spacing(20)
                        .height(Length::Shrink)
                } else {
                    column![editor]
                }
            } else {
                column![]
            },
            if self.show_markdown {
                if self.is_loading_note {
                    scrollable(
                        column![text("Loading Preview").size(24).width(Length::Fill),]
                            .spacing(20)
                            .height(Length::Shrink),
                    )
                } else {
                    scrollable(preview).spacing(10).height(Fill)
                }
            } else {
                scrollable(column![])
            }
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    pub fn tool_view(&self) -> Element<Message> {
        row![
            Tooltip::new(
                button(Svg::from_path("icons/toggle-sidebar.svg"))
                    .on_press(Message::Notes(NotesPageMessage::ToggleSidebar))
                    .style(if self.show_sidebar {
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
                    .style(if self.show_markdown {
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
                    .style(if self.show_editor {
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
        ]
        .width(Length::FillPortion(1))
        .into()
    }
}

impl Default for NotesPage {
    fn default() -> Self {
        Self::new()
    }
}
