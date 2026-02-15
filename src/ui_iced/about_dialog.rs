use iced::widget::{button, column, container, text, Space};
use iced::{Element, Length, Theme};

use super::app::Message;
use super::theme::AppTheme;

pub fn view_about_dialog<'a>(theme: &AppTheme) -> Element<'a, Message> {
    let t_text = theme.text;
    let t_text_dim = theme.text_dim;
    let t_accent = theme.accent;
    let t_button_bg = theme.button_bg;
    let t_dialog_bg = theme.dialog_bg;
    let t_border = theme.border;

    let content = column![
        text("Notepad+++").size(22).color(t_text),
        text(format!("Version {}", env!("CARGO_PKG_VERSION")))
            .size(13)
            .color(t_text_dim),
        Space::with_height(8),
        text("A fast, native text editor for programmers")
            .size(13)
            .color(t_text),
        text("Built with Rust + Iced")
            .size(13)
            .color(t_text_dim),
        Space::with_height(12),
        button(text("Close").size(13))
            .on_press(Message::CloseAbout)
            .padding([4, 20])
            .style(move |_theme: &Theme, status| {
                let bg = match status {
                    button::Status::Hovered | button::Status::Pressed => t_accent,
                    _ => t_button_bg,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: t_text,
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
        .align_x(iced::alignment::Horizontal::Center)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_dialog_bg)),
            border: iced::Border {
                color: t_border,
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
            ..Default::default()
        })
        .into()
}
