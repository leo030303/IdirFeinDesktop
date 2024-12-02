use std::path::PathBuf;

use iced::event::Status;
use iced::keyboard::key::Named;
use iced::keyboard::Key;
use iced::widget::image::Handle;
use iced::widget::scrollable;
use iced::widget::scrollable::Viewport;
use iced::{event, keyboard, Element, Event, Task};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::app::Message;

use super::update::update;
use super::view::{main_view, tool_view};

pub(crate) const NUM_IMAGES_IN_ROW: usize = 4;
pub(crate) const IMAGE_HEIGHT: f32 = 220.0;
pub(crate) const ROW_BATCH_SIZE: usize = 10;
pub(crate) const ARROW_KEY_SCROLL_AMOUNT: f32 = 50.0;
pub(crate) const PAGE_KEY_SCROLL_AMOUNT: f32 = 500.0;
pub(crate) const THUMBNAIL_SIZE: u32 = 200;
pub(crate) const THUMBNAIL_FOLDER_NAME: &str = ".thumbnails";

pub(crate) static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GalleryPageConfig {
    pub default_folder: Option<PathBuf>,
}

pub struct GalleryPage {
    pub(crate) selected_folder: Option<PathBuf>,
    pub(crate) selected_image: Option<PathBuf>,
    pub(crate) first_loaded_row_index: usize,
    pub(crate) gallery_row_list: Vec<ImageRow>,
    pub(crate) gallery_paths_list: Vec<PathBuf>,
    pub(crate) scrollable_viewport_option: Option<Viewport>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageRow {
    pub loaded: bool,
    pub index: usize,
    pub images_data: Vec<(PathBuf, Option<Handle>)>,
}

#[derive(Debug, Clone)]
pub enum GalleryPageMessage {
    PickGalleryFolder,
    SetGalleryFolder(Option<PathBuf>),
    LoadGalleryFolder,
    SelectImageForBigView(Option<PathBuf>),
    SetGalleryFilesList(Vec<Vec<PathBuf>>),
    LoadImageRows(Vec<ImageRow>),
    UnloadImageRows(Vec<ImageRow>),
    SetImageRows(Vec<ImageRow>),
    GalleryScrolled(Viewport),
    ArrowDownKeyPressed,
    ArrowUpKeyPressed,
    PageDownKeyPressed,
    PageUpKeyPressed,
    EscapeKeyPressed,
    SelectPreviousImage,
    SelectNextImage,
}

impl GalleryPage {
    pub fn new(config: &GalleryPageConfig) -> Self {
        Self {
            selected_folder: config.default_folder.clone(),
            gallery_row_list: vec![],
            first_loaded_row_index: 0,
            scrollable_viewport_option: None,
            selected_image: None,
            gallery_paths_list: vec![],
        }
    }

    pub fn opening_task() -> Task<Message> {
        Task::done(Message::Gallery(GalleryPageMessage::LoadGalleryFolder))
    }

    pub fn closing_task(&mut self) -> Task<Message> {
        Task::none()
    }

    pub fn update(&mut self, message: GalleryPageMessage) -> Task<Message> {
        update(self, message)
    }

    pub fn view(&self) -> Element<Message> {
        main_view(self)
    }
    pub fn subscription() -> iced::Subscription<Message> {
        event::listen_with(|event, status, _id| match (event, status) {
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: Key::Named(Named::Escape),
                    ..
                }),
                Status::Ignored,
            ) => Some(Message::Gallery(GalleryPageMessage::EscapeKeyPressed)),
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: Key::Named(Named::ArrowUp),
                    ..
                }),
                Status::Ignored,
            ) => Some(Message::Gallery(GalleryPageMessage::ArrowUpKeyPressed)),
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: Key::Named(Named::ArrowDown),
                    ..
                }),
                Status::Ignored,
            ) => Some(Message::Gallery(GalleryPageMessage::ArrowDownKeyPressed)),
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: Key::Named(Named::PageUp),
                    ..
                }),
                Status::Ignored,
            ) => Some(Message::Gallery(GalleryPageMessage::PageUpKeyPressed)),
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: Key::Named(Named::PageDown),
                    ..
                }),
                Status::Ignored,
            ) => Some(Message::Gallery(GalleryPageMessage::PageDownKeyPressed)),
            _ => None,
        })
    }

    pub fn tool_view(&self) -> Element<Message> {
        tool_view(self)
    }
}
