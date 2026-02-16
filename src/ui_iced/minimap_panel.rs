use iced::widget::{column, container, mouse_area, scrollable};
use iced::{Element, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::AppTheme;

/// Render a minimap panel — a narrow, zoomed-out view of the source code
/// using colored blocks instead of unreadable tiny text.
pub fn view_minimap<'a>(state: &'a NotepadIced, theme: &AppTheme) -> Element<'a, Message> {
    let panel_bg = theme.tab_bar_bg;
    let text_color = theme.text_dim;
    let border_color = iced::Color::from_rgb(0.25, 0.25, 0.30);
    let keyword_color = theme.accent;

    let content_text = state
        .tab_contents
        .get(state.tab_manager.active_index())
        .map(|tc| tc.content.text())
        .unwrap_or_default();

    let max_lines = 5000;
    let line_height = 1.5_f32;
    let max_width = 80.0_f32;

    let mut col = column![].spacing(0);
    for (i, line) in content_text.lines().enumerate() {
        if i >= max_lines {
            break;
        }
        let width = ((line.len() as f32) * 0.8).min(max_width).max(2.0);
        let line_trimmed = line.trim();

        let color = if line_trimmed.is_empty() {
            iced::Color::TRANSPARENT
        } else if line_trimmed.starts_with("fn ")
            || line_trimmed.starts_with("pub ")
            || line_trimmed.starts_with("def ")
            || line_trimmed.starts_with("class ")
            || line_trimmed.starts_with("function ")
            || line_trimmed.starts_with("struct ")
        {
            keyword_color
        } else {
            text_color
        };

        let line_idx = i;
        let bar: Element<'a, Message> = mouse_area(
            container(iced::widget::Space::new(
                Length::Fixed(width),
                Length::Fixed(line_height),
            ))
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(color)),
                ..Default::default()
            }),
        )
        .on_press(Message::GotoSymbol(line_idx))
        .into();

        col = col.push(bar);
    }

    let scrollable_minimap = scrollable(col).height(Length::Fill);

    container(scrollable_minimap)
        .width(Length::Fixed(100.0))
        .height(Length::Fill)
        .clip(true)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(panel_bg)),
            border: iced::Border {
                color: border_color,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}
