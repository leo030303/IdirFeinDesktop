use iced::widget::text_editor;
use regex::Regex;
use std::{
    collections::HashMap,
    error::Error,
    ffi::OsStr,
    fs::{self, File},
    io::BufWriter,
    os::linux::fs::MetadataExt,
    path::PathBuf,
};

use pulldown_cmark::Options;
use walkdir::WalkDir;

use super::page::{Note, NoteCategory, NotesPage, SerializableColour, LORO_NOTE_ID};

#[derive(Debug, Clone)]
pub struct NoteStatistics {
    pub char_count: u64,
    pub word_count: u64,
    pub reading_time_in_mins: u64,
}

pub async fn read_file_to_note(
    new_filepath: PathBuf,
    old_filepath: Option<PathBuf>,
    old_content: String,
) -> String {
    if let Some(old_filepath) = old_filepath {
        fs::write(old_filepath, old_content).unwrap();
    };
    fs::read_to_string(new_filepath.as_path()).unwrap()
}

#[derive(Debug, Clone)]
pub enum ListAction {
    NoAction,
    AddUnorderedListItem {
        list_char: char,
        indent_amount: usize,
    },
    DeleteUnorderedListItem {
        cursor_x_pos: usize,
        indent_amount: usize,
    },
}

pub fn apply_edit_to_note(state: &mut NotesPage, edit_action: text_editor::Edit) {
    let mut editor_offset = get_editor_offset(&state.editor_content);
    let selected_text_option = get_selection_location(&state.editor_content, editor_offset);
    let skip_deletes_due_to_selections = selected_text_option.is_some();
    if let Some((start_offset, selection_length)) = selected_text_option {
        state
            .note_crdt
            .get_text(LORO_NOTE_ID)
            .delete_utf8(start_offset, selection_length)
            .unwrap();
        editor_offset = start_offset;
    }
    state
        .editor_content
        .perform(text_editor::Action::Edit(edit_action.clone()));

    match edit_action {
        text_editor::Edit::Insert(insert_content) => {
            state
                .note_crdt
                .get_text(LORO_NOTE_ID)
                .insert(editor_offset, &insert_content.to_string())
                .unwrap();
        }
        text_editor::Edit::Paste(paste_content) => {
            state
                .note_crdt
                .get_text(LORO_NOTE_ID)
                .insert(editor_offset, &paste_content)
                .unwrap();
        }
        text_editor::Edit::Enter => {
            if state.note_crdt.get_text(LORO_NOTE_ID).len_unicode() == editor_offset + 1 {
                if let Some("\n") = state
                    .note_crdt
                    .get_text(LORO_NOTE_ID)
                    .to_string()
                    .get(editor_offset..(editor_offset + 1))
                {
                } else {
                    state
                        .note_crdt
                        .get_text(LORO_NOTE_ID)
                        .insert(editor_offset, "\n")
                        .unwrap();
                }
            } else {
                state
                    .note_crdt
                    .get_text(LORO_NOTE_ID)
                    .insert(editor_offset, "\n")
                    .unwrap();
            }
        }
        text_editor::Edit::Backspace => {
            if !skip_deletes_due_to_selections && editor_offset > 0 {
                let _ = state
                    .note_crdt
                    .get_text(LORO_NOTE_ID)
                    .delete(editor_offset - 1, 1);
            }
        }
        text_editor::Edit::Delete => {
            if !skip_deletes_due_to_selections {
                let _ = state
                    .note_crdt
                    .get_text(LORO_NOTE_ID)
                    .delete(editor_offset, 1);
            }
        }
    }
    if let Some('\n') = state.note_crdt.get_text(LORO_NOTE_ID).to_string().pop() {
    } else {
        state
            .note_crdt
            .get_text(LORO_NOTE_ID)
            .insert(state.note_crdt.get_text(LORO_NOTE_ID).len_unicode(), "\n")
            .unwrap();
    }
    state
        .undo_manager
        .record_new_checkpoint(&state.note_crdt)
        .unwrap();
}

