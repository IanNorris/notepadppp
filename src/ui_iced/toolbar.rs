// Toolbar icons from Tabler Icons (MIT license) — https://github.com/tabler/tabler-icons
use iced::widget::{button, container, row, svg, text, tooltip, Space};
use iced::{Color, Element, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::AppTheme;

const ICON_SIZE: f32 = 18.0;

/// Render the toolbar row between menu bar and tab bar.
pub fn view_toolbar<'a>(state: &NotepadIced) -> Element<'a, Message> {
    let t = &state.theme;
    let t_toolbar_bg = t.tab_bar_bg;
    let t_separator = t.separator;

    let mut items = row![].spacing(2).padding([2, 4]);

    // File actions
    items = items.push(tool_button(include_bytes!("../../assets/icons/file-plus.svg"), "New (Ctrl+N)", Message::NewTab, t));
    items = items.push(tool_button(include_bytes!("../../assets/icons/folder-open.svg"), "Open (Ctrl+O)", Message::OpenFile, t));
    items = items.push(tool_button(include_bytes!("../../assets/icons/device-floppy.svg"), "Save (Ctrl+S)", Message::Save, t));
    items = items.push(tool_button(include_bytes!("../../assets/icons/device-floppy.svg"), "Save All", Message::SaveAll, t));
    items = items.push(vseparator(t_separator));

    // Edit actions (cut/copy/paste handled natively by the editor widget)
    items = items.push(tool_button_disabled(include_bytes!("../../assets/icons/cut.svg"), "Cut (Ctrl+X)", t));
    items = items.push(tool_button_disabled(include_bytes!("../../assets/icons/copy.svg"), "Copy (Ctrl+C)", t));
    items = items.push(tool_button_disabled(include_bytes!("../../assets/icons/clipboard.svg"), "Paste (Ctrl+V)", t));
    items = items.push(vseparator(t_separator));

    // Undo / Redo
    items = items.push(tool_button(include_bytes!("../../assets/icons/arrow-back-up.svg"), "Undo (Ctrl+Z)", Message::Undo, t));
    items = items.push(tool_button(include_bytes!("../../assets/icons/arrow-forward-up.svg"), "Redo (Ctrl+Y)", Message::Redo, t));
    items = items.push(vseparator(t_separator));

    // Search
    items = items.push(tool_button(include_bytes!("../../assets/icons/search.svg"), "Find (Ctrl+F)", Message::ShowFind, t));
    items = items.push(tool_button(include_bytes!("../../assets/icons/replace.svg"), "Replace (Ctrl+H)", Message::ShowReplace, t));
    items = items.push(vseparator(t_separator));

    // Zoom
    items = items.push(tool_button(include_bytes!("../../assets/icons/zoom-in.svg"), "Zoom In (Ctrl+=)", Message::ZoomIn, t));
    items = items.push(tool_button(include_bytes!("../../assets/icons/zoom-out.svg"), "Zoom Out (Ctrl+-)", Message::ZoomOut, t));
    items = items.push(vseparator(t_separator));

    // Toggles
    items = items.push(tool_button_toggle(include_bytes!("../../assets/icons/text-wrap.svg"), "Word Wrap", state.word_wrap, Message::ToggleWordWrap, t));
    items = items.push(tool_button_toggle(include_bytes!("../../assets/icons/eye.svg"), "Show Whitespace", state.show_whitespace, Message::ToggleWhitespace, t));

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

fn icon_svg(bytes: &'static [u8], color: Color) -> svg::Svg<'static, Theme> {
    let handle = svg::Handle::from_memory(bytes);
    svg(handle)
        .width(ICON_SIZE)
        .height(ICON_SIZE)
        .style(move |_theme: &Theme, _status| svg::Style {
            color: Some(color),
        })
}

fn tool_button<'a>(
    icon_bytes: &'static [u8],
    tip: &str,
    msg: Message,
    t: &AppTheme,
) -> Element<'a, Message> {
    let t_text = t.text;
    let t_menu_hover = t.menu_hover;
    let t_toolbar_bg = t.tab_bar_bg;

    let btn = button(icon_svg(icon_bytes, t_text))
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
    icon_bytes: &'static [u8],
    tip: &str,
    t: &AppTheme,
) -> Element<'a, Message> {
    let t_text_dim = t.text_dim;
    let t_toolbar_bg = t.tab_bar_bg;
    let t_dialog_bg = t.dialog_bg;
    let t_border = t.border;

    let btn = button(icon_svg(icon_bytes, t_text_dim))
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

fn tool_button_toggle<'a>(
    icon_bytes: &'static [u8],
    tip: &str,
    active: bool,
    msg: Message,
    t: &AppTheme,
) -> Element<'a, Message> {
    let icon_color = if active { t.accent } else { t.text };
    let t_text = t.text;
    let t_menu_hover = t.menu_hover;
    let t_toolbar_bg = t.tab_bar_bg;
    let t_accent = t.accent;

    let btn = button(icon_svg(icon_bytes, icon_color))
        .on_press(msg)
        .padding([3, 6])
        .style(move |_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => t_menu_hover,
                _ => if active { t_accent.scale_alpha(0.15) } else { t_toolbar_bg },
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
