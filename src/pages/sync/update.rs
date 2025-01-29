use crate::pages::sync::page::IGNORE_LIST_FILE_NAME;
use crate::pages::sync::page::SYNC_LIST_FILE_NAME;
use std::fs;

use iced::Task;
use rfd::FileDialog;
use uuid::Uuid;

use crate::app::Message;
use crate::constants::APP_ID;

use super::page::{SyncPage, SyncPageMessage};

pub fn update(state: &mut SyncPage, message: SyncPageMessage) -> Task<Message> {
    match message {
        SyncPageMessage::UpdateIgnoreListEditor(s) => state.ignore_list_editor_text = s,
        SyncPageMessage::AddToIgnoreList => {
            if !state.ignore_list_editor_text.is_empty() {
                state
                    .ignore_string_list
                    .push(state.ignore_list_editor_text.clone());
                state.ignore_list_editor_text = String::new();

                match fs::write(
                    dirs::config_dir()
                        .unwrap()
                        .join(APP_ID)
                        .join(IGNORE_LIST_FILE_NAME),
                    serde_json::to_string(&state.ignore_string_list).unwrap(),
                ) {
                    Ok(_) => {
                        return Task::done(Message::ShowToast(
                            true,
                            String::from(
                                "Close and reopen the app to start using the new ignore list",
                            ),
                        ));
                    }
                    Err(err) => {
                        return Task::done(Message::ShowToast(
                            false,
                            format!("Error saving ignore list: {err:?}"),
                        ));
                    }
                }
            }
        }
        SyncPageMessage::DeleteFromIgnoreList(index_of_item_to_delete) => {
            state.ignore_string_list.remove(index_of_item_to_delete);
            match fs::write(
                dirs::config_dir()
                    .unwrap()
                    .join(APP_ID)
                    .join(IGNORE_LIST_FILE_NAME),
                serde_json::to_string(&state.ignore_string_list).unwrap(),
            ) {
                Ok(_) => {
                    return Task::done(Message::ShowToast(
                        true,
                        String::from("Close and reopen the app to start using the new ignore list"),
                    ));
                }
                Err(err) => {
                    return Task::done(Message::ShowToast(
                        false,
                        format!("Error saving ignore list: {err:?}"),
                    ));
                }
            }
        }
        SyncPageMessage::PickNewSyncListFolder => {
            return Task::perform(
                async {
                    FileDialog::new()
                        .set_directory("/")
                        .set_can_create_directories(true)
                        .pick_folder()
                },
                |selected_folder| {
                    Message::Sync(SyncPageMessage::SetNewSyncListFolder(selected_folder))
                },
            );
        }
        SyncPageMessage::SetNewSyncListFolder(selected_folder_option) => {
            if let Some(selected_folder) = selected_folder_option {
                state
                    .folders_to_sync
                    .insert(Uuid::new_v4().to_string(), selected_folder);
                match fs::write(
                    dirs::config_dir()
                        .unwrap()
                        .join(APP_ID)
                        .join(SYNC_LIST_FILE_NAME),
                    serde_json::to_string(&state.folders_to_sync).unwrap(),
                ) {
                    Ok(_) => {
                        return Task::done(Message::ShowToast(
                            true,
                            String::from(
                                "Close and reopen the app to start using the new sync list",
                            ),
                        ));
                    }
                    Err(err) => {
                        return Task::done(Message::ShowToast(
                            false,
                            format!("Error saving sync list: {err:?}"),
                        ));
                    }
                }
            }
        }
        SyncPageMessage::DeleteFromFolderList(uuid_of_item_to_delete) => {
            state.folders_to_sync.remove(&uuid_of_item_to_delete);
            state
                .ignore_string_list
                .push(uuid_of_item_to_delete.to_string());
            match fs::write(
                dirs::config_dir()
                    .unwrap()
                    .join(APP_ID)
                    .join(SYNC_LIST_FILE_NAME),
                serde_json::to_string(&state.folders_to_sync).unwrap(),
            ) {
                Ok(_) => {
                    return Task::done(Message::ShowToast(
                        true,
                        String::from("Close and reopen the app to start using the new sync list"),
                    ));
                }
                Err(err) => {
                    return Task::done(Message::ShowToast(
                        false,
                        format!("Error saving sync list: {err:?}"),
                    ));
                }
            }
        }
    }
    Task::none()
}
