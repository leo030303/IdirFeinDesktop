use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container,
        image::{self, Handle},
        row, scrollable, stack, svg, text, Image, MouseArea, Space, Svg, Tooltip,
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
    container(column![
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
                "Close Image View (Esc)",
                iced::widget::tooltip::Position::Bottom
            ),
        ],
        row![
            column![
                Space::with_height(Length::Fill),
                Tooltip::new(
                    button(Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/previous.svg"
                    ))))
                    .on_press(Message::Gallery(GalleryPageMessage::SelectPreviousImage))
                    .height(Length::Fixed(40.0))
                    .width(Length::Fixed(50.0)),
                    "Previous Image (Left Arrow)",
                    iced::widget::tooltip::Position::Bottom
                ),
                Space::with_height(Length::Fill),
            ],
            image::viewer(Handle::from_path(
                state.selected_image.clone().expect("Shouldn't fail"),
            ))
            .width(Length::Fill)
            .height(Length::Fill),
            column![
                Space::with_height(Length::Fill),
                Tooltip::new(
                    button(Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/next.svg"
                    ))))
                    .on_press(Message::Gallery(GalleryPageMessage::SelectNextImage))
                    .height(Length::Fixed(40.0))
                    .width(Length::Fixed(50.0)),
                    "Next Image (Right Arrow)",
                    iced::widget::tooltip::Position::Bottom
                ),
                Space::with_height(Length::Fill),
            ],
        ],
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .style(container::dark)
    .into()
}

fn gallery_grid(state: &GalleryPage) -> Element<Message> {
    scrollable(column(state.gallery_row_list.iter().map(|image_row| {
        if image_row.loaded {
            row(image_row
                .images_data
                .iter()
                .map(|(photo_path, handle_option)| {
                    if let Some(image_handle) = handle_option {
                        container(
                            MouseArea::new(
                                Image::new(image_handle)
                                    .content_fit(iced::ContentFit::ScaleDown)
                                    .filter_method(image::FilterMethod::Nearest),
                            )
                            .on_press(Message::Gallery(
                                GalleryPageMessage::SelectImageForBigView(Some(
                                    photo_path.to_path_buf(),
                                )),
                            )),
                        )
                        .height(Length::Fill)
                        .padding(5)
                        .width(Length::FillPortion(1))
                        .into()
                    } else {
                        Space::with_height(Length::Fixed(IMAGE_HEIGHT)).into()
                    }
                }))
            .height(Length::Fixed(IMAGE_HEIGHT))
            .into()
        } else {
            row![text("LOADING").center().width(Length::Fill)]
                .height(Length::Fixed(IMAGE_HEIGHT))
                .into()
        }
    })))
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
