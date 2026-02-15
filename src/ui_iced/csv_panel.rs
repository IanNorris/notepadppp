use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Font, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::AppColors;
use crate::tools::csv_viewer::parse_csv;

const PANEL_BG: iced::Color = iced::Color::from_rgb(0.12, 0.12, 0.14);
const HEADER_BG: iced::Color = iced::Color::from_rgb(0.16, 0.16, 0.20);
const ROW_EVEN: iced::Color = iced::Color::from_rgb(0.13, 0.13, 0.16);
const ROW_ODD: iced::Color = iced::Color::from_rgb(0.15, 0.15, 0.18);
const CELL_BORDER: iced::Color = iced::Color::from_rgb(0.22, 0.22, 0.26);

pub fn view_csv_viewer<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let content_text = state
        .tab_contents
        .get(state.tab_manager.active_index())
        .map(|tc| tc.content.text())
        .unwrap_or_default();

    let csv_data = parse_csv(&content_text, ',', state.csv_has_headers);

    if csv_data.headers.is_empty() {
        return container(
            text("No CSV data to display").size(14).color(AppColors::TEXT_DIM),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(PANEL_BG)),
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
        text("CSV Viewer").size(13).color(AppColors::TEXT),
        Space::with_width(Length::Fill),
        button(
            text(if state.csv_has_headers { "Headers: On" } else { "Headers: Off" })
                .size(12)
                .color(AppColors::TEXT),
        )
        .on_press(Message::CsvToggleHeaders)
        .style(|_theme: &Theme, _status| button::Style {
            background: Some(iced::Background::Color(HEADER_BG)),
            text_color: AppColors::TEXT,
            border: iced::Border {
                color: CELL_BORDER,
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
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(HEADER_BG)),
            border: iced::Border {
                color: CELL_BORDER,
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
            text("#").size(12).color(AppColors::TEXT_DIM).font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::DEFAULT
            }),
        )
        .width(Length::Fixed(40.0))
        .padding([4, 6])
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(HEADER_BG)),
            border: iced::Border {
                color: CELL_BORDER,
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
                        .color(AppColors::TEXT)
                        .font(Font {
                            weight: iced::font::Weight::Bold,
                            ..Font::DEFAULT
                        }),
                )
                .padding([4, 6])
                .width(Length::Fixed(cell_width)),
            )
            .on_press(Message::CsvSortColumn(i))
            .style(|_theme: &Theme, status| {
                let bg = match status {
                    button::Status::Hovered | button::Status::Pressed => ROW_ODD,
                    _ => HEADER_BG,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: AppColors::TEXT,
                    border: iced::Border {
                        color: CELL_BORDER,
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

    // Data rows
    let mut data_rows = column![].spacing(0);

    for (row_idx, data_row) in csv_data.rows.iter().enumerate() {
        let bg = if row_idx % 2 == 0 { ROW_EVEN } else { ROW_ODD };

        let mut cells: Vec<Element<'a, Message>> = Vec::new();

        // Row number
        cells.push(
            container(
                text(format!("{}", row_idx + 1))
                    .size(12)
                    .color(AppColors::TEXT_DIM),
            )
            .width(Length::Fixed(40.0))
            .padding([3, 6])
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    color: CELL_BORDER,
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
                    text(cell_text.to_string()).size(12).color(AppColors::TEXT),
                )
                .width(Length::Fixed(cell_width))
                .padding([3, 6])
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(bg)),
                    border: iced::Border {
                        color: CELL_BORDER,
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

    let table = column![header_row, scrollable(data_rows).height(Length::Fill)];

    let panel = column![toolbar_bar, scrollable(table).direction(scrollable::Direction::Both {
        vertical: scrollable::Scrollbar::default(),
        horizontal: scrollable::Scrollbar::default(),
    })];

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(PANEL_BG)),
            ..Default::default()
        })
        .into()
}
