use crate::{
    pages::tasks::page::{TaskViewType, NEW_PROJECT_TEXT_INPUT_ID},
    LOCALES,
};
use fluent_templates::Loader;
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, row, scrollable, svg, text, text_editor, text_input, MouseArea,
        Row, Space, Svg, Tooltip,
    },
    Alignment::Center,
    Element, Font, Length,
};
use iced_aw::{drag_and_drop::droppable, drop_down, DropDown};

use crate::app::Message;

use super::page::{
    TaskCompletionState, TaskData, TasksPage, TasksPageMessage, BACKLOG_ID, DOING_ID, DONE_ID,
    RENAME_PROJECT_TEXT_INPUT_ID, TASK_TITLE_TEXT_INPUT_ID, TODO_ID,
};

pub fn main_view(state: &TasksPage) -> Element<Message> {
    if state.selected_folder.is_none() {
        no_project_folder_selected_view(state)
    } else {
        row![
            if state.show_sidebar {
                sidebar_view(state)
            } else {
                column![].into()
            },
            if state.current_project_file.is_some() {
                column![
                    row![
                        text(
                            state
                                .current_project_file
                                .as_ref()
                                .expect("Can't fail")
                                .file_stem()
                                .unwrap_or_default()
                                .to_str()
                                .unwrap_or(&LOCALES.lookup(&state.locale, "couldnt-read-filename"))
                                .to_string(),
                        )
                        .size(28)
                        .width(Length::FillPortion(1))
                        .align_x(Center),
                        text_input(
                            &LOCALES.lookup(&state.locale, "filter"),
                            &state.filter_tasks_text
                        )
                        .on_input(|s| Message::Tasks(TasksPageMessage::UpdateTasksFilter(s)))
                        .width(Length::FillPortion(1))
                    ],
                    if state.show_confirm_before_delete_dialog {
                        confirm_delete_view(state)
                    } else {
                        column![].into()
                    },
                    if state.show_task_edit_dialog {
                        task_edit_dialog(state)
                    } else {
                        column![].into()
                    },
                    match state.task_view_type {
                        TaskViewType::Kanban => kanban_view(state),
                        TaskViewType::List => list_view(state),
                    }
                ]
                .width(Length::FillPortion(2))
                .padding(20)
            } else {
                column![text(LOCALES.lookup(&state.locale, "no-project-selected"))
                    .width(Length::Fill)
                    .height(100)
                    .align_x(Center)
                    .align_y(Center)
                    .size(24)]
                .width(Length::FillPortion(2))
            }
        ]
        .into()
    }
}

