
use iced::{widget::{pick_list, progress_bar, rich_text, scrollable, span}, Alignment::Center, Element, Font, Length};

use crate::app::Message;
use iced::widget::{button, column, container, row, text, text_input, Space};

use super::{constants::COUNTRY_CODES_FOR_WIFI_SETUP, page::{SetupProgressBarValue, SetupType, SetupWizard, SetupWizardMessage, SetupWizardStep}};

pub fn main_view(state: &SetupWizard) -> Element<Message> {
    match state.current_step {
        SetupWizardStep::DecideWhetherToSetupServer => decide_whether_to_setup_server_view(state),
        SetupWizardStep::EnterServerUrlAndTotpSecret => {
            enter_server_url_and_totp_secret_view(state)
        }
        SetupWizardStep::ConfirmConnection => confirm_connection_view(state),
        SetupWizardStep::ChooseRemoteFoldersToSync => choose_remote_folders_to_sync_view(state),
        SetupWizardStep::CloseWizard => close_wizard_view(state),
        SetupWizardStep::OptionalSetupRemoteAccess => optional_setup_remote_access_view(state),
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
        SetupWizardStep::SettingUpSdCard => setting_up_sd_card_view(state),
        SetupWizardStep::SdCardSetupCompletePleaseEject => {
            sd_card_setup_complete_please_eject_view(state)
        }
        SetupWizardStep::PlugInServerConfirmConnection => {
            plug_in_server_confirm_connection_view(state)
        }
        SetupWizardStep::DownloadExtraAppData => download_extra_app_data(state) ,
        SetupWizardStep::GetWifiDetails => get_wifi_details(state),
        SetupWizardStep::SetRpiServerPassword => set_rpi_server_password(state),
        SetupWizardStep::CreateServerUsers => create_server_users(state),
    }
}

fn set_rpi_server_password(state: &SetupWizard) -> Element<Message> {
    container(column![
        text("Set Password for the server")
            .width(Length::Fill)
            .center()
            .size(24),
        text("Pick the password for the server. You'll never need this for standard use of IdirFéin, but its important for securing your server, so make sure its sufficiently complex.")
            .width(Length::Fill)
            .center()
            .size(20),
        text_input("Server Password", &state.work_in_progress_server_config.server_password).on_input(|s| Message::SetupWizard(SetupWizardMessage::UpdateServerPassword(s))),
        if state.work_in_progress_server_config.server_password.is_empty() {
            button(text("Continue").width(Length::Fill).center()).width(Length::Fill)
        } else {
            button(text("Continue").width(Length::Fill).center()).width(Length::Fill)
                .on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(SetupWizardStep::CreateServerUsers)))
        }
    ].max_width(800)
    .padding(20)
    .spacing(10)).center_x(Length::Fill)
    .into()
    
}

fn create_server_users(state: &SetupWizard) -> Element<Message> {
    container(column![
        text("Create server users")
            .width(Length::Fill)
            .center()
            .size(24),
        text("Create the user accounts for the server. You can't make more later, and you'll need to tell each of the users what their username and password is for them to be able to connect. All users have access to the same data on the server.")
            .width(Length::Fill)
            .center()
            .size(20),
        row![
            text_input("New user name", &state.new_user_name_input_text).on_input(|s| Message::SetupWizard(SetupWizardMessage::UpdateNewUserNameInputText(s))),
            button(text("Add user").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::AddNewUser))
        ].spacing(10),
        text("Users list")
            .width(Length::Fill)
            .center()
            .size(20),
        scrollable(
            column(state.work_in_progress_server_config.users_list.keys().map(|username| {
                container(
                    row![text(username)
                        .font(Font {
                            weight: iced::font::Weight::Medium,
                            ..Default::default()
                        })
                        .align_y(Center)
                        .width(Length::Fill)
                        .height(Length::Shrink)
                    ].padding(10).width(Length::Fill)
                )
                .style(container::bordered_box)
                .into()
            })).spacing(5)
        ),
        button(text("Continue").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(SetupWizardStep::InsertTargetHardriveIntoThisComputer))).width(Length::Fill),
    ].max_width(800)
    .padding(20)
    .spacing(10)).center_x(Length::Fill)
    .into()
    
}

