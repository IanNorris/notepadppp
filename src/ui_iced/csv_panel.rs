use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Font, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::AppTheme;
use crate::tools::csv_viewer::parse_csv;

pub fn view_csv_viewer<'a>(state: &'a NotepadIced, theme: &AppTheme) -> Element<'a, Message> {
    let t_background = theme.background;
    let t_tab_bar_bg = theme.tab_bar_bg;
    let t_border = theme.border;
    let t_text = theme.text;
    let t_text_dim = theme.text_dim;
    let content_text = state
        .tab_contents
        .get(state.tab_manager.active_index())
        .map(|tc| tc.content.text())
        .unwrap_or_default();

    if content_text.trim().is_empty() {
        return container(
            text("No CSV data to display").size(14).color(t_text_dim),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_background)),
            ..Default::default()
        })
        .into();
    }

    let csv_data = parse_csv(&content_text, ',', state.csv_has_headers);

    // Guard: data with no columns or only 1 column and many rows likely isn't CSV
    let looks_like_csv = csv_data.headers.len() >= 2
        || (csv_data.headers.len() == 1 && csv_data.rows.len() <= 1);

    if csv_data.headers.is_empty() || !looks_like_csv || csv_data.headers.len() > 50 {
        let msg = if csv_data.headers.len() > 50 {
            "This data doesn't appear to be CSV (too many columns detected). Try a different delimiter or check the file format."
        } else if csv_data.headers.len() == 1 && csv_data.rows.len() > 1 {
            "This data doesn't appear to be CSV (only 1 column detected). Try a different delimiter or check the file format."
        } else {
            "No CSV data to display"
        };
        return container(
            text(msg).size(14).color(t_text_dim),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_background)),
            ..Default::default()
        })
        .into();
    }

    // Apply sort if set
    let mut csv_data = csv_data;
    if let Some(col) = state.csv_sort_column {
        crate::tools::csv_viewer::sort_by_column(&mut csv_data, col, state.csv_sort_ascending);
    }

    // Toolbar
    let toolbar = row![
        text("CSV Viewer").size(13).color(t_text),
        Space::with_width(Length::Fill),
        button(
            text(if state.csv_has_headers { "Headers: On" } else { "Headers: Off" })
                .size(12)
                .color(t_text),
        )
        .on_press(Message::CsvToggleHeaders)
        .style(move |_theme: &Theme, _status| button::Style {
            background: Some(iced::Background::Color(t_tab_bar_bg)),
            text_color: t_text,
            border: iced::Border {
                color: t_border,
                width: 1.0,
                radius: 3.0.into(),
            },
            ..Default::default()
        }),
    ]
    .align_y(iced::Alignment::Center)
    .padding([6, 10])
    .spacing(8);

    let toolbar_bar = container(toolbar)
        .width(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_tab_bar_bg)),
            border: iced::Border {
                color: t_border,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

    // Header row
    let col_count = csv_data.headers.len();
    let cell_width = 120.0;

    let mut header_cells: Vec<Element<'a, Message>> = Vec::new();
    // Row number header
    header_cells.push(
        container(
            text("#").size(12).color(t_text_dim).font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::DEFAULT
            }),
        )
        .width(Length::Fixed(40.0))
        .padding([4, 6])
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_tab_bar_bg)),
            border: iced::Border {
                color: t_border,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into(),
    );

    for (i, h) in csv_data.headers.iter().enumerate() {
        let sort_indicator = if state.csv_sort_column == Some(i) {
            if state.csv_sort_ascending { " ^" } else { " v" }
        } else {
            ""
        };
        let label = format!("{}{}", h, sort_indicator);

        header_cells.push(
            button(
                container(
                    text(label)
                        .size(12)
                        .color(t_text)
                        .font(Font {
                            weight: iced::font::Weight::Bold,
                            ..Font::DEFAULT
                        }),
                )
                .padding([4, 6])
                .width(Length::Fixed(cell_width)),
            )
            .on_press(Message::CsvSortColumn(i))
            .style(move |_theme: &Theme, status| {
                let bg = match status {
                    button::Status::Hovered | button::Status::Pressed => t_tab_bar_bg,
                    _ => t_tab_bar_bg,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: t_text,
                    border: iced::Border {
                        color: t_border,
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }
            })
            .into(),
        );
    }

    let header_row = container(row(header_cells))
        .width(Length::Fill);

    // Data rows (cap at 10,000 to prevent UI lag)
    let max_display_rows = 10_000;
    let mut data_rows = column![].spacing(0);
    let display_count = csv_data.rows.len().min(max_display_rows);

    for (row_idx, data_row) in csv_data.rows.iter().take(max_display_rows).enumerate() {
        let bg = if row_idx % 2 == 0 { t_background } else { t_tab_bar_bg };

        let mut cells: Vec<Element<'a, Message>> = Vec::new();

        // Row number
        cells.push(
            container(
                text(format!("{}", row_idx + 1))
                    .size(12)
                    .color(t_text_dim),
            )
            .width(Length::Fixed(40.0))
            .padding([3, 6])
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    color: t_border,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into(),
        );

        for col_idx in 0..col_count {
            let cell_text = data_row
                .get(col_idx)
                .map(|s| s.as_str())
                .unwrap_or("");

            cells.push(
                container(
                    text(cell_text.to_string()).size(12).color(t_text),
                )
                .width(Length::Fixed(cell_width))
                .padding([3, 6])
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(bg)),
                    border: iced::Border {
                        color: t_border,
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                })
                .into(),
            );
        }

        data_rows = data_rows.push(row(cells));
    }

    if csv_data.rows.len() > max_display_rows {
        data_rows = data_rows.push(
            container(
                text(format!(
                    "Showing {} of {} rows",
                    display_count,
                    csv_data.rows.len()
                ))
                .size(12)
                .color(t_text_dim),
            )
            .padding([6, 10]),
        );
    }

    let table = column![header_row, scrollable(data_rows).height(Length::Fill)];

    let panel = column![toolbar_bar, scrollable(table).direction(scrollable::Direction::Both {
        vertical: scrollable::Scrollbar::default(),
        horizontal: scrollable::Scrollbar::default(),
    })];

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_background)),
            ..Default::default()
        })
        .into()
}
