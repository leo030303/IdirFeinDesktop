use std::{fs, path::Path};

use crate::config::AppConfig;

pub async fn save_settings_to_file(config: AppConfig) -> (bool, String) {
    // TODO Make config save path platform specific
    let config_file_path = "/home/leoring/.config/idirfein_desktop/config.json";
    if let Some(parent) = Path::new(config_file_path).parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            return (false, format!("Failed to make parent directory: {err:?}"));
        };
    };
    match serde_json::to_string(&config) {
        Ok(serialised_config) => match fs::write(config_file_path, serialised_config) {
            Ok(_) => (true, String::from("Settings saved")),
            Err(err) => (false, format!("Failed on file write: {err:?}")),
        },
        Err(err) => (
            false,
            format!("Couldn't serialise AppConfig object to JSON: {err:?}"),
        ),
    }
}
pub fn load_settings_from_file() -> AppConfig {
    // TODO Make config save path platform specific
    let config_file_path = "/home/leoring/.config/idirfein_desktop/config.json";
    if let Ok(config_json) = fs::read_to_string(config_file_path) {
        let app_config: AppConfig = serde_json::from_str(&config_json).unwrap();
        app_config
    } else {
        AppConfig::default()
    }
}