fn decide_whether_to_setup_server_view(state: &SetupWizard) -> Element<Message> {
    column![
        button(text("Test").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(SetupWizardStep::SettingUpSdCard ))).width(Length::Fill),
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
                if matches!(state.setup_type, SetupType::NoServerSetup) {
                    button(text("Confirm Choice").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(SetupWizardStep::DownloadExtraAppData))).width(Length::Fill)
                        .style(button::success)
                } else {
                    button(text("Use Offline").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::SetSetupType(SetupType::NoServerSetup))).width(Length::Fill)
                }
            ])
        .style(container::bordered_box).padding(10),
            container(column![
                text("Connect to an existing server").width(Length::Fill).center().size(24),
                text("Connect to a server that you or someone else has previously set up. To do this, you will need the server url, your username, and your auth token. Ask your admin for these, and never share them with anyone as they will have full access to your server.").width(Length::Fill).center(),
                Space::with_height(Length::Fill),
                if matches!(state.setup_type, SetupType::ConnectToExistingServerSetup) {
                    button(text("Confirm Choice").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(SetupWizardStep::EnterServerUrlAndTotpSecret))).width(Length::Fill)
                        .style(button::success)
                } else {
                    button(text("Connect to existing server").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::SetSetupType(SetupType::ConnectToExistingServerSetup))).width(Length::Fill)
                }
            ])
        .style(container::bordered_box).padding(10),
            container(column![
                text("Setup a new server").width(Length::Fill).center().size(24),
                text("Setup a new server from scratch. To do this, you will need:\n 1. A Raspberry Pi Zero or similar\n 2. An SD card to use with the device\n 3. A harddrive for data storage\n 4. You must be connected to the internet router you intend to host the server from, and you need to be able to access the router settings\n\nSetup should take around 30 minutes").width(Length::Fill).center(),
                Space::with_height(Length::Fill),
                if matches!(state.setup_type, SetupType::FullServerSetup) {
                    button(text("Confirm Choice").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(SetupWizardStep::GetWifiDetails))).width(Length::Fill)
                        .style(button::success)
                } else {
                    button(text("Setup a new server").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::SetSetupType(SetupType::FullServerSetup))).width(Length::Fill)
                }
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
                SetupWizardStep::DownloadExtraAppData
            )))
        .width(Length::Fill)
    ].max_width(1200)
    .padding(20)
    .spacing(10)).center_x(Length::Fill)
    .into()
}
fn close_wizard_view(state: &SetupWizard) -> Element<Message> {
    container(column![
        text("All Done")
            .width(Length::Fill)
            .center()
            .size(24),
        match state.setup_type {
            SetupType::FullServerSetup => {
                text("The server is connected and your data is being synced, click the button below to continue to the app.")
                    .width(Length::Fill)
                    .center()
                    .size(20)
            },
            SetupType::ConnectToExistingServerSetup => {
                text("The server is connected and your data is being synced, click the button below to continue to the app.")
                    .width(Length::Fill)
                    .center()
                    .size(20)
            },
            SetupType::NoServerSetup => {
                text("You've chosen offline mode, so IdirFéin isn't syncing any data. All other features of the app work as normal. You can still connect to a server in the settings page")
                    .width(Length::Fill)
                    .center()
                    .size(20)
            },
            SetupType::NoneSelectedYet=> {
                text("How did you even make it to this screen without selecting a setup type, strange one")
                    .width(Length::Fill)
                    .center()
                    .size(20)
                
            }
        },
        button(text("Continue").width(Length::Fill).center()).on_press(Message::FinishSetup).width(Length::Fill)
    ].max_width(800)
    .padding(20)
    .spacing(10)).center_x(Length::Fill)
    .into()
}
fn optional_setup_remote_access_view(state: &SetupWizard) -> Element<Message> {
    column![
        text("Setup remote access (optional)")
            .width(Length::Fill)
            .center()
            .size(24),
        text("If you'd like to be able to access the server from anywhere you can set up remote access. If you don't, you'll only be able to sync when you're connected to the same WiFi as the server. If you want to change this later, you'll need to set the server up from scratch.")
            .width(Length::Fill)
            .center()
            .size(20),
        row![
            container(column![
                text("Setup remote access").width(Length::Fill).center().size(24),
                text("Set static local IP for server:").width(Length::Fill).center(),
                text_input("Static IP (x.x.x.x)", &state.work_in_progress_server_config.static_ip).on_input(|s| Message::SetupWizard(SetupWizardMessage::UpdateServerStaticIp(s))),
                rich_text![span("Go to "), span("https://portforward.com/router.htm").underline(true).link(Message::SetupWizard(SetupWizardMessage::LinkClicked("https://portforward.com/router.htm"))), span(format!(" and find your router on the list. This will give you a guide on how to setup port forwarding. You need to set port 443 to go to port {0} on IP {1}", state.work_in_progress_server_config.port, state.work_in_progress_server_config.static_ip))].width(Length::Fill).center(),
                rich_text![span("When you're done that, go to "), span("https://www.duckdns.org/").underline(true).link(Message::SetupWizard(SetupWizardMessage::LinkClicked("https://www.duckdns.org/"))), span(" and make an account. Once thats done, add your desired domain, and copy the domain (the yourdomain part of yourdomain.duckdns.org) and your token into the fields below.")].width(Length::Fill).center(),
                text_input("DuckDNS Domain", &state.work_in_progress_server_config.duckdns_domain).on_input(|s| Message::SetupWizard(SetupWizardMessage::UpdateServerDuckDnsDomain(s))),
                text_input("DuckDNS Token", &state.work_in_progress_server_config.duckdns_token).on_input(|s| Message::SetupWizard(SetupWizardMessage::UpdateServerDuckDnsToken(s))),
                Space::with_height(Length::Fill),
                button(text("Finish remote access setup").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::ConfirmRemoteAccessDetails)).width(Length::Fill)
                    .style(button::success)
            ].spacing(5))
            .style(container::bordered_box).padding(10),
            container(column![
                text("Use on local network only").width(Length::Fill).center().size(24),
                text("You won't be able to access the server when you aren't connected to the same WiFi as the server. You can't change this later without setting up the server from scratch.").width(Length::Fill).center(),
                Space::with_height(Length::Fill),
                    button(text("Use on local network only").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(SetupWizardStep::SetRpiServerPassword))).width(Length::Fill)
            ])
        .style(container::bordered_box).padding(10),
        ].spacing(20)
    ]
    .padding(20)
    .spacing(10)
    .into()
}
fn insert_target_hardrive_into_this_computer_view(state: &SetupWizard) -> Element<Message> {
    container(column![
        text("Insert the hardrive you want to use for storage")
            .width(Length::Fill)
            .center()
            .size(24),
        text("Plug the harddrive you want to use into this computer. Once you have, press List Disks, and then select it from the list. If you're unsure which disk it is, try removing it, clicking List Disks, taking note of all of them, and then plugging the device back in and refreshing the list to see which one is new. On your server, its best to not have multiple external harddrives plugged in, as the target harddrive is identified by its size, which could overlap with another device.")
            .width(Length::Fill)
            .center()
            .size(16),
        button(text("List Disks").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::GetListOfDisks)).width(Length::Fill),
        scrollable(column(state.list_of_disks.iter().map(|disk_info| text(format!("Name: {} Total Size: {} Is Removable: {}\n\n\n", disk_info.name, bytesize::to_string(disk_info.total_space, false), disk_info.is_removable)).into())))
    ].max_width(800)
    .padding(20)
    .spacing(10)).center_x(Length::Fill)
    .into()
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
fn setting_up_sd_card_view(state: &SetupWizard) -> Element<Message> {
    container(column![
        text("Setting up SD card")
            .width(Length::Fill)
            .center()
            .size(24),
        text("Do not close the app. Do not turn off your device. Do not remove the SD card. This will take a few minutes. You will be asked to authorise the app to write to the SD card.")
            .width(Length::Fill)
            .center()
            .size(16),
            match &state.progress_bar_value {
    SetupProgressBarValue::WaitingToStart => {
        column![button(text("Start").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::DownloadImg)).width(Length::Fill)]
        
    },
    SetupProgressBarValue::DownloadingFile(progress) => {
            column![row![
                text("Downloading files").height(Length::Fixed(20.0)),
                Space::with_width(Length::Fixed(20.0)),
                container(
                    progress_bar(0.0..=100.0, *progress)
                        .width(Length::Fill)
                        .height(Length::Fixed(10.0))
                        .style(progress_bar::primary)
                )
                .height(Length::Fixed(20.0))
                .align_y(Center),
                Space::with_width(Length::Fixed(10.0)),
                text(format!("{:.2}%", progress))
                    .height(Length::Fixed(20.0))
                    .width(Length::Fixed(50.0)),
                Space::with_width(Length::Fixed(10.0)),
            ]
            .padding(10)]
        
    },
    SetupProgressBarValue::ExtractingImg(progress) => {
            column![row![
                text("Extracting IMG File").height(Length::Fixed(20.0)),
                Space::with_width(Length::Fixed(20.0)),
                container(
                    progress_bar(0.0..=100.0, *progress)
                        .width(Length::Fill)
                        .height(Length::Fixed(10.0))
                        .style(progress_bar::primary)
                )
                .height(Length::Fixed(20.0))
                .align_y(Center),
                Space::with_width(Length::Fixed(10.0)),
                text(format!("{:.2}%", progress))
                    .height(Length::Fixed(20.0))
                    .width(Length::Fixed(50.0)),
                Space::with_width(Length::Fixed(10.0)),
            ]
            .padding(10)]
        
    },
    SetupProgressBarValue::FlashingSdCard => {
            column![
                text("Flashing SD Card")
                    .width(Length::Fill)
                    .center()
                    .size(20),
            ]
        
    },
    SetupProgressBarValue::WritingConfig => {
            column![
                text("Writing Config")
                    .width(Length::Fill)
                    .center()
                    .size(20)
            ]
        
    },
    SetupProgressBarValue::Finished => {column![text("All Done").style(text::success)]},
}
    ].max_width(1200)
    .padding(20)
    .spacing(10)).center_x(Length::Fill)
    .into()
}
fn sd_card_setup_complete_please_eject_view(_state: &SetupWizard) -> Element<Message> {
    "sd_card_setup_complete_please_eject_view".into()
}
fn plug_in_server_confirm_connection_view(_state: &SetupWizard) -> Element<Message> {
    "plug_in_server_confirm_connection_view".into() // Should direct to download extra app data
}

