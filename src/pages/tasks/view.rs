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

fn kanban_view_item<'a>(state: &'a TasksPage, task: &'a TaskData) -> Element<'a, Message> {
    droppable(
        container(
            column![
                text(&task.title)
                    .size(20)
                    .width(Length::Fill)
                    .align_x(Center),
                text(&task.description),
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
                            "Backlog",
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
                            "To Do",
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
                            "Doing",
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
                            "Done",
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
                "New Task"
            } else {
                "Edit Task"
            })
            .width(Length::Fill)
            .size(24),
            Tooltip::new(
                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/close.svg"
                ))))
                .on_press(Message::Tasks(TasksPageMessage::ToggleShowTaskEditDialog))
                .width(Length::Fixed(50.0)),
                "Close Edit Dialog (Esc)",
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
            button(text("Cancel (Esc)").align_x(Center).width(Length::Fill))
                .width(Length::Fill)
                .on_press(Message::Tasks(TasksPageMessage::ToggleShowTaskEditDialog)),
            button(
                text("Save Task (Ctrl+S)")
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
                text("Backlog").size(24),
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
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(
                            task.completion_state,
                            TaskCompletionState::Backlog
                        ))
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
                text("To Do").size(24),
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
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(task.completion_state, TaskCompletionState::ToDo))
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
                text("Doing").size(24),
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
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(task.completion_state, TaskCompletionState::Doing))
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
                text("Done").size(24),
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
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(task.completion_state, TaskCompletionState::Done))
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
            text("Backlog").width(Length::Fill).align_x(Center).size(20),
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(
                            task.completion_state,
                            TaskCompletionState::Backlog
                        ))
                        .map(|task| list_view_item(task))
                )
                .spacing(10)
            )
        ],
        column![
            text("To Do").width(Length::Fill).align_x(Center).size(20),
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(task.completion_state, TaskCompletionState::ToDo))
                        .map(|task| list_view_item(task))
                )
                .spacing(10)
            )
        ],
        column![
            text("Doing").width(Length::Fill).align_x(Center).size(20),
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(task.completion_state, TaskCompletionState::Doing))
                        .map(|task| list_view_item(task))
                )
                .spacing(10)
            )
        ],
        column![
            text("Done").width(Length::Fill).align_x(Center).size(20),
            scrollable(
                column(
                    state
                        .tasks_list
                        .iter()
                        .filter(|task| matches!(task.completion_state, TaskCompletionState::Done))
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
            text("Delete This Task").width(Length::Fill).size(24),
            row![
                button(text("Cancel (Esc)").align_x(Center).width(Length::Fill))
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
        Space::with_height(10),
        if state.is_creating_new_project {
            row![
                text_input("New Project Name", &state.new_project_name_entry_content)
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
                    "Create",
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
                    "Cancel",
                    iced::widget::tooltip::Position::Bottom
                ),
            ]
        } else {
            row![
                button(text("New Project").width(Length::Fill).align_x(Center))
                    .width(Length::Fill)
                    .style(button::success)
                    .on_press(Message::Tasks(TasksPageMessage::StartCreatingNewProject))
            ]
        },
        Space::with_height(20),
        scrollable(
            column(state.projects_list.iter().map(|project| {
                if state
                    .current_project_being_managed
                    .clone()
                    .is_some_and(|selected_project| selected_project == *project)
                {
                    if state.display_rename_view {
                        row![
                            text_input("Rename Project", &state.rename_project_entry_text)
                                .width(Length::Fill)
                                .on_input(|s| Message::Tasks(
                                    TasksPageMessage::SetRenameProjectEntryText(s)
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
                                "Rename",
                                iced::widget::tooltip::Position::Bottom
                            ),
                            Tooltip::new(
                                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                                    "../../../icons/close.svg"
                                ))))
                                .on_press(Message::Tasks(TasksPageMessage::ToggleRenameProjectView))
                                .style(button::danger)
                                .width(Length::Fixed(50.0))
                                .height(Length::Fixed(30.0)),
                                "Cancel",
                                iced::widget::tooltip::Position::Bottom
                            ),
                        ]
                        .into()
                    } else if state.display_delete_view {
                        row![
                            button(text("Delete").width(Length::Fill).align_x(Center))
                                .style(button::danger)
                                .width(Length::Fill)
                                .on_press(Message::Tasks(TasksPageMessage::DeleteProject)),
                            button(text("Cancel").width(Length::Fill).align_x(Center))
                                .width(Length::Fill)
                                .on_press(Message::Tasks(
                                    TasksPageMessage::ToggleDeleteProjectView
                                )),
                        ]
                        .into()
                    } else {
                        row![
                            button(text("Rename").width(Length::Fill).align_x(Center))
                                .width(Length::Fill)
                                .on_press(Message::Tasks(
                                    TasksPageMessage::ToggleRenameProjectView
                                )),
                            button(text("Delete").width(Length::Fill).align_x(Center))
                                .style(button::danger)
                                .width(Length::Fill)
                                .on_press(Message::Tasks(
                                    TasksPageMessage::ToggleDeleteProjectView
                                )),
                            Tooltip::new(
                                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                                    "../../../icons/close.svg"
                                ))))
                                .on_press(Message::Tasks(TasksPageMessage::ShowMenuForProject(
                                    None
                                )))
                                .width(Length::Fixed(50.0))
                                .height(Length::Fixed(30.0)),
                                "Close",
                                iced::widget::tooltip::Position::Right,
                            )
                        ]
                        .spacing(5)
                        .into()
                    }
                } else {
                    row![
                        button(
                            text(
                                project
                                    .file_stem()
                                    .unwrap_or_default()
                                    .to_str()
                                    .unwrap_or("Couldn't read filename"),
                            )
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
                                .clone()
                                .is_some_and(|value| value == *project)
                            {
                                button::secondary
                            } else {
                                button::primary
                            },
                        )
                        .width(Length::Fill)
                        .on_press(Message::Tasks(
                            TasksPageMessage::PickProjectFile(project.to_path_buf(),)
                        )),
                        Tooltip::new(
                            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                                "../../../icons/view-more.svg"
                            ))))
                            .on_press(Message::Tasks(TasksPageMessage::ShowMenuForProject(Some(
                                project.to_path_buf()
                            ))))
                            .height(Length::Fixed(30.0))
                            .width(Length::Fixed(50.0)),
                            "Manage Details",
                            iced::widget::tooltip::Position::Right,
                        )
                    ]
                    .spacing(5)
                    .into()
                }
            }))
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
        "More Tools",
        iced::widget::tooltip::Position::Bottom,
    );
    let overlay = column![button(
        text("Select Projects Folder")
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
                "Toggle Sidebar (Ctrl+B)",
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
                    TaskViewType::Kanban => "Use List View (Ctrl+L)",
                    TaskViewType::List => "Use Kanban View (Ctrl+L)",
                },
                iced::widget::tooltip::Position::Bottom
            ),
            Tooltip::new(
                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/add.svg"
                ))))
                .on_press(Message::Tasks(TasksPageMessage::StartCreatingNewTask)),
                "New Task (Ctrl+N)",
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
