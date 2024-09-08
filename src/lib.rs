use crate::pages::file_manager::page::FileManagerPageMessage;
use crate::pages::gallery::page::GalleryPageMessage;
use crate::pages::notes::page::NotesPageMessage;
use crate::pages::passwords::page::PasswordsPageMessage;
use crate::pages::settings::page::SettingsPageMessage;
use crate::pages::tasks::page::TasksPageMessage;

pub mod pages;
pub mod utils;

#[derive(Debug, Clone)]
pub enum Message {
    ChangePage(Page),
    CloseWindowRequest,
    None,
    Passwords(PasswordsPageMessage),
    Notes(NotesPageMessage),
    Tasks(TasksPageMessage),
    Gallery(GalleryPageMessage),
    FileManager(FileManagerPageMessage),
    Settings(SettingsPageMessage),
}

#[derive(Debug, Clone)]
pub enum SettingsSubpage {
    Sync,
    Config,
    General,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Page {
    Settings,
    Passwords,
    FileManager,
    Gallery,
    Notes,
    Tasks,
}

impl Page {
    pub fn name(&self) -> &'static str {
        match self {
            Page::Settings => "Settings",
            Page::Passwords => "Passwords",
            Page::FileManager => "Files",
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
