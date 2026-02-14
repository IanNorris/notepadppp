use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Element, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::AppColors;

const DIALOG_BG: iced::Color = iced::Color::from_rgb(0.18, 0.18, 0.22);
const BUTTON_BG: iced::Color = iced::Color::from_rgb(0.25, 0.25, 0.30);

pub fn view_goto_dialog<'a>(state: &NotepadIced) -> Element<'a, Message> {
    let label = text("Go to Line:").size(14).color(AppColors::TEXT);

    let input = text_input("Line number", &state.goto_line_input)
        .on_input(Message::GotoLineInputChanged)
        .on_submit(Message::GotoLineConfirm)
        .size(14)
        .width(Length::Fixed(200.0));

    let go_btn = button(text("Go").size(13))
        .on_press(Message::GotoLineConfirm)
        .padding([4, 16])
        .style(|_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => AppColors::ACCENT,
                _ => BUTTON_BG,
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
        });

    let cancel_btn = button(text("Cancel").size(13))
        .on_press(Message::GotoLineClose)
        .padding([4, 12])
        .style(|_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => {
                    iced::Color::from_rgb(0.30, 0.30, 0.35)
                }
                _ => BUTTON_BG,
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
