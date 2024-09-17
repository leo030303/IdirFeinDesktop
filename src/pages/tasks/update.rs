use iced::Task;

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
        TasksPageMessage::PickTasksFolder => todo!(),
        TasksPageMessage::PickProjectFile => todo!(),
        TasksPageMessage::SetProjectsList(projects_list) => state.projects_list = projects_list,
        TasksPageMessage::SetTasksList(tasks_list) => state.tasks_list = tasks_list,
        TasksPageMessage::SelectTaskToEdit(task_uuid) => state.current_task_id = task_uuid,
        TasksPageMessage::DeleteTask(_) => todo!(),
        TasksPageMessage::SetTaskCompletionState(_) => todo!(),
        TasksPageMessage::CreateNewTask => {
            if let Some(tasks_list) = &mut state.tasks_list {
                tasks_list.push(TaskData {
                    title: state.current_task_title_text.clone(),
                    description: state.current_task_description_text.clone(),
                    ..Default::default()
                })
            }
        }
        TasksPageMessage::UpdateTaskContent => todo!(),
        TasksPageMessage::SaveProject => todo!(),
    }
    Task::none()
}
