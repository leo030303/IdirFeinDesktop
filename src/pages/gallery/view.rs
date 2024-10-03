use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, container,
        image::{self, Handle},
        list, row, scrollable, text, Image,
    },
    Alignment::Center,
    Element, Length,
};
use iced_aw::{Grid, GridRow};

use crate::app::Message;

use super::page::{GalleryPage, GalleryPageMessage, IMAGE_HEIGHT};

pub fn main_view(state: &GalleryPage) -> Element<Message> {
    if state.selected_folder.is_none() {
        no_gallery_folder_selected_view(state)
    } else {
        gallery_grid(state)
    }
}

fn gallery_grid(state: &GalleryPage) -> Element<Message> {
    scrollable(list(
        &state.gallery_list,
        |_index, (_photo_path, handle_option)| {
            if let Some(image_handle) = handle_option {
                container(Image::new(image_handle).content_fit(iced::ContentFit::ScaleDown))
                    .height(Length::Fixed(IMAGE_HEIGHT))
                    .padding(20)
                    .width(Length::FillPortion(1))
                    .into()
            } else {
                container(
                    text("Loading")
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
        },
    ))
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
