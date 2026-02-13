use egui::{self, Color32, RichText, Ui};

use crate::search::{SearchEngine, SearchHistory, SearchMatch, SearchMode};

/// Action returned from the search bar UI to the parent app.
pub enum SearchAction {
    None,
    Close,
    FindNext,
    FindPrev,
    ReplaceNext,
    ReplaceAll,
    /// Query or options changed — re-run find_all
    QueryChanged,
}

/// Persistent state for the search bar UI.
pub struct SearchBarState {
    pub query: String,
    pub replace_text: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub use_regex: bool,
    /// Whether to focus the search input on next frame
    pub request_focus: bool,
}

impl Default for SearchBarState {
    fn default() -> Self {
        Self {
            query: String::new(),
            replace_text: String::new(),
            case_sensitive: false,
            whole_word: false,
            use_regex: false,
            request_focus: false,
        }
    }
}

impl SearchBarState {
    /// Sync search bar state into a SearchEngine.
    pub fn apply_to_engine(&self, engine: &mut SearchEngine) {
        engine.query = self.query.clone();
        engine.replace_text = self.replace_text.clone();
        engine.case_sensitive = self.case_sensitive;
        engine.whole_word = self.whole_word;
        engine.use_regex = self.use_regex;
        engine.search_mode = if self.use_regex {
            SearchMode::Regex
        } else {
            SearchMode::Normal
        };
    }
}

/// Render the search bar (and optionally replace bar) at the top of the editor area.
/// Returns a `SearchAction` indicating what the caller should do.
pub fn render_search_bar(
    ui: &mut Ui,
    state: &mut SearchBarState,
    show_replace: bool,
    match_count: usize,
    current_match_index: Option<usize>,
) -> SearchAction {
    let mut action = SearchAction::None;
    let prev_query = state.query.clone();
    let prev_cs = state.case_sensitive;
    let prev_ww = state.whole_word;
    let prev_re = state.use_regex;

    // Find row
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;

        // Search input
        let search_id = ui.id().with("search_input");
        let response = ui.add(
            egui::TextEdit::singleline(&mut state.query)
                .id(search_id)
                .desired_width(250.0)
                .hint_text("Find...")
        );

        if state.request_focus {
            response.request_focus();
            state.request_focus = false;
        }

        // Enter/Shift+Enter in search field
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if ui.input(|i| i.modifiers.shift) {
                action = SearchAction::FindPrev;
            } else {
                action = SearchAction::FindNext;
            }
            response.request_focus();
        }

        // Toggle buttons
        let cs_color = if state.case_sensitive { Color32::from_rgb(60, 130, 220) } else { ui.visuals().text_color() };
        if ui.add(egui::Button::new(RichText::new("Aa").color(cs_color).strong()).min_size(egui::vec2(28.0, 20.0))).on_hover_text("Match Case").clicked() {
            state.case_sensitive = !state.case_sensitive;
        }

        let ww_color = if state.whole_word { Color32::from_rgb(60, 130, 220) } else { ui.visuals().text_color() };
        if ui.add(egui::Button::new(RichText::new("W").color(ww_color).strong()).min_size(egui::vec2(28.0, 20.0))).on_hover_text("Whole Word").clicked() {
            state.whole_word = !state.whole_word;
        }

        let re_color = if state.use_regex { Color32::from_rgb(60, 130, 220) } else { ui.visuals().text_color() };
        if ui.add(egui::Button::new(RichText::new(".*").color(re_color).strong()).min_size(egui::vec2(28.0, 20.0))).on_hover_text("Regex").clicked() {
            state.use_regex = !state.use_regex;
        }

        ui.separator();

        // Navigation
        if ui.add(egui::Button::new("↑").min_size(egui::vec2(24.0, 20.0))).on_hover_text("Previous Match (Shift+Enter)").clicked() {
            action = SearchAction::FindPrev;
        }
        if ui.add(egui::Button::new("↓").min_size(egui::vec2(24.0, 20.0))).on_hover_text("Next Match (Enter)").clicked() {
            action = SearchAction::FindNext;
        }

        // Match count
        if !state.query.is_empty() {
            let label = if match_count == 0 {
                "No matches".to_string()
            } else if let Some(idx) = current_match_index {
                format!("{} of {} matches", idx + 1, match_count)
            } else {
                format!("{} matches", match_count)
            };
            ui.label(RichText::new(label).weak());
        }

        // Close button (right-aligned)
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(egui::Button::new("×").min_size(egui::vec2(24.0, 20.0))).on_hover_text("Close (Esc)").clicked() {
                action = SearchAction::Close;
            }
        });
    });

    // Replace row (if shown)
    if show_replace {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            ui.add(
                egui::TextEdit::singleline(&mut state.replace_text)
                    .desired_width(250.0)
                    .hint_text("Replace...")
            );

            if ui.button("Replace").on_hover_text("Replace current match").clicked() {
                action = SearchAction::ReplaceNext;
            }
            if ui.button("Replace All").on_hover_text("Replace all matches").clicked() {
                action = SearchAction::ReplaceAll;
            }
        });
    }

    // Detect changes in query or options
    if matches!(action, SearchAction::None) {
        if state.query != prev_query
            || state.case_sensitive != prev_cs
            || state.whole_word != prev_ww
            || state.use_regex != prev_re
        {
            action = SearchAction::QueryChanged;
        }
    }

    action
}

/// Render the search results panel below the editor.
pub fn render_search_results_panel(
    ui: &mut Ui,
    matches: &[SearchMatch],
    current_match_index: Option<usize>,
    history: &SearchHistory,
) -> Option<usize> {
    let mut clicked_index: Option<usize> = None;

    ui.horizontal(|ui| {
        ui.strong("Search Results");
        ui.label(format!("({} matches)", matches.len()));
    });

    ui.separator();

    egui::ScrollArea::vertical()
        .max_height(180.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for (i, m) in matches.iter().enumerate() {
                let is_current = current_match_index == Some(i);
                let text = format!("  Ln {}: {}", m.line + 1, m.line_text.trim());
                let label = if is_current {
                    RichText::new(&text).strong().color(Color32::from_rgb(60, 130, 220))
                } else {
                    RichText::new(&text)
                };
                if ui.selectable_label(is_current, label).clicked() {
                    clicked_index = Some(i);
                }
            }

            // Show recent search history if no current matches
            if matches.is_empty() && !history.entries().is_empty() {
                ui.separator();
                ui.label(RichText::new("Recent Searches").weak());
                for entry in history.entries().iter().rev().take(10) {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🔍").weak());
                        ui.label(
                            RichText::new(format!(
                                "\"{}\" — {} matches",
                                entry.query,
                                entry.results.len()
                            ))
                            .weak(),
                        );
                    });
                }
            }
        });

    clicked_index
}
