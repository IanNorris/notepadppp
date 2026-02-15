use iced::widget::{button, column, container, mouse_area, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::AppColors;

const PANEL_BG: iced::Color = iced::Color::from_rgb(0.16, 0.16, 0.20);
const TITLE_BAR_BG: iced::Color = iced::Color::from_rgb(0.13, 0.13, 0.17);
const TOGGLE_ON: iced::Color = iced::Color::from_rgb(0.0, 0.47, 0.84);
const TOGGLE_OFF: iced::Color = iced::Color::from_rgb(0.25, 0.25, 0.30);
const MATCH_BG: iced::Color = iced::Color::from_rgb(0.18, 0.18, 0.22);
const MATCH_HOVER: iced::Color = iced::Color::from_rgb(0.25, 0.25, 0.30);
const FILE_HEADER_BG: iced::Color = iced::Color::from_rgb(0.14, 0.14, 0.18);

pub fn view_find_in_files_panel<'a>(state: &NotepadIced) -> Element<'a, Message> {
    // Draggable title bar
    let title_bar_content = row![
        text("Find in Files").size(13).color(AppColors::TEXT),
        Space::with_width(Length::Fill),
        nav_button("x", Message::CloseFindInFiles),
    ]
    .align_y(iced::Alignment::Center)
    .padding([4, 8]);

    let title_bar: Element<'_, Message> = mouse_area(
        container(title_bar_content)
            .width(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(TITLE_BAR_BG)),
                border: iced::Border {
                    radius: iced::border::Radius {
                        top_left: 6.0,
                        top_right: 6.0,
                        bottom_left: 0.0,
                        bottom_right: 0.0,
                    },
                    ..Default::default()
                },
                ..Default::default()
            }),
    )
    .on_press(Message::DragFifStart)
    .on_move(Message::DragFifMove)
    .on_release(Message::DragFifEnd)
    .into();

    // Directory input
    let dir_row = row![
        text("Directory:").size(12).color(AppColors::TEXT_DIM).width(Length::Fixed(70.0)),
        text_input("Search directory...", &state.fif_directory)
            .on_input(Message::FifDirectoryChanged)
            .size(13)
            .width(Length::Fill),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center)
    .padding([2, 8]);

    // Query input
    let query_row = row![
        text("Find what:").size(12).color(AppColors::TEXT_DIM).width(Length::Fixed(70.0)),
        text_input("Search query...", &state.fif_query)
            .on_input(Message::FifQueryChanged)
            .on_submit(Message::FifSearch)
            .size(13)
            .width(Length::Fill),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center)
    .padding([2, 8]);

    // File filter input
    let filter_row = row![
        text("Filters:").size(12).color(AppColors::TEXT_DIM).width(Length::Fixed(70.0)),
        text_input("*.rs;*.txt", &state.fif_file_filter)
            .on_input(Message::FifFileFilterChanged)
            .size(13)
            .width(Length::Fill),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center)
    .padding([2, 8]);

    // Toggle buttons and search button
    let recursive_btn = toggle_button("Recursive", state.fif_recursive, Message::FifToggleRecursive);
    let case_btn = toggle_button("Aa", state.fif_case_sensitive, Message::FifToggleCaseSensitive);
    let regex_btn = toggle_button(".*", state.fif_use_regex, Message::FifToggleRegex);

    let search_btn = action_button(
        if state.fif_searching { "Searching..." } else { "Search" },
        Message::FifSearch,
    );

    let options_row = row![
        recursive_btn,
        case_btn,
        regex_btn,
        Space::with_width(Length::Fill),
        search_btn,
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center)
    .padding([2, 8]);

    let mut panel = column![title_bar, dir_row, query_row, filter_row, options_row].spacing(2);

    // Results area
    if !state.fif_results.is_empty() {
        let mut total_matches = 0usize;
        let mut results_col = column![].spacing(1);

        for file_result in &state.fif_results {
            total_matches += file_result.matches.len();
            let path_display = file_result.path.display().to_string();

            // File header
            let file_header = container(
                text(format!("📄 {} ({} matches)", path_display, file_result.matches.len()))
                    .size(11)
                    .color(AppColors::TEXT),
            )
            .width(Length::Fill)
            .padding([3, 6])
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(FILE_HEADER_BG)),
                border: iced::Border {
                    radius: 2.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
            results_col = results_col.push(file_header);

            // Match lines
            for m in &file_result.matches {
                let line_num = format!("{:>5}: ", m.line + 1);
                let line_text = m.line_text.trim().to_string();
                let path_clone = file_result.path.clone();
                let line = m.line;

                let result_btn = button(
                    row![
                        text(line_num).size(11).color(AppColors::TEXT_DIM),
                        text(line_text).size(11).color(AppColors::TEXT),
                    ]
                    .spacing(4),
                )
                .on_press(Message::FifClickResult(path_clone, line))
                .width(Length::Fill)
                .padding([2, 6])
                .style(|_theme: &Theme, status| {
                    let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
                    button::Style {
                        background: Some(iced::Background::Color(if hover { MATCH_HOVER } else { MATCH_BG })),
                        text_color: AppColors::TEXT,
                        border: iced::Border {
                            radius: 2.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                });

                results_col = results_col.push(result_btn);
            }
        }

        let summary = text(format!(
            "{} matches in {} files",
            total_matches,
            state.fif_results.len()
        ))
        .size(11)
        .color(AppColors::TEXT_DIM);

        panel = panel.push(container(summary).padding([2, 8]));

        let results = scrollable(results_col).height(Length::Fixed(250.0));
        panel = panel.push(container(results).padding([4, 8]));
    } else if state.fif_searching {
        panel = panel.push(
            container(text("Searching...").size(12).color(AppColors::TEXT_DIM))
                .padding([8, 8]),
        );
    }

    // Wrap in window-like container
    container(panel)
        .width(Length::Fixed(600.0))
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(PANEL_BG)),
            border: iced::Border {
                color: iced::Color::from_rgb(0.35, 0.35, 0.40),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                offset: iced::Vector::new(2.0, 4.0),
                blur_radius: 12.0,
            },
            ..Default::default()
        })
        .into()
}

fn toggle_button<'a>(label: &str, active: bool, msg: Message) -> Element<'a, Message> {
    let bg = if active { TOGGLE_ON } else { TOGGLE_OFF };
    button(text(label.to_string()).size(12))
        .on_press(msg)
        .padding([3, 6])
        .style(move |_theme: &Theme, _status| button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: AppColors::TEXT,
            border: iced::Border {
                radius: 3.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn nav_button<'a>(label: &str, msg: Message) -> Element<'a, Message> {
    button(text(label.to_string()).size(14))
        .on_press(msg)
        .padding([2, 6])
        .style(|_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => {
                    Some(iced::Background::Color(TOGGLE_OFF))
                }
                _ => None,
            };
            button::Style {
                background: bg,
                text_color: AppColors::TEXT,
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

fn action_button<'a>(label: &str, msg: Message) -> Element<'a, Message> {
    button(text(label.to_string()).size(12))
        .on_press(msg)
        .padding([3, 8])
        .style(|_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => TOGGLE_ON,
                _ => TOGGLE_OFF,
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
        })
        .into()
}
