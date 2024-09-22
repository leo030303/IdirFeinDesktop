use iced::Size;
use idirfein_desktop_iced::app::AppState;

pub fn main() -> iced::Result {
    iced::application("IdirFéin Desktop", AppState::update, AppState::view)
        .theme(AppState::theme)
        .exit_on_close_request(false)
        .window_size(Size::INFINITY)
        .subscription(AppState::subscription)
        .run_with(AppState::new)
}
