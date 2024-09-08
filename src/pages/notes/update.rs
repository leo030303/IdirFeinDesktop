use std::fs;

use iced::{
    widget::{markdown, text_editor},
    Task,
};
use rfd::FileDialog;

use crate::Message;

use super::{
    notes_utils::{read_file_to_note, read_notes_from_folder},
    page::{NotesPage, NotesPageMessage},
};

pub fn update(state: &mut NotesPage, message: NotesPageMessage) -> Task<Message> {
    match message {
        NotesPageMessage::Edit(action) => {
            let is_edit = action.is_edit();

            state.editor_content.perform(action);

            if is_edit {
                state.markdown_preview_items =
                    markdown::parse(&state.editor_content.text()).collect();
            }
        }
        NotesPageMessage::LinkClicked(link) => {
            println!("{link}");
        }
        NotesPageMessage::ToggleSidebar => state.show_sidebar = !state.show_sidebar,
        NotesPageMessage::ToggleMarkdown => state.show_markdown = !state.show_markdown,
        NotesPageMessage::NewNote => {
            // Save current file content
            if let Some(current_file) = &state.current_file {
                fs::write(current_file, state.editor_content.text()).unwrap();
            };
            state.current_file = None;
            state.editor_content = text_editor::Content::new();
            state.markdown_preview_items = markdown::parse(&state.editor_content.text()).collect();
        }
        NotesPageMessage::SaveNote => todo!(),
        NotesPageMessage::OpenFilePicker => {
            let selected_folder = FileDialog::new().set_directory("/").pick_folder();
            state.selected_folder = selected_folder.clone();
            if let Some(selected_folder) = selected_folder {
                return Task::perform(read_notes_from_folder(selected_folder), |notes_list| {
                    Message::Notes(NotesPageMessage::LoadFolderAsNotesList(notes_list))
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
        NotesPageMessage::LoadFolderAsNotesList(notes_list) => {
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
        NotesPageMessage::ExportPDF => todo!(),
        NotesPageMessage::ExportToWebsite => todo!(),
        NotesPageMessage::ToggleDocumentStatisticsView => {
            state.show_document_statistics_view = !state.show_document_statistics_view
        }
        NotesPageMessage::ToggleRenameNoteView => {
            state.show_rename_note_view = !state.show_rename_note_view
        }
    }
    Task::none()
}
