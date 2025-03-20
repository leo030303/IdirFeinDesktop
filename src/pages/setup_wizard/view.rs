
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use fluent_templates::Loader;
use iced::widget::image::FilterMethod;
use iced::{widget::{image::Handle, progress_bar, rich_text, scrollable, span, Image}, Alignment::Center, Element, Font, Length};

use crate::app::Message;
use crate::LOCALES;
use iced::widget::{button, column, container, row, svg, text, text_input, Space, Svg, Tooltip};

use super::page::{SetupProgressBarValue, SetupType, SetupWizard, SetupWizardMessage, SetupWizardStep};

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
        SetupWizardStep::DownloadExtraAppData => download_extra_app_data(state) ,
        SetupWizardStep::CreateServerUsers => create_server_users(state),
        SetupWizardStep::ListUsersChooseYours => list_users_choose_yours(state),
        SetupWizardStep::ImagerSetupSteps => imager_setup_steps(state) ,
        SetupWizardStep::SshConnectionStepsRunScript => ssh_connection_steps_run_script(state),
    }
}

fn ssh_connection_steps_run_script(_state: &SetupWizard) -> Element<Message> {
    container(column![
        text("Finish Server Setup")
            .width(Length::Fill)
            .center()
            .size(24),
        container(scrollable(
            column![
                text("1. Make sure the Raspberry Pi is still powered on and the ethernet cable is still plugged into the router. Also ensure your computer is connected to the same WiFi.").width(Length::Fill),
                text("2. Open the terminal app on your computer. If you can't find it search for terminal").width(Length::Fill),
                text("3. Enter \"ssh username@address\", replacing username with the username you had set (the default is the same username as the user on the computer you set it up on), and address with the address you had set (the default address is raspberrypi.station)").width(Length::Fill),
                text("4. When prompted, enter \"yes\" to connect to the server, and enter the password you set for the Raspberry Pi (the default is the same password as the computer you set it up on)").width(Length::Fill),
                text("5. Type \"nano script.sh\" and press enter. Then click the button below to copy the setup script").width(Length::Fill),
                button("Click this button to copy the setup script").on_press(Message::SetupWizard(SetupWizardMessage::CopySetupScript)),
                text("6. Use Ctrl+V to paste the script into the editor open in the terminal. Then press Ctrl+X and then y and then enter to exit the editor.").width(Length::Fill),
                text("7. Type \"chmod +x script.sh\" and hit enter. Then type \"./script.sh\" and press enter. This is going to take a while. Wait until the terminal says \"All Done\", then you can close the terminal app.").width(Length::Fill),
                text("8. Unplug the server, then plug it back in and wait around 3 minutes before continuing, ensuring the hardrive and ethernet cables are inserted. Then press the Continue button to finish setup").width(Length::Fill),
                button(text("Continue").width(Length::Fill).center()).width(Length::Fill)
                    .on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(SetupWizardStep::ListUsersChooseYours)))
            ].spacing(10)
        )).style(container::bordered_box).padding(20),
    ]
    .padding(20)
    .spacing(10)).center_x(Length::Fill)
    .into()
    
}