fn get_wifi_details(state: &SetupWizard) -> Element<Message> {
    container(column![
        text("Enter WiFi Details")
            .width(Length::Fill)
            .center()
            .size(24),
        text("Before you start, log onto your router and ensure there is a separate 2.4Ghz WiFi channel, and enter the SSID for that channel. The Raspberry Pi Zero only works with 2.4Ghz, not 5Ghz. If you don't set this up, the server will be unable to connect to WiFi.")
            .width(Length::Fill).style(text::danger)
            .center()
            .size(20),
        text_input("WiFi SSID", &state.work_in_progress_server_config.wifi_ssid).on_input(|s| Message::SetupWizard(SetupWizardMessage::UpdateWifiSsid(s))),
        text_input("WiFi Password", &state.work_in_progress_server_config.wifi_password).on_input(|s| Message::SetupWizard(SetupWizardMessage::UpdateWifiPassword(s))),
        column![
            text("Select your country"),
            pick_list(
                COUNTRY_CODES_FOR_WIFI_SETUP
                    .into_iter()
                    .map(|(country_name, _country_code)| country_name)
                    .collect::<Vec<&'static str>>(),
                state.work_in_progress_server_config.country_name_option,
                |country_name| Message::SetupWizard(SetupWizardMessage::SetWifiCountryName(country_name)),
            )
        ]
        .spacing(10),
        button(text("Continue").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::SetWifiDetails)).width(Length::Fill)
    ].max_width(800)
    .padding(20)
    .spacing(10)).center_x(Length::Fill)
    .into()
    
}