/// Returns the starting offset and the length of the current selection in the editor
fn get_selection_location(
    editor_content: &text_editor::Content,
    editor_offset: usize,
) -> Option<(usize, usize)> {
    if let Some(selected_text) = editor_content.selection() {
        let pattern = Regex::new(&regex::escape(&selected_text)).unwrap();
        let mut matches_vec = vec![];
        let editor_text = editor_content.text();
        let mut editor_text_str = editor_text.as_str();
        let mut editor_text_str_offset = 0;
        while let Some(matched) = pattern.find(editor_text_str) {
            matches_vec.push((editor_text_str_offset, matched));
            if let Some(new_text_tuple) = editor_text_str.split_at_checked(matched.start() + 1) {
                editor_text_str = new_text_tuple.1;
                editor_text_str_offset += new_text_tuple.0.len();
            } else {
                break;
            }
        }
        let selected_matched = matches_vec.into_iter().find(|(match_offset, match_val)| {
            (match_offset + match_val.start()..(match_offset + match_val.end() + 1))
                .contains(&editor_offset)
        });
        selected_matched
            .map(|(match_offset, match_val)| (match_offset + match_val.start(), match_val.len()))
    } else {
        None
    }
}

fn get_editor_offset(editor_content: &text_editor::Content) -> usize {
    let (cursor_y, cursor_x) = editor_content.cursor_position();
    editor_content
        .lines()
        .take(cursor_y)
        .fold(cursor_x, |accumulator, current_line| {
            accumulator + current_line.chars().count() + 1
        })
}

pub fn move_cursor_to_position(
    editor_content: &mut text_editor::Content,
    x_pos: usize,
    y_pos: usize,
) {
    editor_content.perform(text_editor::Action::Move(
        text_editor::Motion::DocumentStart,
    ));
    for _ in 0..y_pos {
        editor_content.perform(text_editor::Action::Move(text_editor::Motion::Down));
    }
    for _ in 0..x_pos {
        editor_content.perform(text_editor::Action::Move(text_editor::Motion::Right));
    }
}

pub fn select_specific_string_in_editor(
    editor_content: &mut text_editor::Content,
    string_start_index: usize,
) {
    editor_content.perform(text_editor::Action::Move(
        text_editor::Motion::DocumentStart,
    ));
    let mut offset_so_far: usize = 0;
    let mut lines_to_move_down = 0;
    editor_content.lines().for_each(|line| {
        if line.len() < (string_start_index - offset_so_far) {
            lines_to_move_down += 1;
            offset_so_far += line.len();
        }
    });
    lines_to_move_down -= 1;
    println!("Lines to move down: {lines_to_move_down}");
    (0..lines_to_move_down).for_each(|_| {
        println!("Moved down");
        editor_content.perform(text_editor::Action::Move(text_editor::Motion::Down));
    });
    println!("Remaining offset: {}", (string_start_index - offset_so_far));
    (0..(string_start_index - offset_so_far)).for_each(|_| {
        println!("Moved right");
        editor_content.perform(text_editor::Action::Move(text_editor::Motion::Right));
    });
    editor_content.perform(text_editor::Action::Select(text_editor::Motion::Right));
}

// TODO Get this working for ordered lists
pub fn parse_markdown_lists(state: &mut NotesPage) -> ListAction {
    let asterisk_pattern = Regex::new(r"^([ ]*)\*[ ]").unwrap();
    let asterisk_with_text_after_pattern = Regex::new(r"^([ ]*)\*[ ].").unwrap();
    let dash_pattern = Regex::new(r"^([ ]*)\-[ ]").unwrap();
    let dash_with_text_after_pattern = Regex::new(r"^([ ]*)\-[ ].").unwrap();
    if let Some(indent_amount) = state
        .editor_content
        .line(state.editor_content.cursor_position().0)
        .and_then(|current_line| {
            asterisk_pattern
                .captures(&current_line)
                .map(|caps| caps[1].len())
        })
    {
        if state
            .editor_content
            .line(state.editor_content.cursor_position().0)
            .is_some_and(|current_line| asterisk_with_text_after_pattern.is_match(&current_line))
        {
            ListAction::AddUnorderedListItem {
                list_char: '*',
                indent_amount,
            }
        } else {
            ListAction::DeleteUnorderedListItem {
                cursor_x_pos: state.editor_content.cursor_position().1,
                indent_amount,
            }
        }
    } else if let Some(indent_amount) = state
        .editor_content
        .line(state.editor_content.cursor_position().0)
        .and_then(|current_line| {
            dash_pattern
                .captures(&current_line)
                .map(|caps| caps[1].len())
        })
    {
        if state
            .editor_content
            .line(state.editor_content.cursor_position().0)
            .is_some_and(|current_line| dash_with_text_after_pattern.is_match(&current_line))
        {
            ListAction::AddUnorderedListItem {
                list_char: '-',
                indent_amount: 0,
            }
        } else {
            ListAction::DeleteUnorderedListItem {
                cursor_x_pos: state.editor_content.cursor_position().1,
                indent_amount,
            }
        }
    } else {
        ListAction::NoAction
    }
}

