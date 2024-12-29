use iced::{Element, Task};
use serde::{Deserialize, Serialize};

use crate::app::Message;

use super::update::update;
use super::view::{main_view, tool_view};

use std::path::PathBuf;

use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncPageConfig {
    pub server_url: String,
}

pub struct SyncPage {
    pub ignore_list_editor_text: String,
    pub is_connected_to_server: bool,
    pub ignore_string_list: Vec<String>,
    pub ignore_folder_id_list: Vec<Uuid>,
    pub folders_to_sync: Vec<(Uuid, PathBuf)>,
}

#[derive(Debug, Clone)]
pub enum SyncPageMessage {
    UpdateIgnoreListEditor(String),
    AddToIgnoreList,
    DeleteFromIgnoreList(usize),
    PickNewSyncListFolder,
    SetNewSyncListFolder(Option<PathBuf>),
    DeleteFromFolderList(usize),
}

impl SyncPage {
    pub fn new(_config: &SyncPageConfig) -> Self {
        // TODO Read from disk
        Self {
            is_connected_to_server: false,
            ignore_list_editor_text: String::new(),
            ignore_string_list: vec![],
            folders_to_sync: vec![],
            ignore_folder_id_list: vec![],
        }
    }

    pub fn opening_task() -> Task<Message> {
        Task::none()
    }

    pub fn closing_task(&mut self) -> Task<Message> {
        Task::none()
    }

    pub fn update(&mut self, message: SyncPageMessage) -> Task<Message> {
        update(self, message)
    }

    pub fn view(&self) -> Element<Message> {
        main_view(self)
    }

    pub fn tool_view(&self) -> Element<Message> {
        tool_view(self)
    }
}
