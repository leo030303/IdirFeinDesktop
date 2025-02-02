use std::collections::HashMap;
use std::path::PathBuf;

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
    ExtractingImg(f32),
    FlashingSdCard,
    WritingConfig,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: PathBuf,
    pub total_space: u64,
    pub available_space: u64,
    pub is_removable: bool,
}
impl DiskInfo {
    pub fn from_disk(disk: &sysinfo::Disk) -> Self {
        Self {
            name: disk.name().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_path_buf(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
            is_removable: disk.is_removable(),
        }
    }
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
    pub port: u32,
    pub static_ip: String,
    pub users_list: HashMap<String, Vec<u8>>,
    pub is_lan_only: bool,
    pub duckdns_domain: String,
    pub duckdns_token: String,
    pub storage_harddrive_uuid: String,
    pub wifi_ssid: String,
    pub wifi_password: String,
    pub country_name_option: Option<&'static str>,
    pub wpa_supplicant_file_content: String,
    pub server_password: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 1987,
            static_ip: String::new(),
            users_list: HashMap::new(),
            is_lan_only: true,
            storage_harddrive_uuid: String::new(),
            duckdns_domain: String::new(),
            duckdns_token: String::new(),
            wifi_ssid: String::new(),
            wifi_password: String::new(),
            country_name_option: None,
            wpa_supplicant_file_content: String::new(),
            server_password: String::new(),
        }
    }
}

pub struct SetupWizard {
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
    pub list_of_disks: Vec<DiskInfo>,
    pub progress_bar_value: SetupProgressBarValue,
}

#[derive(Debug, Clone)]
pub enum SetupWizardMessage {
    GoToStep(SetupWizardStep),
    UpdateServerStaticIp(String),
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
    GetListOfDisks,
    SetProgressBarValue(SetupProgressBarValue),
    DownloadImg,
    ExtractImg,
    FlashSdCard,
    DownloadExtraData,
    ConfirmRemoteAccessDetails,
    LinkClicked(&'static str),
    UpdateServerDuckDnsDomain(String),
    UpdateServerDuckDnsToken(String),
    UpdateWifiSsid(String),
    UpdateWifiPassword(String),
    SetWifiCountryName(&'static str),
    SetWifiDetails,
    UpdateServerPassword(String),
}

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

    CloseWizard,

    // Full server setup path
    /// Say to setup 2.4ghz, get wifi ssid and password and country code, create wpa_supplicant file which gets written
    GetWifiDetails,

    /// Ask if the user wants to setup external WLAN access, explain what this means
    /// Prompts user to set up port forwarding, give links to guides, says what port to use, confirm when done
    /// Prompts user to go to DuckDNS and set up credentials, enter when done
    OptionalSetupRemoteAccess,

    /// Get the user to set the passwords for the raspberry pi server
    SetRpiServerPassword,

    /// Get the user to create the list of users for server syncing
    CreateServerUsers,

    /// Plug in the harddrive to be used, select it from list so config can store its UUID
    InsertTargetHardriveIntoThisComputer,
    /// Confirm this is the harddrive you want to use
    ConfirmTargetHarddrive,
    /// Eject harddrive and tell the user to plug it into the server
    RemoveTargetHarddrive,
    /// Tells user to insert SD card into computer, user clicks to confirm they have
    InsertSdCardIntoThisComputer,
    /// All usb devices get displayed, user picks SD card
    PickDeviceWhichIsTargetSdCard,
    /// Confirm this is the device you want, all info will be wiped
    ConfirmSdCardChoice,
    /// Please wait while we setup your device, flash OS, flash server software, write config etc
    SettingUpSdCard,
    /// SD card setup is complete, it has been ejected and is safe to remove
    SdCardSetupCompletePleaseEject,
    /// Please put the sd in your rpi zero and plug it in, a pop up appears when the app connects to the server successfully
    PlugInServerConfirmConnection,

    /// Downloading extra data
    DownloadExtraAppData,
}

impl SetupWizard {
    pub fn new() -> Self {
        let server_config = ServerConfig::default();
        Self {
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
            list_of_disks: vec![],
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
