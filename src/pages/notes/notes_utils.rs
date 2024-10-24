use regex::Regex;
use std::{
    error::Error,
    ffi::OsStr,
    fs,
    os::linux::fs::MetadataExt,
    path::{Component, Path, PathBuf},
};

use pulldown_cmark::Options;
use walkdir::WalkDir;

use super::page::{Note, NoteCategory, NotesPage, SerializableColour};

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
    AddUnorderedListItem(char),
    DeleteUnorderedListItem(usize),
}

// TODO Get this working for ordered lists
pub fn parse_markdown_lists(state: &mut NotesPage) -> ListAction {
    if state
        .editor_content
        .line(state.editor_content.cursor_position().0)
        .is_some_and(|current_line| current_line.starts_with("* "))
    {
        if state
            .editor_content
            .line(state.editor_content.cursor_position().0)
            .is_some_and(|current_line| *current_line != *"* ")
        {
            ListAction::AddUnorderedListItem('*')
        } else {
            ListAction::DeleteUnorderedListItem(state.editor_content.cursor_position().1)
        }
    } else if state
        .editor_content
        .line(state.editor_content.cursor_position().0)
        .is_some_and(|current_line| current_line.starts_with("- "))
    {
        if state
            .editor_content
            .line(state.editor_content.cursor_position().0)
            .is_some_and(|current_line| *current_line != *"- ")
        {
            ListAction::AddUnorderedListItem('-')
        } else {
            ListAction::DeleteUnorderedListItem(state.editor_content.cursor_position().1)
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
                category_name: find_nested_folder_name(&selected_folder, file_path.path()),
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
    todo!()
    // let html_content = convert_to_html(&text_to_convert);

    // if let Ok(pdf_app) = PdfApplication::new() {
    //     let export_path = md_file_path
    //         .clone()
    //         .unwrap_or(PathBuf::from("export.md"))
    //         .with_extension("pdf");
    //     let pdfout = pdf_app
    //         .builder()
    //         .orientation(Orientation::Landscape)
    //         .margin(Size::Inches(2))
    //         .title(
    //             &md_file_path
    //                 .and_then(|filepath| {
    //                     filepath
    //                         .file_stem()
    //                         .map(|os_str| os_str.to_str().map(|str_val| str_val.to_string()))
    //                 })
    //                 .flatten()
    //                 .unwrap_or(String::from("No Title")),
    //         )
    //         .build_from_html(&html_content);
    //     match pdfout {
    //         Ok(mut pdfout) => match pdfout.save(export_path.clone()) {
    //             Ok(_) => (
    //                 true,
    //                 format!("PDF successfully exported to {export_path:?}"),
    //             ),
    //             Err(err) => (false, format!("PDF export failed: {err:?}")),
    //         },
    //         Err(err) => (false, format!("PDF export failed: {err:?}")),
    //     }
    // } else {
    //     (false, String::from("Failed to init PDF application"))
    // }
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
