use crate::pages::tasks::page::{TaskViewType, NEW_PROJECT_TEXT_INPUT_ID};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, row, scrollable, svg, text, text_editor, text_input, Row, Space,
        Svg, Tooltip,
    },
    Alignment::Center,
    Element, Font, Length,
};

use crate::app::Message;

use super::page::{
    TaskCompletionState, TaskData, TasksPage, TasksPageMessage, TASK_TITLE_TEXT_INPUT_ID,
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
                    text(
                        state
                            .current_project_file
                            .clone()
                            .expect("Can't fail")
                            .file_stem()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or("Couldn't read filename")
                            .to_string(),
                    )
                    .size(28)
                    .width(Length::Fill)
                    .align_x(Center),
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
                column![text("No Project Selected")
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

fn kanban_view_item(task: &TaskData) -> Element<Message> {
    // TODO
    column![
        text(&task.title)
            .size(20)
            .width(Length::Fill)
            .align_x(Center),
        text(&task.description),
        {
            let mut state_setter_row = Row::new();
            if !matches!(task.completion_state, TaskCompletionState::Backlog) {
                state_setter_row = state_setter_row.push(Tooltip::new(
                    button(Svg::from_path("icons/1.svg"))
                        .on_press(Message::Tasks(TasksPageMessage::SetTaskCompletionState(
                            task.id,
                            TaskCompletionState::Backlog,
                        )))
                        .width(Length::Fill),
                    "Backlog",
                    iced::widget::tooltip::Position::Bottom,
                ));
            }
            if !matches!(task.completion_state, TaskCompletionState::ToDo) {
                state_setter_row = state_setter_row.push(Tooltip::new(
                    button(Svg::from_path("icons/2.svg"))
                        .on_press(Message::Tasks(TasksPageMessage::SetTaskCompletionState(
                            task.id,
                            TaskCompletionState::ToDo,
                        )))
                        .width(Length::Fill),
                    "To Do",
                    iced::widget::tooltip::Position::Bottom,
                ));
            }
            if !matches!(task.completion_state, TaskCompletionState::Doing) {
                state_setter_row = state_setter_row.push(Tooltip::new(
                    button(Svg::from_path("icons/3.svg"))
                        .on_press(Message::Tasks(TasksPageMessage::SetTaskCompletionState(
                            task.id,
                            TaskCompletionState::Doing,
                        )))
                        .width(Length::Fill),
                    "Doing",
                    iced::widget::tooltip::Position::Bottom,
                ));
            }
            if !matches!(task.completion_state, TaskCompletionState::Done) {
                state_setter_row = state_setter_row.push(Tooltip::new(
                    button(Svg::from_path("icons/4.svg"))
                        .on_press(Message::Tasks(TasksPageMessage::SetTaskCompletionState(
                            task.id,
                            TaskCompletionState::Done,
                        )))
                        .width(Length::Fill),
                    "Done",
                    iced::widget::tooltip::Position::Bottom,
                ));
            }
            state_setter_row
        },
        row![
            button(Svg::from_path("icons/edit.svg"))
                .width(Length::Fill)
                .on_press(Message::Tasks(TasksPageMessage::OpenEditDialogForTask(
                    task.id
                ))),
            button(Svg::from_path("icons/delete.svg"))
                .style(button::danger)
                .width(Length::Fill)
                .on_press(Message::Tasks(
                    TasksPageMessage::DeleteTaskWithConfirmationCheck(task.id)
                ))
        ]
    ]
    .padding(5)
    .into()
}

fn list_view_item(task: &TaskData) -> Element<Message> {
    // TODO
    text(&task.title).into()
}

fn task_edit_dialog(state: &TasksPage) -> Element<Message> {
    column![
        row![
            text(if state.current_task_id.is_none() {
                "New Task"
            } else {
                "Edit Task"
            })
            .width(Length::Fill)
            .size(24),
            Tooltip::new(
                button(Svg::from_path("icons/close.svg"))
                    .on_press(Message::Tasks(TasksPageMessage::ToggleShowTaskEditDialog))
                    .width(Length::Fixed(50.0)),
                "Close Edit Dialog",
                iced::widget::tooltip::Position::Bottom
            ),
        ],
        text_input("Task Title", &state.current_task_title_text)
            .on_input(|s| Message::Tasks(TasksPageMessage::UpdateTaskTitle(s)))
            .on_submit(Message::Tasks(TasksPageMessage::UpdateCurrentTask))
            .id(text_input::Id::new(TASK_TITLE_TEXT_INPUT_ID)),
        text_editor(&state.current_task_description_content)
            .placeholder("Task Description")
            .on_action(|action| Message::Tasks(TasksPageMessage::UpdateTaskDescription(action)))
            .height(Length::Fixed(300.0))
            .padding(10)
            .font(Font::MONOSPACE),
        row![
            button(text("Cancel").align_x(Center).width(Length::Fill))
                .width(Length::Fill)
                .on_press(Message::Tasks(TasksPageMessage::ToggleShowTaskEditDialog)),
            button(text("Save Task").align_x(Center).width(Length::Fill))
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
                text("Backlog").size(24),
                Svg::from_path("icons/1.svg")
                    .style(|theme, _status| svg::Style {
                        color: text::base(theme).color
                    })
                    .height(Length::Fixed(24.0)),
                Space::with_width(Length::Fill),
            ]
            .padding(5),
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(
                            task.completion_state,
                            TaskCompletionState::Backlog
                        ))
                        .map(|task| kanban_view_item(task))
                )
                .padding(5)
                .spacing(5)
            )
        ]
        .width(Length::Fill)
        .align_x(Center),
        column![
            row![
                Space::with_width(Length::Fill),
                text("To Do").size(24),
                Svg::from_path("icons/2.svg")
                    .style(|theme, _status| svg::Style {
                        color: text::base(theme).color
                    })
                    .height(Length::Fixed(24.0)),
                Space::with_width(Length::Fill),
            ]
            .padding(5),
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(task.completion_state, TaskCompletionState::ToDo))
                        .map(|task| kanban_view_item(task))
                )
                .padding(5)
                .spacing(5)
            )
        ]
        .width(Length::Fill)
        .align_x(Center),
        column![
            row![
                Space::with_width(Length::Fill),
                text("Doing").size(24),
                Svg::from_path("icons/3.svg")
                    .style(|theme, _status| svg::Style {
                        color: text::base(theme).color
                    })
                    .height(Length::Fixed(24.0)),
                Space::with_width(Length::Fill),
            ]
            .padding(5),
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(task.completion_state, TaskCompletionState::Doing))
                        .map(|task| kanban_view_item(task))
                )
                .padding(5)
                .spacing(5)
            )
        ]
        .width(Length::Fill)
        .align_x(Center),
        column![
            row![
                Space::with_width(Length::Fill),
                text("Done").size(24),
                Svg::from_path("icons/4.svg")
                    .style(|theme, _status| svg::Style {
                        color: text::base(theme).color
                    })
                    .height(Length::Fixed(24.0)),
                Space::with_width(Length::Fill),
            ]
            .padding(5),
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(task.completion_state, TaskCompletionState::Done))
                        .map(|task| kanban_view_item(task))
                )
                .padding(5)
                .spacing(5)
            )
        ]
        .width(Length::Fill)
        .align_x(Center),
    ]
    .into()
}

