use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container,
        image::{self, Handle},
        list, row, scrollable, stack, svg, text, Image, MouseArea, Space, Svg, Tooltip,
    },
    Alignment::Center,
    Element, Length,
};

use crate::app::Message;

use super::page::{GalleryPage, GalleryPageMessage, IMAGE_HEIGHT, SCROLLABLE_ID};

pub fn main_view(state: &GalleryPage) -> Element<Message> {
    if state.selected_folder.is_none() {
        no_gallery_folder_selected_view(state)
    } else {
        let gallery_grid = gallery_grid(state);
        if state.selected_image.is_some() {
            stack([gallery_grid, big_image_viewer(state)]).into()
        } else {
            gallery_grid
        }
    }
}

fn big_image_viewer(state: &GalleryPage) -> Element<Message> {
    column![
        row![
            Space::with_width(Length::Fill),
            Tooltip::new(
                button(Svg::new(svg::Handle::from_memory(include_bytes!(
                    "../../../icons/close.svg"
                ))))
                .on_press(Message::Gallery(GalleryPageMessage::SelectImageForBigView(
                    None
                )))
                .width(Length::Fixed(50.0)),
                "Close Image View",
                iced::widget::tooltip::Position::Bottom
            ),
        ],
        image::viewer(Handle::from_path(
            state.selected_image.clone().expect("Shouldn't fail"),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
    ]
    .into()
}

fn gallery_grid(state: &GalleryPage) -> Element<Message> {
    scrollable(list(&state.gallery_list, |_index, photos_vec| {
        row(photos_vec.iter().map(|(photo_path, handle_option)| {
            if let Some(image_handle) = handle_option {
                container(
                    MouseArea::new(
                        Image::new(image_handle).content_fit(iced::ContentFit::ScaleDown),
                    )
                    .on_press(Message::Gallery(
                        GalleryPageMessage::SelectImageForBigView(Some(photo_path.to_path_buf())),
                    )),
                )
                .height(Length::Fixed(IMAGE_HEIGHT))
                .padding(20)
                .width(Length::FillPortion(1))
                .into()
            } else {
                container(
                    text(photo_path.to_str().unwrap_or("File is loading"))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Center)
                        .align_y(Center),
                )
                .height(Length::Fixed(IMAGE_HEIGHT))
                .padding(20)
                .width(Length::FillPortion(1))
                .into()
            }
        }))
        .into()
    }))
    .id(SCROLLABLE_ID.clone())
    .on_scroll(|viewport| Message::Gallery(GalleryPageMessage::GalleryScrolled(viewport)))
    .into()
}

fn no_gallery_folder_selected_view(_state: &GalleryPage) -> Element<Message> {
    container(
        button(
            text("Select Gallery Folder")
                .size(20)
                .height(Length::Fixed(40.0))
                .align_y(Center)
                .width(Length::Fill)
                .align_x(Center),
        )
        .on_press(Message::Gallery(GalleryPageMessage::PickGalleryFolder))
        .width(Length::Fixed(250.0)),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .into()
}

pub fn tool_view(_state: &GalleryPage) -> Element<Message> {
    row![].width(Length::FillPortion(1)).into()
}
