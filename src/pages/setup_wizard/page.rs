use std::collections::HashMap;

use iced::{Element, Task};
use serde::{Deserialize, Serialize};

use crate::app::Message;
use crate::config::AppConfig;

use super::update::update;
use super::view::main_view;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u32,
    pub static_ip: String,
    pub users_list: HashMap<String, Vec<u8>>,
    pub is_lan_only: bool,
    pub storage_harddrive_uuid: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 1987,
            static_ip: String::new(),
            users_list: HashMap::new(),
            is_lan_only: true,
            storage_harddrive_uuid: String::new(),
        }
    }
}

pub struct SetupWizard {
    pub current_step: SetupWizardStep,
    pub work_in_progress_server_config: ServerConfig,
    pub work_in_progress_client_config: AppConfig,
    pub port_input_text: String,
    pub static_ip_input_text: String,
    pub new_user_name_input_text: String,
    pub server_url_input_text: String,
    pub client_username_input_text: String,
    pub client_secret_input_text: String,
    pub is_successful_connection: Option<bool>,
}

#[derive(Debug, Clone)]
pub enum SetupWizardMessage {
    GoToStep(SetupWizardStep),
    SetServerPort,
    UpdateServerPortInputText(String),
    SetServerStaticIp,
    UpdateServerStaticIpInputText(String),
    AddNewUser,
    UpdateNewUserNameInputText(String),
    SetIsLanOnly(bool),
    UpdateServerUrlInputText(String),
    UpdateClientUsernameInputText(String),
    UpdateClientSecretInputText(String),
    SetExistingServerDetails,
    TestConnection,
    SetConnectionSuccess(Option<bool>),
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
    /// Device is setup and ready to use
    ExistingServerCloseWizard,

    // Use offline path
    /// Remind the user they have no sync capabilities, but they can change this at any time in the settings
    OfflineSetupCloseWizard,

    // Full server setup path
    /// Ask if the user wants to setup external WLAN access, explain what this means
    OptionalSetupWlanChoice,

    // Optional WLAN setup
    /// Prompts user to set up port forwarding, give links to guides, says what port to use, confirm when done
    SetupPortForwarding,
    /// Prompts user to go to DuckDNS and set up credentials, enter when done
    SetupDuckDns,

    // Continued path
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
    FlashingSdCard,
    /// SD card setup is complete, it has been ejected and is safe to remove
    SdCardSetupCompletePleaseEject,
    /// Please put the sd in your rpi zero and plug it in, a pop up appears when the app connects to the server successfully
    PlugInServerConfirmConnection,
    /// Server setup is complete, click close, opens up regular device view
    FullSetupCloseWizard,
}

impl SetupWizard {
    pub fn new() -> Self {
        let server_config = ServerConfig::default();
        Self {
            current_step: SetupWizardStep::DecideWhetherToSetupServer,
            port_input_text: format!("{:?}", server_config.port),
            static_ip_input_text: format!("{:?}", server_config.static_ip),
            new_user_name_input_text: String::new(),
            work_in_progress_server_config: server_config,
            work_in_progress_client_config: AppConfig::default(),
            server_url_input_text: String::new(),
            client_username_input_text: String::new(),
            client_secret_input_text: String::new(),
            is_successful_connection: None,
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
