use iced::widget::{button, column, container, mouse_area, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::AppTheme;

pub fn view_find_in_files_panel<'a>(state: &NotepadIced, theme: &AppTheme) -> Element<'a, Message> {
    let panel_bg = theme.dialog_bg;
    let title_bar_bg = theme.background;
    let match_bg = theme.menu_bg;
    let match_hover = theme.menu_hover;
    let file_header_bg = theme.tab_bar_bg;
    let text_color = theme.text;
    let text_dim = theme.text_dim;
    let t_border = theme.border;
    // Draggable title bar
    let title_bar_content = row![
        text("Find in Files").size(13).color(text_color),
        Space::with_width(Length::Fill),
        nav_button("x", Message::CloseFindInFiles, theme),
    ]
    .align_y(iced::Alignment::Center)
    .padding([4, 8]);

    let title_bar: Element<'_, Message> = mouse_area(
        container(title_bar_content)
            .width(Length::Fill)
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(title_bar_bg)),
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
        text("Directory:").size(12).color(text_dim).width(Length::Fixed(70.0)),
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
        text("Find what:").size(12).color(text_dim).width(Length::Fixed(70.0)),
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
        text("Filters:").size(12).color(text_dim).width(Length::Fixed(70.0)),
        text_input("*.rs;*.txt", &state.fif_file_filter)
            .on_input(Message::FifFileFilterChanged)
            .size(13)
            .width(Length::Fill),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center)
    .padding([2, 8]);

    // Toggle buttons and search button
    let recursive_btn = toggle_button("Recursive", state.fif_recursive, Message::FifToggleRecursive, theme);
    let case_btn = toggle_button("Aa", state.fif_case_sensitive, Message::FifToggleCaseSensitive, theme);
    let regex_btn = toggle_button(".*", state.fif_use_regex, Message::FifToggleRegex, theme);

    let search_btn = action_button(
        if state.fif_searching { "Searching..." } else { "Search" },
        Message::FifSearch,
        theme,
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
                    .color(text_color),
            )
            .width(Length::Fill)
            .padding([3, 6])
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(file_header_bg)),
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
                        text(line_num).size(11).color(text_dim),
                        text(line_text).size(11).color(text_color),
                    ]
                    .spacing(4),
                )
                .on_press(Message::FifClickResult(path_clone, line))
                .width(Length::Fill)
                .padding([2, 6])
                .style(move |_theme: &Theme, status| {
                    let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
                    button::Style {
                        background: Some(iced::Background::Color(if hover { match_hover } else { match_bg })),
                        text_color: text_color,
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
        .color(text_dim);

        panel = panel.push(container(summary).padding([2, 8]));

        let results = scrollable(results_col).height(Length::Fixed(250.0));
        panel = panel.push(container(results).padding([4, 8]));
    } else if state.fif_searching {
        panel = panel.push(
            container(text("Searching...").size(12).color(text_dim))
                .padding([8, 8]),
        );
    }

    // Wrap in window-like container
    container(panel)
        .width(Length::Fixed(600.0))
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(panel_bg)),
            border: iced::Border {
                color: t_border,
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

fn toggle_button<'a>(label: &str, active: bool, msg: Message, theme: &AppTheme) -> Element<'a, Message> {
    let t_text = theme.text;
    let bg = if active { theme.accent } else { theme.button_bg };
    button(text(label.to_string()).size(12))
        .on_press(msg)
        .padding([3, 6])
        .style(move |_theme: &Theme, _status| button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: t_text,
            border: iced::Border {
                radius: 3.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn nav_button<'a>(label: &str, msg: Message, theme: &AppTheme) -> Element<'a, Message> {
    let t_text = theme.text;
    let t_button_bg = theme.button_bg;
    button(text(label.to_string()).size(14))
        .on_press(msg)
        .padding([2, 6])
        .style(move |_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => {
                    Some(iced::Background::Color(t_button_bg))
                }
                _ => None,
            };
            button::Style {
                background: bg,
                text_color: t_text,
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

fn action_button<'a>(label: &str, msg: Message, theme: &AppTheme) -> Element<'a, Message> {
    let t_text = theme.text;
    let t_accent = theme.accent;
    let t_button_bg = theme.button_bg;
    button(text(label.to_string()).size(12))
        .on_press(msg)
        .padding([3, 8])
        .style(move |_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => t_accent,
                _ => t_button_bg,
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
        })
        .into()
}
