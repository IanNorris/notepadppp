use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::AppTheme;

/// A single search results entry (one search operation).
#[derive(Debug, Clone)]
pub struct SearchResultsEntry {
    pub query: String,
    pub matches: Vec<SearchResultMatch>,
    pub collapsed: bool,
}

/// A single match within a search results entry.
#[derive(Debug, Clone)]
pub struct SearchResultMatch {
    pub line: usize,
    pub line_text: String,
    /// Optional file path for Find in Files results.
    pub file_path: Option<std::path::PathBuf>,
}

/// Manager for accumulating search results across multiple searches.
#[derive(Debug, Clone, Default)]
pub struct SearchResultsManager {
    pub entries: Vec<SearchResultsEntry>,
}

impl SearchResultsManager {
    pub fn add_search(&mut self, query: String, matches: Vec<SearchResultMatch>) {
        self.entries.push(SearchResultsEntry {
            query,
            matches,
            collapsed: false,
        });
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn toggle_collapse(&mut self, index: usize) {
        if let Some(entry) = self.entries.get_mut(index) {
            entry.collapsed = !entry.collapsed;
        }
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn total_matches(&self) -> usize {
        self.entries.iter().map(|e| e.matches.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

pub fn view_search_results_panel<'a>(state: &NotepadIced, theme: &AppTheme) -> Element<'a, Message> {
    let t_text = theme.text;
    let t_text_dim = theme.text_dim;
    let t_accent = theme.accent;
    let t_menu_hover = theme.menu_hover;
    let t_dialog_bg = theme.dialog_bg;
    let t_menu_bg = theme.menu_bg;
    let t_border = theme.border;
    let t_button_bg = theme.button_bg;

    let total = state.search_results_panel.total_matches();
    let entries = state.search_results_panel.entry_count();

    let header = row![
        text(format!("Search Results ({} searches, {} matches)", entries, total))
            .size(12)
            .color(t_text),
        Space::with_width(Length::Fill),
        {
            let bg = t_button_bg;
            button(text("Clear").size(11))
                .on_press(Message::ClearSearchResults)
                .padding([2, 6])
                .style(move |_theme: &Theme, status| {
                    let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
                    button::Style {
                        background: Some(iced::Background::Color(if hover { t_accent } else { bg })),
                        text_color: t_text,
                        border: iced::Border { radius: 3.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                })
        },
        {
            let bg = t_button_bg;
            button(text("x").size(11))
                .on_press(Message::ToggleSearchResultsPanel)
                .padding([2, 6])
                .style(move |_theme: &Theme, status| {
                    let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
                    button::Style {
                        background: Some(iced::Background::Color(if hover { t_accent } else { bg })),
                        text_color: t_text,
                        border: iced::Border { radius: 3.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                })
        },
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center)
    .padding([4, 8]);

    let mut content = column![header].spacing(2);

    for (entry_idx, entry) in state.search_results_panel.entries.iter().enumerate() {
        let arrow = if entry.collapsed { ">" } else { "v" };
        let entry_header = {
            let bg = t_menu_bg;
            button(
                row![
                    text(format!("{} \"{}\" — {} matches", arrow, entry.query, entry.matches.len()))
                        .size(11)
                        .color(t_text_dim),
                ]
            )
            .on_press(Message::ToggleSearchResultCollapse(entry_idx))
            .width(Length::Fill)
            .padding([2, 8])
            .style(move |_theme: &Theme, status| {
                let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
                button::Style {
                    background: Some(iced::Background::Color(if hover { t_menu_hover } else { bg })),
                    text_color: t_text_dim,
                    border: iced::Border { radius: 2.0.into(), ..Default::default() },
                    ..Default::default()
                }
            })
        };
        content = content.push(entry_header);

        if !entry.collapsed {
            for (match_idx, m) in entry.matches.iter().enumerate().take(200) {
                let line_num = format!("{:>5}: ", m.line + 1);
                let line_text = m.line_text.trim().to_string();
                let bg = t_menu_bg;

                let result_btn = button(
                    row![
                        text(line_num).size(10).color(t_text_dim),
                        text(line_text).size(10).color(t_text),
                    ]
                    .spacing(4),
                )
                .on_press(Message::ClickSearchResultEntry(entry_idx, match_idx))
                .width(Length::Fill)
                .padding([1, 12])
                .style(move |_theme: &Theme, status| {
                    let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
                    button::Style {
                        background: Some(iced::Background::Color(if hover { t_menu_hover } else { bg })),
                        text_color: t_text,
                        border: iced::Border { radius: 1.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                });

                content = content.push(result_btn);
            }
        }
    }

    let scrollable_content = scrollable(content).height(Length::Fill);

    container(scrollable_content)
        .width(Length::Fill)
        .height(Length::Fixed(180.0))
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_dialog_bg)),
            border: iced::Border {
                color: t_border,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}
