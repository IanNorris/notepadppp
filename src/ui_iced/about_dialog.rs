use iced::widget::{button, column, container, text, Space};
use iced::{Element, Length, Theme};

use super::app::Message;
use super::theme::AppColors;

const DIALOG_BG: iced::Color = iced::Color::from_rgb(0.18, 0.18, 0.22);

pub fn view_about_dialog<'a>() -> Element<'a, Message> {
    let content = column![
        text("Notepad+++").size(22).color(AppColors::TEXT),
        text(format!("Version {}", env!("CARGO_PKG_VERSION")))
            .size(13)
            .color(AppColors::TEXT_DIM),
        Space::with_height(8),
        text("A fast, native text editor for programmers")
            .size(13)
            .color(AppColors::TEXT),
        text("Built with Rust + Iced")
            .size(13)
            .color(AppColors::TEXT_DIM),
        Space::with_height(12),
        button(text("Close").size(13))
            .on_press(Message::CloseAbout)
            .padding([4, 20])
            .style(|_theme: &Theme, status| {
                let bg = match status {
                    button::Status::Hovered | button::Status::Pressed => AppColors::ACCENT,
                    _ => iced::Color::from_rgb(0.25, 0.25, 0.30),
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: AppColors::TEXT,
                    border: iced::Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }),
    ]
    .spacing(4)
    .padding(20)
    .align_x(iced::Alignment::Center);

    container(content)
        .width(Length::Fixed(320.0))
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(DIALOG_BG)),
            border: iced::Border {
                color: iced::Color::from_rgb(0.30, 0.30, 0.35),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}