fn imager_setup_steps(_state: &SetupWizard) -> Element<Message> {
    container(column![
        text("Setup server")
            .width(Length::Fill)
            .center()
            .size(24),
        container(scrollable(
            column![
                text("There are a few steps to take:")
                    .width(Length::Fill)
                    .center()
                    .size(20),
                row![
                    rich_text![span("1. Install Raspberry Pi Imager from Flathub here, just like you installed this app from Flathub "), span("https://flathub.org/apps/org.raspberrypi.rpi-imager").underline(true).link(Message::SetupWizard(SetupWizardMessage::LinkClicked("https://flathub.org/apps/org.raspberrypi.rpi-imager")))].width(Length::Fill),
                    Space::with_width(Length::Fill)
                ],
                row![
                    text("2. Once installed, ensure you are connected to the internet, then insert your SD card you want to flash, and for \"Choose Device\", pick Raspberry Pi Zero 2 W").width(Length::Fill).align_y(Center).height(Length::Fixed(130.0)),
                    Image::new(Handle::from_bytes(include_bytes!("../../../icons/setup_page_images/rpi_zero.png").to_vec()))
                        .content_fit(iced::ContentFit::ScaleDown)
                        .filter_method(FilterMethod::Nearest).height(Length::Fixed(130.0)).width(Length::Fill),
                ],
                row![
                    text("3. Next, in \"Choose OS\", scroll down and choose \"Raspberry Pi OS (other)\"").width(Length::Fill).align_y(Center).height(Length::Fixed(130.0)),
                    Image::new(Handle::from_bytes(include_bytes!("../../../icons/setup_page_images/other_os.png").to_vec()))
                        .content_fit(iced::ContentFit::ScaleDown)
                        .filter_method(FilterMethod::Nearest).height(Length::Fixed(130.0)).width(Length::Fill),
                ],
                row![
                    text("4. In the resulting menu, choose \"Raspberry Pi OS Lite (64-bit)\"").width(Length::Fill).align_y(Center).height(Length::Fixed(130.0)),
                    Image::new(Handle::from_bytes(include_bytes!("../../../icons/setup_page_images/raspberry_pi_os.png").to_vec()))
                        .content_fit(iced::ContentFit::ScaleDown)
                        .filter_method(FilterMethod::Nearest).height(Length::Fixed(130.0)).width(Length::Fill),
                ],
                row![
                    text("5. Next in \"Choose Storage\", select the SD card you want to use. Make sure its inserted into your device. Be absolutely certain you have picked the correct one as it will be wiped.").width(Length::Fill),
                    Space::with_width(Length::Fill)
                ],
                row![
                    text("6. Click \"Next\"").width(Length::Fill).align_y(Center).height(Length::Fixed(130.0)),
                    Image::new(Handle::from_bytes(include_bytes!("../../../icons/setup_page_images/next.png").to_vec()))
                        .content_fit(iced::ContentFit::ScaleDown)
                        .filter_method(FilterMethod::Nearest).height(Length::Fixed(130.0)).width(Length::Fill),
                ],
                row![
                    text("7. In the popup, select \"Edit Settings\"").width(Length::Fill).align_y(Center).height(Length::Fixed(130.0)),
                    Image::new(Handle::from_bytes(include_bytes!("../../../icons/setup_page_images/os_customisation.png").to_vec()))
                        .content_fit(iced::ContentFit::ScaleDown)
                        .filter_method(FilterMethod::Nearest).height(Length::Fixed(130.0)).width(Length::Fill),
                ],
                row![
                    text("8. Select \"Set username and password\" and deselect the rest of the checkboxes. Set the username and password to something memorable.").width(Length::Fill).align_y(Center).height(Length::Fixed(300.0)),
                    Image::new(Handle::from_bytes(include_bytes!("../../../icons/setup_page_images/general_custom.png").to_vec()))
                        .content_fit(iced::ContentFit::ScaleDown)
                        .filter_method(FilterMethod::Nearest).height(Length::Fixed(300.0)).width(Length::Fill),
                ],
                row![
                    text("9. Select \"Services\" from the top navigation bar, and select \"Enable SSH\" and pick \"Use password authentication\". Then press \"Save\", and on the next menu press \"Yes\"").width(Length::Fill).align_y(Center).height(Length::Fixed(300.0)),
                    Image::new(Handle::from_bytes(include_bytes!("../../../icons/setup_page_images/services_custom.png").to_vec()))
                        .content_fit(iced::ContentFit::ScaleDown)
                        .filter_method(FilterMethod::Nearest).height(Length::Fixed(300.0)).width(Length::Fill),
                ],
                row![
                    text("10. Wait for the install to complete, then remove the SD card once you are told to.").width(Length::Fill),
                    Space::with_width(Length::Fill)
                ],
                row![
                    text("11. Assemble the ethernet hat, insert the SD card into the Raspberry Pi, plug in your harddrive to the hat and your ethernet cable, then plug the other end of the ethernet cable into your router. Plug in the power cable into your Raspberry Pi. The light on the hat should glow red, and on the Raspberry Pi there should be a red light and an intermittently flashing green light. Leave the device plugged in and press the Continue button below.").width(Length::Fill),
                    Space::with_width(Length::Fill)
                ],
                button(text("When finished setup: Continue").width(Length::Fill).center()).width(Length::Fill)
                    .on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(SetupWizardStep::OptionalSetupRemoteAccess)))
            ].spacing(10)
        )).style(container::bordered_box).padding(20),
    ]
    .padding(20)
    .spacing(10)).center_x(Length::Fill)
    .into()
    
}
fn list_users_choose_yours(state: &SetupWizard) -> Element<Message> {
    container(column![
        text("Select your user")
            .width(Length::Fill)
            .center()
            .size(24),
        text("Select the user for this device from the list of users. Also copy the username and authentication token for each user as you won't be able to access them later. Ensure to use secure communication methods to send the authentication details to each user.")
            .width(Length::Fill)
            .center()
            .size(20),
        container(column(state.work_in_progress_server_config.users_list.iter().map(|(username, auth_token)| 
            row![
                text(format!("Username: {username}")).width(Length::Fill), 
                text(format!("Auth Token: {}", BASE64_STANDARD.encode(auth_token))).width(Length::Fill), 
                Tooltip::new(
                    button(
                        Svg::new(svg::Handle::from_memory(include_bytes!(
                            "../../../icons/copy.svg"
                        )))
                        .height(Length::Fixed(20.0))
                    )
                    .on_press(Message::CopyValueToClipboard(format!("Username: {} Auth Token: {}", username.clone(), BASE64_STANDARD.encode(auth_token)))
                    ).width(Length::Fixed(50.0)),
                    text(LOCALES.lookup(&state.locale, "copy")),
                    iced::widget::tooltip::Position::Bottom,
                ),
                button(if state.work_in_progress_client_config.sync_config.client_credentials.as_ref().is_some_and(|credentials| credentials.client_id == *username){"Selected"} else {"Select"}).style(if state.work_in_progress_client_config.sync_config.client_credentials.as_ref().is_some_and(|credentials| credentials.client_id == *username) {button::secondary} else {button::primary}).on_press(Message::SetupWizard(SetupWizardMessage::SetUserForThisDevice { username: username.clone(), auth_token: BASE64_STANDARD.encode(auth_token) }))
        
        ].spacing(20).padding(10).into())))
        .style(container::bordered_box).padding(10),
        if state.work_in_progress_client_config.sync_config.client_credentials.is_none() {
            button(text("Continue").width(Length::Fill).center()).width(Length::Fill)
        } else {
            button(text("Continue").width(Length::Fill).center()).width(Length::Fill)
                .on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(SetupWizardStep::ConfirmConnection)))
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
        if state.work_in_progress_server_config.users_list.is_empty() {
            button(text("Continue").width(Length::Fill).center()).width(Length::Fill)
        } else {
            button(text("Continue").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(SetupWizardStep::DownloadExtraAppData))).width(Length::Fill)
        }
    ].max_width(800)
    .padding(20)
    .spacing(10)).center_x(Length::Fill)
    .into()
    
}

