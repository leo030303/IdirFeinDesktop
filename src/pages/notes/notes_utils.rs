use shiva::core::TransformerTrait;
use std::{
    fs,
    path::{Component, Path, PathBuf},
    time::SystemTime,
};

use walkdir::WalkDir;

use super::page::Note;

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
                category: find_nested_folder_name(&selected_folder, file_path.path()),
                file_path: file_path.path().to_path_buf(),
                last_edited: file_path
                    .metadata()
                    .unwrap()
                    .accessed()
                    .unwrap()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            }
        })
        .collect();
    notes_list
}

pub fn take_first_n_chars(input: &str, n: usize) -> String {
    let end_index = input
        .char_indices()
        .nth(n)
        .map(|(i, _)| i)
        .unwrap_or(input.len());

    input[..end_index].to_string()
}

pub fn find_nested_folder_name(original_folder: &PathBuf, file_path: &Path) -> Option<String> {
    if let Ok(relative_path) = file_path.strip_prefix(original_folder) {
        let mut components = relative_path.components();

        if let Some(Component::Normal(folder_name)) = components.next() {
            // Checks if there's another component of the path, that tells you its a directory as the root filename wouldn't have another component
            if components.next().is_some() {
                return folder_name.to_str().map(|s| s.to_string());
            }
        }
    }

    None
}

pub async fn export_pdf(text_to_convert: String, md_file_path: Option<PathBuf>) -> (bool, String) {
    let input_bytes = bytes::Bytes::from(text_to_convert);
    match shiva::markdown::Transformer::parse(&input_bytes) {
        Ok(document) => match shiva::pdf::Transformer::generate(&document) {
            Ok(output_bytes) => {
                let export_path = md_file_path
                    .unwrap_or(PathBuf::from("export.md"))
                    .with_extension("pdf");
                match std::fs::write(export_path.clone(), output_bytes) {
                    Ok(_) => (
                        true,
                        format!("PDF successfully exported to {export_path:?}"),
                    ),
                    Err(err) => (false, format!("PDF export failed: {err:?}")),
                }
            }
            Err(err) => (false, format!("PDF export failed: {err:?}")),
        },
        Err(err) => (false, format!("PDF export failed: {err:?}")),
    }
}

pub async fn export_to_website(
    text_to_convert: String,
    md_file_path: Option<PathBuf>,
) -> (bool, String) {
    let input_bytes = bytes::Bytes::from(text_to_convert);
    match shiva::markdown::Transformer::parse(&input_bytes) {
        Ok(document) => match shiva::html::Transformer::generate(&document) {
            Ok(output_bytes) => {
                let export_path = md_file_path
                    .unwrap_or(PathBuf::from("export.md"))
                    .with_extension("html");
                match std::fs::write(export_path.clone(), output_bytes) {
                    Ok(_) => {
                        todo!();
                        (
                            true,
                            format!("HTML successfully exported to {export_path:?}"),
                        )
                    }
                    Err(err) => (false, format!("HTML export failed: {err:?}")),
                }
            }
            Err(err) => (false, format!("HTML export failed: {err:?}")),
        },
        Err(err) => (false, format!("HTML export failed: {err:?}")),
    }
}
