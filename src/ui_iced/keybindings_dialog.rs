use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Theme};

use crate::io::keybindings::KeyBindings;

use super::app::Message;
use super::theme::AppColors;

const DIALOG_BG: iced::Color = iced::Color::from_rgb(0.18, 0.18, 0.22);
const ROW_ALT_BG: iced::Color = iced::Color::from_rgb(0.16, 0.16, 0.20);

pub fn view_keybindings_dialog<'a>() -> Element<'a, Message> {
    let kb = KeyBindings::defaults();
    let actions = kb.all_actions_sorted();

    let title = text("Keyboard Shortcuts").size(18).color(AppColors::TEXT);

    let close_x = button(text("x").size(14))
        .on_press(Message::CloseKeybindingsDialog)
        .padding([2, 6])
        .style(|_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => AppColors::CLOSE_HOVER,
                _ => iced::Color::TRANSPARENT,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: AppColors::TEXT,
                border: iced::Border::default(),
                ..Default::default()
            }
        });

    let header = row![
        title,
        Space::with_width(Length::Fill),
        close_x,
    ]
    .align_y(iced::Alignment::Center);

    // Column headers
    let col_header = row![
        container(
            text("Action").size(12).color(AppColors::TEXT_DIM)
        )
        .width(Length::Fixed(220.0)),
        container(
            text("Shortcut").size(12).color(AppColors::TEXT_DIM)
        )
        .width(Length::Fill),
    ]
    .spacing(8)
    .padding([4, 0]);

    // Build rows
    let mut rows = column![].spacing(0);
    for (i, (_action, display_name, shortcut)) in actions.iter().enumerate() {
        let bg = if i % 2 == 0 { DIALOG_BG } else { ROW_ALT_BG };
        let name = display_name.clone();
        let sc = shortcut.clone();

        let r = container(
            row![
                container(
                    text(name).size(13).color(AppColors::TEXT)
                )
                .width(Length::Fixed(220.0)),
                container(
                    text(sc).size(13).color(AppColors::ACCENT)
                )
                .width(Length::Fill),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .padding([4, 8])
        .width(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(bg)),
            ..Default::default()
        });

        rows = rows.push(r);
    }

    let scrollable_list = scrollable(rows)
        .height(Length::Fixed(400.0))
        .width(Length::Fill);

    let content = column![
        header,
        Space::with_height(8),
        col_header,
        scrollable_list,
    ]
    .spacing(4)
    .padding(16)
    .width(Length::Fixed(420.0));

    container(content)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(DIALOG_BG)),
            border: iced::Border {
                color: iced::Color::from_rgb(0.30, 0.30, 0.35),
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
