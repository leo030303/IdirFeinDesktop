use std::path::PathBuf;

use iced::{Element, Task};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::Message;

use super::update::update;
use super::view::{main_view, tool_view};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskViewType {
    Kanban,
    List,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskCompletionState {
    Backlog,
    ToDo,
    Doing,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskData {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub completion_state: TaskCompletionState,
}

impl Default for TaskData {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            title: String::new(),
            description: String::new(),
            completion_state: TaskCompletionState::Backlog,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPageConfig {
    pub default_folder: Option<PathBuf>,
    pub default_project_file: Option<PathBuf>,
    pub default_task_view_type: TaskViewType,
    pub show_sidebar_on_start: bool,
    pub confirm_before_delete: bool,
    pub compact_task_view: bool,
}

impl Default for TaskPageConfig {
    fn default() -> Self {
        Self {
            default_folder: None,
            default_project_file: None,
            default_task_view_type: TaskViewType::Kanban,
            show_sidebar_on_start: true,
            confirm_before_delete: true,
            compact_task_view: false,
        }
    }
}

pub struct TasksPage {
    pub(crate) selected_folder: Option<PathBuf>,
    pub(crate) current_project_file: Option<PathBuf>,
    pub(crate) task_view_type: TaskViewType,
    pub(crate) projects_list: Vec<PathBuf>,
    pub(crate) tasks_list: Vec<TaskData>,
    pub(crate) show_sidebar: bool,
    pub(crate) show_task_edit_dialog: bool,
    pub(crate) current_task_title_text: String,
    pub(crate) current_task_description_text: String,
    pub(crate) current_task_id: Option<Uuid>,
    pub(crate) confirm_before_delete: bool,
    pub(crate) show_confirm_before_delete_dialog: bool,
    pub(crate) compact_task_view: bool,
    pub(crate) is_dirty: bool,
}

#[derive(Debug, Clone)]
pub enum TasksPageMessage {
    ToggleShowTaskEditDialog,
    ToggleShowSidebar,
    ToggleCompactTaskView,
    ToggleConfirmBeforeDeleteDialog,
    SetTaskViewType(TaskViewType),
    PickTasksFolder,
    PickProjectFile(PathBuf),
    SetProjectsList(Vec<PathBuf>),
    SetTasksList(Vec<TaskData>),
    SelectTaskToEdit(Option<Uuid>),
    DeleteTask(Uuid),
    SetTaskCompletionState((Uuid, TaskCompletionState)),
    CreateNewTask,
    UpdateTaskContent,
    SaveProject,
}

impl TasksPage {
    pub fn new(config: &TaskPageConfig) -> Self {
        Self {
            selected_folder: config.default_folder.clone(),
            current_project_file: config.default_project_file.clone(),
            task_view_type: config.default_task_view_type.clone(),
            tasks_list: vec![],
            projects_list: vec![],
            show_sidebar: config.show_sidebar_on_start,
            confirm_before_delete: config.confirm_before_delete,
            show_confirm_before_delete_dialog: false,
            show_task_edit_dialog: false,
            current_task_title_text: String::new(),
            current_task_description_text: String::new(),
            compact_task_view: config.compact_task_view,
            current_task_id: None,
            is_dirty: false,
        }
    }

    pub fn opening_task() -> Task<Message> {
        Task::none()
    }

    pub fn closing_task(&mut self) -> Task<Message> {
        println!("Closing task from tasks");
        Task::none()
    }

    pub fn update(&mut self, message: TasksPageMessage) -> Task<Message> {
        update(self, message)
    }

    pub fn view(&self) -> Element<Message> {
        main_view(self)
    }

    pub fn tool_view(&self) -> Element<Message> {
        tool_view(self)
    }
}
