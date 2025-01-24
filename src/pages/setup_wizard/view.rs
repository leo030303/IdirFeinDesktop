use iced::{widget::scrollable, Element, Length};

use crate::app::Message;
use iced::widget::{button, column, container, row, text, text_input, Space};

use super::page::{SetupWizard, SetupWizardMessage, SetupWizardStep};

pub fn main_view(state: &SetupWizard) -> Element<Message> {
    match state.current_step {
        SetupWizardStep::DecideWhetherToSetupServer => decide_whether_to_setup_server_view(state),
        SetupWizardStep::EnterServerUrlAndTotpSecret => {
            enter_server_url_and_totp_secret_view(state)
        }
        SetupWizardStep::ConfirmConnection => confirm_connection_view(state),
        SetupWizardStep::ChooseRemoteFoldersToSync => choose_remote_folders_to_sync_view(state),
        SetupWizardStep::ExistingServerCloseWizard => existing_server_close_wizard_view(state),
        SetupWizardStep::OfflineSetupCloseWizard => offline_setup_close_wizard_view(state),
        SetupWizardStep::OptionalSetupWlanChoice => optional_setup_wlan_choice_view(state),
        SetupWizardStep::SetupPortForwarding => setup_port_forwarding_view(state),
        SetupWizardStep::SetupDuckDns => setup_duck_dns_view(state),
        SetupWizardStep::InsertTargetHardriveIntoThisComputer => {
            insert_target_hardrive_into_this_computer_view(state)
        }
        SetupWizardStep::ConfirmTargetHarddrive => confirm_target_harddrive_view(state),
        SetupWizardStep::RemoveTargetHarddrive => remove_target_harddrive_view(state),
        SetupWizardStep::InsertSdCardIntoThisComputer => {
            insert_sd_card_into_this_computer_view(state)
        }
        SetupWizardStep::PickDeviceWhichIsTargetSdCard => {
            pick_device_which_is_target_sd_card_view(state)
        }
        SetupWizardStep::ConfirmSdCardChoice => confirm_sd_card_choice_view(state),
        SetupWizardStep::FlashingSdCard => flashing_sd_card_view(state),
        SetupWizardStep::SdCardSetupCompletePleaseEject => {
            sd_card_setup_complete_please_eject_view(state)
        }
        SetupWizardStep::PlugInServerConfirmConnection => {
            plug_in_server_confirm_connection_view(state)
        }
        SetupWizardStep::FullSetupCloseWizard => full_setup_close_wizard_view(state),
    }
}

fn decide_whether_to_setup_server_view(_state: &SetupWizard) -> Element<Message> {
    column![
        text("Welcome to IdirFéin!")
            .width(Length::Fill)
            .center()
            .size(24),
        text("Please choose from one of the options below")
            .width(Length::Fill)
            .center()
            .size(20),
        row![
            container(column![
                text("Use Offline").width(Length::Fill).center().size(24),
                text("Use IdirFéin straight away with no setup required, but with no data syncing between devices. All other features will work as normal. You can always enter your details in settings later to connect to an existing server, but you will not be able to setup a new server without resetting your settings").width(Length::Fill).center(),
                Space::with_height(Length::Fill),
                button(text("Use Offline").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(SetupWizardStep::OfflineSetupCloseWizard))).width(Length::Fill)
            ])
        .style(container::bordered_box).padding(10),
            container(column![
                text("Connect to an existing server").width(Length::Fill).center().size(24),
                text("Connect to a server that you or someone else has previously set up. To do this, you will need the server url, your username, and your auth token. Ask your admin for these, and never share them with anyone as they will have full access to your server.").width(Length::Fill).center(),
                Space::with_height(Length::Fill),
                button(text("Connect to existing server").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(SetupWizardStep::EnterServerUrlAndTotpSecret))).width(Length::Fill)
            ])
        .style(container::bordered_box).padding(10),
            container(column![
                text("Setup a new server").width(Length::Fill).center().size(24),
                text("Setup a new server from scratch. To do this, you will need:\n 1. A Raspberry Pi Zero or similar\n 2. An SD card to use with the device\n 3. A harddrive for data storage\n 4. You must be connected to the internet router you intend to host the server from, and you need to be able to access the router settings\n\nSetup should take around 30 minutes").width(Length::Fill).center(),
                Space::with_height(Length::Fill),
                button(text("Setup a new server").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(SetupWizardStep::OptionalSetupWlanChoice))).width(Length::Fill)
            ])
        .style(container::bordered_box).padding(10),
        ].spacing(20)
    ]
    .padding(20)
    .spacing(10)
    .into()
}