pub async fn read_notes_from_folder(selected_folder: PathBuf) -> Vec<Note> {
    let notes_list: Vec<Note> = WalkDir::new(selected_folder.clone())
        .into_iter()
        .filter(|file_path_option| {
            file_path_option.as_ref().unwrap().path().is_file()
                && file_path_option
                    .as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    == Some("md")
        })
        .map(|file_path_option| {
            let file_path = file_path_option.unwrap();
            Note {
                button_title: take_first_n_chars(
                    file_path.path().file_stem().unwrap().to_str().unwrap(),
                    30,
                ),
                file_path: file_path.path().to_path_buf(),
                last_edited: file_path.metadata().unwrap().st_mtime() as u64,
            }
        })
        .collect();
    notes_list
}

pub fn get_colour_for_category(
    categories_list: &[NoteCategory],
    category_name: &str,
) -> iced::Color {
    categories_list
        .iter()
        .find(|category| category.name == category_name)
        .map_or(SerializableColour::default(), |category| {
            category.colour.clone()
        })
        .to_iced_colour()
}

pub fn take_first_n_chars(input: &str, n: usize) -> String {
    let end_index = input
        .char_indices()
        .nth(n)
        .map(|(i, _)| i)
        .unwrap_or(input.len());

    input[..end_index].to_string()
}

pub async fn export_pdf(text_to_convert: String, md_file_path: Option<PathBuf>) -> (bool, String) {
    let mut pdf_config = mdproof::Config::default();
    let export_path = md_file_path
        .clone()
        .unwrap_or(PathBuf::from("export.md"))
        .with_extension("pdf");
    pdf_config.title = String::from(
        export_path
            .file_stem()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("Title"),
    );
    if let Ok(pdf_document) = mdproof::markdown_to_pdf(&text_to_convert, &pdf_config) {
        match File::create(&export_path) {
            Ok(result_file) => match pdf_document.save(&mut BufWriter::new(result_file)) {
                Ok(_) => (
                    true,
                    format!("PDF successfully exported to {export_path:?}"),
                ),
                Err(err) => (false, format!("PDF export failed: {err:?}")),
            },
            Err(err) => (false, format!("PDF export failed: {err:?}")),
        }
    } else {
        (false, String::from("Failed to init PDF application"))
    }
}

fn add_html_to_template(
    html_content: &str,
    mut website_folder: PathBuf,
) -> Result<String, Box<dyn Error>> {
    website_folder.push("template.html");
    let template_file_content = fs::read_to_string(website_folder)?;
    let re = Regex::new(r"__CONTENT__")?;
    Ok(re.replace(&template_file_content, html_content).to_string())
}

pub fn convert_to_html(text_to_convert: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = pulldown_cmark::Parser::new_ext(text_to_convert, options);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    html_output
}

