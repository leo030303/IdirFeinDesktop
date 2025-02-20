use iced::{Settings, Size};
use idirfein::{app::AppState, constants::APP_ID};

pub fn main() -> iced::Result {
    iced::application("IdirFéin Desktop", AppState::update, AppState::view)
        .settings(Settings {
            id: Some(APP_ID.to_owned()),
            ..Default::default()
        })
        .theme(AppState::theme)
        .exit_on_close_request(false)
        .default_font(iced::Font::with_name("Cantarell"))
        .window_size(Size::INFINITY)
        .subscription(AppState::subscription)
        .run_with(AppState::new)
}