fn enter_server_url_and_totp_secret_view(state: &SetupWizard) -> Element<Message> {
    container(column![
        text("Enter Server Details")
            .width(Length::Fill)
            .center()
            .size(24),
        text("Enter your server url, username, and authentication token below. Ask your admin if you're missing any of these, and if you are the admin, you can find these in the config.json on your servers SD card.")
            .width(Length::Fill)
            .center()
            .size(20),
        text_input("Server url", &state.server_url_input_text).on_input(|s| Message::SetupWizard(SetupWizardMessage::UpdateServerUrlInputText(s))),
        text_input("Username", &state.client_username_input_text).on_input(|s| Message::SetupWizard(SetupWizardMessage::UpdateClientUsernameInputText(s))),
        text_input("Client secret", &state.client_secret_input_text).on_input(|s| Message::SetupWizard(SetupWizardMessage::UpdateClientSecretInputText(s))),
        button(text("Continue").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::SetExistingServerDetails)).width(Length::Fill)
    ].max_width(800)
    .padding(20)
    .spacing(10)).center_x(Length::Fill)
    .into()
}

fn confirm_connection_view(state: &SetupWizard) -> Element<Message> {
    container(
        if state.connection_has_been_attempted {
            if state.remote_folders_info.is_some(){
                column![
                    text("Connection Successful!")
                        .width(Length::Fill)
                        .center()
                        .size(24)
                        .style(text::success),
                    button(text("Continue").width(Length::Fill).center())
                        .on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(
                            SetupWizardStep::ChooseRemoteFoldersToSync
                        )))
                        .width(Length::Fill)
                ]
            } else {
                column![
                    text("Connection Failed")
                        .width(Length::Fill)
                        .center()
                        .size(24).style(text::danger),
        text("Possible causes: \nYour details may be wrong\nYour internet connection is poor\nThe server is off or disconnected from the internet")
            .width(Length::Fill)
            .center()
            .size(20),
                    button(text("Back to details page").width(Length::Fill).center())
                        .on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(
                            SetupWizardStep::EnterServerUrlAndTotpSecret
                        )))
                        .width(Length::Fill),
                    button(text("Retry").width(Length::Fill).center())
                        .on_press(Message::SetupWizard(SetupWizardMessage::TestConnection))
                        .width(Length::Fill)
                ]
            }
        } else {
            column![
                text("Testing connection")
                    .width(Length::Fill)
                    .center()
                    .size(24),
        text("This might take a minute. Make sure you're connected to the entire and the server is on.")
            .width(Length::Fill)
            .center()
            .size(20),
            ]
        }
        .max_width(800)
        .padding(20)
        .spacing(10),
    )
    .center_x(Length::Fill)
    .into()
}

