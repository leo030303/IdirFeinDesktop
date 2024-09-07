use iced::{
    alignment::Horizontal,
    widget::{self, button, column, row, text, tooltip::Position, Space, Svg, Tooltip},
    ContentFit, Length, Task, Theme,
};
use idirfein_desktop_iced::{
    pages::{
        file_manager::FileManagerPage, gallery::GalleryPage, notes::NotesPage,
        passwords::PasswordsPage, settings::SettingsPage, tasks::TasksPage,
    },
    Message, Page,
};

fn navbar_button(page: Page) -> iced::Element<'static, Message> {
    Tooltip::new(
        button(Svg::from_path(page.icon_path()).content_fit(ContentFit::ScaleDown))
            .on_press(Message::ChangePage(page.clone())),
        page.name(),
        Position::Bottom,
    )
    .into()
}

pub fn main() -> iced::Result {
    iced::application("IdirFéin Desktop", AppState::update, AppState::view)
        .theme(AppState::theme)
        .run_with(AppState::new)
}

struct AppState {
    theme: Theme,
    current_page: Page,
    notes_page: NotesPage,
    passwords_page: PasswordsPage,
    tasks_page: TasksPage,
    settings_page: SettingsPage,
    file_manager_page: FileManagerPage,
    gallery_page: GalleryPage,
}

impl AppState {
    fn new() -> (Self, Task<Message>) {
        let theme = Theme::TokyoNight;
        (
            Self {
                current_page: Page::Notes,
                theme,
                notes_page: NotesPage::new(),
                passwords_page: PasswordsPage::new(),
                tasks_page: TasksPage::new(),
                settings_page: SettingsPage::new(),
                file_manager_page: FileManagerPage::new(),
                gallery_page: GalleryPage::new(),
            },
            widget::focus_next(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChangePage(new_page) => self.current_page = new_page,
            Message::Passwords(m) => return self.passwords_page.update(m),
            Message::Notes(m) => return self.notes_page.update(m),
            Message::Tasks(m) => return self.tasks_page.update(m),
            Message::Gallery(m) => return self.gallery_page.update(m),
            Message::FileManager(m) => return self.file_manager_page.update(m),
            Message::Settings(m) => return self.settings_page.update(m),
        }
        Task::none()
    }

    fn view(&self) -> iced::Element<Message> {
        let nav_bar = row![
            navbar_button(Page::Notes),
            navbar_button(Page::Tasks),
            navbar_button(Page::Passwords),
            navbar_button(Page::FileManager),
            navbar_button(Page::Gallery),
            navbar_button(Page::Settings),
        ]
        .width(Length::FillPortion(1));

        let tool_view = match self.current_page {
            Page::Settings => self.settings_page.tool_view(),
            Page::Passwords => self.passwords_page.tool_view(),
            Page::FileManager => self.file_manager_page.tool_view(),
            Page::Gallery => self.gallery_page.tool_view(),
            Page::Notes => self.notes_page.tool_view(),
            Page::Tasks => self.tasks_page.tool_view(),
        };

        let main_view = match self.current_page {
            Page::Settings => self.settings_page.view(),
            Page::Passwords => self.passwords_page.view(),
            Page::FileManager => self.file_manager_page.view(),
            Page::Gallery => self.gallery_page.view(),
            Page::Notes => self.notes_page.view(),
            Page::Tasks => self.tasks_page.view(),
        };

        column![
            row![
                tool_view,
                text(self.current_page.name())
                    .size(24)
                    .width(Length::FillPortion(1))
                    .align_x(Horizontal::Center),
                nav_bar,
            ],
            Space::with_height(20),
            main_view
        ]
        .into()
    }
    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
