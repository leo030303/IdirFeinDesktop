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
    /// The current language locale ID
    pub(crate) locale: fluent_templates::LanguageIdentifier,
    /// The path to the folder of project files to display, if any
    pub(crate) selected_folder: Option<PathBuf>,
    /// The path to the project file chosen to display, if any
    pub(crate) current_project_file: Option<PathBuf>,
    /// The format to display the tasks in, either Kanban or List
    pub(crate) task_view_type: TaskViewType,
    /// The list of paths of the project files in the selected directory
    pub(crate) projects_list: Vec<PathBuf>,
    /// The list of tasks in the selected project
    pub(crate) tasks_list: Vec<TaskData>,
    /// Whether to show the sidebar UI
    pub(crate) show_sidebar: bool,
    /// Whether to show the dialog to edit the currently selected task
    pub(crate) show_task_edit_dialog: bool,
    /// The contents of the task title field
    pub(crate) current_task_title_text: String,
    /// The contents of the task description field
    pub(crate) current_task_description_content: text_editor::Content,
    /// The ID of the task selected for management
    pub(crate) current_task_id: Option<Uuid>,
    /// Whether a confirmation dialog should be shown before deleting a task
    pub(crate) should_confirm_before_delete: bool,
    /// Whether to show the delete confirmation dialog UI on screen
    pub(crate) show_confirm_before_delete_dialog: bool,
    /// Whether the current project has been modified without being saved to disk
    pub(crate) is_dirty: bool,
    /// Whether the user is in the middle of creating a new project file
    pub(crate) is_creating_new_project: bool,
    /// The content of the new project name field
    pub(crate) new_project_name_field_content: String,
    /// Whether to show the UI to change a tasks completion state on each task in Kanban view
    pub(crate) show_task_completion_toolbar: bool,
    /// Whether to display the list of extra tools in the tool bar
    pub(crate) show_extra_tools_menu: bool,
    /// The project who's details are being edited, if any
    pub(crate) current_project_being_managed: Option<PathBuf>,
    /// Whether to display the UI to rename the selected project
    pub(crate) display_rename_project_view: bool,
    /// Whether to display the UI to delete the selected project
    pub(crate) display_delete_project_view: bool,
    /// Whether to display the UI to archive/unarchive the selected project
    pub(crate) display_archive_project_view: bool,
    /// The content of the rename project title field
    pub(crate) rename_project_field_text: String,
    /// Whether to show the task editing UI after right clicking a task in Kanban mode
    pub(crate) right_click_to_edit_task: bool,
    /// The string to filter the list of tasks by, titles and descriptions
    pub(crate) filter_tasks_text: String,
    /// The string to filter the list of projects by
    pub(crate) filter_projects_text: String,
    /// The filenames of the prjects in the current folder which have been archived
    pub(crate) archived_list: Vec<String>,
    /// Whether to list projects which have been archived
    pub(crate) show_archived_projects: bool,
}

#[derive(Debug, Clone)]
pub enum TasksPageMessage {
    PickProjectsFolder,
    SetProjectsFolder(Option<PathBuf>),
    LoadProjectsList,
    PickProjectFile(Option<PathBuf>),
    SetProjectsList(Vec<PathBuf>, Vec<String>),
    SetTasksList(Vec<TaskData>, PathBuf),
    SelectTaskToEdit(Option<Uuid>),
    DeleteTask(Uuid),
    DeleteTaskWithConfirmationCheck(Uuid),
    UpdateTaskTitle(String),
    UpdateTaskDescription(text_editor::Action),
    SetTaskCompletionState(Uuid, TaskCompletionState),
    UpdateCurrentTask,
    SaveProject,
    OpenEditDialogForTask(Uuid),
    StartCreatingNewTask,
    StartCreatingNewProject,
    CreateNewProject,
    CancelCreateNewProject,
    UpdateNewProjectNameEntry(String),
    EscapeKeyPressed,
    DropTask(Uuid, iced::Point, iced::Rectangle),
    HandleTaskDropZones(Uuid, Vec<(iced::advanced::widget::Id, iced::Rectangle)>),
    ShowMenuForProject(Option<PathBuf>),
    UpdateRenameProjectEntryText(String),
    RenameProject,
    ArchiveProject,
    UnarchiveProject,
    DeleteProject,
    SetShowTaskCompletionToolbar(bool),
    SetConfirmBeforeDelete(bool),
    SetRightClickToEditTask(bool),
    UpdateTasksFilter(String),
    UpdateProjectsFilter(String),
    ToggleExtraToolsMenu,
    ToggleRenameProjectView,
    ToggleDeleteProjectView,
    ToggleArchiveProjectView,
    ToggleShowArchivedProjects,
    ToggleShowTaskEditDialog,
    ToggleShowSidebar,
    ToggleConfirmBeforeDeleteDialog,
    ToggleTaskViewType,
}

impl TasksPage {
    pub fn new(config: &TaskPageConfig) -> Self {
        let locale: fluent_templates::LanguageIdentifier = current_locale::current_locale()
            .expect("Can't get locale")
            .parse()
            .expect("Failed to parse locale");
        Self {
            locale,
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
            should_confirm_before_delete: config.confirm_before_delete,
            show_confirm_before_delete_dialog: false,
            show_task_edit_dialog: false,
            current_task_title_text: String::new(),
            current_task_description_content: text_editor::Content::default(),
            current_task_id: None,
            is_dirty: false,
            is_creating_new_project: false,
            new_project_name_field_content: String::new(),
            show_task_completion_toolbar: config.show_task_completion_toolbar,
            show_extra_tools_menu: false,
            current_project_being_managed: None,
            display_rename_project_view: false,
            display_archive_project_view: false,
            display_delete_project_view: false,
            rename_project_field_text: String::new(),
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
        // Keyboard shortcuts
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
