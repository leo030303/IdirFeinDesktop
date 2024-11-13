use iced::{
    widget::{markdown, text_editor, text_input},
    Task,
};
use loro::{LoroDoc, UndoManager, VersionVector};
use rfd::FileDialog;
use std::{fs, path::PathBuf};

use crate::{
    app::Message,
    pages::notes::notes_utils::{move_cursor_to_position, parse_markdown_lists},
};

use super::{
    notes_utils::{
        apply_edit_to_note, export_pdf, export_to_website, read_file_to_note,
        read_notes_from_folder, NoteStatistics,
    },
    page::{
        NoteCategory, NotesPage, NotesPageMessage, SerializableColour, INITIAL_ORIGIN_STR,
        LORO_NOTE_ID, MAX_UNDO_STEPS, NEW_NOTE_TEXT_INPUT_ID, RENAME_NOTE_TEXT_INPUT_ID,
    },
};

pub fn update(state: &mut NotesPage, message: NotesPageMessage) -> Task<Message> {
    match message {
        NotesPageMessage::Edit(action) => {
            let is_edit = action.is_edit();

            let mut is_action_performed = false;

            if state.autocomplete_brackets_etc {
                if let text_editor::Action::Edit(text_editor::Edit::Insert(inserted_char)) = action
                {
                    match inserted_char {
                        '(' => {
                            apply_edit_to_note(state, text_editor::Edit::Insert('('));
                            apply_edit_to_note(state, text_editor::Edit::Insert(')'));
                            is_action_performed = true;
                        }
                        '[' => {
                            apply_edit_to_note(state, text_editor::Edit::Insert('['));
                            apply_edit_to_note(state, text_editor::Edit::Insert(']'));
                            is_action_performed = true;
                        }
                        '{' => {
                            apply_edit_to_note(state, text_editor::Edit::Insert('{'));
                            apply_edit_to_note(state, text_editor::Edit::Insert('}'));
                            is_action_performed = true;
                        }
                        '"' => {
                            apply_edit_to_note(state, text_editor::Edit::Insert('"'));
                            apply_edit_to_note(state, text_editor::Edit::Insert('"'));
                            is_action_performed = true;
                        }
                        '`' => {
                            apply_edit_to_note(state, text_editor::Edit::Insert('`'));
                            apply_edit_to_note(state, text_editor::Edit::Insert('`'));
                            is_action_performed = true;
                        }
                        _ => (),
                    }
                }
            }
            if state.autocomplete_lists {
                if let text_editor::Action::Edit(text_editor::Edit::Enter) = action {
                    let list_action = parse_markdown_lists(state);
                    match list_action {
                        crate::pages::notes::notes_utils::ListAction::NoAction => {
                            apply_edit_to_note(state, text_editor::Edit::Enter);
                        }
                        crate::pages::notes::notes_utils::ListAction::AddUnorderedListItem {
                            list_char,
                            indent_amount,
                        } => {
                            apply_edit_to_note(state, text_editor::Edit::Enter);
                            for _ in 0..indent_amount {
                                apply_edit_to_note(state, text_editor::Edit::Insert(' '));
                            }
                            apply_edit_to_note(state, text_editor::Edit::Insert(list_char));
                            apply_edit_to_note(state, text_editor::Edit::Insert(' '));
                        }
                        crate::pages::notes::notes_utils::ListAction::DeleteUnorderedListItem {
                            cursor_x_pos: cursor_position_in_line,
                            indent_amount,
                        } => {
                            let num_chars_to_remove = indent_amount + 2;
                            for _ in 0..cursor_position_in_line {
                                apply_edit_to_note(state, text_editor::Edit::Backspace);
                            }
                            for _ in cursor_position_in_line..num_chars_to_remove {
                                apply_edit_to_note(state, text_editor::Edit::Delete);
                            }
                        }
                    }
                    is_action_performed = true;
                }
            }
            if !is_action_performed {
                if let text_editor::Action::Edit(edit_action) = action {
                    apply_edit_to_note(state, edit_action);
                } else {
                    state.editor_content.perform(action);
                }
            }

            if is_edit {
                state.note_is_dirty = true;

                state.markdown_preview_items =
                    markdown::parse(&state.editor_content.text()).collect();
            }
        }
        NotesPageMessage::LinkClicked(link) => {
            opener::open(link.as_str()).unwrap();
        }
        NotesPageMessage::ToggleSidebar => state.show_sidebar = !state.show_sidebar,
        NotesPageMessage::ToggleMarkdown => {
            state.show_markdown = !state.show_markdown;
            if state.show_markdown {
                state.markdown_preview_items =
                    markdown::parse(&state.editor_content.text()).collect();
            }
        }
        NotesPageMessage::SaveNote => {
            if state.note_is_dirty {
                if let Some(current_file) = state.current_file.clone() {
                    let note_text = state.editor_content.text();
                    return Task::done(Message::Notes(NotesPageMessage::LoadFolderAsNotesList))
                        .chain(Task::perform(
                            async { fs::write(current_file, note_text) },
                            |result| match result {
                                Ok(_) => Message::None,
                                Err(err) => Message::ShowToast(
                                    false,
                                    format!("Failed to save note: {err:?}"),
                                ),
                            },
                        ));
                }
            }
        }
        NotesPageMessage::OpenFilePicker => {
            return Task::perform(
                async {
                    FileDialog::new()
                        .set_directory("/")
                        .set_can_create_directories(true)
                        .pick_folder()
                },
                |selected_folder| Message::Notes(NotesPageMessage::SetNotesFolder(selected_folder)),
            );
        }
        NotesPageMessage::SetNotesFolder(selected_folder) => {
            state.selected_folder = selected_folder;
            return Task::done(Message::Notes(NotesPageMessage::LoadFolderAsNotesList))
                .chain(Task::done(Message::Notes(NotesPageMessage::LoadCategories)))
                .chain(Task::done(Message::Notes(
                    NotesPageMessage::LoadArchivedNotesList,
                )));
        }
        NotesPageMessage::LoadFolderAsNotesList => {
            if let Some(selected_folder) = state.selected_folder.clone() {
                return Task::perform(read_notes_from_folder(selected_folder), |notes_list| {
                    Message::Notes(NotesPageMessage::SetNotesList(notes_list))
                });
            }
        }
        NotesPageMessage::OpenFile(new_filepath) => {
            state.is_loading_note = true;
            // Save current file content
            let old_filepath = state.current_file.take();
            state.current_file = Some(new_filepath.clone());
            return Task::perform(
                read_file_to_note(new_filepath, old_filepath, state.editor_content.text()),
                |new_content| Message::Notes(NotesPageMessage::SetTextEditorContent(new_content)),
            );
        }
        NotesPageMessage::ToggleEditor => state.show_editor = !state.show_editor,
        NotesPageMessage::FilterNotesList(s) => state.notes_list_filter = s,
        NotesPageMessage::SetNotesList(notes_list) => {
            state.notes_list = notes_list;
            state
                .notes_list
                .sort_unstable_by_key(|note| note.last_edited);
            state.notes_list.reverse();
        }
        NotesPageMessage::SetTextEditorContent(new_content) => {
            state.editor_content = text_editor::Content::with_text(&new_content);
            state.note_crdt = LoroDoc::new();
            let temp_crdt = LoroDoc::new();
            let _ = temp_crdt.get_text(LORO_NOTE_ID).insert(0, &new_content);
            state
                .note_crdt
                .import_with(
                    &temp_crdt.export_from(&VersionVector::new()),
                    INITIAL_ORIGIN_STR,
                )
                .unwrap();
            state.undo_manager = UndoManager::new(&state.note_crdt);
            state.undo_manager.set_max_undo_steps(MAX_UNDO_STEPS);
            state
                .undo_manager
                .add_exclude_origin_prefix(INITIAL_ORIGIN_STR);
            state.markdown_preview_items = markdown::parse(&state.editor_content.text()).collect();
            state.is_loading_note = false;
        }
        NotesPageMessage::ToggleExtraToolsMenu => {
            state.show_extra_tools_menu = !state.show_extra_tools_menu
        }
        NotesPageMessage::ExportPDF => {
            return Task::perform(
                export_pdf(state.editor_content.text(), state.current_file.clone()),
                |(success, content)| Message::ShowToast(success, content),
            );
        }
        NotesPageMessage::ExportToWebsite => {
            return Task::perform(
                export_to_website(
                    state.editor_content.text(),
                    state.current_file.clone(),
                    state.website_folder.clone(),
                ),
                |(success, content)| Message::ShowToast(success, content),
            );
        }
        NotesPageMessage::ToggleDocumentStatisticsView => {
            state.show_document_statistics_view = !state.show_document_statistics_view;
            if state.show_document_statistics_view {
                return Task::done(Message::Notes(NotesPageMessage::CalculateNoteStatistics));
            }
        }
        NotesPageMessage::ToggleManageCategoriesView => {
            state.show_manage_categories_view = !state.show_manage_categories_view
        }
        NotesPageMessage::CalculateNoteStatistics => {
            let note_text = state.editor_content.text();
            return Task::perform(
                async move {
                    let char_count = note_text.chars().count() as u64;
                    let word_count = note_text.split(' ').count() as u64;
                    let reading_time_in_mins = word_count / 200;
                    NoteStatistics {
                        char_count,
                        word_count,
                        reading_time_in_mins,
                    }
                },
                |note_statistics| {
                    Message::Notes(NotesPageMessage::SetNoteStatistics(note_statistics))
                },
            );
        }
        NotesPageMessage::SetNoteStatistics(note_statistics) => {
            state.current_note_statistics = note_statistics;
        }
        NotesPageMessage::SetAutoCompleteLists(b) => {
            state.autocomplete_lists = b;
        }
        NotesPageMessage::SetConfirmBeforeDelete(b) => {
            state.confirm_before_delete_note = b;
        }
        NotesPageMessage::CreateNewNote => {
            if let Some(selected_folder) = state.selected_folder.as_ref() {
                if let Some(current_file) = &state.current_file {
                    fs::write(current_file, state.editor_content.text()).unwrap();
                };
                state.note_is_dirty = true;
                state.editor_content = text_editor::Content::new();
                state.note_crdt = LoroDoc::new();
                state.undo_manager = UndoManager::new(&state.note_crdt);
                state.undo_manager.set_max_undo_steps(MAX_UNDO_STEPS);
                state
                    .undo_manager
                    .add_exclude_origin_prefix(INITIAL_ORIGIN_STR);
                state.markdown_preview_items =
                    markdown::parse(&state.editor_content.text()).collect();
                let mut new_path = selected_folder.clone();
                new_path.push(&state.new_note_title_entry_content);
                new_path.set_extension("md");
                state.current_file = Some(new_path);
                state.new_note_title_entry_content = String::new();
                state.is_creating_new_note = false;
                return Task::done(Message::Notes(NotesPageMessage::SaveNote)).chain(Task::done(
                    Message::Notes(NotesPageMessage::LoadFolderAsNotesList),
                ));
            } else {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("No selected folder to save note into"),
                ));
            }
        }
        NotesPageMessage::UpdateNewNoteTitleEntry(s) => {
            state.new_note_title_entry_content = s;
        }
        NotesPageMessage::CancelCreateNewNote => {
            state.is_creating_new_note = false;
            state.new_note_title_entry_content = String::new();
        }
        NotesPageMessage::StartCreatingNewNote => {
            state.is_creating_new_note = true;
            return text_input::focus(text_input::Id::new(NEW_NOTE_TEXT_INPUT_ID));
        }
        NotesPageMessage::SetRenameNoteText(s) => state.rename_note_entry_text = s,
        NotesPageMessage::RenameNote => {
            if let Some(_selected_folder) = state.selected_folder.as_ref() {
                if let Some(current_note_being_managed_path) =
                    state.current_note_being_managed_path.as_ref()
                {
                    let mut new_path = current_note_being_managed_path
                        .with_file_name(&state.rename_note_entry_text);
                    new_path.set_extension("md");
                    fs::rename(current_note_being_managed_path, &new_path).unwrap();
                    if state.current_file == state.current_note_being_managed_path {
                        state.current_file = Some(new_path);
                    }
                    state.rename_note_entry_text = String::new();
                    state.display_rename_view = false;
                    state.display_delete_view = false;
                    state.current_note_being_managed_path = None;
                }
                return Task::done(Message::Notes(NotesPageMessage::SaveNote)).chain(Task::done(
                    Message::Notes(NotesPageMessage::LoadFolderAsNotesList),
                ));
            } else {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("No selected folder to save note into"),
                ));
            }
        }
        NotesPageMessage::ToggleRenameNoteView => {
            state.display_rename_view = !state.display_rename_view;
            if state.display_rename_view {
                return text_input::focus(text_input::Id::new(RENAME_NOTE_TEXT_INPUT_ID));
            } else {
                state.rename_note_entry_text = String::new();
            }
        }
        NotesPageMessage::DeleteNote => {
            if let Some(current_note_being_managed_path) =
                state.current_note_being_managed_path.as_ref()
            {
                fs::remove_file(current_note_being_managed_path).unwrap();
                if state.current_file == state.current_note_being_managed_path {
                    state.current_file = None;
                }
                state.current_note_being_managed_path = None;
                state.editor_content = text_editor::Content::new();
                state.note_crdt = LoroDoc::new();
                state.undo_manager = UndoManager::new(&state.note_crdt);
                state.undo_manager.set_max_undo_steps(MAX_UNDO_STEPS);
                state
                    .undo_manager
                    .add_exclude_origin_prefix(INITIAL_ORIGIN_STR);
                state.markdown_preview_items =
                    markdown::parse(&state.editor_content.text()).collect();
                return Task::done(Message::Notes(NotesPageMessage::LoadFolderAsNotesList)).chain(
                    Task::done(Message::ShowToast(true, String::from("Note deleted"))),
                );
            }
        }
        NotesPageMessage::ToggleDeleteNoteView => {
            state.display_delete_view = !state.display_delete_view
        }
        NotesPageMessage::ShowMenuForNote(note_path) => {
            state.display_rename_view = false;
            state.display_delete_view = false;
            state.current_note_being_managed_path = note_path;
        }
        NotesPageMessage::LoadCategories => {
            if let Some(mut selected_folder) = state.selected_folder.clone() {
                return Task::perform(
                    async move {
                        selected_folder.push(".categories.json");
                        let categories_file = selected_folder;
                        if let Ok(categories_json) = fs::read_to_string(categories_file) {
                            let categories_list: Vec<NoteCategory> =
                                serde_json::from_str(&categories_json).unwrap_or_default();
                            categories_list
                        } else {
                            vec![]
                        }
                    },
                    |categories_list| {
                        Message::Notes(NotesPageMessage::SetCategoriesList(categories_list))
                    },
                );
            }
        }
        NotesPageMessage::SetCategoriesList(categories_list) => {
            state.categories_list = categories_list
        }
        NotesPageMessage::SaveCategoriesList => {
            let categories_list = state.categories_list.clone();
            if let Some(mut selected_folder) = state.selected_folder.clone() {
                return Task::perform(
                    async move {
                        selected_folder.push(".categories.json");
                        let categories_file = selected_folder;
                        match serde_json::to_string(&categories_list) {
                            Ok(serialised_categories) => {
                                match fs::write(categories_file, serialised_categories) {
                                    Ok(_) => (true, String::new()),
                                    Err(err) => {
                                        (false, format!("Failed to save categories: {err:?}"))
                                    }
                                }
                            }
                            Err(err) => (false, format!("Failed to save categories: {err:?}")),
                        }
                    },
                    |(success, toast_message)| {
                        if success {
                            Message::None
                        } else {
                            Message::ShowToast(success, toast_message)
                        }
                    },
                );
            }
        }
        NotesPageMessage::LoadArchivedNotesList => {
            if let Some(mut selected_folder) = state.selected_folder.clone() {
                return Task::perform(
                    async move {
                        selected_folder.push(".archived.json");
                        let archive_file = selected_folder;
                        if let Ok(archived_json) = fs::read_to_string(archive_file) {
                            let archived_notes_list: Vec<PathBuf> =
                                serde_json::from_str(&archived_json).unwrap_or_default();
                            archived_notes_list
                        } else {
                            vec![]
                        }
                    },
                    |archived_notes_list| {
                        Message::Notes(NotesPageMessage::SetArchivedNotesList(archived_notes_list))
                    },
                );
            }
        }
        NotesPageMessage::SetArchivedNotesList(archived_notes_list) => {
            state.archived_notes_list = archived_notes_list
        }
        NotesPageMessage::SaveArchivedNotesList => {
            let archived_notes_list = state.archived_notes_list.clone();
            if let Some(mut selected_folder) = state.selected_folder.clone() {
                return Task::perform(
                    async move {
                        selected_folder.push(".archived.json");
                        let archived_notes_file = selected_folder;
                        match serde_json::to_string(&archived_notes_list) {
                            Ok(serialised_archived_notes_list) => {
                                match fs::write(archived_notes_file, serialised_archived_notes_list)
                                {
                                    Ok(_) => (true, String::new()),
                                    Err(err) => {
                                        (false, format!("Failed to save archived list: {err:?}"))
                                    }
                                }
                            }
                            Err(err) => (false, format!("Failed to save archived list: {err:?}")),
                        }
                    },
                    |(success, toast_message)| {
                        if success {
                            Message::None
                        } else {
                            Message::ShowToast(success, toast_message)
                        }
                    },
                );
            }
        }
        NotesPageMessage::AddCategory => {
            let serializable_colour =
                SerializableColour::from_iced_color(state.current_color_picker_colour);
            if serializable_colour != SerializableColour::default() {
                if !state.new_category_entry_text.is_empty() {
                    let invalid_chars = r#"/\?%*:|"<>"#;
                    if !state
                        .new_category_entry_text
                        .chars()
                        .any(|c| invalid_chars.contains(c) || c.is_control())
                    {
                        state.categories_list.push(NoteCategory {
                            name: state.new_category_entry_text.clone(),
                            colour: serializable_colour,
                        });
                        return Task::done(Message::Notes(NotesPageMessage::SaveCategoriesList));
                    } else {
                        return Task::done(Message::ShowToast(
                            false,
                            format!("Category name can't contain any of {invalid_chars}"),
                        ));
                    }
                } else {
                    return Task::done(Message::ShowToast(
                        false,
                        String::from("Choose a name for the category"),
                    ));
                }
            } else {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("Select a colour for the category"),
                ));
            }
        }
        NotesPageMessage::DeleteCategory => todo!(),
        NotesPageMessage::SetNewCategoryText(s) => state.new_category_entry_text = s,
        NotesPageMessage::SetColourPickerColour(colour) => {
            state.current_color_picker_colour = colour;
            state.show_colour_picker = false;
        }
        NotesPageMessage::ToggleColourPicker => {
            state.show_colour_picker = !state.show_colour_picker;
        }
        NotesPageMessage::SetWebsiteFolder(selected_folder) => {
            state.website_folder = selected_folder
        }
        NotesPageMessage::Undo => {
            if state.undo_manager.undo(&state.note_crdt).is_ok() {
                let (cursor_y, cursor_x) = state.editor_content.cursor_position();
                state.editor_content = text_editor::Content::with_text(
                    &state.note_crdt.get_text(LORO_NOTE_ID).to_string(),
                );
                move_cursor_to_position(&mut state.editor_content, cursor_x, cursor_y);
                state.note_is_dirty = true;

                state.markdown_preview_items =
                    markdown::parse(&state.editor_content.text()).collect();
            }
        }
        NotesPageMessage::Redo => {
            if state.undo_manager.redo(&state.note_crdt).is_ok() {
                let (cursor_y, cursor_x) = state.editor_content.cursor_position();
                state.editor_content = text_editor::Content::with_text(
                    &state.note_crdt.get_text(LORO_NOTE_ID).to_string(),
                );
                move_cursor_to_position(&mut state.editor_content, cursor_x, cursor_y);
                state.note_is_dirty = true;

                state.markdown_preview_items =
                    markdown::parse(&state.editor_content.text()).collect();
            }
        }
        NotesPageMessage::SetAutocompleteBrackets(b) => state.autocomplete_brackets_etc = b,
    }
    Task::none()
}
