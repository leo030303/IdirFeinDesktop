use crate::pages::tasks::page::TaskViewType;
use iced::{
    widget::{button, column, row, scrollable, text, text_editor, text_input, Svg, Tooltip},
    Alignment::Center,
    Element, Font, Length,
};

use crate::app::Message;

use super::page::{TaskCompletionState, TaskData, TasksPage, TasksPageMessage};

pub fn main_view(state: &TasksPage) -> Element<Message> {
    row![
        if state.show_sidebar {
            column![text("Sidebar")].width(Length::FillPortion(1))
        } else {
            column![]
        },
        column![
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
        .spacing(10)
    ]
    .into()
}

fn kanban_view_item(task: &TaskData) -> Element<Message> {
    // TODO
    column![
        text(&task.title),
        text(&task.description),
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
            .on_input(|s| Message::Tasks(TasksPageMessage::UpdateTaskTitle(s))),
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
            text("Backlog").width(Length::Fill).align_x(Center).size(20),
            scrollable(column(
                state
                    .tasks_list
                    .iter()
                    .filter(|task| matches!(task.completion_state, TaskCompletionState::Backlog))
                    .map(|task| kanban_view_item(task))
            ))
        ],
        column![
            text("To Do").width(Length::Fill).align_x(Center).size(20),
            scrollable(column(
                state
                    .tasks_list
                    .iter()
                    .filter(|task| matches!(task.completion_state, TaskCompletionState::ToDo))
                    .map(|task| kanban_view_item(task))
            ))
        ],
        column![
            text("Doing").width(Length::Fill).align_x(Center).size(20),
            scrollable(column(
                state
                    .tasks_list
                    .iter()
                    .filter(|task| matches!(task.completion_state, TaskCompletionState::Doing))
                    .map(|task| kanban_view_item(task))
            ))
        ],
        column![
            text("Done").width(Length::Fill).align_x(Center).size(20),
            scrollable(column(
                state
                    .tasks_list
                    .iter()
                    .filter(|task| matches!(task.completion_state, TaskCompletionState::Done))
                    .map(|task| kanban_view_item(task))
            ))
        ],
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

pub fn tool_view(state: &TasksPage) -> Element<Message> {
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
    .into()
}
