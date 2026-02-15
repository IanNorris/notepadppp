use iced::widget::{button, container, row, text, tooltip, Space};
use iced::{Element, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::AppTheme;

/// Render the toolbar row between menu bar and tab bar.
pub fn view_toolbar<'a>(state: &NotepadIced) -> Element<'a, Message> {
    let t = &state.theme;
    let t_toolbar_bg = t.tab_bar_bg;
    let t_separator = t.separator;

    let mut items = row![].spacing(2).padding([2, 4]);

    // File actions
    items = items.push(tool_button("\u{1F4C4}", "New (Ctrl+N)", Message::NewTab, t));
    items = items.push(tool_button("\u{1F4C2}", "Open (Ctrl+O)", Message::OpenFile, t));
    items = items.push(tool_button("\u{1F4BE}", "Save (Ctrl+S)", Message::Save, t));
    items = items.push(tool_button("\u{1F4BE}", "Save All", Message::SaveAll, t));
    items = items.push(vseparator(t_separator));

    // Edit actions (cut/copy/paste handled natively by the editor widget)
    items = items.push(tool_button_disabled("\u{2702}\u{FE0F}", "Cut (Ctrl+X)", t));
    items = items.push(tool_button_disabled("\u{1F4CB}", "Copy (Ctrl+C)", t));
    items = items.push(tool_button_disabled("\u{1F4CC}", "Paste (Ctrl+V)", t));
    items = items.push(vseparator(t_separator));

    // Undo / Redo
    items = items.push(tool_button("\u{21A9}\u{FE0F}", "Undo (Ctrl+Z)", Message::Undo, t));
    items = items.push(tool_button("\u{21AA}\u{FE0F}", "Redo (Ctrl+Y)", Message::Redo, t));
    items = items.push(vseparator(t_separator));

    // Search
    items = items.push(tool_button("\u{1F50D}", "Find (Ctrl+F)", Message::ShowFind, t));
    items = items.push(tool_button("\u{1F504}", "Replace (Ctrl+H)", Message::ShowReplace, t));
    items = items.push(vseparator(t_separator));

    // Zoom
    items = items.push(tool_button("\u{2B06}\u{FE0F}", "Zoom In (Ctrl+=)", Message::ZoomIn, t));
    items = items.push(tool_button("\u{2B07}\u{FE0F}", "Zoom Out (Ctrl+-)", Message::ZoomOut, t));
    items = items.push(vseparator(t_separator));

    // Toggles
    let wrap_label = if state.word_wrap { "\u{1F4D1}*" } else { "\u{1F4D1}" };
    items = items.push(tool_button(wrap_label, "Word Wrap", Message::ToggleWordWrap, t));

    let ws_label = if state.show_whitespace { "\u{1F441}\u{FE0F}*" } else { "\u{1F441}\u{FE0F}" };
    items = items.push(tool_button(ws_label, "Show Whitespace", Message::ToggleWhitespace, t));

    container(items)
        .width(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_toolbar_bg)),
            border: iced::Border {
                color: t_separator,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn tool_button<'a>(
    icon: &str,
    tip: &str,
    msg: Message,
    t: &AppTheme,
) -> Element<'a, Message> {
    let t_text = t.text;
    let t_menu_hover = t.menu_hover;
    let t_toolbar_bg = t.tab_bar_bg;

    let btn = button(text(icon.to_string()).size(14))
        .on_press(msg)
        .padding([3, 6])
        .style(move |_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => t_menu_hover,
                _ => t_toolbar_bg,
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

    let t_dialog_bg = t.dialog_bg;
    let t_tip_text = t.text;
    let t_border = t.border;

    tooltip(btn, text(tip.to_string()).size(12), tooltip::Position::Bottom)
        .gap(4)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_dialog_bg)),
            text_color: Some(t_tip_text),
            border: iced::Border {
                color: t_border,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn tool_button_disabled<'a>(
    icon: &str,
    tip: &str,
    t: &AppTheme,
) -> Element<'a, Message> {
    let t_text_dim = t.text_dim;
    let t_toolbar_bg = t.tab_bar_bg;
    let t_dialog_bg = t.dialog_bg;
    let t_border = t.border;

    let btn = button(text(icon.to_string()).size(14).color(t_text_dim))
        .padding([3, 6])
        .style(move |_theme: &Theme, _status| button::Style {
            background: Some(iced::Background::Color(t_toolbar_bg)),
            text_color: t_text_dim,
            border: iced::Border {
                radius: 3.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    tooltip(btn, text(tip.to_string()).size(12), tooltip::Position::Bottom)
        .gap(4)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_dialog_bg)),
            text_color: Some(t_text_dim),
            border: iced::Border {
                color: t_border,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn vseparator<'a>(color: iced::Color) -> Element<'a, Message> {
    container(Space::with_width(1))
        .height(Length::Fixed(20.0))
        .padding([0, 3])
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(color)),
            ..Default::default()
        })
        .into()
}