pub async fn export_to_website(
    text_to_convert: String,
    md_file_path_option: Option<PathBuf>,
    website_folder_option: Option<PathBuf>,
) -> (bool, String) {
    if md_file_path_option.is_none() {
        return (
            false,
            String::from("Can't export, filename for doc is not set"),
        );
    }
    if website_folder_option.is_none() {
        return (
            false,
            String::from("Can't export, folder for website files is not set"),
        );
    }
    let md_file_path = md_file_path_option.expect("Can't fail");
    let website_folder = website_folder_option.expect("Can't fail");
    let mut html_export_path = website_folder.clone();
    let mut markdown_export_path = website_folder.clone();
    let initial_html = convert_to_html(&text_to_convert);
    let converted_html_result = add_html_to_template(&initial_html, website_folder);
    if let Err(err) = converted_html_result {
        return (
            false,
            format!("Error inserting HTML into template: {err:?}"),
        );
    }
    let converted_html = converted_html_result.expect("Can't fail");

    if let Some(file_export_filestem) = md_file_path.file_stem() {
        let mut html_export_filename = file_export_filestem.to_os_string();
        html_export_filename.push(OsStr::new(".html"));
        html_export_path.push("www");

        if let Err(err) = fs::create_dir_all(&html_export_path) {
            return (
                false,
                format!("Can't export, failed to create folder for html files: {err:?}"),
            );
        };
        html_export_path.push(html_export_filename);

        let mut markdown_export_filename = file_export_filestem.to_os_string();
        markdown_export_filename.push(OsStr::new(".md"));
        markdown_export_path.push("markdown");
        if let Err(err) = fs::create_dir_all(&markdown_export_path) {
            return (
                false,
                format!("Can't export, failed to create folder for markdown files: {err:?}"),
            );
        };
        markdown_export_path.push(markdown_export_filename);

        if let Err(err) = fs::write(html_export_path, converted_html) {
            return (
                false,
                format!("Can't export, failed to write html file: {err:?}"),
            );
        }
        if let Err(err) = fs::write(markdown_export_path, text_to_convert) {
            return (
                false,
                format!("Can't export, failed to write markdown file: {err:?}"),
            );
        }
        (true, String::from("Successfully exported to website"))
    } else {
        (
            false,
            String::from("Can't export, markdown filename is not set"),
        )
    }
}

