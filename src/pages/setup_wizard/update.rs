use iced::Task;

use crate::app::Message;

use super::page::{SetupWizard, SetupWizardMessage};

pub fn update(state: &mut SetupWizard, message: SetupWizardMessage) -> Task<Message> {
    match message {
        SetupWizardMessage::GoToStep(step_to_go_to) => state.current_step = step_to_go_to,

        SetupWizardMessage::SetServerPort => {
            if let Ok(parsed_port) = state.port_input_text.parse::<u32>() {
                state.work_in_progress_server_config.port = parsed_port;
            }
        }
        SetupWizardMessage::UpdateServerPortInputText(s) => {
            state.port_input_text = s;
        }
        SetupWizardMessage::SetServerStaticIp => {
            if !state.static_ip_input_text.is_empty() {
                state.work_in_progress_server_config.static_ip = state.static_ip_input_text.clone();
            }
        }
        SetupWizardMessage::UpdateServerStaticIpInputText(s) => {
            state.static_ip_input_text = s;
        }
        SetupWizardMessage::AddNewUser => {
            if !state.new_user_name_input_text.is_empty() {
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
    }
    Task::none()
}
