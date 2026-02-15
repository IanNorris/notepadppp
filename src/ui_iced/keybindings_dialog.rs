use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Theme};

use crate::io::keybindings::KeyBindings;

use super::app::Message;
use super::theme::AppTheme;

pub fn view_keybindings_dialog<'a>(theme: &AppTheme) -> Element<'a, Message> {
    let t_text = theme.text;
    let t_text_dim = theme.text_dim;
    let t_accent = theme.accent;
    let t_close_hover = theme.close_hover;
    let t_dialog_bg = theme.dialog_bg;
    let t_border = theme.border;
    let t_menu_bg = theme.menu_bg;

    let kb = KeyBindings::defaults();
    let actions = kb.all_actions_sorted();

    let title = text("Keyboard Shortcuts").size(18).color(t_text);

    let close_x = button(text("x").size(14))
        .on_press(Message::CloseKeybindingsDialog)
        .padding([2, 6])
        .style(move |_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => t_close_hover,
                _ => iced::Color::TRANSPARENT,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: t_text,
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
            text("Action").size(12).color(t_text_dim)
        )
        .width(Length::Fixed(220.0)),
        container(
            text("Shortcut").size(12).color(t_text_dim)
        )
        .width(Length::Fill),
    ]
    .spacing(8)
    .padding([4, 0]);

    // Build rows
    let mut rows = column![].spacing(0);
    for (i, (_action, display_name, shortcut)) in actions.iter().enumerate() {
        let bg = if i % 2 == 0 { t_dialog_bg } else { t_menu_bg };
        let name = display_name.clone();
        let sc = shortcut.clone();

        let r = container(
            row![
                container(
                    text(name).size(13).color(t_text)
                )
                .width(Length::Fixed(220.0)),
                container(
                    text(sc).size(13).color(t_accent)
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
