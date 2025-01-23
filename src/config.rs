use iced::Theme;
use serde::{Deserialize, Serialize};

use crate::{
    pages::{
        gallery::page::GalleryPageConfig, notes::page::NotesPageConfig,
        passwords::page::PasswordPageConfig, sync::page::SyncPageConfig,
        tasks::page::TaskPageConfig,
    },
    Page,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    theme_string: String,
    pub default_page_on_open: Page,
    pub notes_config: NotesPageConfig,
    pub passwords_config: PasswordPageConfig,
    pub gallery_config: GalleryPageConfig,
    pub tasks_config: TaskPageConfig,
    pub sync_config: SyncPageConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme_string: Self::get_string_from_theme(Theme::TokyoNightStorm),
            default_page_on_open: Page::Notes,
            notes_config: NotesPageConfig::default(),
            passwords_config: PasswordPageConfig::default(),
            gallery_config: GalleryPageConfig::default(),
            tasks_config: TaskPageConfig::default(),
            sync_config: SyncPageConfig::default(),
        }
    }
}
impl AppConfig {
    pub fn get_theme(&self) -> Option<Theme> {
        Self::get_theme_from_string(&self.theme_string)
    }
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme_string = Self::get_string_from_theme(theme);
    }
    fn get_string_from_theme(theme: Theme) -> String {
        format!("{theme:?}")
            .split("::")
            .last()
            .unwrap_or_default()
            .to_string()
    }
    fn get_theme_from_string(theme_string: &str) -> Option<Theme> {
        match theme_string {
            "CatppuccinFrappe" => Some(Theme::CatppuccinFrappe),
            "CatppuccinLatte" => Some(Theme::CatppuccinLatte),
            "CatppuccinMacchiato" => Some(Theme::CatppuccinMacchiato),
            "CatppuccinMocha" => Some(Theme::CatppuccinMocha),
            "Dark" => Some(Theme::Dark),
            "Dracula" => Some(Theme::Dracula),
            "Light" => Some(Theme::Light),
            "Ferra" => Some(Theme::Ferra),
            "GruvboxDark" => Some(Theme::GruvboxDark),
            "GruvboxLight" => Some(Theme::GruvboxLight),
            "KanagawaDragon" => Some(Theme::KanagawaDragon),
            "KanagawaLotus" => Some(Theme::KanagawaLotus),
            "KanagawaWave" => Some(Theme::KanagawaWave),
            "Moonfly" => Some(Theme::Moonfly),
            "Nightfly" => Some(Theme::Nightfly),
            "Nord" => Some(Theme::Nord),
            "Oxocarbon" => Some(Theme::Oxocarbon),
            "SolarizedDark" => Some(Theme::SolarizedDark),
            "SolarizedLight" => Some(Theme::SolarizedLight),
            "TokyoNight" => Some(Theme::TokyoNight),
            "TokyoNightLight " => Some(Theme::TokyoNightLight),
            "TokyoNightStorm" => Some(Theme::TokyoNightStorm),
            _ => None,
        }
    }
}