fn kanban_view_item<'a>(state: &'a TasksPage, task: &'a TaskData) -> Element<'a, Message> {
    let task_details: Element<'a, Message> = column![
        text(&task.title)
            .size(20)
            .width(Length::Fill)
            .align_x(Center),
        text(&task.description),
    ]
    .into();
    droppable(
        container(
            column![
                if state.right_click_to_edit_task {
                    MouseArea::new(task_details)
                        .on_right_release(Message::Tasks(TasksPageMessage::OpenEditDialogForTask(
                            task.id,
                        )))
                        .into()
                } else {
                    task_details
                },
                if state.show_task_completion_toolbar {
                    let mut state_setter_row = Row::new();
                    if !matches!(task.completion_state, TaskCompletionState::Backlog) {
                        state_setter_row = state_setter_row.push(Tooltip::new(
                            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                                "../../../icons/1.svg"
                            ))))
                            .on_press(Message::Tasks(TasksPageMessage::SetTaskCompletionState(
                                task.id,
                                TaskCompletionState::Backlog,
                            )))
                            .width(Length::Fill),
                            text(LOCALES.lookup(&state.locale, "backlog")),
                            iced::widget::tooltip::Position::Bottom,
                        ));
                    }
                    if !matches!(task.completion_state, TaskCompletionState::ToDo) {
                        state_setter_row = state_setter_row.push(Tooltip::new(
                            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                                "../../../icons/2.svg"
                            ))))
                            .on_press(Message::Tasks(TasksPageMessage::SetTaskCompletionState(
                                task.id,
                                TaskCompletionState::ToDo,
                            )))
                            .width(Length::Fill),
                            text(LOCALES.lookup(&state.locale, "todo")),
                            iced::widget::tooltip::Position::Bottom,
                        ));
                    }
                    if !matches!(task.completion_state, TaskCompletionState::Doing) {
                        state_setter_row = state_setter_row.push(Tooltip::new(
                            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                                "../../../icons/3.svg"
                            ))))
                            .on_press(Message::Tasks(TasksPageMessage::SetTaskCompletionState(
                                task.id,
                                TaskCompletionState::Doing,
                            )))
                            .width(Length::Fill),
                            text(LOCALES.lookup(&state.locale, "doing")),
                            iced::widget::tooltip::Position::Bottom,
                        ));
                    }
                    if !matches!(task.completion_state, TaskCompletionState::Done) {
                        state_setter_row = state_setter_row.push(Tooltip::new(
                            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                                "../../../icons/4.svg"
                            ))))
                            .on_press(Message::Tasks(TasksPageMessage::SetTaskCompletionState(
                                task.id,
                                TaskCompletionState::Done,
                            )))
                            .width(Length::Fill)
                            .style(button::success),
                            text(LOCALES.lookup(&state.locale, "done")),
                            iced::widget::tooltip::Position::Bottom,
                        ));
                    }
                    state_setter_row
                } else {
                    row![]
                },
                row![
                    button(Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/edit.svg"
                    ))))
                    .width(Length::Fill)
                    .on_press(Message::Tasks(
                        TasksPageMessage::OpenEditDialogForTask(task.id)
                    )),
                    button(Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/delete.svg"
                    ))))
                    .style(button::danger)
                    .width(Length::Fill)
                    .on_press(Message::Tasks(
                        TasksPageMessage::DeleteTaskWithConfirmationCheck(task.id)
                    ))
                ]
            ]
            .padding(5),
        )
        .style(container::bordered_box),
    )
    .on_drop(|point, rectangle| {
        Message::Tasks(TasksPageMessage::DropTask(task.id, point, rectangle))
    })
    .into()
}

fn list_view_item(task: &TaskData) -> Element<Message> {
    text(&task.title).into()
}

fn task_edit_dialog(state: &TasksPage) -> Element<Message> {
    column![
        row![
            text(if state.current_task_id.is_none() {
                LOCALES.lookup(&state.locale, "new-task")
            } else {
                LOCALES.lookup(&state.locale, "edit-task")
            })
            .width(Length::Fill)
            .size(24),
            Tooltip::new(
                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/close.svg"
                ))))
                .on_press(Message::Tasks(TasksPageMessage::ToggleShowTaskEditDialog))
                .width(Length::Fixed(50.0)),
                text(LOCALES.lookup(&state.locale, "close-edit-dialog-shortcut")),
                iced::widget::tooltip::Position::Bottom
            ),
        ],
        text_input(
            &LOCALES.lookup(&state.locale, "task-title"),
            &state.current_task_title_text
        )
        .on_input(|s| Message::Tasks(TasksPageMessage::UpdateTaskTitle(s)))
        .on_submit(Message::Tasks(TasksPageMessage::UpdateCurrentTask))
        .id(text_input::Id::new(TASK_TITLE_TEXT_INPUT_ID)),
        text_editor(&state.current_task_description_content)
            .placeholder(LOCALES.lookup(&state.locale, "task-description"))
            .on_action(|action| Message::Tasks(TasksPageMessage::UpdateTaskDescription(action)))
            .height(Length::Fixed(300.0))
            .padding(10)
            .font(Font::MONOSPACE),
        row![
            button(
                text(LOCALES.lookup(&state.locale, "cancel-shortcut"))
                    .align_x(Center)
                    .width(Length::Fill)
            )
            .width(Length::Fill)
            .on_press(Message::Tasks(TasksPageMessage::ToggleShowTaskEditDialog)),
            button(
                text(LOCALES.lookup(&state.locale, "save-task-shortcut"))
                    .align_x(Center)
                    .width(Length::Fill)
            )
            .width(Length::Fill)
            .style(button::success)
            .on_press(Message::Tasks(TasksPageMessage::UpdateCurrentTask))
        ]
        .spacing(20)
    ]
    .padding(20)
    .into()
}

