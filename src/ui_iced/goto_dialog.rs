use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Element, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::AppTheme;

pub fn view_goto_dialog<'a>(state: &NotepadIced, theme: &AppTheme) -> Element<'a, Message> {
    let t_text = theme.text;
    let t_accent = theme.accent;
    let t_button_bg = theme.button_bg;
    let t_dialog_bg = theme.dialog_bg;
    let t_border = theme.border;

    let label = text("Go to Line:").size(14).color(t_text);

    let input = text_input("Line number", &state.goto_line_input)
        .on_input(Message::GotoLineInputChanged)
        .on_submit(Message::GotoLineConfirm)
        .size(14)
        .width(Length::Fixed(200.0));

    let go_btn = button(text("Go").size(13))
        .on_press(Message::GotoLineConfirm)
        .padding([4, 16])
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
        });

    let t_menu_hover = theme.menu_hover;
    let cancel_btn = button(text("Cancel").size(13))
        .on_press(Message::GotoLineClose)
        .padding([4, 12])
        .style(move |_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => t_menu_hover,
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
        });

    let content = column![
        label,
        row![input, Space::with_width(8), go_btn, cancel_btn]
            .spacing(4)
            .align_y(iced::Alignment::Center),
    ]
    .spacing(6)
    .padding(12);

    container(content)
        .width(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_dialog_bg)),
            border: iced::Border {
                color: t_border,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}
