use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use zspell::Dictionary;

use iced::event::Status;
use iced::keyboard::{Key, Modifiers};
use iced::widget::{markdown, text_editor};
use iced::{event, keyboard, time, Event, Subscription, Task};
use iced::{Element, Theme};
use loro::{LoroDoc, UndoManager};
use serde::{Deserialize, Serialize};

use crate::app::Message;
use crate::constants::APP_ID;

use super::notes_utils::NoteStatistics;
use super::update::update;
use super::view::{main_view, tool_view};

pub const CATEGORIES_FILE_NAME: &str = ".categories";
pub const ARCHIVED_FILE_NAME: &str = ".archived";
pub const TEXT_EDITOR_ID: &str = "TEXT_EDITOR_ID";
pub const NEW_NOTE_TEXT_INPUT_ID: &str = "NEW_NOTE_TEXT_INPUT_ID";
pub const RENAME_NOTE_TEXT_INPUT_ID: &str = "RENAME_NOTE_TEXT_INPUT_ID";
pub const LORO_NOTE_ID: &str = "LORO_NOTE_ID";
pub const INITIAL_ORIGIN_STR: &str = "initial";
pub const MAX_UNDO_STEPS: usize = 10000;

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
    pub autocomplete_brackets_etc: bool,
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
            autocomplete_brackets_etc: true,
            autocomplete_lists: true,
            website_folder: None,
        }
    }
}

pub struct NotesPage {
    pub(crate) editor_content: text_editor::Content,
    pub(crate) note_crdt: LoroDoc,
    pub(crate) undo_manager: UndoManager,
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
    pub(crate) new_note_title_entry_content: String,
    pub(crate) is_creating_new_note: bool,
    pub(crate) current_note_being_managed_path: Option<PathBuf>,
    pub(crate) display_rename_view: bool,
    pub(crate) rename_note_entry_text: String,
    pub(crate) display_delete_view: bool,
    pub(crate) categories_list: Vec<NoteCategory>,
    pub(crate) categorised_notes_list: HashMap<String, Vec<String>>,
    pub(crate) new_category_entry_text: String,
    pub(crate) current_color_picker_colour: iced::Color,
    pub(crate) show_colour_picker: bool,
    pub(crate) website_folder: Option<PathBuf>,
    pub(crate) autocomplete_brackets_etc: bool,
    pub(crate) spelling_corrections_list: Vec<(usize, String)>,
    pub(crate) show_spell_check_view: bool,
    pub(crate) spell_check_dictionary: Dictionary,
    pub(crate) display_archive_view: bool,
    pub(crate) archived_notes_list: Vec<String>,
    pub(crate) show_archived_notes: bool,
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
    SetAutoCompleteLists(bool),
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
    SetCategoriesList((Vec<NoteCategory>, HashMap<String, Vec<String>>)),
    SaveCategoriesList,
    AddCategory,
    DeleteCategory(String),
    SetNewCategoryText(String),
    SetColourPickerColour(iced::Color),
    ToggleColourPicker,
    SetWebsiteFolder(Option<PathBuf>),
    SetAutocompleteBrackets(bool),
    Undo,
    Redo,
    CalculateSpellingCorrectionsList,
    ToggleSpellCheckView,
    SetSpellingCorrectionsList(Vec<(usize, String)>),
    GoToSpellingMistake(usize, String),
    ArchiveNote,
    UnarchiveNote,
    ToggleArchiveNoteView,
    ToggleShowArchivedNotes,
    LoadArchivedList,
    CategoriseNote(Option<String>),
}

impl NotesPage {
    pub fn new(config: &NotesPageConfig) -> Self {
        let theme = Theme::TokyoNight;

        let loro_doc = LoroDoc::new();
        let mut undo_manager = UndoManager::new(&loro_doc);
        undo_manager.set_max_undo_steps(MAX_UNDO_STEPS);
        undo_manager.add_exclude_origin_prefix(INITIAL_ORIGIN_STR);

        let idirfein_data_dir = dirs::data_dir()
            .expect("Can't find data dir")
            .as_path()
            .join(APP_ID);
        let locale = current_locale::current_locale().expect("Can't get locale");
        let aff_content = fs::read_to_string(
            idirfein_data_dir
                .join("dictionaries")
                .join(locale.to_uppercase())
                .join("index.aff"),
        )
        .unwrap_or_default();

        let dic_content = fs::read_to_string(
            idirfein_data_dir
                .join("dictionaries")
                .join(locale.to_uppercase())
                .join("index.dic"),
        )
        .unwrap_or_default();

        let spell_check_dictionary: Dictionary = zspell::builder()
            .config_str(&aff_content)
            .dict_str(&dic_content)
            .build()
            .expect("failed to build dictionary!");

        Self {
            editor_content: text_editor::Content::with_text(""),
            undo_manager,
            note_crdt: loro_doc,
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
            autocomplete_lists: config.autocomplete_lists,
            new_note_title_entry_content: String::new(),
            is_creating_new_note: false,
            current_note_being_managed_path: None,
            display_rename_view: false,
            rename_note_entry_text: String::new(),
            display_delete_view: false,
            categories_list: vec![],
            new_category_entry_text: String::new(),
            current_color_picker_colour: iced::Color::default(),
            show_colour_picker: false,
            website_folder: config.website_folder.clone(),
            autocomplete_brackets_etc: config.autocomplete_brackets_etc,
            spelling_corrections_list: vec![],
            show_spell_check_view: false,
            spell_check_dictionary,
            display_archive_view: false,
            archived_notes_list: vec![],
            show_archived_notes: false,
            categorised_notes_list: HashMap::new(),
        }
    }

    pub fn opening_task() -> Task<Message> {
        Task::done(Message::Notes(NotesPageMessage::LoadFolderAsNotesList))
            .chain(Task::done(Message::Notes(NotesPageMessage::LoadCategories)))
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
                    } else if pressed_char.as_ref() == "z" || pressed_char.as_ref() == "Z" {
                        Some(Message::Notes(NotesPageMessage::Undo))
                    } else if pressed_char.as_ref() == "y" || pressed_char.as_ref() == "Y" {
                        Some(Message::Notes(NotesPageMessage::Redo))
                    } else if pressed_char.as_ref() == "k" || pressed_char.as_ref() == "K" {
                        Some(Message::Notes(NotesPageMessage::ToggleSpellCheckView))
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
