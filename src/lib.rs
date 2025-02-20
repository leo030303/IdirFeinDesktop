use std::fmt::Display;

use iced::widget::svg::{self, Handle};
use serde::{Deserialize, Serialize};

#[rustfmt::skip]
pub mod constants;
pub mod app;
pub mod config;
pub mod pages;
pub mod utils;

fluent_templates::static_loader! {
    pub static LOCALES = {
        locales: "./locales",
        fallback_language: "en-GB",
    };
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Page {
    Settings,
    Passwords,
    Sync,
    Gallery,
    Notes,
    Tasks,
}

impl Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Page::Settings => write!(f, "Settings"),
            Page::Passwords => write!(f, "Passwords"),
            Page::Sync => write!(f, "Sync"),
            Page::Gallery => write!(f, "Gallery"),
            Page::Notes => write!(f, "Notes"),
            Page::Tasks => write!(f, "Tasks"),
        }
    }
}

impl Page {
    pub fn get_all() -> [Page; 6] {
        [
            Page::Settings,
            Page::Notes,
            Page::Passwords,
            Page::Tasks,
            Page::Sync,
            Page::Gallery,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Page::Settings => "Settings",
            Page::Passwords => "Passwords",
            Page::Sync => "Sync",
            Page::Gallery => "Gallery",
            Page::Notes => "Notes",
            Page::Tasks => "Tasks",
        }
    }

    pub fn icon_handle(&self) -> Handle {
        match self {
            Page::Settings => svg::Handle::from_memory(include_bytes!("../icons/settings.svg")),
            Page::Passwords => svg::Handle::from_memory(include_bytes!("../icons/key.svg")),
            Page::Sync => svg::Handle::from_memory(include_bytes!("../icons/sync.svg")),
            Page::Gallery => svg::Handle::from_memory(include_bytes!("../icons/image-round.svg")),
            Page::Notes => svg::Handle::from_memory(include_bytes!("../icons/notepad.svg")),
            Page::Tasks => svg::Handle::from_memory(include_bytes!("../icons/task.svg")),
        }
    }
}
