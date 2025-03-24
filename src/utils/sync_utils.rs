use std::{
    collections::HashMap,
    fs::{self, File},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use fast_rsync::{Signature, SignatureOptions};
use loro::LoroDoc;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::constants::{APP_ID, LORO_NOTE_ID};
use crate::pages::sync::page::SYNC_LIST_FILE_NAME;

const DEFAULT_CRYPTO_HASH_SIZE: u32 = 16;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerFileRequest {
    /// The file at the given index is being requested by the remote server
    RequestNewFile(usize),
    /// This new folder from the server should be created on the client
    CreateFolder(String),
    /// The signature of the file at the given index is being requested so it can create a diff to update it
    RequestSignature(usize),
    /// The client should apply the given diff to the file at the given index
    ApplyDiff(usize, Vec<u8>),
    /// The diff between the file at the given index and the signature made up from these bytes is being requested to update the server file
    RequestDiff(usize, Vec<u8>),
    /// The given file data should be written to the given relative path in the given folder id
    CreateFile(Vec<u8>, PathBuf, String),
    /// The file at the given index should be deleted on the client
    Delete(usize),
    /// The loro file at the given index is being requested to update the server file, and the server loro file is given to update the client version
    RequestLoroUpdate(usize, Vec<u8>),
}

impl ServerFileRequest {
    /// Important for ordering the requests, as its important some requests are handled before others
    fn get_ordering_val(&self) -> usize {
        match self {
            ServerFileRequest::RequestNewFile(_) => 7,
            ServerFileRequest::CreateFolder(_) => 1,
            ServerFileRequest::RequestSignature(_) => 3,
            ServerFileRequest::ApplyDiff(_, _) => 6,
            ServerFileRequest::RequestLoroUpdate(_, _) => 5,
            ServerFileRequest::RequestDiff(_, _) => 4,
            ServerFileRequest::CreateFile(_, _, _) => 8,
            ServerFileRequest::Delete(_) => 2,
        }
    }
}

impl Ord for ServerFileRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_ordering_val().cmp(&other.get_ordering_val())
    }
}

impl PartialOrd for ServerFileRequest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ServerFileRequest {
    fn eq(&self, other: &Self) -> bool {
        self.get_ordering_val().eq(&other.get_ordering_val())
    }
}

