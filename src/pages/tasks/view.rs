use iced::{
    widget::{button, column, row, text, Svg, Tooltip},
    Element, Length,
};

use crate::app::Message;

use super::page::{TasksPage, TasksPageMessage};

pub fn main_view(state: &TasksPage) -> Element<Message> {
    row![if state.show_sidebar {
        column![text("Sidebar")]
    } else {
        column![]
    }]
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
        // Tooltip::new(
        //     button(match state.task_view_type {
        //         super::page::TaskViewType::Kanban => Svg::from_path("icons/kanban.svg"),
        //         super::page::TaskViewType::List => Svg::from_path("icons/list.svg"),
        //     })
        //     .on_press(Message::Tasks(TasksPageMessage::ToggleEditor)),
        //     "Toggle Editor",
        //     iced::widget::tooltip::Position::Bottom
        // ),
        Tooltip::new(
            button(Svg::from_path("icons/add.svg"))
                .on_press(Message::Tasks(TasksPageMessage::ToggleShowTaskEditDialog)),
            "New Task",
            iced::widget::tooltip::Position::Bottom
        ),
    ]
    .width(Length::FillPortion(1))
    .into()
}
