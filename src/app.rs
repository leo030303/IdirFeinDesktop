use std::time::Duration;

use iced::{
    alignment::Horizontal,
    event,
    keyboard::{self, Key, Modifiers},
    widget::{self, button, column, container, row, svg, text, tooltip::Position, Svg, Tooltip},
    window, Alignment, ContentFit, Event, Length, Subscription, Task, Theme,
};

use crate::{
    config::AppConfig,
    pages::{
        gallery::page::{GalleryPage, GalleryPageMessage},
        notes::page::{NotesPage, NotesPageMessage},
        passwords::page::{PasswordsPage, PasswordsPageMessage},
        settings::{
            page::{SettingsPage, SettingsPageMessage},
            settings_utils::{load_settings_from_file, save_settings_to_file},
        },
        sync::page::{SyncPage, SyncPageMessage},
        tasks::page::{TasksPage, TasksPageMessage},
    },
    utils::socket_utils::{self, ServerMessage},
    Page,
};

pub const APP_ID: &str = "idirfein_desktop";

#[derive(Debug, Clone)]
pub enum Message {
    ChangePage(Page),
    CloseWindowRequest,
    None,
    Passwords(PasswordsPageMessage),
    Notes(NotesPageMessage),
    Tasks(TasksPageMessage),
    Gallery(GalleryPageMessage),
    Sync(SyncPageMessage),
    Settings(SettingsPageMessage),
    ShowToast(bool, String),
    ToastExpired,
    SaveConfig,
    ServerMessageEvent(socket_utils::Event),
    SendServerMessage(String),
}

pub struct AppState {
    config: AppConfig,
    is_closing: bool,
    current_page: Page,
    notes_page: NotesPage,
    passwords_page: PasswordsPage,
    tasks_page: TasksPage,
    settings_page: SettingsPage,
    sync_page: SyncPage,
    gallery_page: GalleryPage,
    show_toast: bool,
    is_good_toast: bool,
    toast_text: String,
    server_connection_state: ServerConnectionState,
}

