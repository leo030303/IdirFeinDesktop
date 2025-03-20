use std::{collections::HashMap, time::Duration};

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use iced::Task;
use url::Url;

use crate::app::Message;
use crate::utils::auth_utils::AuthCredentials;

use super::setup_wizard_utils::is_valid_ip;
use super::{
    page::{SetupWizard, SetupWizardMessage, SetupWizardStep},
    setup_wizard_utils,
};

pub fn update(state: &mut SetupWizard, message: SetupWizardMessage) -> Task<Message> {
    match message {
        SetupWizardMessage::GoToStep(step_to_go_to) => {
            state.current_step = step_to_go_to;
            state.connection_has_been_attempted = false;
            if matches!(state.current_step, SetupWizardStep::ConfirmConnection) {
                return Task::done(Message::SetupWizard(SetupWizardMessage::TestConnection));
            }
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
                        state.work_in_progress_client_config.sync_config.should_sync = true;
                        state
                            .work_in_progress_client_config
                            .sync_config
                            .client_credentials = Some(AuthCredentials {
                            client_id: state.client_username_input_text.clone(),
                            client_secret: decoded_secret,
                        });
                        return Task::done(Message::SetupWizard(SetupWizardMessage::GoToStep(
                            SetupWizardStep::ConfirmConnection,
                        )));
                    }
                } else {
                    return Task::done(Message::ShowToast(
                        false,
                        String::from("Secret token is incorrect, are you sure you typed it right?"),
                    ));
                }
            } else {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("Need to fill all fields"),
                ));
            }
        }
        SetupWizardMessage::TestConnection => {
            let client_secret = base64::prelude::BASE64_STANDARD
                .decode(&state.client_secret_input_text)
                .expect("Just checked this");
            let client_id = state.client_username_input_text.clone();
            let auth_credentials = AuthCredentials {
                client_id,
                client_secret,
            };
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
                        Url::parse(&(String::from("https://") + &server_url))
                            .unwrap()
                            .join("/sync/first_sync")
                            .unwrap();
                    server_url_with_auth
                        .query_pairs_mut()
                        .append_pair("client_id", &auth_credentials.client_id);

                    let auth_token = auth_credentials.calculate_totp();
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
        SetupWizardMessage::SetProgressBarValue(progress) => {
            state.progress_bar_value = progress;
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
            if state.work_in_progress_server_config.gateway_ip.is_empty() {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("Gateway IP can't be blank"),
                ));
            }
            if !is_valid_ip(&state.work_in_progress_server_config.gateway_ip) {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("Gateway IP has invalid format"),
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
            if state
                .work_in_progress_server_config
                .certbot_email
                .is_empty()
            {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("Certbot email can't be blank"),
                ));
            }
            state.work_in_progress_server_config.is_lan_only = false;
            return Task::done(Message::SetupWizard(SetupWizardMessage::GoToStep(
                SetupWizardStep::CreateServerUsers,
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
        SetupWizardMessage::UpdateServerCertbotEmail(s) => {
            state.work_in_progress_server_config.certbot_email = s;
        }
        SetupWizardMessage::SetUserForThisDevice {
            username,
            auth_token,
        } => {
            if let Ok(client_secret) = BASE64_STANDARD.decode(auth_token) {
                state
                    .work_in_progress_client_config
                    .sync_config
                    .client_credentials = Some(AuthCredentials {
                    client_id: username,
                    client_secret,
                })
            }
        }
        SetupWizardMessage::CopySetupScript => {
            let setup_script = format!(
                r#"
#!/bin/bash

# Update the system
sudo apt update -y
sudo apt upgrade -y

# Create data dir
sudo mkdir -p /mnt/idirfein_data
sudo chown "$USER":"$(id -gn)" /mnt/idirfein_data

# Automount harddrive on start and set static ip
sudo tee /usr/local/bin/automount.sh > /dev/null << 'EOF'
#!/bin/bash

MOUNT_POINT="/mnt/idirfein_data"

sudo ip addr add {0}/24 dev eth0 && sudo ip route add default via {6}
# Find the first external disk (typically sda, sdb, etc)
# Excluding mmcblk (SD card) and loop devices
external_drive=$(lsblk -pnlo NAME,TYPE | grep -E 'disk' | grep -v 'mmcblk' | grep -v 'loop' | head -n 1 | awk '{{print $1}}')

if [ -z "$external_drive" ]; then
    exit 1
fi

# Find the first partition on the external drive
external_partition="${{external_drive}}1"

# Check if the partition exists
if [ ! -b "$external_partition" ]; then
    # If partition 1 doesn't exist, try using the whole disk
    external_partition="$external_drive"
fi

mount "$external_partition" "$MOUNT_POINT"
EOF
sudo chown "$USER":"$(id -gn)" /usr/local/bin/automount.sh
chmod +x /usr/local/bin/automount.sh

sudo tee /etc/systemd/system/automount.service > /dev/null << 'EOF'
[Unit]
Description=Automatic mounting of external drives
After=network.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/automount.sh
StandardOutput=journal

[Install]
WantedBy=multi-user.target
EOF
sudo systemctl enable automount.service
sudo systemctl start automount.service

# Download the server file
sudo wget -O "/mnt/idirfein_data/idirfein_server" "https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/idirfein_server"
sudo chmod +x /mnt/idirfein_data/idirfein_server
sudo mkdir -p /mnt/idirfein_data/blog_content/www


# Automatically run the binary 'idirfein_server' on startup
sudo tee /usr/local/bin/idirfein_start.sh > /dev/null << 'EOF'
#!/bin/bash
cd /mnt/idirfein_data && ./idirfein_server
EOF
sudo chown "$USER":"$(id -gn)" /usr/local/bin/idirfein_start.sh
chmod +x /usr/local/bin/idirfein_start.sh


sudo tee /etc/systemd/system/idirfein.service > /dev/null << 'EOF'
[Unit]
Description=Idirfein web server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/idirfein_start.sh
Restart=always
RestartSec=5
StandardOutput=journal

[Install]
WantedBy=multi-user.target
EOF
sudo systemctl enable idirfein.service

sudo mkdir -p /usr/local/share/idirfein_server/
sudo tee /usr/local/share/idirfein_server/users.json > /dev/null << 'EOF'
{5}
EOF

if {1}; then
    echo "All Done"
    exit 1
fi

# Set up DuckDNS
mkdir -p ~/duckdns
cd ~/duckdns

cat <<EOL > duck.sh
#!/bin/bash
echo url="https://www.duckdns.org/update?domains={2}&token={3}&ip=" | curl -k -o ~/duckdns/duck.log -K -
EOL

chmod 700 duck.sh

# Add the DuckDNS script to crontab
(crontab -l 2>/dev/null; echo "*/5 * * * * ~/duckdns/duck.sh >/dev/null 2>&1") | crontab -

# Set up SSL certificates using Let's Encrypt
sudo apt install -y certbot
sudo certbot certonly --standalone --http-01-port 8000 -d {2}.duckdns.org --non-interactive --agree-tos --email {4}
(crontab -l 2>/dev/null; echo "0 0 * * * sudo systemctl stop idirfein.service && certbot renew --http-01-port 8000 --quiet && sudo systemctl start idirfein.service
") | crontab -

mkdir ~/.certs
ln -s /etc/letsencrypt/live/{2}.duckdns.org/fullchain.pem ~/.certs/fullchain
ln -s /etc/letsencrypt/live/{2}.duckdns.org/privkey.pem ~/.certs/privkey


echo "All Done"
                "#,
                state.work_in_progress_server_config.static_ip,
                state.work_in_progress_server_config.is_lan_only,
                state.work_in_progress_server_config.duckdns_domain,
                state.work_in_progress_server_config.duckdns_token,
                state.work_in_progress_server_config.certbot_email,
                serde_json::to_string(&state.work_in_progress_server_config.users_list)
                    .expect("Malformed Users List"),
                state.work_in_progress_server_config.gateway_ip
            );
            return Task::done(Message::CopyValueToClipboard(setup_script));
        }
        SetupWizardMessage::UpdateServerGatewayIp(s) => {
            state.work_in_progress_server_config.gateway_ip = s;
        }
        SetupWizardMessage::ConfirmLocalAccessDetails => {
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
            if state.work_in_progress_server_config.gateway_ip.is_empty() {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("Gateway IP can't be blank"),
                ));
            }
            if !is_valid_ip(&state.work_in_progress_server_config.gateway_ip) {
                return Task::done(Message::ShowToast(
                    false,
                    String::from("Gateway IP has invalid format"),
                ));
            }
            state.work_in_progress_server_config.is_lan_only = true;
            state.work_in_progress_client_config.sync_config.server_url =
                format!("{}:8000", state.work_in_progress_server_config.static_ip);
            return Task::done(Message::SetupWizard(SetupWizardMessage::GoToStep(
                SetupWizardStep::CreateServerUsers,
            )));
        }
    }
    Task::none()
}