fn kanban_view(state: &TasksPage) -> Element<Message> {
    row![
        column![
            row![
                Space::with_width(Length::Fill),
                text(LOCALES.lookup(&state.locale, "backlog")).size(24),
                Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/1.svg"
                )))
                .style(|theme, _status| svg::Style {
                    color: text::base(theme).color
                })
                .height(Length::Fixed(24.0)),
                Space::with_width(Length::Fill),
            ]
            .padding(5),
            text(format!(
                "{} {}",
                LOCALES.lookup(&state.locale, "tasks-count"),
                state
                    .tasks_list
                    .iter()
                    .filter(|task| matches!(task.completion_state, TaskCompletionState::Backlog))
                    .count()
            ))
            .font(Font {
                style: iced::font::Style::Italic,
                ..Default::default()
            })
            .width(Length::Fill)
            .align_x(Center),
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(
                            task.completion_state,
                            TaskCompletionState::Backlog
                        ))
                        .filter(|task| {
                            task.title
                                .to_lowercase()
                                .contains(&state.filter_tasks_text.to_lowercase())
                                || task
                                    .description
                                    .to_lowercase()
                                    .contains(&state.filter_tasks_text.to_lowercase())
                        })
                        .map(|task| kanban_view_item(state, task))
                )
                .padding(5)
                .spacing(10)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .id(scrollable::Id::new(BACKLOG_ID))
        ]
        .width(Length::Fill)
        .align_x(Center),
        column![
            row![
                Space::with_width(Length::Fill),
                text(LOCALES.lookup(&state.locale, "todo")).size(24),
                Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/2.svg"
                )))
                .style(|theme, _status| svg::Style {
                    color: text::base(theme).color
                })
                .height(Length::Fixed(24.0)),
                Space::with_width(Length::Fill),
            ]
            .padding(5),
            text(format!(
                "{} {}",
                LOCALES.lookup(&state.locale, "tasks-count"),
                state
                    .tasks_list
                    .iter()
                    .filter(|task| matches!(task.completion_state, TaskCompletionState::ToDo))
                    .count()
            ))
            .font(Font {
                style: iced::font::Style::Italic,
                ..Default::default()
            })
            .width(Length::Fill)
            .align_x(Center),
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(task.completion_state, TaskCompletionState::ToDo))
                        .filter(|task| {
                            task.title
                                .to_lowercase()
                                .contains(&state.filter_tasks_text.to_lowercase())
                                || task
                                    .description
                                    .to_lowercase()
                                    .contains(&state.filter_tasks_text.to_lowercase())
                        })
                        .map(|task| kanban_view_item(state, task))
                )
                .padding(5)
                .spacing(10)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .id(scrollable::Id::new(TODO_ID))
        ]
        .width(Length::Fill)
        .align_x(Center),
        column![
            row![
                Space::with_width(Length::Fill),
                text(LOCALES.lookup(&state.locale, "doing")).size(24),
                Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/3.svg"
                )))
                .style(|theme, _status| svg::Style {
                    color: text::base(theme).color
                })
                .height(Length::Fixed(24.0)),
                Space::with_width(Length::Fill),
            ]
            .padding(5),
            text(format!(
                "{} {}",
                LOCALES.lookup(&state.locale, "tasks-count"),
                state
                    .tasks_list
                    .iter()
                    .filter(|task| matches!(task.completion_state, TaskCompletionState::Doing))
                    .count()
            ))
            .font(Font {
                style: iced::font::Style::Italic,
                ..Default::default()
            })
            .width(Length::Fill)
            .align_x(Center),
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(task.completion_state, TaskCompletionState::Doing))
                        .filter(|task| {
                            task.title
                                .to_lowercase()
                                .contains(&state.filter_tasks_text.to_lowercase())
                                || task
                                    .description
                                    .to_lowercase()
                                    .contains(&state.filter_tasks_text.to_lowercase())
                        })
                        .map(|task| kanban_view_item(state, task))
                )
                .padding(5)
                .spacing(10)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .id(scrollable::Id::new(DOING_ID))
        ]
        .width(Length::Fill)
        .align_x(Center),
        column![
            row![
                Space::with_width(Length::Fill),
                text(LOCALES.lookup(&state.locale, "done")).size(24),
                Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/4.svg"
                )))
                .style(|theme, _status| svg::Style {
                    color: text::base(theme).color
                })
                .height(Length::Fixed(24.0)),
                Space::with_width(Length::Fill),
            ]
            .padding(5),
            text(format!(
                "{} {}",
                LOCALES.lookup(&state.locale, "tasks-count-arg"),
                state
                    .tasks_list
                    .iter()
                    .filter(|task| matches!(task.completion_state, TaskCompletionState::Done))
                    .count()
            ))
            .font(Font {
                style: iced::font::Style::Italic,
                ..Default::default()
            })
            .width(Length::Fill)
            .align_x(Center),
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(task.completion_state, TaskCompletionState::Done))
                        .filter(|task| {
                            task.title
                                .to_lowercase()
                                .contains(&state.filter_tasks_text.to_lowercase())
                                || task
                                    .description
                                    .to_lowercase()
                                    .contains(&state.filter_tasks_text.to_lowercase())
                        })
                        .map(|task| kanban_view_item(state, task))
                )
                .padding(5)
                .spacing(10)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .id(scrollable::Id::new(DONE_ID))
        ]
        .width(Length::Fill)
        .align_x(Center),
    ]
    .into()
}

