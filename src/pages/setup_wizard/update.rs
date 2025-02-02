use std::{collections::HashMap, time::Duration};

use base64::Engine;
use iced::Task;
use url::Url;

use crate::constants::APP_ID;
use crate::{app::Message, utils::auth_utils};

use super::constants::COUNTRY_CODES_FOR_WIFI_SETUP;
use super::setup_wizard_utils::is_valid_ip;
use super::{
    page::{DiskInfo, SetupWizard, SetupWizardMessage, SetupWizardStep},
    setup_wizard_utils,
};

const RPI_OS_IMAGE_ARCHIVE_FILENAME: &str = "rpi_os_lite.img.xz";
const RPI_OS_IMAGE_EXTRACTED_FILENAME: &str = "rpi_os_lite.img";
const RASPBERRY_PI_OS_LITE_DOWNLOAD_LINK: &str = "https://downloads.raspberrypi.com/raspios_lite_arm64/images/raspios_lite_arm64-2024-11-19/2024-11-19-raspios-bookworm-arm64-lite.img.xz";

pub fn update(state: &mut SetupWizard, message: SetupWizardMessage) -> Task<Message> {
    match message {
        SetupWizardMessage::GoToStep(step_to_go_to) => {
            state.current_step = step_to_go_to;
            state.connection_has_been_attempted = false;
        }

        SetupWizardMessage::UpdateServerStaticIp(s) => {
            state.work_in_progress_server_config.static_ip = s;
        }
        SetupWizardMessage::AddNewUser => {
            if !state.new_user_name_input_text.is_empty()
                && !state
                    .work_in_progress_server_config
                    .users_list
                    .contains_key(&state.new_user_name_input_text)
            {
                let new_user_name = state.new_user_name_input_text.clone();
                state.new_user_name_input_text = String::new();
                let new_user_auth_token: Vec<u8> = loop {
                    if let Ok(secret_bytes) = totp_rs::Secret::generate_secret().to_bytes() {
                        break secret_bytes;
                    }
                };
                state
                    .work_in_progress_server_config
                    .users_list
                    .insert(new_user_name, new_user_auth_token);
            }
        }
        SetupWizardMessage::UpdateNewUserNameInputText(s) => {
            state.new_user_name_input_text = s;
        }
        SetupWizardMessage::SetIsLanOnly(b) => {
            state.work_in_progress_server_config.is_lan_only = b;
        }
        SetupWizardMessage::UpdateServerUrlInputText(s) => {
            state.server_url_input_text = s;
        }
        SetupWizardMessage::UpdateClientUsernameInputText(s) => {
            state.client_username_input_text = s;
        }
        SetupWizardMessage::UpdateClientSecretInputText(s) => {
            state.client_secret_input_text = s;
        }
        SetupWizardMessage::SetExistingServerDetails => {
            if !state.server_url_input_text.is_empty()
                && !state.client_username_input_text.is_empty()
                && !state.client_secret_input_text.is_empty()
            {
                if let Ok(decoded_secret) =
                    base64::prelude::BASE64_STANDARD.decode(&state.client_secret_input_text)
                {
                    if let Some(parsed_url) = state
                        .server_url_input_text
                        .split("://")
                        .last()
                        .map(|item| item.to_string())
                    {
                        state.work_in_progress_client_config.sync_config.server_url = parsed_url;
                        state
                            .work_in_progress_client_config
                            .sync_config
                            .client_username = Some(state.client_username_input_text.clone());
                        state
                            .work_in_progress_client_config
                            .sync_config
                            .client_secret = Some(decoded_secret);
                        return Task::done(Message::SetupWizard(SetupWizardMessage::GoToStep(
                            SetupWizardStep::ConfirmConnection,
                        )))
                        .chain(Task::done(Message::SetupWizard(
                            SetupWizardMessage::TestConnection,
                        )));
                    }
                }
            }
        }
        SetupWizardMessage::TestConnection => {
            let client_secret = base64::prelude::BASE64_STANDARD
                .decode(&state.client_secret_input_text)
                .expect("Just checked this");
            let client_username = state.client_username_input_text.clone();
            let server_url = state
                .server_url_input_text
                .split("://")
                .last()
                .map(|item| item.to_string())
                .expect("Just checked this");
            state.connection_has_been_attempted = false;

            return Task::perform(
                async move {
                    let mut server_url_with_auth =
                        Url::parse(&(String::from("http://") + &server_url)) // TODO Change to https for prod
                            .unwrap()
                            .join("/sync/first_sync")
                            .unwrap();
                    server_url_with_auth
                        .query_pairs_mut()
                        .append_pair("client_id", &client_username);

                    let auth_token = auth_utils::calculate_totp(&client_secret);
                    let mut retry_counter = 0;
                    loop {
                        if retry_counter >= 3 {
                            break None;
                        }
                        match reqwest::Client::new()
                            .get(server_url_with_auth.as_ref())
                            .bearer_auth(&auth_token)
                            .send()
                            .await
                        {
                            Ok(res) => {
                                if res.status().is_success() {
                                    break res.bytes().await.ok().and_then(|res_bytes| {
                                        serde_json::from_slice::<HashMap<String, Vec<String>>>(
                                            &res_bytes,
                                        )
                                        .ok()
                                    });
                                }
                            }
                            Err(err) => {
                                println!("Sync get error, will retry: {err:?}");
                            }
                        }
                        retry_counter += 1;
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                },
                |res_option| {
                    Message::SetupWizard(SetupWizardMessage::SetRemoteFolderInfo(res_option))
                },
            );
        }
        SetupWizardMessage::SetRemoteFolderInfo(remote_folders_info_option) => {
            state.connection_has_been_attempted = true;
            state.remote_folders_info = remote_folders_info_option;
        }
        SetupWizardMessage::SetSelectedRemoteFolder(selected_remote_folder) => {
            state.selected_remote_folder = selected_remote_folder;
        }
        SetupWizardMessage::IgnoreFolderId(id_to_ignore) => {
            state
                .work_in_progress_client_config
                .sync_config
                .ignored_remote_folder_ids
                .push(id_to_ignore);
        }
        SetupWizardMessage::UnignoreFolderId(index_to_remove) => {
            state
                .work_in_progress_client_config
                .sync_config
                .ignored_remote_folder_ids
                .remove(index_to_remove);
        }
        SetupWizardMessage::SetSyncFrequency(sync_frequency) => {
            state
                .work_in_progress_client_config
                .sync_config
                .sync_frequency_settings = sync_frequency;
        }
        SetupWizardMessage::SetSetupType(setup_type) => {
            state.setup_type = setup_type;
        }
        SetupWizardMessage::GetListOfDisks => {
            let disks = sysinfo::Disks::new_with_refreshed_list();
            state.list_of_disks = disks.list().iter().map(DiskInfo::from_disk).collect();
        }
        SetupWizardMessage::SetProgressBarValue(progress) => {
            state.progress_bar_value = progress;
        }
        SetupWizardMessage::DownloadImg => {
            let mut iso_download_file_path =
                dirs::data_local_dir().expect("No config directory, big problem");
            iso_download_file_path.push(APP_ID);
            iso_download_file_path.push("rpi_os_lite.img.xz");
            return Task::run(
                setup_wizard_utils::download_file(
                    RASPBERRY_PI_OS_LITE_DOWNLOAD_LINK.to_owned(),
                    iso_download_file_path,
                ),
                |progress_result| match progress_result {
                    Ok(progress) => {
                        Message::SetupWizard(SetupWizardMessage::SetProgressBarValue(progress))
                    }
                    Err(err) => Message::ShowToast(false, format!("Error with download: {err}")),
                },
            )
            .chain(Task::done(Message::SetupWizard(
                SetupWizardMessage::ExtractImg,
            )));
        }
        SetupWizardMessage::ExtractImg => {
            let mut img_archive_download_file_path =
                dirs::data_local_dir().expect("No config directory, big problem");
            img_archive_download_file_path.push(APP_ID);
            img_archive_download_file_path.push(RPI_OS_IMAGE_ARCHIVE_FILENAME);
            let mut extracted_img_file_path =
                dirs::data_local_dir().expect("No config directory, big problem");
            extracted_img_file_path.push(APP_ID);
            extracted_img_file_path.push(RPI_OS_IMAGE_EXTRACTED_FILENAME);
            return Task::run(
                setup_wizard_utils::extract_img(
                    img_archive_download_file_path,
                    extracted_img_file_path,
                ),
                |progress_result| match progress_result {
                    Ok(progress) => {
                        Message::SetupWizard(SetupWizardMessage::SetProgressBarValue(progress))
                    }
                    Err(err) => Message::ShowToast(false, format!("Error with download: {err}")),
                },
            )
            .chain(Task::done(Message::SetupWizard(
                SetupWizardMessage::FlashSdCard,
            )));
        }
        SetupWizardMessage::FlashSdCard => {
            let mut extracted_img_file_path =
                dirs::data_local_dir().expect("No config directory, big problem");
            extracted_img_file_path.push(APP_ID);
            extracted_img_file_path.push(RPI_OS_IMAGE_EXTRACTED_FILENAME);
            let temp_target_name = String::from("mmcblk0"); // TODO change to selected option
            return Task::run(
                setup_wizard_utils::flash_img_to_sd_card(extracted_img_file_path, temp_target_name),
                |progress_result| match progress_result {
                    Ok(progress) => {
                        Message::SetupWizard(SetupWizardMessage::SetProgressBarValue(progress))
                    }
                    Err(err) => Message::ShowToast(false, format!("Error with flashing: {err}")),
                },
            );
            // .chain(Task::done(Message::SetupWizard(
            //     SetupWizardMessage::WriteConfig,
            // ))); TODO uncomment
        }
        SetupWizardMessage::DownloadExtraData => {
            return Task::run(
                setup_wizard_utils::download_extra_files(),
                |progress_result| match progress_result {
                    Ok(progress) => {
                        Message::SetupWizard(SetupWizardMessage::SetProgressBarValue(progress))
                    }
                    Err(err) => Message::ShowToast(false, format!("Error with download: {err}")),
                },
            );
        }
        SetupWizardMessage::ConfirmRemoteAccessDetails => {
            if state.work_in_progress_server_config.static_ip.is_empty() {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("Static IP can't be blank"),
                ));
            }
            if !is_valid_ip(&state.work_in_progress_server_config.static_ip) {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("Static IP has invalid format"),
                ));
            }
            if state
                .work_in_progress_server_config
                .duckdns_domain
                .is_empty()
            {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("DuckDNS domain can't be blank"),
                ));
            }
            if state
                .work_in_progress_server_config
                .duckdns_token
                .is_empty()
            {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("DuckDNS token can't be blank"),
                ));
            }
            state.work_in_progress_server_config.is_lan_only = false;
            return Task::done(Message::SetupWizard(SetupWizardMessage::GoToStep(
                SetupWizardStep::SetRpiServerPassword,
            )));
        }
        SetupWizardMessage::LinkClicked(link) => {
            opener::open(link).unwrap();
        }
        SetupWizardMessage::UpdateServerDuckDnsDomain(s) => {
            state.work_in_progress_server_config.duckdns_domain = s;
        }
        SetupWizardMessage::UpdateServerDuckDnsToken(s) => {
            state.work_in_progress_server_config.duckdns_token = s;
        }
        SetupWizardMessage::UpdateWifiSsid(s) => {
            state.work_in_progress_server_config.wifi_ssid = s;
        }
        SetupWizardMessage::UpdateWifiPassword(s) => {
            state.work_in_progress_server_config.wifi_password = s;
        }
        SetupWizardMessage::SetWifiCountryName(selected_country_name) => {
            state.work_in_progress_server_config.country_name_option = Some(selected_country_name);
        }
        SetupWizardMessage::SetWifiDetails => {
            if state.work_in_progress_server_config.wifi_ssid.is_empty() {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("SSID can't be blank"),
                ));
            }
            if state
                .work_in_progress_server_config
                .wifi_password
                .is_empty()
            {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("Password can't be blank"),
                ));
            }
            if state
                .work_in_progress_server_config
                .country_name_option
                .is_none()
            {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("Country can't be blank"),
                ));
            }

            let country_code = COUNTRY_CODES_FOR_WIFI_SETUP
                .iter()
                .find_map(|(country_name, country_code)| {
                    if *country_name
                        == state
                            .work_in_progress_server_config
                            .country_name_option
                            .expect("Has to be some, just checked this")
                    {
                        Some(country_code)
                    } else {
                        None
                    }
                })
                .expect("All country names have to be in list");
            state
                .work_in_progress_server_config
                .wpa_supplicant_file_content = format!(
                r#"country={}
update_config=1
ctrl_interface=/var/run/wpa_supplicant

network={{
 scan_ssid=1
 ssid="{}"
 psk="{}"
}}"#,
                country_code,
                state.work_in_progress_server_config.wifi_ssid,
                state.work_in_progress_server_config.wifi_password
            );
            return Task::done(Message::SetupWizard(SetupWizardMessage::GoToStep(
                SetupWizardStep::OptionalSetupRemoteAccess,
            )));
        }
        SetupWizardMessage::UpdateServerPassword(s) => {
            state.work_in_progress_server_config.server_password = s;
        }
    }
    Task::none()
}
