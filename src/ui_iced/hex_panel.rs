use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Element, Font, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::AppColors;
use crate::tools::hex_viewer::HexView;

const PANEL_BG: iced::Color = iced::Color::from_rgb(0.12, 0.12, 0.14);
const HEADER_BG: iced::Color = iced::Color::from_rgb(0.14, 0.14, 0.17);
const OFFSET_COLOR: iced::Color = iced::Color::from_rgb(0.45, 0.50, 0.55);
const HEX_COLOR: iced::Color = iced::Color::from_rgb(0.82, 0.82, 0.82);
const ASCII_COLOR: iced::Color = iced::Color::from_rgb(0.40, 0.82, 0.82);
const SEPARATOR: iced::Color = iced::Color::from_rgb(0.25, 0.25, 0.30);

pub fn view_hex_viewer<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let content_text = state
        .tab_contents
        .get(state.tab_manager.active_index())
        .map(|tc| tc.content.text())
        .unwrap_or_default();

    let hex_view = HexView::from_text(&content_text);
    let bytes_per_row: usize = 16;

    // Header
    let toolbar = container(
        row![
            text("Hex Viewer").size(13).color(AppColors::TEXT),
            Space::with_width(Length::Fill),
            text(format!("{} bytes", hex_view.bytes.len()))
                .size(12)
                .color(AppColors::TEXT_DIM),
        ]
        .align_y(iced::Alignment::Center)
        .padding([6, 10]),
    )
    .width(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(HEADER_BG)),
        ..Default::default()
    });

    // Column headers
    let mut col_header_parts: Vec<Element<'a, Message>> = Vec::new();
    col_header_parts.push(
        text("  Offset  ")
            .size(11)
            .color(OFFSET_COLOR)
            .font(Font::MONOSPACE)
            .into(),
    );
    col_header_parts.push(Space::with_width(8).into());

    let mut hex_hdr = String::new();
    for i in 0..bytes_per_row {
        if i == 8 {
            hex_hdr.push(' ');
        }
        hex_hdr.push_str(&format!("{:02X} ", i));
    }
    col_header_parts.push(
        text(hex_hdr)
            .size(11)
            .color(OFFSET_COLOR)
            .font(Font::MONOSPACE)
            .into(),
    );
    col_header_parts.push(Space::with_width(8).into());
    col_header_parts.push(
        text("ASCII")
            .size(11)
            .color(OFFSET_COLOR)
            .font(Font::MONOSPACE)
            .into(),
    );

    let col_header = container(
        row(col_header_parts).align_y(iced::Alignment::Center),
    )
    .padding([4, 10])
    .width(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(PANEL_BG)),
        border: iced::Border {
            color: SEPARATOR,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    });

    // Hex rows
    let row_count = if hex_view.bytes.is_empty() {
        0
    } else {
        (hex_view.bytes.len() + bytes_per_row - 1) / bytes_per_row
    };

    let mut hex_rows = column![].spacing(0);

    for r in 0..row_count {
        let row_start = r * bytes_per_row;
        let row_end = (row_start + bytes_per_row).min(hex_view.bytes.len());

        // Offset
        let offset_str = format!("{:08X}", row_start);

        // Hex bytes
        let mut hex_str = String::new();
        for i in 0..bytes_per_row {
            if i == 8 {
                hex_str.push(' ');
            }
            let idx = row_start + i;
            if idx < row_end {
                hex_str.push_str(&format!("{:02X} ", hex_view.bytes[idx]));
            } else {
                hex_str.push_str("   ");
            }
        }

        // ASCII
        let mut ascii_str = String::new();
        for i in row_start..row_start + bytes_per_row {
            if i < row_end {
                let b = hex_view.bytes[i];
                if b.is_ascii_graphic() || b == b' ' {
                    ascii_str.push(b as char);
                } else {
                    ascii_str.push('.');
                }
            }
        }

        let hex_row = row![
            text(offset_str).size(12).color(OFFSET_COLOR).font(Font::MONOSPACE),
            Space::with_width(8),
            text(hex_str).size(12).color(HEX_COLOR).font(Font::MONOSPACE),
            Space::with_width(8),
            text(ascii_str).size(12).color(ASCII_COLOR).font(Font::MONOSPACE),
        ]
        .align_y(iced::Alignment::Center);

        hex_rows = hex_rows.push(
            container(hex_row).padding([1, 10]),
        );
    }

    if row_count == 0 {
        hex_rows = hex_rows.push(
            container(text("Empty document").size(13).color(AppColors::TEXT_DIM))
                .padding([10, 10]),
        );
    }

    let body = scrollable(hex_rows).height(Length::Fill);

    let panel = column![toolbar, col_header, body];

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(PANEL_BG)),
            ..Default::default()
        })
        .into()
}