pub fn get_category_for_note(
    categorised_notes_list: &HashMap<String, Vec<String>>,
    note_name: &String,
) -> Option<String> {
    for (key, notes_list) in categorised_notes_list.iter() {
        if notes_list.contains(note_name) {
            return Some(key.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use text_editor::Edit;

    use crate::pages::notes::page::NotesPageConfig;

    use super::*;
    #[test]
    fn loro_state_matches_editor_state() {
        let mut test_state = NotesPage::new(&NotesPageConfig::default());
        let list_of_actions = vec![
            text_editor::Action::Edit(Edit::Insert('e')),
            text_editor::Action::Edit(Edit::Insert('e')),
            text_editor::Action::Edit(Edit::Delete),
            text_editor::Action::Edit(Edit::Paste(Arc::new("appleja".to_string()))),
            text_editor::Action::Edit(Edit::Insert('x')),
            text_editor::Action::Edit(Edit::Insert('p')),
            text_editor::Action::Edit(Edit::Insert(' ')),
            text_editor::Action::Edit(Edit::Insert('u')),
            text_editor::Action::Edit(Edit::Insert('l')),
            text_editor::Action::Edit(Edit::Enter),
            text_editor::Action::Move(text_editor::Motion::Left),
            text_editor::Action::Edit(Edit::Insert('C')),
            text_editor::Action::Edit(Edit::Insert('T')),
            text_editor::Action::Edit(Edit::Insert('Q')),
            text_editor::Action::Select(text_editor::Motion::Left),
            text_editor::Action::Edit(Edit::Backspace),
            text_editor::Action::Edit(Edit::Insert('p')),
            text_editor::Action::Edit(Edit::Delete),
            text_editor::Action::Edit(Edit::Insert('9')),
            text_editor::Action::Edit(Edit::Paste(Arc::new("sancjncq".to_string()))),
            text_editor::Action::Edit(Edit::Paste(Arc::new("i\ncpiqnwoicnsancsa".to_string()))),
            text_editor::Action::Edit(Edit::Insert('k')),
            text_editor::Action::Edit(Edit::Insert('a')),
            text_editor::Action::Edit(Edit::Enter),
            text_editor::Action::Edit(Edit::Backspace),
            text_editor::Action::Edit(Edit::Insert('e')),
            text_editor::Action::Edit(Edit::Insert('g')),
            text_editor::Action::Edit(Edit::Insert('K')),
            text_editor::Action::Move(text_editor::Motion::Right),
            text_editor::Action::Move(text_editor::Motion::WordLeft),
            text_editor::Action::Select(text_editor::Motion::WordLeft),
            text_editor::Action::Edit(Edit::Insert('3')),
            text_editor::Action::Edit(Edit::Enter),
            text_editor::Action::Edit(Edit::Paste(Arc::new("pajspiajoifuewbewnvckd".to_string()))),
            text_editor::Action::Edit(Edit::Backspace),
            text_editor::Action::Edit(Edit::Delete),
            text_editor::Action::Edit(Edit::Insert('e')),
            text_editor::Action::Edit(Edit::Insert('e')),
            text_editor::Action::Edit(Edit::Delete),
            text_editor::Action::Edit(Edit::Paste(Arc::new("appleja".to_string()))),
            text_editor::Action::Select(text_editor::Motion::PageUp),
            text_editor::Action::Edit(Edit::Insert('x')),
            text_editor::Action::Edit(Edit::Insert('p')),
            text_editor::Action::Edit(Edit::Insert('u')),
            text_editor::Action::Move(text_editor::Motion::DocumentStart),
            text_editor::Action::Edit(Edit::Insert('l')),
            text_editor::Action::Edit(Edit::Enter),
            text_editor::Action::Move(text_editor::Motion::Left),
            text_editor::Action::Edit(Edit::Insert('C')),
            text_editor::Action::Edit(Edit::Insert('T')),
            text_editor::Action::Edit(Edit::Insert('Q')),
            text_editor::Action::Edit(Edit::Enter),
            text_editor::Action::Edit(Edit::Backspace),
            text_editor::Action::Move(text_editor::Motion::Up),
            text_editor::Action::Edit(Edit::Insert('p')),
            text_editor::Action::Edit(Edit::Delete),
            text_editor::Action::Edit(Edit::Insert('9')),
            text_editor::Action::Edit(Edit::Paste(Arc::new("sancjncq".to_string()))),
            text_editor::Action::Move(text_editor::Motion::Down),
            text_editor::Action::Edit(Edit::Paste(Arc::new("i\ncpiqnwoicnsancsa".to_string()))),
            text_editor::Action::Edit(Edit::Insert('k')),
            text_editor::Action::Edit(Edit::Insert('a')),
            text_editor::Action::Move(text_editor::Motion::PageDown),
            text_editor::Action::Edit(Edit::Enter),
            text_editor::Action::Edit(Edit::Backspace),
            text_editor::Action::Edit(Edit::Insert('e')),
            text_editor::Action::Edit(Edit::Insert('g')),
            text_editor::Action::Move(text_editor::Motion::DocumentEnd),
            text_editor::Action::Edit(Edit::Insert('K')),
            text_editor::Action::Edit(Edit::Insert('3')),
            text_editor::Action::Edit(Edit::Enter),
            text_editor::Action::Edit(Edit::Paste(Arc::new("pajspiajoifuewbewnvckd".to_string()))),
            text_editor::Action::Edit(Edit::Backspace),
            text_editor::Action::Edit(Edit::Delete),
            text_editor::Action::Edit(Edit::Insert('e')),
            text_editor::Action::Edit(Edit::Insert('e')),
            text_editor::Action::Edit(Edit::Delete),
            text_editor::Action::Edit(Edit::Paste(Arc::new("appleja".to_string()))),
            text_editor::Action::Edit(Edit::Insert('x')),
            text_editor::Action::Edit(Edit::Insert('p')),
            text_editor::Action::Edit(Edit::Insert(' ')),
            text_editor::Action::Edit(Edit::Insert('u')),
            text_editor::Action::Edit(Edit::Insert('l')),
            text_editor::Action::Edit(Edit::Enter),
            text_editor::Action::Move(text_editor::Motion::Left),
            text_editor::Action::Edit(Edit::Insert('C')),
            text_editor::Action::Edit(Edit::Insert('T')),
            text_editor::Action::Edit(Edit::Insert('Q')),
            text_editor::Action::Select(text_editor::Motion::Left),
            text_editor::Action::Edit(Edit::Backspace),
            text_editor::Action::Edit(Edit::Insert('p')),
            text_editor::Action::Edit(Edit::Delete),
            text_editor::Action::Edit(Edit::Insert('9')),
            text_editor::Action::Edit(Edit::Paste(Arc::new("sancjncq".to_string()))),
            text_editor::Action::Edit(Edit::Paste(Arc::new("i\ncpiqnwoicnsancsa".to_string()))),
            text_editor::Action::Edit(Edit::Insert('k')),
            text_editor::Action::Edit(Edit::Insert('a')),
            text_editor::Action::Edit(Edit::Enter),
            text_editor::Action::Edit(Edit::Backspace),
            text_editor::Action::Edit(Edit::Insert('e')),
            text_editor::Action::Edit(Edit::Insert('g')),
            text_editor::Action::Edit(Edit::Insert('K')),
            text_editor::Action::Move(text_editor::Motion::Right),
            text_editor::Action::Move(text_editor::Motion::WordLeft),
            text_editor::Action::Select(text_editor::Motion::WordLeft),
            text_editor::Action::Edit(Edit::Insert('3')),
            text_editor::Action::Edit(Edit::Enter),
            text_editor::Action::Edit(Edit::Paste(Arc::new("pajspiajoifuewbewnvckd".to_string()))),
            text_editor::Action::Edit(Edit::Backspace),
            text_editor::Action::Edit(Edit::Delete),
            text_editor::Action::Edit(Edit::Insert('e')),
            text_editor::Action::Edit(Edit::Insert('e')),
            text_editor::Action::Edit(Edit::Delete),
            text_editor::Action::Edit(Edit::Paste(Arc::new("appleja".to_string()))),
            text_editor::Action::Select(text_editor::Motion::PageUp),
            text_editor::Action::Edit(Edit::Insert('x')),
            text_editor::Action::Edit(Edit::Insert('p')),
            text_editor::Action::Edit(Edit::Insert('u')),
            text_editor::Action::Move(text_editor::Motion::DocumentStart),
            text_editor::Action::Edit(Edit::Insert('l')),
            text_editor::Action::Edit(Edit::Enter),
            text_editor::Action::Move(text_editor::Motion::Left),
            text_editor::Action::Edit(Edit::Insert('C')),
            text_editor::Action::Edit(Edit::Insert('T')),
            text_editor::Action::Edit(Edit::Insert('Q')),
            text_editor::Action::Edit(Edit::Enter),
            text_editor::Action::Edit(Edit::Backspace),
            text_editor::Action::Move(text_editor::Motion::Up),
            text_editor::Action::Edit(Edit::Insert('p')),
            text_editor::Action::Edit(Edit::Delete),
            text_editor::Action::Edit(Edit::Insert('9')),
            text_editor::Action::Edit(Edit::Paste(Arc::new("sancjncq".to_string()))),
            text_editor::Action::Move(text_editor::Motion::Down),
            text_editor::Action::Edit(Edit::Paste(Arc::new("i\ncpiqnwoicnsancsa".to_string()))),
            text_editor::Action::Edit(Edit::Insert('k')),
            text_editor::Action::Edit(Edit::Insert('a')),
            text_editor::Action::Move(text_editor::Motion::PageDown),
            text_editor::Action::Edit(Edit::Enter),
            text_editor::Action::Edit(Edit::Backspace),
            text_editor::Action::Edit(Edit::Insert('e')),
            text_editor::Action::Edit(Edit::Insert('g')),
            text_editor::Action::Move(text_editor::Motion::DocumentEnd),
            text_editor::Action::Edit(Edit::Insert('K')),
            text_editor::Action::Edit(Edit::Insert('3')),
            text_editor::Action::Edit(Edit::Enter),
            text_editor::Action::Edit(Edit::Paste(Arc::new("pajspiajoifuewbewnvckd".to_string()))),
            text_editor::Action::Edit(Edit::Backspace),
            text_editor::Action::Edit(Edit::Delete),
        ];
        list_of_actions
            .into_iter()
            .enumerate()
            .for_each(|(i, action)| {
                println!("Running action {i}: {action:?}");
                let _ =
                    test_state.update(crate::pages::notes::page::NotesPageMessage::Edit(action));
                println!(
                    "Editor: {:?}\nLoro:   {:?}",
                    test_state.editor_content.text(),
                    test_state.note_crdt.get_text(LORO_NOTE_ID).to_string()
                );
                assert_eq!(
                    test_state.editor_content.text(),
                    test_state.note_crdt.get_text(LORO_NOTE_ID).to_string()
                );
            });
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Undo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Undo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Undo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Undo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Undo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Undo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Undo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Undo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Undo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Redo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Redo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Redo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Redo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Redo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Redo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Redo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Redo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Redo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Redo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Redo);
        let _ = test_state.update(crate::pages::notes::page::NotesPageMessage::Redo);
        assert_eq!(
            test_state.editor_content.text(),
            test_state.note_crdt.get_text(LORO_NOTE_ID).to_string()
        );
    }
}
