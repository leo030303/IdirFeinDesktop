use iced::{
    widget::{button, column, row, svg, text, text_input, Svg, Tooltip},
    Element, Length,
};

use crate::app::Message;

use super::page::{SyncPage, SyncPageMessage};

pub fn main_view(state: &SyncPage) -> Element<Message> {
    column![
        if state.is_connected_to_server {
            row![
                text("Connected to server").style(text::success),
                Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/connection.svg"
                )))
            ]
        } else {
            row![
                text("Disconnected from server").style(text::danger),
                Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/no_connection.svg"
                )))
            ]
        },
        row![
            row![
                text_input("New ignore list entry", &state.ignore_list_editor_text)
                    .width(Length::Fixed(200.0))
                    .on_input(|s| Message::Sync(SyncPageMessage::UpdateIgnoreListEditor(s)))
                    .on_submit(Message::Sync(SyncPageMessage::AddToIgnoreList)),
                button("Add").on_press(Message::Sync(SyncPageMessage::AddToIgnoreList))
            ],
            column(
                state
                    .ignore_string_list
                    .iter()
                    .enumerate()
                    .map(|(index, ignore_list_item)| {
                        row![
                            text(ignore_list_item),
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
                        .into()
                    })
            )
        ],
        row![
            row![button("Add folder to sync list")
                .on_press(Message::Sync(SyncPageMessage::PickNewSyncListFolder))],
            column(state.folders_to_sync.iter().enumerate().map(
                |(index, (_folder_id, folder_path))| {
                    row![
                        text(folder_path.to_str().unwrap_or("Error reading folder path")),
                        Tooltip::new(
                            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                                "../../../icons/delete.svg"
                            ))))
                            .on_press(Message::Sync(SyncPageMessage::DeleteFromFolderList(index)))
                            .style(button::danger)
                            .width(Length::Fixed(50.0))
                            .height(Length::Fixed(30.0)),
                            "Remove",
                            iced::widget::tooltip::Position::Bottom
                        ),
                    ]
                    .into()
                }
            ))
        ],
        button("Test sync (this button will be removed)")
            .on_press(Message::SendServerMessage(String::from("Test")))
    ]
    .into()
}

pub fn tool_view(_state: &SyncPage) -> Element<Message> {
    row![].width(Length::FillPortion(1)).into()
}
