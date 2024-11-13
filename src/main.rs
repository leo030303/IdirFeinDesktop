use iced::{
    window::{self, Settings},
    Size,
};
use idirfein_desktop_iced::app::AppState;

pub fn main() -> iced::Result {
    iced::application("IdirFéin Desktop", AppState::update, AppState::view)
        .theme(AppState::theme)
        .window(Settings {
            icon: Some(
                window::icon::from_file_data(
                    include_bytes!("../logos/idirfein_logo_1.jpg"),
                    Some(iced::advanced::graphics::image::image_rs::ImageFormat::Jpeg),
                )
                .expect("Missing the app icon"),
            ),
            ..Default::default()
        })
        .exit_on_close_request(false)
        .default_font(iced::Font::with_name("Cantarell"))
        .window_size(Size::INFINITY)
        .subscription(AppState::subscription)
        .run_with(AppState::new)
}
