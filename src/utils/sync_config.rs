use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncConfig {
    pub server_url: String,
    pub ignore_string_list: Vec<String>,
    pub folders_to_sync: Vec<(Uuid, PathBuf)>,
}
