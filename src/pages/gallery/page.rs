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

use super::gallery_utils::{FaceData, PhotoProcessingProgress};
use super::update::update;
use super::view::{main_view, tool_view};

pub(crate) const NUM_IMAGES_IN_ROW: usize = 4;
pub(crate) const IMAGE_HEIGHT: f32 = 220.0;
pub(crate) const ROW_BATCH_SIZE: usize = 10;
pub(crate) const ARROW_KEY_SCROLL_AMOUNT: f32 = 50.0;
pub(crate) const PAGE_KEY_SCROLL_AMOUNT: f32 = 500.0;
pub(crate) const THUMBNAIL_SIZE: u32 = 200;
pub(crate) const RENAME_PERSON_INPUT_ID: &str = "RENAME_PERSON_INPUT_ID";
pub(crate) const UNNAMED_STRING: &str = "Unnamed";
pub(crate) const THUMBNAIL_FOLDER_NAME: &str = ".thumbnails";
pub(crate) const FACE_DATA_FOLDER_NAME: &str = ".face_data";
pub(crate) const FACE_DATA_FILE_NAME: &str = "extracted_faces.json";
pub(crate) const PATH_TO_FACE_RECOGNITION_MODEL: &str =
    "models/face_recognition_sface_2021dec.onnx";
pub(crate) const PATH_TO_FACE_EXTRACTION_MODEL: &str = "models/blazefaces-640.onnx";
pub(crate) const PATH_TO_TEXT_RECOGNITION_MODEL: &str = "models/text-recognition.rten";
pub(crate) const PATH_TO_TEXT_DETECTION_MODEL: &str = "models/text-detection.rten";

pub(crate) static GALLERY_SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
pub(crate) static LIST_PEOPLE_SCROLL_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GalleryPageConfig {
    pub default_folder: Option<PathBuf>,
    pub run_thumbnail_generation_on_start: bool,
    pub run_face_extraction_on_start: bool,
    pub run_face_recognition_on_start: bool,
}

pub struct GalleryPage {
    pub(crate) selected_folder: Option<PathBuf>,
    pub(crate) selected_image: Option<(PathBuf, Vec<FaceData>)>,
    pub(crate) first_loaded_row_index: usize,
    pub(crate) gallery_row_list: Vec<ImageRow>,
    pub(crate) gallery_paths_list: Vec<PathBuf>,
    pub(crate) gallery_parents_list: Vec<PathBuf>,
    pub(crate) gallery_scrollable_viewport_option: Option<Viewport>,
    pub(crate) people_list_scrollable_viewport_option: Option<Viewport>,
    pub(crate) photo_process_progress: PhotoProcessingProgress,
    pub(crate) photo_process_abort_handle: Option<iced::task::Handle>,
    pub(crate) person_to_manage: Option<(PathBuf, FaceData)>,
    pub(crate) rename_person_editor_text: String,
    pub(crate) show_ignore_person_confirmation: bool,
    pub(crate) show_rename_confirmation: bool,
    pub(crate) show_people_view: bool,
    pub(crate) people_list: Vec<(String, PathBuf)>,
    pub(crate) person_to_view: Option<PersonToView>,
    pub(crate) run_thumbnail_generation_on_start: bool,
    pub(crate) run_face_extraction_on_start: bool,
    pub(crate) run_face_recognition_on_start: bool,
    pub(crate) current_image_ocr_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageRow {
    pub is_loaded: bool,
    pub index: usize,
    pub images_data: Vec<(PathBuf, Option<Handle>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PersonToView {
    pub name: String,
    pub list_of_image_paths: Vec<PathBuf>,
    pub list_of_rows: Vec<ImageRow>,
}

#[derive(Debug, Clone)]
pub enum GalleryPageMessage {
    PickGalleryFolder,
    SetGalleryFolder(Option<PathBuf>),
    LoadGalleryFolder,
    SelectImageForBigView(Option<PathBuf>),
    SetGalleryFilesList(Vec<Vec<PathBuf>>, Vec<PathBuf>),
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
    CopySelectedImagePath,
    ExtractAllFaces,
    GenerateAllThumbnails,
    RunFaceRecognition,
    SetPhotoProcessProgress(PhotoProcessingProgress),
    AbortProcess,
    OpenManagePersonView(PathBuf, FaceData),
    CloseManagePersonView,
    MaybeRenamePerson(Option<String>),
    ConfirmRenamePerson,
    CancelRenamePerson,
    UpdateRenamePersonEditor(String),
    MaybeIgnorePerson,
    ConfirmIgnorePerson,
    CancelIgnorePerson,
    TogglePeopleView,
    SetPeopleList(Vec<(String, PathBuf)>),
    SetPersonToViewName(Option<String>),
    SetPersonToViewPaths(Vec<PathBuf>),
    PeopleListScrolled(Viewport),
    RunOcrOnSelectedImage,
    SetCurrentImageOcrText(Option<String>),
    CopyOcrText,
}

impl GalleryPage {
    pub fn new(config: &GalleryPageConfig) -> Self {
        Self {
            selected_folder: config.default_folder.clone(),
            gallery_row_list: vec![],
            first_loaded_row_index: 0,
            gallery_scrollable_viewport_option: None,
            people_list_scrollable_viewport_option: None,
            selected_image: None,
            gallery_paths_list: vec![],
            gallery_parents_list: vec![],
            photo_process_progress: PhotoProcessingProgress::None,
            photo_process_abort_handle: None,
            person_to_manage: None,
            rename_person_editor_text: String::new(),
            show_ignore_person_confirmation: false,
            show_rename_confirmation: false,
            show_people_view: false,
            people_list: vec![],
            person_to_view: None,
            run_thumbnail_generation_on_start: config.run_thumbnail_generation_on_start,
            run_face_extraction_on_start: config.run_face_extraction_on_start,
            run_face_recognition_on_start: config.run_face_recognition_on_start,
            current_image_ocr_text: None,
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
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: Key::Named(Named::ArrowRight),
                    ..
                }),
                Status::Ignored,
            ) => Some(Message::Gallery(GalleryPageMessage::SelectNextImage)),
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: Key::Named(Named::ArrowLeft),
                    ..
                }),
                Status::Ignored,
            ) => Some(Message::Gallery(GalleryPageMessage::SelectPreviousImage)),
            _ => None,
        })
    }

    pub fn tool_view(&self) -> Element<Message> {
        tool_view(self)
    }
}
