use iced::Element;

use crate::app::Message;

use super::page::{SetupWizard, SetupWizardStep};

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
    "decide_whether_to_setup_server_view".into()
}

fn enter_server_url_and_totp_secret_view(_state: &SetupWizard) -> Element<Message> {
    "EnterServerUrlAndTotpSecret ".into()
}

fn confirm_connection_view(_state: &SetupWizard) -> Element<Message> {
    "ConfirmConnection ".into()
}

fn choose_remote_folders_to_sync_view(_state: &SetupWizard) -> Element<Message> {
    "ChooseRemoteFoldersToSync ".into()
}
fn existing_server_close_wizard_view(_state: &SetupWizard) -> Element<Message> {
    "existing_server_close_wizard_view".into()
}
fn offline_setup_close_wizard_view(_state: &SetupWizard) -> Element<Message> {
    "offline_setup_close_wizard_view".into()
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
