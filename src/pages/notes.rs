use std::fs;
use std::os::linux::fs::MetadataExt;
use std::path::PathBuf;

use iced::alignment::Horizontal;
use iced::widget::{
    button, column, markdown, row, scrollable, text, text_editor, Scrollable, Svg, Tooltip,
};
use iced::{highlighter, Length};
use iced::{Element, Fill, Font, Theme};
use rfd::FileDialog;

use crate::Message;

// TODO Select folder in sidebar if none selected
// TODO Async file load
// TODO Async folder dir list
// TODO List of all markdown files from folder in sidebar
// TODO Sync scrolling between editor and preview
// TODO Filter files in sidebar
// TODO Handles categorised, notes in folders
// TODO Autosave file
// TODO Rename file
// TODO Handle links correctly, websites open browser, within file jumps to that section, files that are markdown opens that file, images??

fn take_first_n_chars(input: &str, n: usize) -> String {
    let end_index = input
        .char_indices()
        .map(|(i, _)| i)
        .take(n)
        .last()
        .unwrap_or(input.len());

    input[..end_index].to_string()
}

struct Note {
    pub button_title: String,
    pub file_path: PathBuf,
    pub last_edited: i64,
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
    OpenFile(PathBuf),
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
        }
    }

    pub fn update(&mut self, message: NotesPageMessage) {
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
                    self.notes_list = fs::read_dir(selected_folder)
                        .unwrap()
                        .map(|file_path_option| {
                            let file_path = file_path_option.unwrap();
                            Note {
                                button_title: take_first_n_chars(
                                    file_path.path().file_stem().unwrap().to_str().unwrap(),
                                    30,
                                ),
                                file_path: file_path.path().clone(),
                                last_edited: file_path.metadata().unwrap().st_ctime(),
                            }
                        })
                        .collect();
                    self.notes_list
                        .sort_unstable_by_key(|note| note.last_edited);
                }
            }
            NotesPageMessage::OpenFile(path) => {
                // Save current file content
                if let Some(current_file) = &self.current_file {
                    fs::write(current_file, self.editor_content.text()).unwrap();
                };
                self.editor_content =
                    text_editor::Content::with_text(&fs::read_to_string(path.as_path()).unwrap());
                self.markdown_preview_items =
                    markdown::parse(&self.editor_content.text()).collect();
                self.current_file = Some(path);
            }
            NotesPageMessage::ToggleEditor => self.show_editor = !self.show_editor,
        }
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
                Scrollable::new(column(self.notes_list.iter().map(|note| {
                    button(
                        text(note.button_title.clone())
                            .font(Font {
                                weight: iced::font::Weight::Semibold,
                                ..Default::default()
                            })
                            .width(Length::Fill)
                            .align_x(Horizontal::Center),
                    )
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
                })))
                .into()
            } else {
                button("Select Notes Folder")
                    .on_press(Message::Notes(NotesPageMessage::OpenFilePicker))
                    .into()
            }
        } else {
            column![].into()
        };
        row![
            sidebar,
            if self.show_editor {
                column![editor]
            } else {
                column![]
            },
            if self.show_markdown {
                scrollable(preview).spacing(10).height(Fill)
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
