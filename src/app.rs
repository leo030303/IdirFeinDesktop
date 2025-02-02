use std::time::Duration;

use iced::{
    alignment::Horizontal,
    event,
    keyboard::{self, Key, Modifiers},
    widget::{self, button, column, container, row, svg, text, tooltip::Position, Svg, Tooltip},
    window, Alignment, ContentFit, Element, Event, Length, Subscription, Task, Theme,
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
        setup_wizard::page::{SetupWizard, SetupWizardMessage},
        sync::page::{SyncPage, SyncPageMessage},
        tasks::page::{TasksPage, TasksPageMessage},
    },
    utils::socket_utils::{self, ServerMessage},
    Page,
};

#[derive(Debug, Clone)]
pub enum Message {
    ChangePage(Page),
    CloseWindowRequest,
    None,
    SetupWizard(SetupWizardMessage),
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
    FinishSetup,
}

pub struct AppState {
    config: AppConfig,
    is_setting_up_server: bool,
    is_closing: bool,
    current_page: Page,
    setup_wizard: SetupWizard,
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
        let config_option = load_settings_from_file();
        let is_setting_up_server = config_option.is_none();
        let config = config_option.unwrap_or_default();
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
                is_setting_up_server,
                setup_wizard: SetupWizard::new(),
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
            Message::SetupWizard(m) => return self.setup_wizard.update(m),
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
            Message::FinishSetup => {
                self.is_setting_up_server = false;
                self.config = self.setup_wizard.work_in_progress_client_config.clone();
                return Task::done(Message::SaveConfig);
            }
            Message::None => (),
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions_vec = vec![];
        subscriptions_vec.push(event::listen_with(|event, status, _id| {
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
        }));
        if !self.is_setting_up_server {
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
            if self.config.sync_config.should_sync {
                subscriptions_vec.push(
                    Subscription::run_with_id(
                        "server_connection_subscription",
                        socket_utils::connect(
                            self.config.sync_config.server_url.clone(),
                            self.sync_page.folders_to_sync.clone(),
                            self.sync_page.ignore_string_list.clone(),
                            self.config.sync_config.default_data_storage_folder.clone(),
                            self.config.sync_config.ignored_remote_folder_ids.clone(),
                        ),
                    )
                    .map(Message::ServerMessageEvent),
                );
            }
        }
        Subscription::batch(subscriptions_vec)
    }

    pub fn view(&self) -> Element<Message> {
        let nav_bar = row![
            navbar_button(self, Page::Notes, self.current_page == Page::Notes, 1),
            navbar_button(self, Page::Tasks, self.current_page == Page::Tasks, 2),
            navbar_button(
                self,
                Page::Passwords,
                self.current_page == Page::Passwords,
                3
            ),
            navbar_button(self, Page::Sync, self.current_page == Page::Sync, 4),
            navbar_button(self, Page::Gallery, self.current_page == Page::Gallery, 5),
            navbar_button(self, Page::Settings, self.current_page == Page::Settings, 6),
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
            if self.is_setting_up_server {
                column![
                    if self.show_toast {
                        toast_widget(self)
                    } else {
                        container(row![]).height(10).into()
                    },
                    self.setup_wizard.view()
                ]
                .into()
            } else {
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
                        toast_widget(self)
                    } else {
                        container(row![]).height(10).into()
                    },
                    main_view
                ]
                .into()
            }
        } else {
            app_is_closing_view(self)
        }
    }
    pub fn theme(&self) -> Theme {
        self.config.get_theme().unwrap_or(Theme::Light)
    }
}

fn toast_widget(state: &AppState) -> Element<Message> {
    container(
        row![
            text(&state.toast_text)
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
            .style(if state.is_good_toast {
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
        if state.is_good_toast {
            container::Style::default()
                .background(theme.extended_palette().success.base.color)
                .color(theme.extended_palette().success.base.text)
        } else {
            container::Style::default()
                .background(theme.extended_palette().danger.base.color)
                .color(theme.extended_palette().danger.base.text)
        }
    })
    .into()
}

fn app_is_closing_view(_state: &AppState) -> Element<Message> {
    text("Finishing up, please wait")
        .size(24)
        .width(Length::Fill)
        .align_x(Horizontal::Center)
        .into()
}
fn navbar_button(
    _state: &AppState,
    page: Page,
    selected: bool,
    index: u8,
) -> iced::Element<Message> {
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
