use std::path::PathBuf;

use iced::event::Status;
use iced::keyboard::key::Named;
use iced::keyboard::{self, Key, Modifiers};
use iced::widget::text_editor;
use iced::{event, Element, Event, Task};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::Message;

use super::update::update;
use super::view::{main_view, tool_view};

pub const ARCHIVED_FILE_NAME: &str = ".archived";
pub const TASK_TITLE_TEXT_INPUT_ID: &str = "TASK_TITLE_TEXT_INPUT_ID";
pub const NEW_PROJECT_TEXT_INPUT_ID: &str = "NEW_PROJECT_TEXT_INPUT_ID";
pub const RENAME_PROJECT_TEXT_INPUT_ID: &str = "RENAME_PROJECT_TEXT_INPUT_ID";
pub const BACKLOG_ID: &str = "BACKLOG_ID";
pub const TODO_ID: &str = "TODO_ID";
pub const DOING_ID: &str = "DOING_ID";
pub const DONE_ID: &str = "DONE_ID";

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
    pub kanban_task_view_is_default: bool,
    pub show_sidebar_on_start: bool,
    pub confirm_before_delete: bool,
    pub show_task_completion_toolbar: bool,
    pub right_click_to_edit_task: bool,
}

impl Default for TaskPageConfig {
    fn default() -> Self {
        Self {
            default_folder: None,
            default_project_file: None,
            kanban_task_view_is_default: true,
            show_sidebar_on_start: true,
            confirm_before_delete: true,
            show_task_completion_toolbar: false,
            right_click_to_edit_task: true,
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
    pub(crate) current_task_description_content: text_editor::Content,
    pub(crate) current_task_id: Option<Uuid>,
    pub(crate) confirm_before_delete: bool,
    pub(crate) show_confirm_before_delete_dialog: bool,
    pub(crate) is_dirty: bool,
    pub(crate) is_creating_new_project: bool,
    pub(crate) new_project_name_entry_content: String,
    pub(crate) show_task_completion_toolbar: bool,
    pub(crate) show_extra_tools_menu: bool,
    pub(crate) current_project_being_managed: Option<PathBuf>,
    pub(crate) display_rename_view: bool,
    pub(crate) display_delete_view: bool,
    pub(crate) display_archive_view: bool,
    pub(crate) rename_project_entry_text: String,
    pub(crate) right_click_to_edit_task: bool,
    pub(crate) filter_tasks_text: String,
    pub(crate) filter_projects_text: String,
    pub(crate) archived_list: Vec<String>,
    pub(crate) show_archived_projects: bool,
}

#[derive(Debug, Clone)]
pub enum TasksPageMessage {
    ToggleShowTaskEditDialog,
    ToggleShowSidebar,
    ToggleConfirmBeforeDeleteDialog,
    ToggleTaskViewType,
    PickProjectsFolder,
    SetProjectsFolder(Option<PathBuf>),
    LoadProjectsList,
    PickProjectFile(Option<PathBuf>),
    SetProjectsList(Vec<PathBuf>, Vec<String>),
    SetTasksList(Vec<TaskData>, PathBuf),
    SelectTaskToEdit(Option<Uuid>),
    DeleteTask(Uuid),
    DeleteTaskWithConfirmationCheck(Uuid),
    OpenEditDialogForTask(Uuid),
    UpdateTaskTitle(String),
    UpdateTaskDescription(text_editor::Action),
    SetTaskCompletionState(Uuid, TaskCompletionState),
    UpdateCurrentTask,
    SaveProject,
    StartCreatingNewTask,
    ClearAndCloseTaskEditDialog,
    StartCreatingNewProject,
    CreateNewProject,
    UpdateNewProjectNameEntry(String),
    CancelCreateNewProject,
    EscapeKeyPressed,
    DropTask(Uuid, iced::Point, iced::Rectangle),
    HandleTaskDropZones(Uuid, Vec<(iced::advanced::widget::Id, iced::Rectangle)>),
    ToggleExtraToolsMenu,
    ShowMenuForProject(Option<PathBuf>),
    SetRenameProjectEntryText(String),
    RenameProject,
    ArchiveProject,
    UnarchiveProject,
    DeleteProject,
    ToggleRenameProjectView,
    ToggleDeleteProjectView,
    ToggleArchiveProjectView,
    SetShowTaskCompletionToolbar(bool),
    SetConfirmBeforeDelete(bool),
    SetRightClickToEditTask(bool),
    UpdateTasksFilter(String),
    UpdateProjectsFilter(String),
    ToggleShowArchivedProjects,
}

impl TasksPage {
    pub fn new(config: &TaskPageConfig) -> Self {
        Self {
            selected_folder: config.default_folder.clone(),
            current_project_file: config.default_project_file.clone(),
            task_view_type: if config.kanban_task_view_is_default {
                TaskViewType::Kanban
            } else {
                TaskViewType::List
            },
            tasks_list: vec![],
            projects_list: vec![],
            show_sidebar: config.show_sidebar_on_start,
            confirm_before_delete: config.confirm_before_delete,
            show_confirm_before_delete_dialog: false,
            show_task_edit_dialog: false,
            current_task_title_text: String::new(),
            current_task_description_content: text_editor::Content::with_text(""),
            current_task_id: None,
            is_dirty: false,
            is_creating_new_project: false,
            new_project_name_entry_content: String::new(),
            show_task_completion_toolbar: config.show_task_completion_toolbar,
            show_extra_tools_menu: false,
            current_project_being_managed: None,
            display_rename_view: false,
            display_archive_view: false,
            display_delete_view: false,
            rename_project_entry_text: String::new(),
            right_click_to_edit_task: config.right_click_to_edit_task,
            filter_tasks_text: String::new(),
            filter_projects_text: String::new(),
            archived_list: vec![],
            show_archived_projects: false,
        }
    }

    pub fn opening_task() -> Task<Message> {
        Task::done(Message::Tasks(TasksPageMessage::LoadProjectsList))
    }

    pub fn closing_task(&mut self) -> Task<Message> {
        Task::done(Message::Tasks(TasksPageMessage::SaveProject))
    }

    pub fn update(&mut self, message: TasksPageMessage) -> Task<Message> {
        update(self, message)
    }

    pub fn view(&self) -> Element<Message> {
        main_view(self)
    }

    pub fn subscription() -> iced::Subscription<Message> {
        event::listen_with(|event, status, _id| match (event, status) {
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: Key::Character(pressed_char),
                    modifiers: Modifiers::CTRL,
                    ..
                }),
                _,
            ) => {
                if pressed_char.as_ref() == "n" || pressed_char.as_ref() == "N" {
                    Some(Message::Tasks(TasksPageMessage::StartCreatingNewTask))
                } else if pressed_char.as_ref() == "b" || pressed_char.as_ref() == "B" {
                    Some(Message::Tasks(TasksPageMessage::ToggleShowSidebar))
                } else if pressed_char.as_ref() == "l" || pressed_char.as_ref() == "L" {
                    Some(Message::Tasks(TasksPageMessage::ToggleTaskViewType))
                } else if pressed_char.as_ref() == "s" || pressed_char.as_ref() == "S" {
                    Some(Message::Tasks(TasksPageMessage::UpdateCurrentTask))
                } else {
                    None
                }
            }
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: Key::Named(Named::Escape),
                    ..
                }),
                Status::Ignored,
            ) => Some(Message::Tasks(TasksPageMessage::EscapeKeyPressed)),
            _ => None,
        })
    }

    pub fn tool_view(&self) -> Element<Message> {
        tool_view(self)
    }
}
