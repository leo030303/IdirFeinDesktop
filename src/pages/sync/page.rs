use iced::{Element, Task};
use serde::{Deserialize, Serialize};

use crate::app::{Message, APP_ID};

use super::update::update;
use super::view::{main_view, tool_view};

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub const IGNORE_LIST_FILE_NAME: &str = "sync_ignore_list.json";
pub const SYNC_LIST_FILE_NAME: &str = "sync_folder_list.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPageConfig {
    pub server_url: String,
    pub default_data_storage_folder: PathBuf,
    pub should_sync: bool,
    pub client_username: Option<String>,
    pub client_secret: Option<Vec<u8>>,
    pub ignored_remote_folder_ids: Vec<String>,
}

impl Default for SyncPageConfig {
    fn default() -> Self {
        Self {
            server_url: String::new(),
            default_data_storage_folder: dirs::home_dir().unwrap().join("idirfein"),
            should_sync: false,
            client_username: None,
            client_secret: None,
            ignored_remote_folder_ids: vec![],
        }
    }
}

pub struct SyncPage {
    pub ignore_list_editor_text: String,
    pub is_connected_to_server: bool,
    pub ignore_string_list: Vec<String>,
    pub folders_to_sync: HashMap<String, PathBuf>,
}

#[derive(Debug, Clone)]
pub enum SyncPageMessage {
    UpdateIgnoreListEditor(String),
    AddToIgnoreList,
    DeleteFromIgnoreList(usize),
    PickNewSyncListFolder,
    SetNewSyncListFolder(Option<PathBuf>),
    DeleteFromFolderList(String),
}

impl SyncPage {
    pub fn new(_config: &SyncPageConfig) -> Self {
        let ignore_string_list: Vec<String> = serde_json::from_str(
            &fs::read_to_string(
                dirs::config_dir()
                    .unwrap()
                    .join(APP_ID)
                    .join(IGNORE_LIST_FILE_NAME),
            )
            .unwrap_or_default(),
        )
        .unwrap_or_default();
        let folders_to_sync: HashMap<String, PathBuf> = serde_json::from_str(
            &fs::read_to_string(
                dirs::config_dir()
                    .unwrap()
                    .join(APP_ID)
                    .join(SYNC_LIST_FILE_NAME),
            )
            .unwrap_or_default(),
        )
        .unwrap_or_default();
        Self {
            is_connected_to_server: false,
            ignore_list_editor_text: String::new(),
            ignore_string_list,
            folders_to_sync,
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