fn choose_remote_folders_to_sync_view(state: &SetupWizard) -> Element<Message> {
    container(column![
        text("Select which remote folders to sync here")
            .width(Length::Fill)
            .center()
            .size(24),
        text("You're successfully connected to the server. Choose from the list of remote folders to decide which ones you want downloaded to this device and which you want to ignore")
            .width(Length::Fill)
            .center()
            .size(20),
        row![
            column![
            text("Remote Folders").size(20).width(Length::Fill).center(),
            scrollable(column(state.remote_folders_info.as_ref().expect("Shouldn't have been able to get to that page if this is None, please report this as a bug").iter().filter(|(folder_id, _)| !state.work_in_progress_client_config.sync_config.ignored_remote_folder_ids.contains(folder_id)).map(|(folder_id, list_of_paths)| row![
                button(text(format!("ID: {folder_id} Files Count: {}", list_of_paths.len()))).on_press(Message::SetupWizard(SetupWizardMessage::SetSelectedRemoteFolder(Some(folder_id.clone())))).width(Length::Fill),
                button("Ignore").style(button::danger).on_press(Message::SetupWizard(SetupWizardMessage::IgnoreFolderId(folder_id.clone())))
            ].spacing(5).into())).spacing(5)).width(Length::Fill).height(Length::FillPortion(1)),
            Space::with_height(Length::Fixed(10.0)),
            text("Ignored folders").size(20).width(Length::Fill).center(),
            scrollable(column(state.work_in_progress_client_config.sync_config.ignored_remote_folder_ids.iter().enumerate().map(|(index, folder_id)| row![
                button(text(format!("Ignored: {folder_id}"))).on_press(Message::SetupWizard(SetupWizardMessage::SetSelectedRemoteFolder(Some(folder_id.clone())))).width(Length::Fill),
                button("Unignore").on_press(Message::SetupWizard(SetupWizardMessage::UnignoreFolderId(index)))
            ].spacing(5).into())).spacing(5)).width(Length::Fill).height(Length::FillPortion(1)),
                
            ],
            if let Some(selected_remote_folder_id) = state.selected_remote_folder.as_ref() {
            column![
                scrollable(
                    column![
                        text(format!("ID: {selected_remote_folder_id}")).size(20).width(Length::Fill).center(),
                        text(
                            state.remote_folders_info.as_ref().expect("Shouldn't have been able to get to that page if this is None, please report this as a bug")
                            .get(selected_remote_folder_id)
                            .expect("This should be an item in the array, please report this as a bug")
                            .join("\n")
                            )
                    ]
                )
            ].width(Length::Fill)
            } else {
                column![]
            }
        ].spacing(10),
        button(text("Continue").width(Length::Fill).center()).on_press(Message::SetupWizard(
            SetupWizardMessage::GoToStep(
                SetupWizardStep::ExistingServerCloseWizard
            )))
        .width(Length::Fill)
    ].max_width(1200)
    .padding(20)
    .spacing(10)).center_x(Length::Fill)
    .into()
}
fn existing_server_close_wizard_view(_state: &SetupWizard) -> Element<Message> {
    container(column![
        text("All Done")
            .width(Length::Fill)
            .center()
            .size(24),
        text("The server is connected and your data is being synced, click the button below to continue to the app.")
            .width(Length::Fill)
            .center()
            .size(20),
        button(text("Continue").width(Length::Fill).center()).on_press(Message::FinishSetup).width(Length::Fill)
    ].max_width(800)
    .padding(20)
    .spacing(10)).center_x(Length::Fill)
    .into()
}
fn offline_setup_close_wizard_view(_state: &SetupWizard) -> Element<Message> {
    container(column![
        text("All Done")
            .width(Length::Fill)
            .center()
            .size(24),
        text("You've chosen offline mode, so IdirFéin isn't syncing any data. All other features of the app work as normal. You can still connect to a server in the settings page")
            .width(Length::Fill)
            .center()
            .size(20),
        button(text("Continue").width(Length::Fill).center()).on_press(Message::FinishSetup).width(Length::Fill)
    ].max_width(800)
    .padding(20)
    .spacing(10)).center_x(Length::Fill)
    .into()
}
fn optional_setup_wlan_choice_view(_state: &SetupWizard) -> Element<Message> {
    "optional_setup_wlan_choice_view".into()
}
fn setup_port_forwarding_view(_state: &SetupWizard) -> Element<Message> {
    "setup_port_forwarding_view".into()
}
fn setup_duck_dns_view(_state: &SetupWizard) -> Element<Message> {
    "setup_duck_dns_view".into()
}
fn insert_target_hardrive_into_this_computer_view(_state: &SetupWizard) -> Element<Message> {
    "insert_target_hardrive_into_this_computer_view".into()
}
fn confirm_target_harddrive_view(_state: &SetupWizard) -> Element<Message> {
    "confirm_target_harddrive_view".into()
}
fn remove_target_harddrive_view(_state: &SetupWizard) -> Element<Message> {
    "remove_target_harddrive_view".into()
}
fn insert_sd_card_into_this_computer_view(_state: &SetupWizard) -> Element<Message> {
    "insert_sd_card_into_this_computer_view".into()
}
fn pick_device_which_is_target_sd_card_view(_state: &SetupWizard) -> Element<Message> {
    "pick_device_which_is_target_sd_card_view".into()
}
fn confirm_sd_card_choice_view(_state: &SetupWizard) -> Element<Message> {
    "confirm_sd_card_choice_view".into()
}
fn flashing_sd_card_view(_state: &SetupWizard) -> Element<Message> {
    "flashing_sd_card_view".into()
}
fn sd_card_setup_complete_please_eject_view(_state: &SetupWizard) -> Element<Message> {
    "sd_card_setup_complete_please_eject_view".into()
}
fn plug_in_server_confirm_connection_view(_state: &SetupWizard) -> Element<Message> {
    "plug_in_server_confirm_connection_view".into()
}
fn full_setup_close_wizard_view(_state: &SetupWizard) -> Element<Message> {
    "full_setup_close_wizard_view".into()
}