fn list_view(state: &TasksPage) -> Element<Message> {
    scrollable(column![
        column![
            text(LOCALES.lookup(&state.locale, "backlog"))
                .width(Length::Fill)
                .align_x(Center)
                .size(20),
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(
                            task.completion_state,
                            TaskCompletionState::Backlog
                        ))
                        .filter(|task| {
                            task.title
                                .to_lowercase()
                                .contains(&state.filter_tasks_text.to_lowercase())
                                || task
                                    .description
                                    .to_lowercase()
                                    .contains(&state.filter_tasks_text.to_lowercase())
                        })
                        .map(|task| list_view_item(task))
                )
                .spacing(10)
            )
        ],
        column![
            text(LOCALES.lookup(&state.locale, "todo"))
                .width(Length::Fill)
                .align_x(Center)
                .size(20),
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(task.completion_state, TaskCompletionState::ToDo))
                        .filter(|task| {
                            task.title
                                .to_lowercase()
                                .contains(&state.filter_tasks_text.to_lowercase())
                                || task
                                    .description
                                    .to_lowercase()
                                    .contains(&state.filter_tasks_text.to_lowercase())
                        })
                        .map(|task| list_view_item(task))
                )
                .spacing(10)
            )
        ],
        column![
            text(LOCALES.lookup(&state.locale, "doing"))
                .width(Length::Fill)
                .align_x(Center)
                .size(20),
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(task.completion_state, TaskCompletionState::Doing))
                        .filter(|task| {
                            task.title
                                .to_lowercase()
                                .contains(&state.filter_tasks_text.to_lowercase())
                                || task
                                    .description
                                    .to_lowercase()
                                    .contains(&state.filter_tasks_text.to_lowercase())
                        })
                        .map(|task| list_view_item(task))
                )
                .spacing(10)
            )
        ],
        column![
            text(LOCALES.lookup(&state.locale, "done"))
                .width(Length::Fill)
                .align_x(Center)
                .size(20),
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(task.completion_state, TaskCompletionState::Done))
                        .filter(|task| {
                            task.title
                                .to_lowercase()
                                .contains(&state.filter_tasks_text.to_lowercase())
                                || task
                                    .description
                                    .to_lowercase()
                                    .contains(&state.filter_tasks_text.to_lowercase())
                        })
                        .map(|task| list_view_item(task))
                )
                .spacing(10)
            )
        ],
    ])
    .into()
}

fn confirm_delete_view(state: &TasksPage) -> Element<Message> {
    if state.current_task_id.is_some() {
        column![
            text(LOCALES.lookup(&state.locale, "delete-this-task"))
                .width(Length::Fill)
                .size(24),
            row![
                button(
                    text(LOCALES.lookup(&state.locale, "cancel-shortcut"))
                        .align_x(Center)
                        .width(Length::Fill)
                )
                .width(Length::Fill)
                .on_press(Message::Tasks(
                    TasksPageMessage::ToggleConfirmBeforeDeleteDialog
                )),
                button(
                    text(LOCALES.lookup(&state.locale, "delete"))
                        .align_x(Center)
                        .width(Length::Fill)
                )
                .width(Length::Fill)
                .style(button::danger)
                .on_press(Message::Tasks(TasksPageMessage::DeleteTask(
                    state.current_task_id.expect("Checked this was some")
                )))
            ]
            .spacing(20)
        ]
    } else {
        column![text(LOCALES.lookup(&state.locale, "no-task-selected"))]
    }
    .into()
}

