use std::path::PathBuf;
use std::time::Duration;

use iced::event::Status;
use iced::keyboard::{Key, Modifiers};
use iced::widget::{markdown, text_editor};
use iced::{event, keyboard, time, Event, Subscription, Task};
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
// TODO Export as HTML and add to website

pub const NEW_NOTE_TEXT_INPUT_ID: &str = "NEW_NOTE_TEXT_INPUT_ID";
pub const RENAME_NOTE_TEXT_INPUT_ID: &str = "RENAME_NOTE_TEXT_INPUT_ID";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteCategory {
    pub name: String,
    pub colour: SerializableColour,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SerializableColour {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub transparency: f32,
}

impl SerializableColour {
    pub fn from_iced_color(value: iced::Color) -> Self {
        Self {
            red: value.r,
            green: value.g,
            blue: value.b,
            transparency: value.a,
        }
    }

    pub fn to_iced_colour(&self) -> iced::Color {
        iced::Color::from_rgba(self.red, self.green, self.blue, self.transparency)
    }
}

#[derive(Debug, Clone)]
pub struct Note {
    pub button_title: String,
    pub category_name: Option<String>,
    pub file_path: PathBuf,
    pub last_edited: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotesPageConfig {
    pub default_folder: Option<PathBuf>,
    pub show_sidebar_on_start: bool,
    pub show_markdown_on_start: bool,
    pub show_editor_on_start: bool,
    pub confirm_before_delete: bool,
    pub show_format_toolbar: bool,
    pub autocomplete_lists: bool,
    pub website_folder: Option<PathBuf>,
}

impl Default for NotesPageConfig {
    fn default() -> Self {
        Self {
            default_folder: None,
            show_sidebar_on_start: true,
            show_markdown_on_start: true,
            show_editor_on_start: true,
            confirm_before_delete: true,
            show_format_toolbar: true,
            autocomplete_lists: true,
            website_folder: None,
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
    pub(crate) show_manage_categories_view: bool,
    pub(crate) current_note_statistics: NoteStatistics,
    pub(crate) confirm_before_delete_note: bool,
    pub(crate) note_is_dirty: bool,
    pub(crate) autocomplete_lists: bool,
    pub(crate) show_format_toolbar: bool,
    pub(crate) new_note_title_entry_content: String,
    pub(crate) is_creating_new_note: bool,
    pub(crate) current_note_being_managed_path: Option<PathBuf>,
    pub(crate) display_rename_view: bool,
    pub(crate) rename_note_entry_text: String,
    pub(crate) display_delete_view: bool,
    pub(crate) archived_notes_list: Vec<PathBuf>,
    pub(crate) categories_list: Vec<NoteCategory>,
    pub(crate) new_category_entry_text: String,
    pub(crate) current_color_picker_colour: iced::Color,
    pub(crate) show_colour_picker: bool,
    pub(crate) website_folder: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum NotesPageMessage {
    Edit(text_editor::Action),
    LinkClicked(markdown::Url),
    ToggleSidebar,
    ToggleMarkdown,
    ToggleEditor,
    SaveNote,
    OpenFilePicker,
    SetNotesFolder(Option<PathBuf>),
    SetNotesList(Vec<Note>),
    SetTextEditorContent(String),
    OpenFile(PathBuf),
    FilterNotesList(String),
    ToggleExtraToolsMenu,
    ExportPDF,
    ExportToWebsite,
    ToggleDocumentStatisticsView,
    ToggleManageCategoriesView,
    CalculateNoteStatistics,
    SetNoteStatistics(NoteStatistics),
    LoadFolderAsNotesList,
    InsertTitle,
    SetAutoCompleteLists(bool),
    SetShowFormatToolbar(bool),
    SetConfirmBeforeDelete(bool),
    CreateNewNote,
    UpdateNewNoteTitleEntry(String),
    CancelCreateNewNote,
    StartCreatingNewNote,
    SetRenameNoteText(String),
    RenameNote,
    ToggleRenameNoteView,
    DeleteNote,
    ToggleDeleteNoteView,
    ShowMenuForNote(Option<PathBuf>),
    LoadCategories,
    SetCategoriesList(Vec<NoteCategory>),
    SaveCategoriesList,
    AddCategory,
    DeleteCategory,
    LoadArchivedNotesList,
    SetArchivedNotesList(Vec<PathBuf>),
    SaveArchivedNotesList,
    SetNewCategoryText(String),
    SetColourPickerColour(iced::Color),
    ToggleColourPicker,
    SetWebsiteFolder(Option<PathBuf>),
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
            current_note_statistics: NoteStatistics {
                char_count: 0,
                word_count: 0,
                reading_time_in_mins: 0,
            },
            show_manage_categories_view: false,
            confirm_before_delete_note: config.confirm_before_delete,
            note_is_dirty: false,
            show_format_toolbar: config.show_format_toolbar,
            autocomplete_lists: config.autocomplete_lists,
            new_note_title_entry_content: String::new(),
            is_creating_new_note: false,
            current_note_being_managed_path: None,
            display_rename_view: false,
            rename_note_entry_text: String::new(),
            display_delete_view: false,
            archived_notes_list: vec![],
            categories_list: vec![],
            new_category_entry_text: String::new(),
            current_color_picker_colour: iced::Color::default(),
            show_colour_picker: false,
            website_folder: config.website_folder.clone(),
        }
    }

    pub fn opening_task() -> Task<Message> {
        Task::done(Message::Notes(NotesPageMessage::LoadFolderAsNotesList))
            .chain(Task::done(Message::Notes(NotesPageMessage::LoadCategories)))
            .chain(Task::done(Message::Notes(
                NotesPageMessage::LoadArchivedNotesList,
            )))
    }

    pub fn closing_task(&mut self) -> Task<Message> {
        Task::done(Message::Notes(NotesPageMessage::SaveNote))
    }

    pub fn update(&mut self, message: NotesPageMessage) -> Task<Message> {
        update(self, message)
    }

    pub fn view(&self) -> Element<Message> {
        main_view(self)
    }

    pub fn subscription() -> iced::Subscription<Message> {
        Subscription::batch([
            event::listen_with(|event, status, _id| match (event, status) {
                (
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key: Key::Character(pressed_char),
                        modifiers: Modifiers::CTRL,
                        ..
                    }),
                    Status::Ignored,
                ) => {
                    if pressed_char.as_ref() == "n" || pressed_char.as_ref() == "N" {
                        Some(Message::Notes(NotesPageMessage::StartCreatingNewNote))
                    } else if pressed_char.as_ref() == "b" || pressed_char.as_ref() == "B" {
                        Some(Message::Notes(NotesPageMessage::ToggleSidebar))
                    } else if pressed_char.as_ref() == "m" || pressed_char.as_ref() == "M" {
                        Some(Message::Notes(NotesPageMessage::ToggleMarkdown))
                    } else if pressed_char.as_ref() == "e" || pressed_char.as_ref() == "E" {
                        Some(Message::Notes(NotesPageMessage::ToggleEditor))
                    } else {
                        None
                    }
                }
                _ => None,
            }),
            time::every(Duration::from_secs(3)).map(|_| Message::Notes(NotesPageMessage::SaveNote)),
        ])
    }

    pub fn tool_view(&self) -> Element<Message> {
        tool_view(self)
    }
}
