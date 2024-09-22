use std::{
    fs::{self},
    path::PathBuf,
};

use iced::Task;
use rfd::FileDialog;

use crate::app::Message;

use super::page::{TaskData, TasksPage, TasksPageMessage};

pub fn update(state: &mut TasksPage, message: TasksPageMessage) -> Task<Message> {
    match message {
        TasksPageMessage::ToggleShowTaskEditDialog => {
            state.show_task_edit_dialog = !state.show_task_edit_dialog
        }
        TasksPageMessage::ToggleShowSidebar => state.show_sidebar = !state.show_sidebar,
        TasksPageMessage::ToggleCompactTaskView => {
            state.compact_task_view = !state.compact_task_view
        }
        TasksPageMessage::ToggleConfirmBeforeDeleteDialog => {
            state.show_confirm_before_delete_dialog = !state.show_confirm_before_delete_dialog
        }
        TasksPageMessage::SetTaskViewType(task_type) => state.task_view_type = task_type,
        TasksPageMessage::PickTasksFolder => {
            let selected_folder = FileDialog::new().set_directory("/").pick_folder();
            state.selected_folder = selected_folder.clone();
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
        TasksPageMessage::PickProjectFile(path_to_file) => {
            return Task::perform(
                async {
                    if let Ok(task_json) = fs::read_to_string(path_to_file) {
                        let tasks_list: Vec<TaskData> =
                            serde_json::from_str(&task_json).unwrap_or_default();
                        tasks_list
                    } else {
                        vec![]
                    }
                },
                |tasks_list| Message::Tasks(TasksPageMessage::SetTasksList(tasks_list)),
            )
        }
        TasksPageMessage::SetProjectsList(projects_list) => state.projects_list = projects_list,
        TasksPageMessage::SetTasksList(tasks_list) => state.tasks_list = tasks_list,
        TasksPageMessage::SelectTaskToEdit(task_uuid) => state.current_task_id = task_uuid,
        TasksPageMessage::DeleteTask(id_to_delete) => {
            if let Some(task_index) = state.tasks_list.iter().position(|x| x.id == id_to_delete) {
                state.tasks_list.remove(task_index);
                state.is_dirty = true;
            }
            return Task::done(Message::Tasks(TasksPageMessage::SaveProject));
        }
        TasksPageMessage::SetTaskCompletionState((id_to_edit, task_completion_state)) => {
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
        TasksPageMessage::CreateNewTask => {
            state.tasks_list.push(TaskData {
                title: state.current_task_title_text.clone(),
                description: state.current_task_description_text.clone(),
                ..Default::default()
            });
        }
        TasksPageMessage::UpdateTaskContent => todo!(),
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
    }
    Task::none()
}
