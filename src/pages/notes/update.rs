use iced::{
    widget::{markdown, text_editor, text_input},
    Task,
};
use rfd::FileDialog;
use std::fs;

use crate::{app::Message, pages::notes::notes_utils::parse_markdown_lists};

use super::{
    notes_utils::{export_pdf, read_file_to_note, read_notes_from_folder, NoteStatistics},
    page::{NotesPage, NotesPageMessage, NEW_NOTE_TEXT_INPUT_ID, RENAME_NOTE_TEXT_INPUT_ID},
};

pub fn update(state: &mut NotesPage, message: NotesPageMessage) -> Task<Message> {
    match message {
        NotesPageMessage::Edit(action) => {
            let is_edit = action.is_edit();

            if state.autocomplete_lists {
                if let text_editor::Action::Edit(text_editor::Edit::Enter) = action {
                    let list_action = parse_markdown_lists(state);
                    match list_action {
                        crate::pages::notes::notes_utils::ListAction::NoAction => {
                            state
                                .editor_content
                                .perform(text_editor::Action::Edit(text_editor::Edit::Enter));
                        }
                        crate::pages::notes::notes_utils::ListAction::AddUnorderedListItem(
                            list_char,
                        ) => {
                            state
                                .editor_content
                                .perform(text_editor::Action::Edit(text_editor::Edit::Enter));
                            state.editor_content.perform(text_editor::Action::Edit(
                                text_editor::Edit::Insert(list_char),
                            ));
                            state
                                .editor_content
                                .perform(text_editor::Action::Edit(text_editor::Edit::Insert(' ')));
                        }
                        crate::pages::notes::notes_utils::ListAction::DeleteUnorderedListItem => {
                            state
                                .editor_content
                                .perform(text_editor::Action::Edit(text_editor::Edit::Backspace));
                            state
                                .editor_content
                                .perform(text_editor::Action::Edit(text_editor::Edit::Backspace));
                            state
                                .editor_content
                                .perform(text_editor::Action::Edit(text_editor::Edit::Enter));
                        }
                    }
                } else {
                    state.editor_content.perform(action);
                }
            } else {
                state.editor_content.perform(action);
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
            return Task::done(Message::Notes(NotesPageMessage::LoadFolderAsNotesList));
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
        NotesPageMessage::ExportToWebsite => todo!(),
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
        NotesPageMessage::InsertTitle => {
            state.note_is_dirty = true;
            state
                .editor_content
                .perform(text_editor::Action::Edit(text_editor::Edit::Enter));
            state
                .editor_content
                .perform(text_editor::Action::Edit(text_editor::Edit::Insert('#')));
            state
                .editor_content
                .perform(text_editor::Action::Edit(text_editor::Edit::Insert(' ')));
        }
        NotesPageMessage::SetAutoCompleteLists(b) => {
            state.autocomplete_lists = b;
        }
        NotesPageMessage::SetShowFormatToolbar(b) => {
            state.show_format_toolbar = b;
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
    }
    Task::none()
}
