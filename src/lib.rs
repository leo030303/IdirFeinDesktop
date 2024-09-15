use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub mod app;
pub mod config;
pub mod pages;
pub mod utils;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Page {
    Settings,
    Passwords,
    FileManager,
    Gallery,
    Notes,
    Tasks,
}

impl Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Page::Settings => write!(f, "Settings"),
            Page::Passwords => write!(f, "Passwords"),
            Page::FileManager => write!(f, "File Manager"),
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
            Page::FileManager,
            Page::Gallery,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Page::Settings => "Settings",
            Page::Passwords => "Passwords",
            Page::FileManager => "File Manager",
            Page::Gallery => "Gallery",
            Page::Notes => "Notes",
            Page::Tasks => "Tasks",
        }
    }

    pub fn icon_path(&self) -> &'static str {
        match self {
            Page::Settings => "icons/settings.svg",
            Page::Passwords => "icons/key.svg",
            Page::FileManager => "icons/file-manager.svg",
            Page::Gallery => "icons/image-round.svg",
            Page::Notes => "icons/notepad.svg",
            Page::Tasks => "icons/task.svg",
        }
    }
}
