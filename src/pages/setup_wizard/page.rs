use std::collections::HashMap;

use iced::{Element, Task};
use serde::{Deserialize, Serialize};

use crate::app::Message;
use crate::config::AppConfig;
use crate::pages::sync::page::SyncFrequencySettings;

use super::update::update;
use super::view::main_view;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SetupProgressBarValue {
    WaitingToStart,
    DownloadingFile(f32),
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SetupType {
    FullServerSetup,
    ConnectToExistingServerSetup,
    NoServerSetup,
    NoneSelectedYet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub static_ip: String,
    pub users_list: HashMap<String, Vec<u8>>,
    pub is_lan_only: bool,
    pub duckdns_domain: String,
    pub duckdns_token: String,
    pub certbot_email: String,
    pub gateway_ip: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            static_ip: String::new(),
            users_list: HashMap::new(),
            is_lan_only: true,
            duckdns_domain: String::new(),
            duckdns_token: String::new(),
            certbot_email: String::new(),
            gateway_ip: String::new(),
        }
    }
}

pub struct SetupWizard {
    pub locale: fluent_templates::LanguageIdentifier,
    pub current_step: SetupWizardStep,
    pub work_in_progress_server_config: ServerConfig,
    pub work_in_progress_client_config: AppConfig,
    pub new_user_name_input_text: String,
    pub server_url_input_text: String,
    pub client_username_input_text: String,
    pub client_secret_input_text: String,
    pub connection_has_been_attempted: bool,
    pub remote_folders_info: Option<HashMap<String, Vec<String>>>,
    pub selected_remote_folder: Option<String>,
    pub setup_type: SetupType,
    pub progress_bar_value: SetupProgressBarValue,
}

#[derive(Debug, Clone)]
pub enum SetupWizardMessage {
    GoToStep(SetupWizardStep),
    UpdateServerStaticIp(String),
    UpdateServerGatewayIp(String),
    AddNewUser,
    UpdateNewUserNameInputText(String),
    SetIsLanOnly(bool),
    UpdateServerUrlInputText(String),
    UpdateClientUsernameInputText(String),
    UpdateClientSecretInputText(String),
    SetExistingServerDetails,
    TestConnection,
    SetRemoteFolderInfo(Option<HashMap<String, Vec<String>>>),
    SetSelectedRemoteFolder(Option<String>),
    IgnoreFolderId(String),
    UnignoreFolderId(usize),
    SetSyncFrequency(SyncFrequencySettings),
    SetSetupType(SetupType),
    SetProgressBarValue(SetupProgressBarValue),
    DownloadExtraData,
    ConfirmRemoteAccessDetails,
    LinkClicked(&'static str),
    UpdateServerDuckDnsDomain(String),
    UpdateServerDuckDnsToken(String),
    UpdateServerCertbotEmail(String),
    SetUserForThisDevice {
        username: String,
        auth_token: String,
    },
    CopySetupScript,
    ConfirmLocalAccessDetails,
}

// Offline setup
//
// DecideWhetherToSetupServer
// DownloadExtraAppData
// CloseWizard
//
//
// Existing server setup
//
// DecideWhetherToSetupServer
// EnterServerUrlAndTotpSecret
// ConfirmConnection
// ChooseRemoteFoldersToSync
// DownloadExtraAppData
// CloseWizard
//
//
// Full server setup
//
// DecideWhetherToSetupServer
// ImagerSetupSteps
// OptionalSetupRemoteAccess
// CreateServerUsers
// DownloadExtraAppData
// SshConnectionStepsRunScript
// ListUsersChooseYours
// ConfirmConnection
// CloseWizard

#[derive(Debug, Clone)]
pub enum SetupWizardStep {
    /// Three options, setup server, connect to existing server, use offline, include list of necessary materials for each
    DecideWhetherToSetupServer,

    // Connect to existing server path
    /// Please enter your server Url, username, and totp secret, ask your admin for these, share securely, you won't need these again
    EnterServerUrlAndTotpSecret,
    /// Tries to connect to server, if fails theres a back button to reenter previous details
    ConfirmConnection,
    /// Choose which remote folders you want to hold on this device, if any
    ChooseRemoteFoldersToSync,

    // Full server setup path
    /// Ask if the user wants to setup external WLAN access, explain what this means
    /// Prompts user to set up port forwarding, give links to guides, says what port to use, confirm when done
    /// Prompts user to go to DuckDNS and set up credentials, enter when done
    OptionalSetupRemoteAccess,

    /// Get the user to create the list of users for server syncing
    CreateServerUsers,

    /// Downloading extra data
    DownloadExtraAppData,
    CloseWizard,
    ListUsersChooseYours,
    ImagerSetupSteps,
    SshConnectionStepsRunScript,
}

impl SetupWizard {
    pub fn new() -> Self {
        let server_config = ServerConfig::default();
        let locale: fluent_templates::LanguageIdentifier = current_locale::current_locale()
            .expect("Can't get locale")
            .parse()
            .expect("Failed to parse locale");
        Self {
            locale,
            current_step: SetupWizardStep::DecideWhetherToSetupServer,
            new_user_name_input_text: String::new(),
            work_in_progress_server_config: server_config,
            work_in_progress_client_config: AppConfig::default(),
            server_url_input_text: String::new(),
            client_username_input_text: String::new(),
            client_secret_input_text: String::new(),
            connection_has_been_attempted: false,
            remote_folders_info: None,
            selected_remote_folder: None,
            setup_type: SetupType::NoneSelectedYet,
            progress_bar_value: SetupProgressBarValue::WaitingToStart,
        }
    }

    pub fn update(&mut self, message: SetupWizardMessage) -> Task<Message> {
        update(self, message)
    }

    pub fn view(&self) -> Element<Message> {
        main_view(self)
    }
}

impl Default for SetupWizard {
    fn default() -> Self {
        Self::new()
    }
}
