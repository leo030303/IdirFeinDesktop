use iced::Task;
use rfd::FileDialog;
use uuid::Uuid;

use crate::app::Message;

use super::page::{SyncPage, SyncPageMessage};

pub fn update(state: &mut SyncPage, message: SyncPageMessage) -> Task<Message> {
    match message {
        SyncPageMessage::UpdateIgnoreListEditor(s) => state.ignore_list_editor_text = s,
        SyncPageMessage::AddToIgnoreList => {
            // TODO save to disk
            if !state.ignore_list_editor_text.is_empty() {
                state
                    .ignore_string_list
                    .push(state.ignore_list_editor_text.clone());
                state.ignore_list_editor_text = String::new();
                return Task::done(Message::ShowToast(
                    true,
                    String::from("Close and reopen the app to start using the new ignore list"),
                ));
            }
        }
        SyncPageMessage::DeleteFromIgnoreList(index_of_item_to_delete) => {
            // TODO save to disk
            state.ignore_string_list.remove(index_of_item_to_delete);
            return Task::done(Message::ShowToast(
                true,
                String::from("Close and reopen the app to start using the new ignore list"),
            ));
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
            // TODO save to disk
            if let Some(selected_folder) = selected_folder_option {
                state
                    .folders_to_sync
                    .push((Uuid::new_v4(), selected_folder));
                return Task::done(Message::ShowToast(
                    true,
                    String::from("Close and reopen the app to start using the new sync list"),
                ));
            }
        }
        SyncPageMessage::DeleteFromFolderList(index_of_item_to_delete) => {
            // TODO save to disk
            state.folders_to_sync.remove(index_of_item_to_delete);
            return Task::done(Message::ShowToast(
                true,
                String::from("Close and reopen the app to start using the new sync list"),
            ));
        }
    }
    Task::none()
}