impl AppState {
    pub fn new() -> (Self, Task<Message>) {
        let config = load_settings_from_file();
        (
            Self {
                config: config.clone(),
                current_page: config.default_page_on_open.clone(),
                is_closing: false,
                notes_page: NotesPage::new(&config.notes_config),
                passwords_page: PasswordsPage::new(&config.passwords_config),
                tasks_page: TasksPage::new(&config.tasks_config),
                settings_page: SettingsPage::new(&config),
                sync_page: SyncPage::new(&config.sync_config),
                gallery_page: GalleryPage::new(&config.gallery_config),
                show_toast: false,
                is_good_toast: true,
                toast_text: String::new(),
                server_connection_state: ServerConnectionState::Disconnected,
            },
            Task::batch([
                widget::focus_next(),
                NotesPage::opening_task(),
                TasksPage::opening_task(),
                PasswordsPage::opening_task(),
                SettingsPage::opening_task(),
                GalleryPage::opening_task(),
                SyncPage::opening_task(),
            ]),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChangePage(new_page) => self.current_page = new_page,
            Message::Passwords(m) => return self.passwords_page.update(m),
            Message::Notes(m) => return self.notes_page.update(m),
            Message::Tasks(m) => return self.tasks_page.update(m),
            Message::Gallery(m) => return self.gallery_page.update(m),
            Message::Sync(m) => return self.sync_page.update(m),
            Message::Settings(m) => return self.settings_page.update(m, &mut self.config),
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
                    self.sync_page.closing_task(),
                    self.settings_page.closing_task(),
                ])
                .chain(window::get_latest().and_then(window::close));
            }
            Message::SaveConfig => {
                return Task::perform(save_settings_to_file(self.config.clone()), |success| {
                    Message::Settings(SettingsPageMessage::ResultFromSave(success))
                });
            }
            Message::ServerMessageEvent(event) => match event {
                socket_utils::Event::Connected(connection) => {
                    self.server_connection_state = ServerConnectionState::Connected(connection);
                    self.sync_page.is_connected_to_server = true;
                }
                socket_utils::Event::Disconnected => {
                    self.server_connection_state = ServerConnectionState::Disconnected;
                    self.sync_page.is_connected_to_server = false;
                }
                socket_utils::Event::MessageReceived(message) => {
                    println!("Recieved update: {message:?}");
                }
            },
            Message::SendServerMessage(message_string) => {
                if let ServerConnectionState::Connected(connection) =
                    &mut self.server_connection_state
                {
                    connection.send(ServerMessage::User(message_string));
                } else {
                    println!("Disconnected");
                }
            }
            Message::None => (),
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions_vec = vec![event::listen_with(|event, status, _id| {
            match (event, status) {
                (Event::Window(window::Event::CloseRequested), _) => {
                    Some(Message::CloseWindowRequest)
                }
                (
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key: Key::Character(pressed_char),
                        modifiers: Modifiers::ALT,
                        ..
                    }),
                    _,
                ) => {
                    if pressed_char.as_ref() == "1" {
                        Some(Message::ChangePage(Page::Notes))
                    } else if pressed_char.as_ref() == "2" {
                        Some(Message::ChangePage(Page::Tasks))
                    } else if pressed_char.as_ref() == "3" {
                        Some(Message::ChangePage(Page::Passwords))
                    } else if pressed_char.as_ref() == "4" {
                        Some(Message::ChangePage(Page::Sync))
                    } else if pressed_char.as_ref() == "5" {
                        Some(Message::ChangePage(Page::Gallery))
                    } else if pressed_char.as_ref() == "6" {
                        Some(Message::ChangePage(Page::Settings))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })];
        match self.current_page {
            Page::Settings => (),
            Page::Passwords => {
                subscriptions_vec.push(PasswordsPage::subscription());
            }
            Page::Sync => (),
            Page::Gallery => {
                subscriptions_vec.push(GalleryPage::subscription());
            }
            Page::Notes => {
                subscriptions_vec.push(NotesPage::subscription());
            }
            Page::Tasks => {
                subscriptions_vec.push(TasksPage::subscription());
            }
        }
        subscriptions_vec.push(
            Subscription::run_with_id(
                "server_connection_subscription",
                socket_utils::connect(self.config.sync_config.server_url.clone()),
            )
            .map(Message::ServerMessageEvent),
        );
        Subscription::batch(subscriptions_vec)
    }

    pub fn view(&self) -> iced::Element<Message> {
        let nav_bar = row![
            navbar_button(Page::Notes, self.current_page == Page::Notes, 1),
            navbar_button(Page::Tasks, self.current_page == Page::Tasks, 2),
            navbar_button(Page::Passwords, self.current_page == Page::Passwords, 3),
            navbar_button(Page::Sync, self.current_page == Page::Sync, 4),
            navbar_button(Page::Gallery, self.current_page == Page::Gallery, 5),
            navbar_button(Page::Settings, self.current_page == Page::Settings, 6),
        ]
        .width(Length::FillPortion(1));

        let tool_view = match self.current_page {
            Page::Settings => self.settings_page.tool_view(),
            Page::Passwords => self.passwords_page.tool_view(),
            Page::Sync => self.sync_page.tool_view(),
            Page::Gallery => self.gallery_page.tool_view(),
            Page::Notes => self.notes_page.tool_view(),
            Page::Tasks => self.tasks_page.tool_view(),
        };

        let main_view = match self.current_page {
            Page::Settings => self.settings_page.view(&self.config),
            Page::Passwords => self.passwords_page.view(),
            Page::Sync => self.sync_page.view(),
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
                    container(
                        row![
                            text(&self.toast_text)
                                .width(Length::Fill)
                                .align_x(Alignment::Center)
                                .size(18),
                            button(
                                Svg::new(svg::Handle::from_memory(include_bytes!(
                                    "../icons/close.svg"
                                )))
                                .content_fit(ContentFit::Contain)
                            )
                            .width(Length::Fixed(50.0))
                            .height(Length::Fill)
                            .style(if self.is_good_toast {
                                button::success
                            } else {
                                button::danger
                            })
                            .on_press(Message::ToastExpired)
                        ]
                        .padding(10),
                    )
                    .height(50)
                    .style(container::bordered_box)
                    .style(|theme| {
                        if self.is_good_toast {
                            container::Style::default()
                                .background(theme.extended_palette().success.base.color)
                                .color(theme.extended_palette().success.base.text)
                        } else {
                            container::Style::default()
                                .background(theme.extended_palette().danger.base.color)
                                .color(theme.extended_palette().danger.base.text)
                        }
                    })
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
        self.config.get_theme().unwrap_or(Theme::Light)
    }
}

fn navbar_button(page: Page, selected: bool, index: u8) -> iced::Element<'static, Message> {
    Tooltip::new(
        button(Svg::new(page.icon_handle()).content_fit(ContentFit::ScaleDown))
            .style(if selected {
                button::secondary
            } else {
                button::primary
            })
            .on_press(Message::ChangePage(page.clone())),
        text(format!("{} (Alt+{})", page.name(), index)),
        Position::Bottom,
    )
    .into()
}

enum ServerConnectionState {
    Disconnected,
    Connected(socket_utils::Connection),
}
