use std::path::PathBuf;

use iced::widget::{markdown, text_editor};
use iced::Task;
use iced::{Element, Theme};
use serde::{Deserialize, Serialize};

use crate::app::Message;

use super::notes_utils::NoteStatistics;
use super::update::update;
use super::view::{main_view, tool_view};

// TODO Make it all look nice
// TODO Format shortcut bar to insert markdown items
// TODO Add ability to set category for note
// TODO Add ability to set category colours
// TODO Add category filter
// TODO Sync scrolling between editor and preview
// TODO Autosave file
// TODO Rename file
// TODO Export as HTML and add to website
// TODO Lazy load notes list

#[derive(Debug, Clone)]
pub struct Note {
    pub button_title: String,
    pub category: Option<String>,
    pub file_path: PathBuf,
    pub last_edited: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotesPageConfig {
    pub default_folder: Option<PathBuf>,
    pub show_sidebar_on_start: bool,
    pub show_markdown_on_start: bool,
    pub show_editor_on_start: bool,
}

impl Default for NotesPageConfig {
    fn default() -> Self {
        Self {
            default_folder: None,
            show_sidebar_on_start: true,
            show_markdown_on_start: true,
            show_editor_on_start: true,
        }
    }
}

pub struct NotesPage {
    pub(crate) editor_content: text_editor::Content,
    pub(crate) markdown_preview_items: Vec<markdown::Item>,
    pub(crate) theme: Theme,
    pub(crate) show_sidebar: bool,
    pub(crate) show_markdown: bool,
    pub(crate) show_editor: bool,
    pub(crate) notes_list: Vec<Note>,
    pub(crate) selected_folder: Option<PathBuf>,
    pub(crate) current_file: Option<PathBuf>,
    pub(crate) notes_list_filter: String,
    pub(crate) is_loading_note: bool,
    pub(crate) show_extra_tools_menu: bool,
    pub(crate) show_document_statistics_view: bool,
    pub(crate) show_rename_note_view: bool,
    pub(crate) current_note_statistics: NoteStatistics,
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
    SetNotesList(Vec<Note>),
    SetTextEditorContent(String),
    OpenFile(PathBuf),
    FilterNotesList(String),
    ToggleExtraToolsMenu,
    ExportPDF,
    ExportToWebsite,
    ToggleDocumentStatisticsView,
    ToggleRenameNoteView,
    CalculateNoteStatistics,
    SetNoteStatistics(NoteStatistics),
    LoadFolderAsNotesList,
}

impl NotesPage {
    pub fn new(config: &NotesPageConfig) -> Self {
        let theme = Theme::TokyoNight;

        Self {
            editor_content: text_editor::Content::with_text(""),
            markdown_preview_items: markdown::parse("").collect(),
            theme,
            show_sidebar: config.show_sidebar_on_start,
            show_markdown: config.show_markdown_on_start,
            show_editor: config.show_editor_on_start,
            selected_folder: config.default_folder.clone(),
            current_file: None,
            notes_list: vec![],
            notes_list_filter: String::new(),
            is_loading_note: false,
            show_extra_tools_menu: false,
            show_document_statistics_view: false,
            show_rename_note_view: false,
            current_note_statistics: NoteStatistics {
                char_count: 0,
                word_count: 0,
                reading_time_in_mins: 0,
            },
        }
    }

    pub fn closing_task(&mut self) -> Task<Message> {
        println!("Closing task from notes");
        Task::none()
    }

    pub fn update(&mut self, message: NotesPageMessage) -> Task<Message> {
        update(self, message)
    }

    pub fn view(&self) -> Element<Message> {
        main_view(self)
    }

    pub fn tool_view(&self) -> Element<Message> {
        tool_view(self)
    }
}