impl Eq for ServerFileRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientFileResponse {
    /// The response to RequestNewFile
    NewFile(usize, Vec<u8>),
    /// The response to RequestSignature
    Signature(usize, Vec<u8>),
    /// The response to RequestDiff
    Diff(usize, Vec<u8>),
    /// The response to RequestLoroUpdate
    LoroUpdate(usize, Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filedata {
    /// The unique ID of the folder which contains this file
    pub folder_id: String,
    /// The relative path of this file from the root of its base folder
    pub relative_path: PathBuf,
    /// The total size of this file in bytes
    pub size: u64,
    /// The last modification of the file in seconds since the Unix epoch
    pub modification_time: i64,
    /// Whether to delete the file on the remote server
    pub should_delete: bool,
}

impl Filedata {
    pub fn get_absolute_path(&self, folders_list: &HashMap<String, PathBuf>) -> Option<PathBuf> {
        folders_list
            .get(&self.folder_id)
            .map(|base_path| base_path.join(&self.relative_path))
    }
}

pub struct SyncManager {
    pub list_of_folders: HashMap<String, PathBuf>,
    pub client_file_list: Vec<Filedata>,
    pub client_ignore_list: Vec<String>,
    pub client_ignored_remote_folders_list: Vec<String>,
    pub default_data_storage_folder: PathBuf,
}

impl SyncManager {
    pub fn new(
        list_of_folders: HashMap<String, PathBuf>,
        client_ignore_list: Vec<String>,
        default_data_storage_folder: PathBuf,
        client_ignored_remote_folders_list: Vec<String>,
    ) -> Self {
        let previously_synced_client_file_list = vec![]; // TODO read from config
        let client_file_list = SyncManager::get_list_of_files_to_sync(
            &list_of_folders,
            &client_ignore_list,
            &previously_synced_client_file_list,
        );
        Self {
            list_of_folders,
            client_file_list,
            client_ignore_list,
            default_data_storage_folder,
            client_ignored_remote_folders_list,
        }
    }
    pub fn get_list_of_files_to_sync(
        list_of_folders: &HashMap<String, PathBuf>,
        ignore_list: &[String],
        old_client_file_list: &[Filedata],
    ) -> Vec<Filedata> {
        let mut client_file_list: Vec<Filedata> = list_of_folders
            .iter()
            .flat_map(|(folder_id, folder_path)| {
                let mut all_filedata: Vec<Filedata> = WalkDir::new(folder_path)
                    .into_iter()
                    .filter_map(|dir_entry_result| dir_entry_result.ok())
                    .map(|dir_entry| dir_entry.into_path())
                    // Ensure you only list files and not directories too
                    .filter(|filepath| filepath.is_file())
                    // Check ignore list
                    .filter(|filepath| {
                        filepath.to_str().is_some_and(|path_str| {
                            !ignore_list
                                .iter()
                                .any(|ignore_list_item| path_str.contains(ignore_list_item))
                        }) && filepath.metadata().is_ok()
                    })
                    .map(|filepath| {
                        let metadata = filepath.metadata().unwrap();
                        // Truncate file path to get path relative to folder root
                        let relative_path = filepath
                            .strip_prefix(folder_path)
                            .expect("Path should be child of folder path")
                            .to_path_buf();
                        Filedata {
                            folder_id: folder_id.clone(),
                            relative_path,
                            size: metadata.size(),
                            modification_time: metadata.mtime(),
                            should_delete: false,
                        }
                    })
                    .collect();
                all_filedata.sort_unstable_by(|a, b| a.relative_path.cmp(&b.relative_path));
                all_filedata.sort_by(|a, b| a.folder_id.cmp(&b.folder_id));
                all_filedata
            })
            .collect();
        let delete_list: Vec<Filedata> = old_client_file_list
            .iter()
            .filter(|file_data_to_check| {
                !client_file_list.iter().any(|current_file_data| {
                    current_file_data.folder_id == file_data_to_check.folder_id
                        && current_file_data.relative_path == file_data_to_check.relative_path
                })
            })
            .cloned()
            .collect();
        delete_list.into_iter().for_each(|mut item_to_delete| {
            item_to_delete.should_delete = true;
            client_file_list.push(item_to_delete);
        });
        client_file_list
    }

    /// Handles a server request, returns a client response
    pub fn handle_server_request(
        &mut self,
        request: ServerFileRequest,
    ) -> Option<ClientFileResponse> {
        match request {
            ServerFileRequest::RequestNewFile(file_index) => {
                if let Some(path_to_read) = self
                    .client_file_list
                    .get(file_index)
                    .and_then(|file_data| file_data.get_absolute_path(&self.list_of_folders))
                {
                    let file_bytes = fs::read(path_to_read).unwrap();
                    return Some(ClientFileResponse::NewFile(file_index, file_bytes));
                }
            }
            ServerFileRequest::CreateFolder(new_folder_id) => {
                self.list_of_folders.insert(
                    new_folder_id.clone(),
                    self.default_data_storage_folder.join(&new_folder_id),
                );
                let _ = fs::write(
                    dirs::config_dir()
                        .unwrap()
                        .join(APP_ID)
                        .join(SYNC_LIST_FILE_NAME),
                    serde_json::to_string(&self.list_of_folders).unwrap(),
                );
            }
            ServerFileRequest::RequestSignature(file_index) => {
                if let Some(path_to_read) = self
                    .client_file_list
                    .get(file_index)
                    .and_then(|file_data| file_data.get_absolute_path(&self.list_of_folders))
                {
                    return Some(ClientFileResponse::Signature(
                        file_index,
                        Signature::calculate(
                            &fs::read(path_to_read).unwrap(),
                            SignatureOptions {
                                block_size: get_ideal_block_size(
                                    self.client_file_list
                                        .get(file_index)
                                        .expect("Already checked this index")
                                        .size,
                                ),
                                crypto_hash_size: DEFAULT_CRYPTO_HASH_SIZE,
                            },
                        )
                        .into_serialized(),
                    ));
                }
            }
            ServerFileRequest::ApplyDiff(file_index, diff_bytes) => {
                if let Some(path_to_apply) = self
                    .client_file_list
                    .get(file_index)
                    .and_then(|file_data| file_data.get_absolute_path(&self.list_of_folders))
                {
                    let file_bytes = fs::read(&path_to_apply).unwrap();
                    let mut file_handle = File::create(path_to_apply).unwrap();
                    let _ = fast_rsync::apply(&file_bytes, &diff_bytes, &mut file_handle);
                }
            }
            ServerFileRequest::RequestDiff(file_index, signature_bytes) => {
                if let Some(path_to_read) = self
                    .client_file_list
                    .get(file_index)
                    .and_then(|file_data| file_data.get_absolute_path(&self.list_of_folders))
                {
                    if let Ok(signature) = Signature::deserialize(signature_bytes) {
                        let file_bytes = fs::read(&path_to_read).unwrap();
                        let mut diff: Vec<u8> = vec![];
                        if fast_rsync::diff(&signature.index(), &file_bytes, &mut diff).is_ok() {
                            return Some(ClientFileResponse::Diff(file_index, diff));
                        }
                    }
                }
            }
            ServerFileRequest::CreateFile(file_bytes, relative_path, folder_id) => {
                if let Some(path_to_write) = self
                    .list_of_folders
                    .get(&folder_id)
                    .map(|base_path| base_path.join(relative_path))
                {
                    let _ = fs::create_dir_all(path_to_write.parent().unwrap_or(Path::new("/")));
                    let _ = fs::write(path_to_write, file_bytes);
                }
            }
            ServerFileRequest::Delete(file_index) => {
                if let Some(path_to_delete) = self
                    .client_file_list
                    .get(file_index)
                    .and_then(|file_data| file_data.get_absolute_path(&self.list_of_folders))
                {
                    let _ = fs::remove_file(path_to_delete);
                }
            }
            ServerFileRequest::RequestLoroUpdate(file_index, server_loro_bytes) => {
                if let Some(path_to_apply) = self
                    .client_file_list
                    .get(file_index)
                    .and_then(|file_data| file_data.get_absolute_path(&self.list_of_folders))
                {
                    let client_loro_file_bytes = fs::read(&path_to_apply).unwrap();
                    let server_doc = LoroDoc::new();
                    let _ = server_doc.import(&server_loro_bytes);
                    let client_doc = LoroDoc::new();
                    let _ = client_doc.import_batch(&[client_loro_file_bytes, server_loro_bytes]);
                    let client_export_bytes = client_doc.export_from(&server_doc.oplog_vv());
                    let new_client_bytes = client_doc.export_snapshot();
                    let _ = fs::write(&path_to_apply, new_client_bytes);
                    let base_file_path = path_to_apply.parent().unwrap().join(
                        path_to_apply
                            .file_stem()
                            .unwrap()
                            .to_string_lossy()
                            .chars()
                            .skip(1)
                            .collect::<String>(),
                    );
                    let updated_base_file_text = client_doc.get_text(LORO_NOTE_ID).to_string();
                    let _ = fs::write(base_file_path, updated_base_file_text);
                    return Some(ClientFileResponse::LoroUpdate(
                        file_index,
                        client_export_bytes,
                    ));
                }
            }
        }
        None
    }
    pub fn get_initialiser_data(&self) -> String {
        println!("{:?}", self.client_file_list); // TODO Remove this
        serde_json::to_string(&SyncInitialiserData {
            client_file_list: self.client_file_list.clone(),
            client_ignore_list: self.client_ignore_list.clone(),
            client_ignored_remote_folders_list: self.client_ignored_remote_folders_list.clone(),
        })
        .unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncInitialiserData {
    pub client_file_list: Vec<Filedata>,
    pub client_ignore_list: Vec<String>,
    pub client_ignored_remote_folders_list: Vec<String>,
}

/// Get the ideal block size for a given file, based off the rsync implementation https://git.samba.org/?p=rsync.git;a=blob;f=generator.c;h=5538a92dd57ddf8671a2404a7308ada73a710f58;hb=HEAD#l600
pub fn get_ideal_block_size(file_size: u64) -> u32 {
    let sqrt = (file_size as f64).sqrt();
    let rounded = (sqrt / 8.0).round() * 8.0;
    rounded as u32
}
