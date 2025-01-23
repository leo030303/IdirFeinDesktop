use iced::{
    widget::{button, column, container, row, scrollable, svg, text, text_input, Svg, Tooltip},
    Alignment::Center,
    Element, Font, Length,
};

use crate::app::Message;

use super::page::{SyncPage, SyncPageMessage};

pub fn main_view(state: &SyncPage) -> Element<Message> {
    column![
        syncing_progress_bar(state),
        row![ignore_list_manager(state), sync_folder_manager(state)]
            .spacing(20)
            .padding(20),
    ]
    .into()
}

fn syncing_progress_bar(_state: &SyncPage) -> Element<Message> {
    row!["Progress bar"].width(Length::Fill).into()
}

fn sync_status_indicator(state: &SyncPage) -> Element<Message> {
    if state.is_connected_to_server {
        row![
            text("Connected").style(text::success),
            Svg::new(svg::Handle::from_memory(include_bytes!(
                "../../../icons/connection_good.svg"
            )))
        ]
    } else {
        row![
            text("Disconnected").style(text::danger),
            Svg::new(svg::Handle::from_memory(include_bytes!(
                "../../../icons/connection_bad.svg"
            )))
        ]
    }
    .width(150)
    .padding(5)
    .into()
}

fn ignore_list_manager(state: &SyncPage) -> Element<Message> {
    column![
        row![
            text_input("New ignore list entry", &state.ignore_list_editor_text)
                .on_input(|s| Message::Sync(SyncPageMessage::UpdateIgnoreListEditor(s)))
                .on_submit(Message::Sync(SyncPageMessage::AddToIgnoreList))
                .width(Length::Fill),
            button(text("Add")).on_press(Message::Sync(SyncPageMessage::AddToIgnoreList))
        ]
        .width(Length::Fill),
        scrollable(
            column(
                state
                    .ignore_string_list
                    .iter()
                    .enumerate()
                    .map(|(index, ignore_list_item)| {
                        container(
                            row![
                                text(ignore_list_item)
                                    .font(Font {
                                        weight: iced::font::Weight::Medium,
                                        ..Default::default()
                                    })
                                    .align_y(Center)
                                    .width(Length::Fill)
                                    .height(Length::Shrink),
                                Tooltip::new(
                                    button(Svg::new(svg::Handle::from_memory(include_bytes!(
                                        "../../../icons/delete.svg"
                                    ))))
                                    .on_press(Message::Sync(SyncPageMessage::DeleteFromIgnoreList(
                                        index
                                    )))
                                    .style(button::danger)
                                    .width(Length::Fixed(50.0))
                                    .height(Length::Fixed(30.0)),
                                    "Remove",
                                    iced::widget::tooltip::Position::Bottom
                                ),
                            ]
                            .padding(5)
                            .spacing(10),
                        )
                        .style(container::bordered_box)
                        .into()
                    })
            )
            .spacing(10)
        )
    ]
    .spacing(10)
    .width(Length::FillPortion(1))
    .into()
}

fn sync_folder_manager(state: &SyncPage) -> Element<Message> {
    column![
        row![button(
            text("Add folder to sync list")
                .size(20)
                .width(Length::Fill)
                .center()
        )
        .width(Length::Fill)
        .on_press(Message::Sync(SyncPageMessage::PickNewSyncListFolder))],
        scrollable(
            column(
                state
                    .folders_to_sync
                    .iter()
                    .map(|(folder_id, folder_path)| {
                        container(
                            row![
                                text(folder_path.to_str().unwrap_or("Error reading folder path"))
                                    .font(Font {
                                        weight: iced::font::Weight::Medium,
                                        ..Default::default()
                                    })
                                    .align_y(Center)
                                    .width(Length::Fill)
                                    .height(Length::Shrink),
                                Tooltip::new(
                                    button(Svg::new(svg::Handle::from_memory(include_bytes!(
                                        "../../../icons/delete.svg"
                                    ))))
                                    .on_press(Message::Sync(SyncPageMessage::DeleteFromFolderList(
                                        folder_id.clone()
                                    )))
                                    .style(button::danger)
                                    .width(Length::Fixed(50.0))
                                    .height(Length::Fixed(30.0)),
                                    "Remove",
                                    iced::widget::tooltip::Position::Bottom
                                ),
                            ]
                            .padding(5)
                            .spacing(10),
                        )
                        .style(container::bordered_box)
                        .into()
                    })
            )
            .spacing(10)
        )
    ]
    .spacing(10)
    .width(Length::FillPortion(1))
    .into()
}

pub fn tool_view(state: &SyncPage) -> Element<Message> {
    row![sync_status_indicator(state)]
        .width(Length::FillPortion(1))
        .into()
}
