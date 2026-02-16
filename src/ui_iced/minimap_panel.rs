use iced::widget::{column, container, mouse_area, text, scrollable};
use iced::{Element, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::AppTheme;

/// Render a minimap panel — a narrow, zoomed-out view of the source code.
pub fn view_minimap<'a>(state: &'a NotepadIced, theme: &AppTheme) -> Element<'a, Message> {
    let panel_bg = theme.tab_bar_bg;
    let text_color = theme.text_dim;
    let border_color = iced::Color::from_rgb(0.25, 0.25, 0.30);

    let content_text = state
        .tab_contents
        .get(state.tab_manager.active_index())
        .map(|tc| tc.content.text())
        .unwrap_or_default();

    let minimap_font_size = 2.0_f32;
    let max_lines = 5000;

    let mut col = column![].spacing(0);
    for (i, line) in content_text.lines().enumerate() {
        if i >= max_lines {
            col = col.push(text(String::from("...")).size(minimap_font_size).color(text_color));
            break;
        }
        let display = if line.len() > 200 {
            line[..200].to_string()
        } else if line.is_empty() {
            " ".to_string()
        } else {
            line.to_string()
        };
        let line_idx = i;

        let line_text = text(display)
            .size(minimap_font_size)
            .color(text_color);

        let line_widget: Element<'a, Message> = mouse_area(line_text)
            .on_press(Message::GotoSymbol(line_idx))
            .into();

        col = col.push(line_widget);
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
