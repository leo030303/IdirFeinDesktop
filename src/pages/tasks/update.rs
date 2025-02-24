use std::{
    fs::{self, File},
    mem,
    path::PathBuf,
};

use iced::{
    advanced::widget::Id,
    widget::{text_editor, text_input},
    Task,
};
use iced_aw::widget::zones_on_point;
use rfd::FileDialog;

use crate::app::Message;

use super::page::{
    TaskCompletionState, TaskData, TasksPage, TasksPageMessage, ARCHIVED_FILE_NAME, BACKLOG_ID,
    DOING_ID, DONE_ID, NEW_PROJECT_TEXT_INPUT_ID, RENAME_PROJECT_TEXT_INPUT_ID,
    TASK_TITLE_TEXT_INPUT_ID, TODO_ID,
};

pub fn update(state: &mut TasksPage, message: TasksPageMessage) -> Task<Message> {
    match message {
        TasksPageMessage::ToggleShowTaskEditDialog => {
            state.show_task_edit_dialog = !state.show_task_edit_dialog
        }
        TasksPageMessage::ToggleShowSidebar => state.show_sidebar = !state.show_sidebar,
        TasksPageMessage::ToggleConfirmBeforeDeleteDialog => {
            state.show_confirm_before_delete_dialog = !state.show_confirm_before_delete_dialog
        }
        TasksPageMessage::ToggleTaskViewType => match state.task_view_type {
            super::page::TaskViewType::Kanban => {
                state.task_view_type = super::page::TaskViewType::List
            }
            super::page::TaskViewType::List => {
                state.task_view_type = super::page::TaskViewType::Kanban
            }
        },
        TasksPageMessage::LoadProjectsList => {
            let selected_folder = state.selected_folder.clone();
            return Task::perform(
                async {
                    let projects_list: Vec<PathBuf> =
                        if let Some(selected_folder) = selected_folder.as_ref() {
                            match fs::read_dir(selected_folder) {
                                Ok(directory_iterator) => directory_iterator
                                    .filter_map(|read_dir_object| read_dir_object.ok())
                                    .map(|read_dir_object| read_dir_object.path())
                                    .filter(|path| {
                                        path.extension().is_some_and(|extension_os_str| {
                                            extension_os_str
                                                .to_str()
                                                .is_some_and(|extension| extension == "json")
                                        })
                                    })
                                    .collect(),
                                Err(err) => {
                                    println!("Error reading directory: {err:?}");
                                    vec![]
                                }
                            }
                        } else {
                            vec![]
                        };
                    let archived_list: Vec<String> = if let Some(selected_folder) = selected_folder
                    {
                        if let Ok(archived_projects_json) =
                            fs::read_to_string(selected_folder.join(ARCHIVED_FILE_NAME))
                        {
                            serde_json::from_str(&archived_projects_json).unwrap_or_default()
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    };
                    (projects_list, archived_list)
                },
                |(projects_list, archived_list)| {
                    Message::Tasks(TasksPageMessage::SetProjectsList(
                        projects_list,
                        archived_list,
                    ))
                },
            );
        }
        TasksPageMessage::PickProjectsFolder => {
            return Task::perform(
                async {
                    FileDialog::new()
                        .set_directory("/")
                        .set_can_create_directories(true)
                        .pick_folder()
                },
                |selected_folder| {
                    Message::Tasks(TasksPageMessage::SetProjectsFolder(selected_folder))
                },
            );
        }
        TasksPageMessage::SetProjectsFolder(selected_folder) => {
            state.selected_folder = selected_folder;
            return Task::done(Message::Tasks(TasksPageMessage::LoadProjectsList));
        }

        TasksPageMessage::PickProjectFile(path_to_file_option) => {
            if let Some(path_to_file) = path_to_file_option {
                if let Some(selected_folder) = &state.selected_folder {
                    // Check that the project file is a child of the projects folder before loading
                    let canonicalised_folder_result = selected_folder.canonicalize();
                    let canonicalised_file_result = path_to_file.canonicalize();
                    if let Ok(canonicalised_folder) = canonicalised_folder_result {
                        if let Ok(canonicalised_file) = canonicalised_file_result {
                            if canonicalised_file.starts_with(canonicalised_folder) {
                                return Task::batch([
                                    Task::done(Message::Tasks(TasksPageMessage::SaveProject)),
                                    Task::perform(
                                        async move {
                                            if let Ok(task_json) = fs::read_to_string(&path_to_file)
                                            {
                                                let tasks_list: Vec<TaskData> =
                                                    serde_json::from_str(&task_json)
                                                        .unwrap_or_default();
                                                (tasks_list, path_to_file)
                                            } else {
                                                (vec![], path_to_file)
                                            }
                                        },
                                        |(tasks_list, path_to_file)| {
                                            Message::Tasks(TasksPageMessage::SetTasksList(
                                                tasks_list,
                                                path_to_file,
                                            ))
                                        },
                                    ),
                                ]);
                            } else {
                                state.current_project_file = None;
                            }
                        } else {
                            state.current_project_file = None;
                        }
                    }
                }
            }
        }
        TasksPageMessage::SetProjectsList(projects_list, archived_list) => {
            state.projects_list = projects_list;
            state.archived_list = archived_list;
            return Task::done(Message::Tasks(TasksPageMessage::PickProjectFile(
                state.current_project_file.clone(),
            )));
        }
        TasksPageMessage::SetTasksList(tasks_list, project_path) => {
            state.tasks_list = tasks_list;
            state.current_project_file = Some(project_path);
        }
        TasksPageMessage::SelectTaskToEdit(task_uuid) => state.current_task_id = task_uuid,
        TasksPageMessage::DeleteTask(id_to_delete) => {
            state.show_confirm_before_delete_dialog = false;
            if let Some(task_index) = state.tasks_list.iter().position(|x| x.id == id_to_delete) {
                state.tasks_list.remove(task_index);
                state.is_dirty = true;
            }
            return Task::done(Message::Tasks(TasksPageMessage::SaveProject));
        }
        TasksPageMessage::SetTaskCompletionState(id_to_edit, task_completion_state) => {
            if let Some(task_index) = state.tasks_list.iter().position(|x| x.id == id_to_edit) {
                state
                    .tasks_list
                    .get_mut(task_index)
                    .expect("Shouldn't be possible for this to fail")
                    .completion_state = task_completion_state;
                state.is_dirty = true;
            }
            return Task::done(Message::Tasks(TasksPageMessage::SaveProject));
        }
        TasksPageMessage::UpdateCurrentTask => {
            match state.current_task_id {
                Some(task_id) => {
                    if let Some(task_index) = state.tasks_list.iter().position(|x| x.id == task_id)
                    {
                        state
                            .tasks_list
                            .get_mut(task_index)
                            .expect("Shouldn't be possible for this to fail")
                            .title = mem::take(&mut state.current_task_title_text);
                        state
                            .tasks_list
                            .get_mut(task_index)
                            .expect("Shouldn't be possible for this to fail")
                            .description = state.current_task_description_content.text();
                        state.is_dirty = true;
                    }
                }
                None => {
                    state.tasks_list.push(TaskData {
                        title: mem::take(&mut state.current_task_title_text),
                        description: state.current_task_description_content.text(),
                        ..Default::default()
                    });
                    state.is_dirty = true;
                }
            };
            state.current_task_description_content = text_editor::Content::default();
            state.current_task_id = None;
            state.show_task_edit_dialog = false;
            return Task::done(Message::Tasks(TasksPageMessage::SaveProject));
        }
        TasksPageMessage::SaveProject => {
            if state.is_dirty {
                if let Some(current_project_file) = state.current_project_file.clone() {
                    let serialised_tasks_list_option = serde_json::to_string(&state.tasks_list);
                    return Task::perform(
                        async {
                            match serialised_tasks_list_option {
                                Ok(serialised_tasks_list) => {
                                    match fs::write(current_project_file, serialised_tasks_list) {
                                        Ok(_) => Ok(()),
                                        Err(err) => Err(format!("PROJECT SAVE FAILED: Failed on file write: {err:?}")),
                                    }
                                }
                                Err(err) => Err(format!(
                                    "PROJECT SAVE FAILED: Couldn't serialise tasks list object to JSON: {err:?}"
                                )),
                            }
                        },
                        |result| match result {
                            Ok(_) => Message::None,
                            Err(err_string) => Message::ShowToast(false, err_string),
                        },
                    );
                }
            }
        }
        TasksPageMessage::StartCreatingNewTask => {
            state.show_task_edit_dialog = true;
            state.current_task_id = None;
            state.current_task_title_text = String::new();
            state.current_task_description_content = text_editor::Content::with_text("");
            return text_input::focus(text_input::Id::new(TASK_TITLE_TEXT_INPUT_ID));
        }
        TasksPageMessage::UpdateTaskTitle(s) => state.current_task_title_text = s,
        TasksPageMessage::UpdateTaskDescription(action) => {
            state.current_task_description_content.perform(action)
        }
        TasksPageMessage::DeleteTaskWithConfirmationCheck(task_id) => {
            if state.should_confirm_before_delete {
                state.current_task_id = Some(task_id);
                return Task::done(Message::Tasks(
                    TasksPageMessage::ToggleConfirmBeforeDeleteDialog,
                ));
            } else {
                return Task::done(Message::Tasks(TasksPageMessage::DeleteTask(task_id)));
            }
        }
        TasksPageMessage::OpenEditDialogForTask(task_id) => {
            if let Some(task_index) = state.tasks_list.iter().position(|x| x.id == task_id) {
                state.current_task_id = Some(task_id);
                state.show_task_edit_dialog = true;
                state.current_task_title_text = state
                    .tasks_list
                    .get(task_index)
                    .expect("Shouldn't fail")
                    .title
                    .clone();
                state.current_task_description_content = text_editor::Content::with_text(
                    &state
                        .tasks_list
                        .get(task_index)
                        .expect("Shouldn't fail")
                        .description,
                );
                return text_input::focus(text_input::Id::new(TASK_TITLE_TEXT_INPUT_ID));
            }
        }
        TasksPageMessage::StartCreatingNewProject => {
            state.is_creating_new_project = true;
            return text_input::focus(text_input::Id::new(NEW_PROJECT_TEXT_INPUT_ID));
        }
        TasksPageMessage::CreateNewProject => {
            state.is_creating_new_project = false;
            if let Some(mut selected_folder) = state.selected_folder.clone() {
                selected_folder.push(&state.new_project_name_field_content);
                state.new_project_name_field_content = String::new();
                selected_folder.set_extension("json");
                if let Err(err) = File::create(selected_folder) {
                    return Task::done(Message::ShowToast(
                        false,
                        format!("Couldn't create project file: {err:?}"),
                    ));
                } else {
                    return Task::done(Message::Tasks(TasksPageMessage::LoadProjectsList));
                }
            }
        }
        TasksPageMessage::UpdateNewProjectNameEntry(s) => state.new_project_name_field_content = s,
        TasksPageMessage::CancelCreateNewProject => {
            state.is_creating_new_project = false;
            state.new_project_name_field_content = String::new();
        }
        TasksPageMessage::EscapeKeyPressed => {
            state.show_task_edit_dialog = false;
            state.current_task_title_text = String::new();
            state.current_task_description_content = text_editor::Content::with_text("");
            state.current_task_id = None;
            state.show_confirm_before_delete_dialog = false;
        }
        TasksPageMessage::DropTask(task_id, cursor_position, _rectangle) => {
            return zones_on_point(
                move |zones| Message::Tasks(TasksPageMessage::HandleTaskDropZones(task_id, zones)),
                cursor_position,
                None,
                None,
            );
        }
        TasksPageMessage::HandleTaskDropZones(task_id, zones) => {
            if let Some(dropped_zone) = zones.first() {
                if dropped_zone.0 == Id::new(BACKLOG_ID) {
                    return Task::done(Message::Tasks(TasksPageMessage::SetTaskCompletionState(
                        task_id,
                        TaskCompletionState::Backlog,
                    )));
                } else if dropped_zone.0 == Id::new(TODO_ID) {
                    return Task::done(Message::Tasks(TasksPageMessage::SetTaskCompletionState(
                        task_id,
                        TaskCompletionState::ToDo,
                    )));
                } else if dropped_zone.0 == Id::new(DOING_ID) {
                    return Task::done(Message::Tasks(TasksPageMessage::SetTaskCompletionState(
                        task_id,
                        TaskCompletionState::Doing,
                    )));
                } else if dropped_zone.0 == Id::new(DONE_ID) {
                    return Task::done(Message::Tasks(TasksPageMessage::SetTaskCompletionState(
                        task_id,
                        TaskCompletionState::Done,
                    )));
                }
            }
        }
        TasksPageMessage::ToggleExtraToolsMenu => {
            state.show_extra_tools_menu = !state.show_extra_tools_menu
        }
        TasksPageMessage::ShowMenuForProject(project_path) => {
            state.display_rename_project_view = false;
            state.display_delete_project_view = false;
            state.current_project_being_managed = project_path
        }
        TasksPageMessage::UpdateRenameProjectEntryText(s) => state.rename_project_field_text = s,
        TasksPageMessage::DeleteProject => {
            if let Some(current_project_file) = state.current_project_being_managed.as_ref() {
                let _ = fs::remove_file(current_project_file);
                state.current_project_being_managed = None;
                state.display_delete_project_view = false;
                return Task::done(Message::Tasks(TasksPageMessage::LoadProjectsList)).chain(
                    Task::done(Message::ShowToast(true, String::from("Project deleted"))),
                );
            }
        }
        TasksPageMessage::RenameProject => {
            if let Some(_selected_folder) = state.selected_folder.as_ref() {
                if let Some(current_project_file) = state.current_project_being_managed.as_ref() {
                    let mut new_path =
                        current_project_file.with_file_name(&state.rename_project_field_text);

                    new_path.set_extension("json");
                    let _ = fs::rename(current_project_file, &new_path);
                    if state.current_project_file == state.current_project_being_managed {
                        state.current_project_file = Some(new_path);
                    }
                    state.rename_project_field_text = String::new();
                    state.display_rename_project_view = false;
                    state.display_delete_project_view = false;
                    state.current_project_being_managed = None;
                } else {
                    return Task::done(Message::ShowToast(
                        false,
                        String::from("No selected project to rename"),
                    ));
                }
                return Task::done(Message::Tasks(TasksPageMessage::SaveProject)).chain(
                    Task::done(Message::Tasks(TasksPageMessage::LoadProjectsList)),
                );
            } else {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("No selected folder to save renamed project to."),
                ));
            }
        }
        TasksPageMessage::ToggleRenameProjectView => {
            state.display_rename_project_view = !state.display_rename_project_view;
            if state.display_rename_project_view {
                return text_input::focus(text_input::Id::new(RENAME_PROJECT_TEXT_INPUT_ID));
            } else {
                state.rename_project_field_text = String::new();
            }
        }
        TasksPageMessage::ToggleDeleteProjectView => {
            state.display_delete_project_view = !state.display_delete_project_view;
        }
        TasksPageMessage::SetShowTaskCompletionToolbar(b) => {
            state.show_task_completion_toolbar = b;
        }
        TasksPageMessage::SetConfirmBeforeDelete(b) => {
            state.should_confirm_before_delete = b;
        }
        TasksPageMessage::SetRightClickToEditTask(b) => {
            state.right_click_to_edit_task = b;
        }
        TasksPageMessage::UpdateTasksFilter(s) => {
            state.filter_tasks_text = s;
        }
        TasksPageMessage::UpdateProjectsFilter(s) => {
            state.filter_projects_text = s;
        }
        TasksPageMessage::ArchiveProject => {
            if let Some(selected_folder) = state.selected_folder.as_ref() {
                if let Some(current_project_being_managed) =
                    state.current_project_being_managed.as_ref()
                {
                    let project_to_archive = current_project_being_managed
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase();
                    state.archived_list.push(project_to_archive);
                    match serde_json::to_string(&state.archived_list) {
                        Ok(serialised) => {
                            let _ = fs::write(selected_folder.join(ARCHIVED_FILE_NAME), serialised);
                        }
                        Err(err) => {
                            return Task::done(Message::ShowToast(
                                false,
                                format!("Couldn't write archived files list: {err:?}"),
                            ));
                        }
                    }
                }
            }
            state.current_project_being_managed = None;
            state.display_archive_project_view = false;
        }
        TasksPageMessage::UnarchiveProject => {
            if let Some(selected_folder) = state.selected_folder.as_ref() {
                if let Some(current_project_being_managed) =
                    state.current_project_being_managed.as_ref()
                {
                    let project_to_unarchive = current_project_being_managed
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase();
                    if let Some(index_to_remove) = state
                        .archived_list
                        .iter()
                        .position(|item| *item == project_to_unarchive)
                    {
                        state.archived_list.remove(index_to_remove);
                        match serde_json::to_string(&state.archived_list) {
                            Ok(serialised) => {
                                let _ =
                                    fs::write(selected_folder.join(ARCHIVED_FILE_NAME), serialised);
                            }
                            Err(err) => {
                                return Task::done(Message::ShowToast(
                                    false,
                                    format!("Couldn't write archived files list: {err:?}"),
                                ));
                            }
                        }
                    }
                }
            }
            state.current_project_being_managed = None;
            state.display_archive_project_view = false;
        }
        TasksPageMessage::ToggleArchiveProjectView => {
            state.display_archive_project_view = !state.display_archive_project_view;
        }
        TasksPageMessage::ToggleShowArchivedProjects => {
            state.show_archived_projects = !state.show_archived_projects;
        }
    }
    Task::none()
}
