use crate::pages::gallery::page::RENAME_PERSON_INPUT_ID;
use std::path::Path;

use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container,
        image::{self, Handle},
        progress_bar, row, scrollable, svg, text, text_input, Image, MouseArea, Space, Svg,
        Tooltip,
    },
    Alignment::Center,
    Element, Length,
};

use crate::app::Message;

use super::{
    gallery_utils::PhotoProcessingProgress,
    page::{
        GalleryPage, GalleryPageMessage, FACE_DATA_FOLDER_NAME, GALLERY_SCROLLABLE_ID,
        IMAGE_HEIGHT, LIST_PEOPLE_SCROLL_ID, UNNAMED_STRING,
    },
};

pub fn main_view(state: &GalleryPage) -> Element<Message> {
    if state.selected_folder.is_none() {
        no_gallery_folder_selected_view(state)
    } else if state.selected_image.is_some() {
        big_image_viewer(state)
    } else if state.show_people_view {
        list_people_view(state)
    } else {
        gallery_grid(state)
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
            if state.person_to_manage.is_some() {
                person_management_view(state)
            } else {
                people_sidebar(state)
            },
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

fn people_sidebar(state: &GalleryPage) -> Element<Message> {
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
                            MouseArea::new(column![
                                Image::new(image::Handle::from_path(
                                    state
                                        .selected_image
                                        .as_ref()
                                        .expect("Can't fail")
                                        .0
                                        .parent()
                                        .unwrap_or(Path::new("/"))
                                        .join(FACE_DATA_FOLDER_NAME)
                                        .join(&face_data.thumbnail_filename)
                                ))
                                .content_fit(iced::ContentFit::ScaleDown)
                                .filter_method(image::FilterMethod::Nearest),
                                text(
                                    face_data
                                        .name_of_person
                                        .as_ref()
                                        .map_or(UNNAMED_STRING, |v| v)
                                )
                                .width(Length::Fill)
                                .center()
                            ])
                            .on_press(Message::Gallery(GalleryPageMessage::OpenManagePersonView(
                                state.selected_image.as_ref().expect("Can't fail").0.clone(),
                                face_data.clone(),
                            )))
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
    .width(Length::Fixed(100.0))
    .into()
}

fn gallery_grid(state: &GalleryPage) -> Element<Message> {
    let image_rows_to_display = if let Some(person_to_view) = state.person_to_view.as_ref() {
        &person_to_view.list_of_rows
    } else {
        &state.gallery_row_list
    };
    column![
        photo_processing_progress_bar(state),
        if let Some(person_to_view) = state.person_to_view.as_ref() {
            column![text(format!("Photos of {}", &person_to_view.name))
                .width(Length::Fill)
                .center()
                .size(24)]
        } else {
            column![]
        },
        scrollable(column(image_rows_to_display.iter().map(|image_row| {
            if image_row.is_loaded {
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
        .id(GALLERY_SCROLLABLE_ID.clone())
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
                text(format!("{:.2}%", progress))
                    .height(Length::Fixed(20.0))
                    .width(Length::Fixed(50.0)),
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

fn person_management_view(state: &GalleryPage) -> Element<Message> {
    let (image_path, face_data) = state.person_to_manage.as_ref().expect("Can't fail");
    scrollable(if state.show_ignore_person_confirmation {
        column![
            row![
                Space::with_width(Length::Fill),
                Tooltip::new(
                    button(Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/close.svg"
                    ))))
                    .on_press(Message::Gallery(GalleryPageMessage::CloseManagePersonView))
                    .width(Length::Fixed(50.0)),
                    "Close Person Manager",
                    iced::widget::tooltip::Position::Bottom
                ),
            ],
            Image::new(image::Handle::from_path(
                image_path
                    .parent()
                    .unwrap_or(Path::new("/"))
                    .join(FACE_DATA_FOLDER_NAME)
                    .join(&face_data.thumbnail_filename)
            ))
            .content_fit(iced::ContentFit::ScaleDown)
            .filter_method(image::FilterMethod::Nearest),
            text(format!(
                "Ignore {}?",
                face_data
                    .name_of_person
                    .as_ref()
                    .unwrap_or(&String::from(UNNAMED_STRING)),
            ))
            .width(Length::Fill)
            .center(),
            button(text("Confirm").width(Length::Fill).center())
                .on_press(Message::Gallery(GalleryPageMessage::ConfirmIgnorePerson))
                .width(Length::Fill)
                .style(button::danger),
            button(text("Cancel").width(Length::Fill).center())
                .on_press(Message::Gallery(GalleryPageMessage::CancelIgnorePerson))
                .width(Length::Fill),
        ]
        .width(Length::Fixed(200.0))
        .padding(10)
        .spacing(10)
    } else if state.show_rename_confirmation {
        column![
            row![
                Space::with_width(Length::Fill),
                Tooltip::new(
                    button(Svg::new(svg::Handle::from_memory(include_bytes!(
                        "../../../icons/close.svg"
                    ))))
                    .on_press(Message::Gallery(GalleryPageMessage::CloseManagePersonView))
                    .width(Length::Fixed(50.0)),
                    "Close Person Manager",
                    iced::widget::tooltip::Position::Bottom
                ),
            ],
            Image::new(image::Handle::from_path(
                image_path
                    .parent()
                    .unwrap_or(Path::new("/"))
                    .join(FACE_DATA_FOLDER_NAME)
                    .join(&face_data.thumbnail_filename)
            ))
            .content_fit(iced::ContentFit::ScaleDown)
            .filter_method(image::FilterMethod::Nearest)
            .width(Length::Fill),
            text(format!(
                "Rename face from {} to {}?",
                face_data
                    .name_of_person
                    .as_ref()
                    .unwrap_or(&String::from(UNNAMED_STRING)),
                state.rename_person_editor_text
            ))
            .width(Length::Fill)
            .center(),
            button(text("Confirm").width(Length::Fill).center())
                .on_press(Message::Gallery(GalleryPageMessage::ConfirmRenamePerson))
                .width(Length::Fill)
                .style(button::danger),
            button(text("Cancel").width(Length::Fill).center())
                .on_press(Message::Gallery(GalleryPageMessage::CancelRenamePerson))
                .width(Length::Fill),
        ]
        .width(Length::Fixed(200.0))
        .padding(10)
        .spacing(10)
    } else {
        column![row![
            column![
                row![
                    Space::with_width(Length::Fill),
                    Tooltip::new(
                        button(Svg::new(svg::Handle::from_memory(include_bytes!(
                            "../../../icons/close.svg"
                        ))))
                        .on_press(Message::Gallery(GalleryPageMessage::CloseManagePersonView))
                        .width(Length::Fixed(50.0)),
                        "Close Person Manager",
                        iced::widget::tooltip::Position::Bottom
                    ),
                ],
                Image::new(image::Handle::from_path(
                    image_path
                        .parent()
                        .unwrap_or(Path::new("/"))
                        .join(FACE_DATA_FOLDER_NAME)
                        .join(&face_data.thumbnail_filename)
                ))
                .content_fit(iced::ContentFit::ScaleDown)
                .filter_method(image::FilterMethod::Nearest),
                text(
                    face_data
                        .name_of_person
                        .as_deref()
                        .unwrap_or(UNNAMED_STRING),
                )
                .width(Length::Fill)
                .center(),
                button(text("Ignore").width(Length::Fill).center())
                    .on_press(Message::Gallery(GalleryPageMessage::MaybeIgnorePerson))
                    .width(Length::Fill),
                text_input("Rename person", &state.rename_person_editor_text)
                    .on_input(|s| Message::Gallery(GalleryPageMessage::UpdateRenamePersonEditor(s)))
                    .on_submit(Message::Gallery(GalleryPageMessage::MaybeRenamePerson(
                        None
                    )))
                    .id(RENAME_PERSON_INPUT_ID)
                    .width(Length::Fill),
                button(text("Rename").width(Length::Fill).center())
                    .on_press(Message::Gallery(GalleryPageMessage::MaybeRenamePerson(
                        None
                    )))
                    .width(Length::Fill),
            ]
            .width(Length::Fixed(200.0))
            .padding(10)
            .spacing(10),
            column(
                state
                    .people_list
                    .iter()
                    .filter(|(name_of_person, _thumbnail_path)| name_of_person
                        .to_lowercase()
                        .contains(&state.rename_person_editor_text.to_lowercase()))
                    .filter(|(name_of_person, _thumbnail_path)| name_of_person != UNNAMED_STRING)
                    .map(|(name_of_person, _thumbnail_path)| {
                        button(
                            text(format!("Rename to {name_of_person}"))
                                .width(Length::Fill)
                                .center(),
                        )
                        .on_press(Message::Gallery(GalleryPageMessage::MaybeRenamePerson(
                            Some(name_of_person.clone()),
                        )))
                        .width(Length::Fill)
                        .into()
                    })
            )
            .spacing(10)
            .padding(10)
            .width(Length::Fixed(200.0))
        ]]
    })
    .into()
}

fn list_people_view(state: &GalleryPage) -> Element<Message> {
    scrollable(
        row(state
            .people_list
            .iter()
            .map(|(name_of_person, path_to_thumbnail)| {
                MouseArea::new(
                    column![
                        Image::new(image::Handle::from_path(path_to_thumbnail))
                            .content_fit(iced::ContentFit::ScaleDown)
                            .filter_method(image::FilterMethod::Nearest),
                        text(name_of_person).width(Length::Fill).center()
                    ]
                    .width(Length::Fixed(200.0))
                    .spacing(10)
                    .padding(10),
                )
                .on_press(Message::Gallery(GalleryPageMessage::SetPersonToViewName(
                    Some(name_of_person.clone()),
                )))
                .into()
            }))
        .wrap(),
    )
    .id(LIST_PEOPLE_SCROLL_ID.clone())
    .on_scroll(|viewport| Message::Gallery(GalleryPageMessage::PeopleListScrolled(viewport)))
    .width(Length::Fill)
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
        row![Tooltip::new(
            button(Svg::new(if state.show_people_view {
                svg::Handle::from_memory(include_bytes!("../../../icons/image-round.svg"))
            } else {
                svg::Handle::from_memory(include_bytes!("../../../icons/people.svg"))
            }))
            .on_press(Message::Gallery(GalleryPageMessage::TogglePeopleView)),
            if state.show_people_view {
                "Back to main gallery (Esc)"
            } else if state.person_to_view.is_some() {
                "Back to people view (Esc)"
            } else {
                "View recognised people"
            },
            iced::widget::tooltip::Position::Bottom
        ),]
        .width(Length::FillPortion(1))
        .into()
    }
}