fn list_view(state: &TasksPage) -> Element<Message> {
    scrollable(column![
        column![
            text("Backlog").width(Length::Fill).align_x(Center).size(20),
            scrollable(column(
                state
                    .tasks_list
                    .iter()
                    .filter(|task| matches!(task.completion_state, TaskCompletionState::Backlog))
                    .map(|task| list_view_item(task))
            ))
        ],
        column![
            text("To Do").width(Length::Fill).align_x(Center).size(20),
            scrollable(column(
                state
                    .tasks_list
                    .iter()
                    .filter(|task| matches!(task.completion_state, TaskCompletionState::ToDo))
                    .map(|task| list_view_item(task))
            ))
        ],
        column![
            text("Doing").width(Length::Fill).align_x(Center).size(20),
            scrollable(column(
                state
                    .tasks_list
                    .iter()
                    .filter(|task| matches!(task.completion_state, TaskCompletionState::Doing))
                    .map(|task| list_view_item(task))
            ))
        ],
        column![
            text("Done").width(Length::Fill).align_x(Center).size(20),
            scrollable(column(
                state
                    .tasks_list
                    .iter()
                    .filter(|task| matches!(task.completion_state, TaskCompletionState::Done))
                    .map(|task| list_view_item(task))
            ))
        ],
    ])
    .into()
}

