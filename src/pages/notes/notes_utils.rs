use iced::widget::{markdown, text_editor};
use regex::Regex;
use shiva::core::TransformerTrait;
use std::{
    ffi::OsStr,
    fs::{self},
    os::linux::fs::MetadataExt,
    path::PathBuf,
};

use pulldown_cmark::Options;
use walkdir::WalkDir;

use crate::constants::LORO_NOTE_ID;

use super::page::{Note, NotesPage};

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
        has_check_box: bool,
    },
    DeleteUnorderedListItem {
        cursor_x_pos: usize,
        indent_amount: usize,
        has_check_box: bool,
    },
    AddOrderedListItem {
        num_to_insert: u32,
        indent_amount: usize,
        has_check_box: bool,
    },
    DeleteOrderedListItem {
        current_num: u32,
        cursor_x_pos: usize,
        indent_amount: usize,
        has_check_box: bool,
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

pub fn parse_markdown_lists(state: &mut NotesPage) -> ListAction {
    let asterisk_pattern = Regex::new(r"^([ ]*)\*\s(\[( |x)\]\s)?").unwrap();
    let asterisk_with_text_after_pattern = Regex::new(r"^\s*\*\s(.*)").unwrap();

    let dash_pattern = Regex::new(r"^([ ]*)-\s(\[( |x)\]\s)?").unwrap();
    let dash_with_text_after_pattern = Regex::new(r"^\s*-\s(.*)").unwrap();

    let ordered_list_pattern = Regex::new(r"^([ ]*)(\d+)\.\s(\[( |x)\]\s)?").unwrap();
    let ordered_list_with_text_after_pattern = Regex::new(r"^\s*\d+\.\s(.*)").unwrap();

    if let Some((indent_amount, has_check_box)) = state
        .editor_content
        .line(state.editor_content.cursor_position().0)
        .and_then(|current_line| {
            asterisk_pattern
                .captures(&current_line)
                .map(|caps| (caps[1].len(), caps.get(2).is_some()))
        })
    {
        if state
            .editor_content
            .line(state.editor_content.cursor_position().0)
            .is_some_and(|current_line| {
                asterisk_with_text_after_pattern
                    .captures(&current_line)
                    .is_some_and(|caps| match caps.get(1) {
                        Some(captured_text) => {
                            captured_text.as_str() != "[ ] "
                                && captured_text.as_str() != "[x] "
                                && !captured_text.is_empty()
                        }
                        None => false,
                    })
            })
        {
            ListAction::AddUnorderedListItem {
                list_char: '*',
                indent_amount,
                has_check_box,
            }
        } else {
            ListAction::DeleteUnorderedListItem {
                cursor_x_pos: state.editor_content.cursor_position().1,
                indent_amount,
                has_check_box,
            }
        }
    } else if let Some((indent_amount, has_check_box)) = state
        .editor_content
        .line(state.editor_content.cursor_position().0)
        .and_then(|current_line| {
            dash_pattern
                .captures(&current_line)
                .map(|caps| (caps[1].len(), caps.get(2).is_some()))
        })
    {
        if state
            .editor_content
            .line(state.editor_content.cursor_position().0)
            .is_some_and(|current_line| {
                dash_with_text_after_pattern
                    .captures(&current_line)
                    .is_some_and(|caps| match caps.get(1) {
                        Some(captured_text) => {
                            captured_text.as_str() != "[ ] "
                                && captured_text.as_str() != "[x] "
                                && !captured_text.is_empty()
                        }
                        None => false,
                    })
            })
        {
            ListAction::AddUnorderedListItem {
                list_char: '-',
                indent_amount,
                has_check_box,
            }
        } else {
            ListAction::DeleteUnorderedListItem {
                cursor_x_pos: state.editor_content.cursor_position().1,
                indent_amount,
                has_check_box,
            }
        }
    } else if let Some((indent_amount, current_num, has_check_box)) = state
        .editor_content
        .line(state.editor_content.cursor_position().0)
        .and_then(|current_line| {
            ordered_list_pattern.captures(&current_line).map(|caps| {
                (
                    caps[1].len(),
                    caps[2].parse::<u32>().unwrap(),
                    caps.get(3).is_some(),
                )
            })
        })
    {
        if state
            .editor_content
            .line(state.editor_content.cursor_position().0)
            .is_some_and(|current_line| {
                ordered_list_with_text_after_pattern
                    .captures(&current_line)
                    .is_some_and(|caps| match caps.get(1) {
                        Some(captured_text) => {
                            captured_text.as_str() != "[ ] "
                                && captured_text.as_str() != "[x] "
                                && !captured_text.is_empty()
                        }
                        None => false,
                    })
            })
        {
            ListAction::AddOrderedListItem {
                num_to_insert: current_num + 1,
                indent_amount,
                has_check_box,
            }
        } else {
            ListAction::DeleteOrderedListItem {
                cursor_x_pos: state.editor_content.cursor_position().1,
                indent_amount,
                has_check_box,
                current_num,
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

pub fn take_first_n_chars(input: &str, n: usize) -> String {
    let end_index = input
        .char_indices()
        .nth(n)
        .map(|(i, _)| i)
        .unwrap_or(input.len());

    input[..end_index].to_string()
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

fn add_html_to_template(html_content: &str, page_title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">

<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="X-UA-Compatible" content="ie=edge">
  <title>{page_title}</title>
  <link rel="stylesheet" href="styles.css">
</head>

<body>

  <header>
    <h1>{page_title}</h1>
    <nav>
      <a href="/blog/index.html">Home</a>
    </nav>
  </header>

  <main>
    {html_content}
  </main>
</body>

</html>"#,
    )
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
    website_folder: PathBuf,
) -> (bool, String) {
    if md_file_path_option.is_none() {
        return (
            false,
            String::from("Can't export, filename for doc is not set"),
        );
    }
    let md_file_path = md_file_path_option.expect("Can't fail");
    let mut html_export_path = website_folder.clone();
    let initial_html = convert_to_html(&text_to_convert);
    let converted_html = add_html_to_template(
        &initial_html,
        &md_file_path
            .file_stem()
            .map(|file_stem| file_stem.to_string_lossy())
            .unwrap_or_default(),
    );

    if let Some(file_export_filestem) = md_file_path.file_stem() {
        let mut html_export_filename = file_export_filestem.to_os_string();
        html_export_filename.push(OsStr::new(".html"));

        if let Err(err) = fs::create_dir_all(&html_export_path) {
            return (
                false,
                format!("Can't export, failed to create folder for html files: {err:?}"),
            );
        };
        html_export_path.push(html_export_filename);

        if let Err(err) = fs::write(html_export_path, converted_html) {
            return (
                false,
                format!("Can't export, failed to write html file: {err:?}"),
            );
        }
        update_blog_index_file(website_folder);
        (true, String::from("Successfully exported to website"))
    } else {
        (
            false,
            String::from("Can't export, markdown filename is not set"),
        )
    }
}

pub fn update_blog_index_file(website_folder: PathBuf) {
    let list_of_file_links_block: String = WalkDir::new(&website_folder)
        .into_iter()
        .filter_map(|dir_entry_result| dir_entry_result.ok())
        .map(|dir_entry| dir_entry.into_path())
        .filter(|filepath| filepath.is_file())
        .filter(|filepath| {
            filepath.to_str().is_some_and(|path_str| {
                !path_str.ends_with("styles.css") && !path_str.ends_with(".styles.css.loro")
            })
        })
        .map(|filepath| {
            format!(
                r#"<a href="/blog/{0}">{0}</a><br>"#,
                filepath
                    .strip_prefix(&website_folder)
                    .map(|path_item| path_item.to_path_buf())
                    .unwrap_or_default()
                    .to_string_lossy()
            )
        })
        .fold(String::new(), |mut acc, link_content| {
            acc.push_str(&link_content);
            acc
        });
    let index_file_content = add_html_to_template(&list_of_file_links_block, "Index");
    let _ = fs::write(website_folder.join("index.html"), index_file_content);
}

pub fn get_markdown_guide_items() -> Vec<markdown::Item> {
    markdown::parse(r#"# Guide to Markdown

## Paragraph
By writing regular text you are basically writing a paragraph.

```
This is a paragraph.
```
This is a paragraph.

---


## Headings
There are 6 heading variants. The number of '#' symbols, followed by text, indicates the importance of the heading.

```
# Heading 1
## Heading 2
### Heading 3
#### Heading 4
##### Heading 5
###### Heading 6
```

# Heading 1
## Heading 2
### Heading 3
#### Heading 4
##### Heading 5
###### Heading 6

---


## Emphasis
Modifying text is so neat and easy. You can make your text bold, italic and strikethrough.

```
Using two asterisks **this text is bold**.  
Two underscores __work as well__.  
Let's make it *italic now*.  
You guessed it, _one underscore is also enough_.  
Can we combine **_both of that_?** Absolutely.
What if I want to ~~strikethrough~~?
```

Using two asterisks **this text is bold**.  
Two underscores __work as well__.  
Let's make it *italic now*.  
You guessed it, _one underscore is also enough_.  
Can we combine **_both of that_?** Absolutely.  
What if I want to ~~strikethrough~~?

---


## Blockquote
Want to emphasise importance of the text? Say no more.

```
> This is a blockquote.
> Want to write on a new line with space between?
>
> > And nested? No problem at all.
> >
> > > PS. you can **style** your text _as you want_.
```

> This is a blockquote.
> Want to write on a new line with space between?
>
> > And nested? No problem at all.
> >
> > > PS. you can **style** your text _as you want_. :

---


## Images
The best way is to simply drag & drop image from your computer directly. You can also create reference to image and assign it that way.  
Here is the syntax.

```
![text if the image fails to load](auto-generated-path-to-file-when-you-upload-image "Text displayed on hover")

[logo]: auto-generated-path-to-file-when-you-upload-image "Hover me"
![error text][logo]
```

![text if the image fails to load](path/to/file)

---


## Links
Similar to images, links can also be inserted directly or by creating a reference. You can create both inline and block links.

[markdown-cheatsheet]: https://github.com/im-luka/markdown-cheatsheet
[docs]: https://github.com/adam-p/markdown-here

[Like it so far? Follow me on GitHub](https://github.com/im-luka)  
[My Markdown Cheatsheet - star it if you like it][markdown-cheatsheet]  
Find some great docs [here][docs]

---

## Code
You can cerate both inline and full block code snippets. You can also define programming language you were using in your snippet. All by using backticks.

```
    I created `.env` file at the root.
    Backticks inside backticks? `` `No problem.` ``

    ```
    {
      learning: "Markdown",
      showing: "block code snippet"
    }
    ```

    ```js
    const x = "Block code snippet in JS";
    console.log(x);
    ```
```

I created `.env` file at the root.
Backticks inside backticks? `` `No problem.` ``

```
{
  learning: "Markdown",
  showing: "block code snippet"
}
```

```js
const x = "Block code snippet in JS";
console.log(x);
```

---

## Lists
As you can do in HTML, Markdown allows creating of both ordered and unordered lists.


### Ordered List

```
1. HTML
2. CSS
3. Javascript
4. React
7. I'm Frontend Dev now 👨🏼‍🎨
```

1. HTML
2. CSS
3. Javascript
4. React
7. I'm Frontend Dev now 👨🏼‍🎨

### Unordered List

```
- Node.js
+ Express
* Nest.js
- Learning Backend ⌛️
```

- Node.js
+ Express
* Nest.js
- Learning Backend ⌛️

### Mixed List
You can also mix both of the lists and create sublists.  
**PS.** Try not to create lists deeper than two levels. It is the best practice.

```
1. Learn Basics
   1. HTML
   2. CSS
   7. Javascript
2. Learn One Framework
   - React 
     - Router
     - Redux
   * Vue
   + Svelte
```

1. Learn Basics
   1. HTML
   2. CSS
   7. Javascript
2. Learn One Framework
   - React 
     - Router
     - Redux
   * Vue
   + Svelte

---

## Task List
Keeping track of the tasks that are done, and those that need to be done.

```
- [x] Learn Markdown
- [ ] Learn Frontend Development
- [ ] Learn Full Stack Development
```

- [x] Learn Markdown
- [ ] Learn Frontend Development
- [ ] Learn Full Stack Development

---

## Horizontal Line
You can use asterisks, hyphens or underlines (*, -, _) to create horizontal line.  
The only rule is that you must include at least three chars of the symbol.

```
First Horizontal Line

***

Second One

-----

Third

_________
```

First Horizontal Line

***

Second One

-----

Third

_________


---



"#).collect()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use text_editor::Edit;

    use crate::pages::notes::page::NotesPageConfig;

    use super::*;
    #[test]
    fn loro_state_matches_editor_state() {
        let mut test_state = NotesPage::new(&NotesPageConfig::default(), PathBuf::new());
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
