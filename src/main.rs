use std::time::Duration;

use iced::{
    alignment::Horizontal,
    event,
    widget::{self, button, column, container, row, text, tooltip::Position, Svg, Tooltip},
    window, ContentFit, Event, Length, Subscription, Task, Theme,
};
use idirfein_desktop_iced::{
    pages::{
        file_manager::page::FileManagerPage, gallery::page::GalleryPage, notes::page::NotesPage,
        passwords::page::PasswordsPage, settings::page::SettingsPage, tasks::page::TasksPage,
    },
    Message, Page,
};

fn navbar_button(page: Page, selected: bool) -> iced::Element<'static, Message> {
    Tooltip::new(
        button(Svg::from_path(page.icon_path()).content_fit(ContentFit::ScaleDown))
            .style(if selected {
                button::secondary
            } else {
                button::primary
            })
            .on_press(Message::ChangePage(page.clone())),
        page.name(),
        Position::Bottom,
    )
    .into()
}

pub fn main() -> iced::Result {
    iced::application("IdirFéin Desktop", AppState::update, AppState::view)
        .theme(AppState::theme)
        .exit_on_close_request(false)
        .subscription(AppState::subscription)
        .run_with(AppState::new)
}

struct AppState {
    theme: Theme,
    is_closing: bool,
    current_page: Page,
    notes_page: NotesPage,
    passwords_page: PasswordsPage,
    tasks_page: TasksPage,
    settings_page: SettingsPage,
    file_manager_page: FileManagerPage,
    gallery_page: GalleryPage,
    show_toast: bool,
    is_good_toast: bool,
    toast_text: String,
}

impl AppState {
    fn new() -> (Self, Task<Message>) {
        let theme = Theme::TokyoNight;
        (
            Self {
                current_page: Page::Notes,
                theme,
                is_closing: false,
                notes_page: NotesPage::new(),
                passwords_page: PasswordsPage::new(),
                tasks_page: TasksPage::new(),
                settings_page: SettingsPage::new(),
                file_manager_page: FileManagerPage::new(),
                gallery_page: GalleryPage::new(),
                show_toast: false,
                is_good_toast: true,
                toast_text: String::new(),
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
            Message::ShowToast(is_good_toast, content) => {
                self.show_toast = true;
                self.is_good_toast = is_good_toast;
                self.toast_text = content;
                return Task::perform(
                    async { std::thread::sleep(Duration::from_millis(5000)) },
                    |_| Message::ToastExpired,
                );
            }
            Message::ToastExpired => {
                self.show_toast = false;
                self.is_good_toast = true;
                self.toast_text = String::new();
            }
            Message::CloseWindowRequest => {
                self.is_closing = true;
                return Task::batch([
                    self.passwords_page.closing_task(),
                    self.notes_page.closing_task(),
                    self.gallery_page.closing_task(),
                    self.tasks_page.closing_task(),
                    self.file_manager_page.closing_task(),
                    self.settings_page.closing_task(),
                ])
                .chain(window::get_latest().and_then(window::close));
            }
            Message::None => (),
        }
        Task::none()
    }
    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(|event| {
            if let Event::Window(window::Event::CloseRequested) = event {
                Message::CloseWindowRequest
            } else {
                Message::None
            }
        })
    }

    fn view(&self) -> iced::Element<Message> {
        let nav_bar = row![
            navbar_button(Page::Notes, self.current_page == Page::Notes),
            navbar_button(Page::Tasks, self.current_page == Page::Tasks),
            navbar_button(Page::Passwords, self.current_page == Page::Passwords),
            navbar_button(Page::FileManager, self.current_page == Page::FileManager),
            navbar_button(Page::Gallery, self.current_page == Page::Gallery),
            navbar_button(Page::Settings, self.current_page == Page::Settings),
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
        if !self.is_closing {
            column![
                row![
                    tool_view,
                    text(self.current_page.name())
                        .size(24)
                        .width(Length::FillPortion(1))
                        .align_x(Horizontal::Center),
                    nav_bar,
                ],
                if self.show_toast {
                    container(row![
                        // TODO Fix toast layout
                        text(&self.toast_text).width(Length::Fill),
                        button(Svg::from_path("icons/close.svg").content_fit(ContentFit::Contain))
                            .width(Length::Fixed(50.0))
                            .height(Length::Fill)
                            .on_press(Message::ToastExpired)
                    ])
                    .height(50)
                    .style(container::bordered_box) // TODO Add good/bad style
                } else {
                    container(row![]).height(10)
                },
                main_view
            ]
            .into()
        } else {
            text("Finishing up, please wait")
                .size(24)
                .width(Length::Fill)
                .align_x(Horizontal::Center)
                .into()
        }
    }
    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