fn confirm_delete_view(state: &TasksPage) -> Element<Message> {
    if state.current_task_id.is_some() {
        column![
            text("Delete This Task").width(Length::Fill).size(24),
            row![
                button(text("Cancel").align_x(Center).width(Length::Fill))
                    .width(Length::Fill)
                    .on_press(Message::Tasks(
                        TasksPageMessage::ToggleConfirmBeforeDeleteDialog
                    )),
                button(text("Delete").align_x(Center).width(Length::Fill))
                    .width(Length::Fill)
                    .style(button::danger)
                    .on_press(Message::Tasks(TasksPageMessage::DeleteTask(
                        state.current_task_id.expect("Checked this was some")
                    )))
            ]
            .spacing(20)
        ]
    } else {
        column![text("no task")]
    }
    .into()
}

fn no_project_folder_selected_view(_state: &TasksPage) -> Element<Message> {
    container(
        button(
            text("Select Projects Folder")
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
        if state.is_creating_new_project {
            row![
                text_input("New Project Name", &state.new_project_name_entry_content)
                    .width(Length::Fill)
                    .on_input(|s| Message::Tasks(TasksPageMessage::UpdateNewProjectNameEntry(s)))
                    .on_submit(Message::Tasks(TasksPageMessage::CreateNewProject))
                    .id(text_input::Id::new(NEW_PROJECT_TEXT_INPUT_ID)),
                Tooltip::new(
                    button(Svg::from_path("icons/ok.svg"))
                        .on_press(Message::Tasks(TasksPageMessage::CreateNewProject))
                        .style(button::success)
                        .width(Length::Fixed(50.0))
                        .height(Length::Fixed(30.0)),
                    "Create",
                    iced::widget::tooltip::Position::Bottom
                ),
                Tooltip::new(
                    button(Svg::from_path("icons/close.svg"))
                        .on_press(Message::Tasks(TasksPageMessage::CancelCreateNewProject))
                        .style(button::danger)
                        .width(Length::Fixed(50.0))
                        .height(Length::Fixed(30.0)),
                    "Cancel",
                    iced::widget::tooltip::Position::Bottom
                ),
            ]
        } else {
            row![
                button(text("New Project").width(Length::Fill).align_x(Center))
                    .width(Length::Fill)
                    .on_press(Message::Tasks(TasksPageMessage::StartCreatingNewProject))
            ]
        },
        Space::with_height(20),
        scrollable(column(state.projects_list.iter().map(|project| {
            button(
                text(
                    project
                        .file_stem()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or("Couldn't read filename"),
                )
                .width(Length::Fill)
                .align_x(Center),
            )
            .style(
                if state
                    .current_project_file
                    .clone()
                    .is_some_and(|value| value == *project)
                {
                    button::secondary
                } else {
                    button::primary
                },
            )
            .width(Length::Fill)
            .on_press(Message::Tasks(TasksPageMessage::PickProjectFile(
                project.to_path_buf(),
            )))
            .into()
        })))
    ]
    .width(Length::FillPortion(1))
    .into()
}

pub fn tool_view(state: &TasksPage) -> Element<Message> {
    if state.selected_folder.is_some() {
        row![
            Tooltip::new(
                button(Svg::from_path("icons/toggle-sidebar.svg"))
                    .on_press(Message::Tasks(TasksPageMessage::ToggleShowSidebar))
                    .style(if state.show_sidebar {
                        button::secondary
                    } else {
                        button::primary
                    }),
                "Toggle Sidebar",
                iced::widget::tooltip::Position::Bottom
            ),
            Tooltip::new(
                button(match state.task_view_type {
                    TaskViewType::Kanban => Svg::from_path("icons/list.svg"),
                    TaskViewType::List => Svg::from_path("icons/kanban.svg"),
                })
                .on_press(Message::Tasks(TasksPageMessage::SetTaskViewType(
                    match state.task_view_type {
                        TaskViewType::Kanban => TaskViewType::List,
                        TaskViewType::List => TaskViewType::Kanban,
                    }
                ))),
                match state.task_view_type {
                    TaskViewType::Kanban => "Use List View",
                    TaskViewType::List => "Use Kanban View",
                },
                iced::widget::tooltip::Position::Bottom
            ),
            Tooltip::new(
                button(Svg::from_path("icons/add.svg"))
                    .on_press(Message::Tasks(TasksPageMessage::StartCreatingNewTask)),
                "New Task",
                iced::widget::tooltip::Position::Bottom
            ),
        ]
        .width(Length::FillPortion(1))
    } else {
        row![].width(Length::FillPortion(1))
    }
    .into()
}