fn download_extra_app_data(state: &SetupWizard) -> Element<Message> {
    container(column![
        text("Downloading extra app data")
            .width(Length::Fill)
            .center()
            .size(24),
        text("Do not close the app. Do not turn off your device. This will take a few minutes. You can see the files being downloaded here: https://github.com/leo030303/idirfein-resources")
            .width(Length::Fill)
            .center()
            .size(16),
            match &state.progress_bar_value {
    SetupProgressBarValue::WaitingToStart => {
        column![button(text("Start").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::DownloadExtraData )).width(Length::Fill)]
        
    },
    SetupProgressBarValue::DownloadingFile(progress) => {
            column![row![
                text("Downloading").height(Length::Fixed(20.0)),
                Space::with_width(Length::Fixed(20.0)),
                container(
                    progress_bar(0.0..=100.0, *progress)
                        .width(Length::Fill)
                        .height(Length::Fixed(10.0))
                        .style(progress_bar::primary)
                )
                .height(Length::Fixed(20.0))
                .align_y(Center),
                Space::with_width(Length::Fixed(10.0)),
                text(format!("{:.2}%", progress))
                    .height(Length::Fixed(20.0))
                    .width(Length::Fixed(50.0)),
                Space::with_width(Length::Fixed(10.0)),
            ]
            .padding(10)]
        
    },
    _ => {column![text("All Done").style(text::success), 
                    button(text("Continue").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(SetupWizardStep::CloseWizard))).width(Length::Fill)
    ]},
}
    ].max_width(1200)
    .padding(20)
    .spacing(10)).center_x(Length::Fill)
    .into()
    
}
