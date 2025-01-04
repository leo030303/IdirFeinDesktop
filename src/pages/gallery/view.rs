use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container,
        image::{self, Handle},
        progress_bar, row, scrollable, svg, text, Image, MouseArea, Space, Svg, Tooltip,
    },
    Alignment::Center,
    Element, Length,
};

use crate::app::Message;

use super::{
    gallery_utils::PhotoProcessingProgress,
    page::{GalleryPage, GalleryPageMessage, FACE_DATA_FOLDER_NAME, IMAGE_HEIGHT, SCROLLABLE_ID},
};

pub fn main_view(state: &GalleryPage) -> Element<Message> {
    if state.selected_folder.is_none() {
        no_gallery_folder_selected_view(state)
    } else {
        let gallery_grid = gallery_grid(state);
        if state.selected_image.is_some() {
            big_image_viewer(state)
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
                text("People").size(18).width(Length::Fill).center(),
                if state
                    .selected_image
                    .as_ref()
                    .expect("Can't fail")
                    .1
                    .iter()
                    .any(|face_data| !face_data.is_ignored)
                {
                    scrollable(
                        column(
                            state
                                .selected_image
                                .as_ref()
                                .expect("Can't fail")
                                .1
                                .iter()
                                .filter(|face_data| !face_data.is_ignored)
                                .map(|face_data| {
                                    column![
                                        Image::new(image::Handle::from_path(
                                            state
                                                .selected_image
                                                .as_ref()
                                                .expect("Can't fail")
                                                .0
                                                .parent()
                                                .unwrap()
                                                .join(FACE_DATA_FOLDER_NAME)
                                                .join(&face_data.thumbnail_filename)
                                        ))
                                        .content_fit(iced::ContentFit::ScaleDown)
                                        .filter_method(image::FilterMethod::Nearest),
                                        text(
                                            face_data
                                                .name_of_person
                                                .as_ref()
                                                .map_or("Unnamed", |v| v)
                                        )
                                        .width(Length::Fill)
                                        .center()
                                    ]
                                    .into()
                                }),
                        )
                        .padding(5)
                        .spacing(10),
                    )
                } else {
                    scrollable(column![text("None found").width(Length::Fill).center()])
                },
            ]
            .width(Length::Fixed(100.0)),
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
                state
                    .selected_image
                    .as_ref()
                    .expect("Shouldn't fail")
                    .0
                    .clone(),
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
    column![
        photo_processing_progress_bar(state),
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
    ]
    .padding(5)
    .spacing(10)
    .into()
}

fn photo_processing_progress_bar(state: &GalleryPage) -> Element<Message> {
    match state.photo_process_progress {
        PhotoProcessingProgress::ThumbnailGeneration(progress) => container(
            row![
                text("Generating Thumbnails").height(Length::Fixed(20.0)),
                Space::with_width(Length::Fixed(20.0)),
                container(
                    progress_bar(0.0..=100.0, progress)
                        .width(Length::Fill)
                        .height(Length::Fixed(10.0))
                        .style(progress_bar::primary)
                )
                .height(Length::Fixed(20.0))
                .align_y(Center),
                Space::with_width(Length::Fixed(10.0)),
                text(format!("{:.2}%", progress)).height(Length::Fixed(20.0)),
                Space::with_width(Length::Fixed(10.0)),
                Tooltip::new(
                    button(Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/close.svg"
                    ))))
                    .on_press(Message::Gallery(GalleryPageMessage::AbortProcess))
                    .width(Length::Fixed(40.0)),
                    "Cancel Process",
                    iced::widget::tooltip::Position::Bottom
                ),
            ]
            .padding(10),
        )
        .style(container::bordered_box),
        PhotoProcessingProgress::FaceExtraction(progress) => container(
            row![
                text("Extracting Faces").height(Length::Fixed(20.0)),
                Space::with_width(Length::Fixed(20.0)),
                container(
                    progress_bar(0.0..=100.0, progress)
                        .width(Length::Fill)
                        .height(Length::Fixed(10.0))
                        .style(progress_bar::primary)
                )
                .height(Length::Fixed(20.0))
                .align_y(Center),
                Space::with_width(Length::Fixed(10.0)),
                text(format!("{:.2}%", progress)).height(Length::Fixed(20.0)),
                Space::with_width(Length::Fixed(10.0)),
                Tooltip::new(
                    button(Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/close.svg"
                    ))))
                    .on_press(Message::Gallery(GalleryPageMessage::AbortProcess))
                    .width(Length::Fixed(40.0)),
                    "Cancel Process",
                    iced::widget::tooltip::Position::Bottom
                ),
            ]
            .padding(10),
        )
        .style(container::bordered_box),
        PhotoProcessingProgress::FaceRecognition(progress) => container(
            row![
                text("Recognising Faces").height(Length::Fixed(20.0)),
                Space::with_width(Length::Fixed(20.0)),
                container(
                    progress_bar(0.0..=100.0, progress)
                        .width(Length::Fill)
                        .height(Length::Fixed(10.0))
                        .style(progress_bar::primary)
                )
                .height(Length::Fixed(20.0))
                .align_y(Center),
                Space::with_width(Length::Fixed(10.0)),
                text(format!("{:.2}%", progress)).height(Length::Fixed(20.0)),
                Space::with_width(Length::Fixed(10.0)),
                Tooltip::new(
                    button(Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/close.svg"
                    ))))
                    .on_press(Message::Gallery(GalleryPageMessage::AbortProcess))
                    .width(Length::Fixed(40.0)),
                    "Cancel Process",
                    iced::widget::tooltip::Position::Bottom
                ),
            ]
            .padding(10),
        )
        .style(container::bordered_box),
        PhotoProcessingProgress::None => container(row![]),
    }
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

pub fn tool_view(state: &GalleryPage) -> Element<Message> {
    if state.selected_image.is_some() {
        row![Tooltip::new(
            button(Svg::new(svg::Handle::from_memory(include_bytes!(
                "../../../icons/copy.svg"
            ))))
            .on_press(Message::Gallery(GalleryPageMessage::CopySelectedImagePath)),
            "Copy image path",
            iced::widget::tooltip::Position::Bottom
        ),]
        .width(Length::FillPortion(1))
        .into()
    } else {
        row![
            button("Extract faces").on_press(Message::Gallery(GalleryPageMessage::ExtractAllFaces)),
            button("Generate thumbnails")
                .on_press(Message::Gallery(GalleryPageMessage::GenerateAllThumbnails))
        ]
        .width(Length::FillPortion(1))
        .into()
    }
}