fn no_project_folder_selected_view(state: &TasksPage) -> Element<Message> {
    container(
        button(
            text(LOCALES.lookup(&state.locale, "select-projects-folder"))
                .size(20)
                .height(Length::Fixed(40.0))
                .align_y(Center)
                .width(Length::Fill)
                .align_x(Center),
        )
        .on_press(Message::Tasks(TasksPageMessage::PickProjectsFolder))
        .width(Length::Fixed(250.0)),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .into()
}

fn sidebar_view(state: &TasksPage) -> Element<Message> {
    column![
        Space::with_height(10),
        if state.is_creating_new_project {
            row![
                text_input(
                    &LOCALES.lookup(&state.locale, "new-project-name"),
                    &state.new_project_name_field_content
                )
                .width(Length::Fill)
                .on_input(|s| Message::Tasks(TasksPageMessage::UpdateNewProjectNameEntry(s)))
                .on_submit(Message::Tasks(TasksPageMessage::CreateNewProject))
                .id(text_input::Id::new(NEW_PROJECT_TEXT_INPUT_ID)),
                Tooltip::new(
                    button(Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/ok.svg"
                    ))))
                    .on_press(Message::Tasks(TasksPageMessage::CreateNewProject))
                    .style(button::success)
                    .width(Length::Fixed(50.0))
                    .height(Length::Fixed(30.0)),
                    text(LOCALES.lookup(&state.locale, "create")),
                    iced::widget::tooltip::Position::Bottom
                ),
                Tooltip::new(
                    button(Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/close.svg"
                    ))))
                    .on_press(Message::Tasks(TasksPageMessage::CancelCreateNewProject))
                    .style(button::danger)
                    .width(Length::Fixed(50.0))
                    .height(Length::Fixed(30.0)),
                    text(LOCALES.lookup(&state.locale, "cancel")),
                    iced::widget::tooltip::Position::Bottom
                ),
            ]
        } else {
            row![
                button(
                    text(if state.show_archived_projects {
                        LOCALES.lookup(&state.locale, "hide-archived")
                    } else {
                        LOCALES.lookup(&state.locale, "show-archived")
                    })
                    .width(Length::Fill)
                    .align_x(Center)
                )
                .width(Length::Fill)
                .on_press(Message::Tasks(TasksPageMessage::ToggleShowArchivedProjects)),
                button(
                    text(LOCALES.lookup(&state.locale, "new-project"))
                        .width(Length::Fill)
                        .align_x(Center)
                )
                .width(Length::Fill)
                .style(button::success)
                .on_press(Message::Tasks(TasksPageMessage::StartCreatingNewProject))
            ]
            .spacing(5)
        },
        Space::with_height(20),
        text_input(
            &LOCALES.lookup(&state.locale, "filter"),
            &state.filter_projects_text
        )
        .on_input(|s| { Message::Tasks(TasksPageMessage::UpdateProjectsFilter(s)) }),
        scrollable(
            column(
                state
                    .projects_list
                    .iter()
                    .filter(|project| !state.archived_list.contains(
                        &project
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_lowercase()
                    ) ^ state.show_archived_projects)
                    .filter(|project| project
                        .file_stem()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or(&LOCALES.lookup(&state.locale, "couldnt-read-filename"))
                        .to_lowercase()
                        .contains(&state.filter_projects_text.to_lowercase()))
                    .map(|project| {
                        if state
                            .current_project_being_managed
                            .as_ref()
                            .is_some_and(|selected_project| *selected_project == *project)
                        {
                            if state.show_archived_projects {
                                row![
                                    button(
                                        text(LOCALES.lookup(&state.locale, "unarchive"))
                                            .width(Length::Fill)
                                            .align_x(Center)
                                    )
                                    .width(Length::Fill)
                                    .on_press(Message::Tasks(TasksPageMessage::UnarchiveProject)),
                                    Tooltip::new(
                                        button(Svg::new(svg::Handle::from_memory(include_bytes!(
                                            "../../../icons/close.svg"
                                        ))))
                                        .on_press(Message::Tasks(
                                            TasksPageMessage::ShowMenuForProject(None)
                                        ))
                                        .width(Length::Fixed(50.0))
                                        .height(Length::Fixed(30.0)),
                                        text(LOCALES.lookup(&state.locale, "close")),
                                        iced::widget::tooltip::Position::Right,
                                    )
                                ]
                                .spacing(5)
                                .into()
                            } else if state.display_rename_project_view {
                                row![
                                    text_input(
                                        &LOCALES.lookup(&state.locale, "rename-project"),
                                        &state.rename_project_field_text
                                    )
                                    .width(Length::Fill)
                                    .on_input(|s| Message::Tasks(
                                        TasksPageMessage::UpdateRenameProjectEntryText(s)
                                    ))
                                    .on_submit(Message::Tasks(TasksPageMessage::RenameProject))
                                    .id(text_input::Id::new(RENAME_PROJECT_TEXT_INPUT_ID)),
                                    Tooltip::new(
                                        button(Svg::new(svg::Handle::from_memory(include_bytes!(
                                            "../../../icons/ok.svg"
                                        ))))
                                        .on_press(Message::Tasks(TasksPageMessage::RenameProject))
                                        .style(button::success)
                                        .width(Length::Fixed(50.0))
                                        .height(Length::Fixed(30.0)),
                                        text(LOCALES.lookup(&state.locale, "rename")),
                                        iced::widget::tooltip::Position::Bottom
                                    ),
                                    Tooltip::new(
                                        button(Svg::new(svg::Handle::from_memory(include_bytes!(
                                            "../../../icons/close.svg"
                                        ))))
                                        .on_press(Message::Tasks(
                                            TasksPageMessage::ToggleRenameProjectView
                                        ))
                                        .style(button::danger)
                                        .width(Length::Fixed(50.0))
                                        .height(Length::Fixed(30.0)),
                                        text(LOCALES.lookup(&state.locale, "cancel")),
                                        iced::widget::tooltip::Position::Bottom
                                    ),
                                ]
                                .spacing(5)
                                .into()
                            } else if state.display_delete_project_view {
                                row![
                                    button(
                                        text(LOCALES.lookup(&state.locale, "delete"))
                                            .width(Length::Fill)
                                            .align_x(Center)
                                    )
                                    .style(button::danger)
                                    .width(Length::Fill)
                                    .on_press(Message::Tasks(TasksPageMessage::DeleteProject)),
                                    button(
                                        text(LOCALES.lookup(&state.locale, "cancel"))
                                            .width(Length::Fill)
                                            .align_x(Center)
                                    )
                                    .width(Length::Fill)
                                    .on_press(Message::Tasks(
                                        TasksPageMessage::ToggleDeleteProjectView
                                    )),
                                ]
                                .spacing(5)
                                .into()
                            } else if state.display_archive_project_view {
                                row![
                                    button(
                                        text(LOCALES.lookup(&state.locale, "archive"))
                                            .width(Length::Fill)
                                            .align_x(Center)
                                    )
                                    .style(button::danger)
                                    .width(Length::Fill)
                                    .on_press(Message::Tasks(TasksPageMessage::ArchiveProject)),
                                    button(
                                        text(LOCALES.lookup(&state.locale, "cancel"))
                                            .width(Length::Fill)
                                            .align_x(Center)
                                    )
                                    .width(Length::Fill)
                                    .on_press(Message::Tasks(
                                        TasksPageMessage::ToggleArchiveProjectView
                                    )),
                                ]
                                .spacing(5)
                                .into()
                            } else {
                                row![
                                    button(
                                        text(LOCALES.lookup(&state.locale, "rename"))
                                            .width(Length::Fill)
                                            .align_x(Center)
                                    )
                                    .width(Length::Fill)
                                    .on_press(Message::Tasks(
                                        TasksPageMessage::ToggleRenameProjectView
                                    )),
                                    button(
                                        text(LOCALES.lookup(&state.locale, "archive"))
                                            .width(Length::Fill)
                                            .align_x(Center)
                                    )
                                    .width(Length::Fill)
                                    .on_press(Message::Tasks(
                                        TasksPageMessage::ToggleArchiveProjectView
                                    )),
                                    button(
                                        text(LOCALES.lookup(&state.locale, "delete"))
                                            .width(Length::Fill)
                                            .align_x(Center)
                                    )
                                    .style(button::danger)
                                    .width(Length::Fill)
                                    .on_press(Message::Tasks(
                                        TasksPageMessage::ToggleDeleteProjectView
                                    )),
                                    Tooltip::new(
                                        button(Svg::new(svg::Handle::from_memory(include_bytes!(
                                            "../../../icons/close.svg"
                                        ))))
                                        .on_press(Message::Tasks(
                                            TasksPageMessage::ShowMenuForProject(None)
                                        ))
                                        .width(Length::Fixed(50.0))
                                        .height(Length::Fixed(30.0)),
                                        text(LOCALES.lookup(&state.locale, "close")),
                                        iced::widget::tooltip::Position::Right,
                                    )
                                ]
                                .spacing(5)
                                .into()
                            }
                        } else {
                            row![
                                button(
                                    text(project.file_stem().unwrap_or_default().to_string_lossy())
                                        .font(Font {
                                            weight: iced::font::Weight::Semibold,
                                            ..Default::default()
                                        })
                                        .width(Length::Fill)
                                        .align_x(Center),
                                )
                                .style(
                                    if state
                                        .current_project_file
                                        .as_ref()
                                        .is_some_and(|value| *value == *project)
                                    {
                                        button::secondary
                                    } else {
                                        button::primary
                                    },
                                )
                                .width(Length::Fill)
                                .on_press(Message::Tasks(
                                    TasksPageMessage::PickProjectFile(Some(project.to_path_buf()),)
                                )),
                                Tooltip::new(
                                    button(Svg::new(svg::Handle::from_memory(include_bytes!(
                                        "../../../icons/view-more.svg"
                                    ))))
                                    .on_press(Message::Tasks(TasksPageMessage::ShowMenuForProject(
                                        Some(project.to_path_buf())
                                    )))
                                    .height(Length::Fixed(30.0))
                                    .width(Length::Fixed(50.0)),
                                    text(LOCALES.lookup(&state.locale, "manage-details")),
                                    iced::widget::tooltip::Position::Right,
                                )
                            ]
                            .spacing(5)
                            .into()
                        }
                    })
            )
            .spacing(5)
        )
        .spacing(5)
    ]
    .width(Length::FillPortion(1))
    .into()
}

