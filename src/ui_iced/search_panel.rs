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

pub fn view_search_panel<'a>(state: &NotepadIced) -> Element<'a, Message> {
    let match_count = state.search_matches.len();
    let match_label = if state.search_query.is_empty() {
        String::new()
    } else if match_count == 0 {
        "No matches".to_string()
    } else {
        let idx = state.current_match_index.map(|i| i + 1).unwrap_or(0);
        format!("{} of {} matches", idx, match_count)
    };

    // Draggable title bar
    let title = if state.show_replace { "Find and Replace" } else { "Find" };
    let title_bar_content = row![
        text(title).size(13).color(AppColors::TEXT),
        Space::with_width(Length::Fill),
        nav_button("x", Message::CloseSearch),
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
    .on_press(Message::DragFindStart)
    .on_move(Message::DragFindMove)
    .on_release(Message::DragFindEnd)
    .into();

    // Toggle buttons
    let case_btn = toggle_button("Aa", state.case_sensitive, Message::ToggleCaseSensitive);
    let word_btn = toggle_button("W", state.whole_word, Message::ToggleWholeWord);
    let regex_btn = toggle_button(".*", state.use_regex, Message::ToggleRegex);

    // Nav buttons
    let prev_btn = nav_button("<", Message::FindPrev);
    let next_btn = nav_button(">", Message::FindNext);

    let find_input = text_input("Find...", &state.search_query)
        .on_input(Message::FindQueryChanged)
        .on_submit(Message::FindNext)
        .size(13)
        .width(Length::Fixed(250.0));

    let find_row = row![
        text("Find what:").size(12).color(AppColors::TEXT_DIM),
        Space::with_width(8),
        find_input,
        Space::with_width(4),
        case_btn,
        word_btn,
        regex_btn,
        Space::with_width(8),
        prev_btn,
        next_btn,
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center)
    .padding([2, 8]);

    let match_info = row![
        text(match_label).size(11).color(AppColors::TEXT_DIM),
    ]
    .padding([0, 8]);

    let mut panel = column![title_bar, find_row, match_info].spacing(2);

    // Replace row
    if state.show_replace {
        let replace_input = text_input("Replace...", &state.replace_text)
            .on_input(Message::ReplaceTextChanged)
            .size(13)
            .width(Length::Fixed(250.0));

        let replace_btn = action_button("Replace", Message::ReplaceNext);
        let replace_all_btn = action_button("Replace All", Message::ReplaceAll);

        let replace_row = row![
            text("Replace with:").size(12).color(AppColors::TEXT_DIM),
            Space::with_width(8),
            replace_input,
            Space::with_width(4),
            replace_btn,
            replace_all_btn,
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center)
        .padding([2, 8]);

        panel = panel.push(replace_row);
    }

    // Results list
    if !state.search_matches.is_empty() {
        let mut results_col = column![].spacing(1);
        for (i, m) in state.search_matches.iter().enumerate().take(100) {
            let line_num = format!("{:>5}: ", m.line + 1);
            let line_text = m.line_text.trim().to_string();
            let is_current = state.current_match_index == Some(i);
            let bg = if is_current { TOGGLE_ON } else { MATCH_BG };

            let result_btn = button(
                row![
                    text(line_num).size(11).color(AppColors::TEXT_DIM),
                    text(line_text).size(11).color(AppColors::TEXT),
                ]
                .spacing(4),
            )
            .on_press(Message::ClickSearchResult(i))
            .width(Length::Fill)
            .padding([2, 6])
            .style(move |_theme: &Theme, status| {
                let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
                button::Style {
                    background: Some(iced::Background::Color(if hover { MATCH_HOVER } else { bg })),
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

        let results = scrollable(results_col).height(Length::Fixed(120.0));
        panel = panel.push(container(results).padding([4, 8]));
    }

    // Wrap in a window-like container
    container(panel)
        .width(Length::Fixed(520.0))
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
