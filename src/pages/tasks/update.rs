use std::{
    fs::{self, File},
    path::PathBuf,
};

use iced::{
    widget::{text_editor, text_input},
    Task,
};
use rfd::FileDialog;

use crate::app::Message;

use super::page::{
    TaskData, TasksPage, TasksPageMessage, NEW_PROJECT_TEXT_INPUT_ID, TASK_TITLE_TEXT_INPUT_ID,
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
                    let projects_list: Vec<PathBuf> = if let Some(selected_folder) = selected_folder
                    {
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
                    projects_list
                },
                |projects_list| Message::Tasks(TasksPageMessage::SetProjectsList(projects_list)),
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

        TasksPageMessage::PickProjectFile(path_to_file) => {
            return Task::batch([
                Task::done(Message::Tasks(TasksPageMessage::SaveProject)),
                Task::perform(
                    async move {
                        if let Ok(task_json) = fs::read_to_string(&path_to_file) {
                            let tasks_list: Vec<TaskData> =
                                serde_json::from_str(&task_json).unwrap_or_default();
                            (tasks_list, path_to_file)
                        } else {
                            (vec![], path_to_file)
                        }
                    },
                    |(tasks_list, path_to_file)| {
                        Message::Tasks(TasksPageMessage::SetTasksList(tasks_list, path_to_file))
                    },
                ),
            ]);
        }
        TasksPageMessage::SetProjectsList(projects_list) => state.projects_list = projects_list,
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
                            .title = state.current_task_title_text.clone();
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
                        title: state.current_task_title_text.clone(),
                        description: state.current_task_description_content.text(),
                        ..Default::default()
                    });
                    state.is_dirty = true;
                }
            };
            return Task::done(Message::Tasks(TasksPageMessage::SaveProject)).chain(Task::done(
                Message::Tasks(TasksPageMessage::ClearAndCloseTaskEditDialog),
            ));
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
        TasksPageMessage::ClearAndCloseTaskEditDialog => {
            state.current_task_title_text = String::new();
            state.current_task_description_content = text_editor::Content::with_text("");
            state.current_task_id = None;
            state.show_task_edit_dialog = false;
        }
        TasksPageMessage::DeleteTaskWithConfirmationCheck(task_id) => {
            if state.confirm_before_delete {
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
                selected_folder.push(&state.new_project_name_entry_content);
                state.new_project_name_entry_content = String::new();
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
        TasksPageMessage::UpdateNewProjectNameEntry(s) => state.new_project_name_entry_content = s,
        TasksPageMessage::CancelCreateNewProject => {
            state.is_creating_new_project = false;
            state.new_project_name_entry_content = String::new();
        }
        TasksPageMessage::EscapeKeyPressed => {
            state.show_task_edit_dialog = false;
            state.current_task_title_text = String::new();
            state.current_task_description_content = text_editor::Content::with_text("");
            state.current_task_id = None;
            state.show_confirm_before_delete_dialog = false;
        }
    }
    Task::none()
}