pub fn tool_view(state: &TasksPage) -> Element<Message> {
    let underlay = Tooltip::new(
        button(Svg::new(svg::Handle::from_memory(include_bytes!(
            "../../../icons/view-more.svg"
        ))))
        .on_press(Message::Tasks(TasksPageMessage::ToggleExtraToolsMenu)),
        text(LOCALES.lookup(&state.locale, "more-tools")),
        iced::widget::tooltip::Position::Bottom,
    );
    let overlay = column![button(
        text(LOCALES.lookup(&state.locale, "select-projects-folder"))
            .width(Length::Fill)
            .align_x(Center),
    )
    .on_press(Message::Tasks(TasksPageMessage::PickProjectsFolder)),]
    .width(Length::Fixed(200.0));

    let drop_down = DropDown::new(underlay, overlay, state.show_extra_tools_menu)
        .on_dismiss(Message::Tasks(TasksPageMessage::ToggleExtraToolsMenu))
        .width(Length::Fill)
        .alignment(drop_down::Alignment::Bottom);
    if state.selected_folder.is_some() {
        row![
            Tooltip::new(
                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/toggle-sidebar.svg"
                ))))
                .on_press(Message::Tasks(TasksPageMessage::ToggleShowSidebar))
                .style(if state.show_sidebar {
                    button::secondary
                } else {
                    button::primary
                }),
                text(LOCALES.lookup(&state.locale, "toggle-sidebar-shortcut")),
                iced::widget::tooltip::Position::Bottom
            ),
            Tooltip::new(
                button(match state.task_view_type {
                    TaskViewType::Kanban => Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/list.svg"
                    ))),
                    TaskViewType::List => Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/kanban.svg"
                    ))),
                })
                .on_press(Message::Tasks(TasksPageMessage::ToggleTaskViewType)),
                match state.task_view_type {
                    TaskViewType::Kanban =>
                        text(LOCALES.lookup(&state.locale, "use-list-view-shortcut")),
                    TaskViewType::List =>
                        text(LOCALES.lookup(&state.locale, "use-kanban-view-shortcut")),
                },
                iced::widget::tooltip::Position::Bottom
            ),
            Tooltip::new(
                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/add.svg"
                ))))
                .on_press(Message::Tasks(TasksPageMessage::StartCreatingNewTask)),
                text(LOCALES.lookup(&state.locale, "new-task-shortcut")),
                iced::widget::tooltip::Position::Bottom
            ),
            drop_down
        ]
        .width(Length::FillPortion(1))
    } else {
        row![].width(Length::FillPortion(1))
    }
    .into()
}
