use std::{os::unix::fs::MetadataExt, path::PathBuf};

use walkdir::WalkDir;

pub struct Filedata {
    pub path: PathBuf,
    pub size: u64,
    pub modification_time: i64,
}

pub fn get_list_of_files_to_sync(
    list_of_folders: Vec<(uuid::Uuid, PathBuf)>,
    ignore_list: Vec<String>,
) -> Vec<(uuid::Uuid, Vec<Filedata>)> {
    list_of_folders
        .into_iter()
        .map(|(folder_id, folder_path)| {
            let all_filedata: Vec<Filedata> = WalkDir::new(folder_path)
                .into_iter()
                .filter_map(|dir_entry_result| dir_entry_result.ok())
                .map(|dir_entry| dir_entry.into_path())
                .filter(|filepath| {
                    filepath.to_str().is_some_and(|path_str| {
                        ignore_list
                            .iter()
                            .any(|ignore_list_item| path_str.contains(ignore_list_item))
                    }) && filepath.metadata().is_ok()
                })
                .map(|filepath| {
                    let metadata = filepath.metadata().unwrap();
                    Filedata {
                        path: filepath,
                        size: metadata.size(),
                        modification_time: metadata.mtime(),
                    }
                })
                .collect();
            (folder_id, all_filedata)
        })
        .collect()
}

pub fn get_delete_list(
    old_list: Vec<(uuid::Uuid, Vec<Filedata>)>,
    new_list: Vec<(uuid::Uuid, Vec<Filedata>)>,
) -> Vec<(uuid::Uuid, Vec<Filedata>)> {
    let all_items_to_keep: Vec<PathBuf> = new_list
        .iter()
        .flat_map(|(_folder_id, filedata_list)| filedata_list)
        .map(|filedata| filedata.path.clone())
        .collect();
    old_list
        .into_iter()
        .map(|(folder_id, filedata_list)| {
            (
                folder_id,
                filedata_list
                    .into_iter()
                    .filter(|filedata| !all_items_to_keep.contains(&filedata.path))
                    .collect(),
            )
        })
        .collect()
}
