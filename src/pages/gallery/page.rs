use std::path::PathBuf;

use iced::event::Status;
use iced::keyboard::key::Named;
use iced::keyboard::Key;
use iced::widget::image::Handle;
use iced::widget::scrollable::Viewport;
use iced::widget::{list, scrollable};
use iced::{event, keyboard, Element, Event, Task};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::app::Message;

use super::update::update;
use super::view::{main_view, tool_view};

pub(crate) const IMAGE_HEIGHT: f32 = 550.0;
pub(crate) const ARROW_KEY_SCROLL_AMOUNT: f32 = 50.0;
pub(crate) const PAGE_KEY_SCROLL_AMOUNT: f32 = 500.0;

pub(crate) static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GalleryPageConfig {
    pub default_folder: Option<PathBuf>,
}

pub struct GalleryPage {
    pub(crate) loaded_image_indexes: Vec<usize>,
    pub(crate) selected_folder: Option<PathBuf>,
    pub(crate) selected_image: Option<PathBuf>,
    pub(crate) last_images_scrolled_past_val: i64,
    pub(crate) gallery_list: list::Content<Vec<(PathBuf, Option<Handle>)>>,
    pub(crate) scrollable_viewport_option: Option<Viewport>,
}

#[derive(Debug, Clone)]
pub enum GalleryPageMessage {
    PickGalleryFolder,
    LoadGalleryFolder,
    SelectImageForBigView(Option<PathBuf>),
    SetGalleryFilesList(Vec<Vec<PathBuf>>),
    LoadImageHandle(usize, Vec<(PathBuf, Option<Handle>)>),
    SetImageHandle(usize, PathBuf, Handle),
    UnloadImageHandle(usize, PathBuf),
    GalleryScrolled(Viewport),
    ArrowDownKeyPressed,
    ArrowUpKeyPressed,
    PageDownKeyPressed,
    PageUpKeyPressed,
}

impl GalleryPage {
    pub fn new(config: &GalleryPageConfig) -> Self {
        Self {
            selected_folder: config.default_folder.clone(),
            gallery_list: list::Content::new(),
            last_images_scrolled_past_val: 0,
            loaded_image_indexes: vec![],
            scrollable_viewport_option: None,
            selected_image: None,
        }
    }

    pub fn opening_task() -> Task<Message> {
        Task::none()
    }

    pub fn closing_task(&mut self) -> Task<Message> {
        println!("Closing task from gallery");
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
