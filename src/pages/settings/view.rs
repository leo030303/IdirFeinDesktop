use iced::{
    border,
    widget::{button, column, row, text},
    Background, Element,
    Length::{self},
    Theme,
};
use iced_aw::Spinner;

use crate::app::Message;
use crate::config::AppConfig;

use super::page::{SettingsPage, SettingsPageMessage};

pub fn main_view<'a>(state: &'a SettingsPage, app_config: &'a AppConfig) -> Element<'a, Message> {
    column![
        text("General Settings").size(24),
        if state.is_saving {
            row![text("Settings are being saved"), Spinner::new()]
        } else if state.save_was_successful {
            row![text(&state.save_message).style(text::success)]
        } else {
            row![text(&state.save_message).style(text::danger)]
        },
        theme_picker(state)
    ]
    .into()
}

fn theme_picker(_state: &SettingsPage) -> Element<Message> {
    row(Theme::ALL.iter().map(|theme| {
        button(text(format!("{theme:?}")))
            .on_press(Message::Settings(SettingsPageMessage::SetTheme(
                theme.clone(),
            )))
            .style(|_, status| {
                let palette = theme.extended_palette();
                let base = button::Style {
                    background: Some(Background::Color(palette.primary.strong.color)),
                    text_color: palette.primary.strong.text,
                    border: border::rounded(2),
                    ..button::Style::default()
                };

                match status {
                    button::Status::Active | button::Status::Pressed => base,
                    button::Status::Hovered => button::Style {
                        background: Some(Background::Color(palette.primary.base.color)),
                        ..base
                    },
                    button::Status::Disabled => base,
                }
            })
            .into()
    }))
    .spacing(20)
    .padding(20)
    .width(Length::Fill)
    .wrap()
    .into()
}

pub fn tool_view(_state: &SettingsPage) -> Element<Message> {
    row![].width(Length::FillPortion(1)).into()
}
