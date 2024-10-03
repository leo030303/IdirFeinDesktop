use std::path::PathBuf;

use iced::widget::image::Handle;
use iced::widget::list;
use iced::widget::scrollable::Viewport;
use iced::{Element, Task};
use serde::{Deserialize, Serialize};

use crate::app::Message;

use super::update::update;
use super::view::{main_view, tool_view};

pub(crate) const IMAGE_HEIGHT: f32 = 350.0;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GalleryPageConfig {
    pub default_folder: Option<PathBuf>,
}

pub struct GalleryPage {
    pub(crate) loaded_image_indexes: Vec<usize>,
    pub(crate) selected_folder: Option<PathBuf>,
    pub(crate) last_images_scrolled_past_val: i64,
    pub(crate) gallery_list: list::Content<(PathBuf, Option<Handle>)>,
}

#[derive(Debug, Clone)]
pub enum GalleryPageMessage {
    PickGalleryFolder,
    LoadGalleryFolder,
    SetGalleryFilesList(Vec<PathBuf>),
    LoadImageHandle(PathBuf),
    SetImageHandle(PathBuf, Handle),
    UnloadImageHandle(PathBuf),
    GalleryScrolled(Viewport),
}

impl GalleryPage {
    pub fn new(config: &GalleryPageConfig) -> Self {
        Self {
            selected_folder: config.default_folder.clone(),
            gallery_list: list::Content::new(),
            last_images_scrolled_past_val: 0,
            loaded_image_indexes: vec![],
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

    pub fn tool_view(&self) -> Element<Message> {
        tool_view(self)
    }
}