fn decide_whether_to_setup_server_view(state: &SetupWizard) -> Element<Message> {
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
                rich_text![span("Setup a new server from scratch. To do this, you will need:\n 1. A Raspberry Pi Zero or similar\n 2. An SD card to use with the device\n 3. A harddrive for data storage\n 4. You must be connected to the internet router you intend to host the server from, which you are able to connect an ethernet cable to, and you need to be able to access the router settings\n 5. An ethernet hat such as this one "), span("https://thepihut.com/products/ethernet-and-usb-hub-hat-for-raspberry-pi").underline(true).link(Message::SetupWizard(SetupWizardMessage::LinkClicked("https://thepihut.com/products/ethernet-and-usb-hub-hat-for-raspberry-pi"))), span("\n 6. An ethernet cable\n\nSetup should take around 30 minutes")].width(Length::Fill).center(),
                Space::with_height(Length::Fill),
                if matches!(state.setup_type, SetupType::FullServerSetup) {
                    button(text("Confirm Choice").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(SetupWizardStep::ImagerSetupSteps))).width(Length::Fill)
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
                    if matches!(state.setup_type, SetupType::FullServerSetup) {SetupWizardStep::CloseWizard} else {SetupWizardStep::ChooseRemoteFoldersToSync}
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
                    if matches!(state.setup_type, SetupType::FullServerSetup) {column![]} else {column![
                        button(text("Back to details page").width(Length::Fill).center())
                            .on_press(Message::SetupWizard(SetupWizardMessage::GoToStep(
                                SetupWizardStep::EnterServerUrlAndTotpSecret
                            )))
                            .width(Length::Fill),
                    ]},
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
                text("This might take a minute. Make sure you're connected to the internet and the server is on. If its a local only server, you must be connected to the same WiFi as the server")
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
        column![
            container(
                column![
                text("Set static local IP for server:").width(Length::Fill).center(),
                text_input("Static IP (x.x.x.x)", &state.work_in_progress_server_config.static_ip).on_input(|s| Message::SetupWizard(SetupWizardMessage::UpdateServerStaticIp(s))),
                text_input("Router Gateway IP (x.x.x.x)", &state.work_in_progress_server_config.gateway_ip).on_input(|s| Message::SetupWizard(SetupWizardMessage::UpdateServerGatewayIp(s))),
                ]
            ).style(container::bordered_box).padding(10),
            row![
            container(column![
                text("Setup remote access").width(Length::Fill).center().size(24),
                rich_text![span("Go to "), span("https://portforward.com/router.htm").underline(true).link(Message::SetupWizard(SetupWizardMessage::LinkClicked("https://portforward.com/router.htm"))), span(format!(" and find your router on the list. This will give you a guide on how to setup port forwarding. You need to set port 443 to go to port 8000 on IP {0}", state.work_in_progress_server_config.static_ip))].width(Length::Fill).center(),
                rich_text![span("When you're done that, go to "), span("https://www.duckdns.org/").underline(true).link(Message::SetupWizard(SetupWizardMessage::LinkClicked("https://www.duckdns.org/"))), span(" and make an account. Once thats done, add your desired domain, and copy the domain (the yourdomain part of yourdomain.duckdns.org) and your token into the fields below.")].width(Length::Fill).center(),
                text_input("DuckDNS Domain", &state.work_in_progress_server_config.duckdns_domain).on_input(|s| Message::SetupWizard(SetupWizardMessage::UpdateServerDuckDnsDomain(s))),
                text_input("DuckDNS Token", &state.work_in_progress_server_config.duckdns_token).on_input(|s| Message::SetupWizard(SetupWizardMessage::UpdateServerDuckDnsToken(s))),
                text("Lastly, enter an email address in order to get the certs needed to access the server securely").width(Length::Fill).center(),
                text_input("Email address", &state.work_in_progress_server_config.certbot_email).on_input(|s| Message::SetupWizard(SetupWizardMessage::UpdateServerCertbotEmail(s))),
                Space::with_height(Length::Fill),
                button(text("Finish remote access setup").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::ConfirmRemoteAccessDetails)).width(Length::Fill)
                    .style(button::success)
            ].spacing(5))
            .style(container::bordered_box).padding(10),
            container(column![
                text("Use on local network only").width(Length::Fill).center().size(24),
                text("You won't be able to access the server when you aren't connected to the same WiFi as the server. You can't change this later without setting up the server from scratch.").width(Length::Fill).center(),
                Space::with_height(Length::Fill),
                    button(text("Use on local network only").width(Length::Fill).center()).on_press(Message::SetupWizard(SetupWizardMessage::ConfirmLocalAccessDetails)).width(Length::Fill)
            ])
        .style(container::bordered_box).padding(10),
        ].spacing(20)].spacing(20)
    ]
    .padding(20)
    .spacing(10)
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
    _ => {column![
            text("All Done").style(text::success), 
            button(text("Continue").width(Length::Fill).center()).on_press(Message::SetupWizard(
                if matches!(state.setup_type, SetupType::FullServerSetup) {SetupWizardMessage::GoToStep(
             SetupWizardStep::SshConnectionStepsRunScript)} else {SetupWizardMessage::GoToStep(
             SetupWizardStep::CloseWizard   
            )}
        )).width(Length::Fill)
    ]},
}
    ].max_width(1200)
    .padding(20)
    .spacing(10)).center_x(Length::Fill)
    .into()
    
}
