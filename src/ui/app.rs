use egui::{self, Align, Layout, RichText, Ui};
use std::path::PathBuf;

use crate::editor::document::{Encoding, LineEnding};
use crate::editor::tab_manager::TabManager;
use crate::io::recent_files::RecentFiles;
use crate::search::{SearchEngine, SearchHistory, SearchMatch};

use super::editor_widget::editor_widget;
use super::search_dialog::{self, SearchAction, SearchBarState};

/// Main application state implementing `eframe::App`.
pub struct NotepadApp {
    tab_manager: TabManager,
    recent_files: RecentFiles,
    show_status_bar: bool,
    #[allow(dead_code)]
    show_toolbar: bool,
    show_line_numbers: bool,
    show_whitespace: bool,
    word_wrap: bool,
    font_size: f32,
    /// Cached String for syncing with egui's TextEdit
    text_cache: String,
    /// Whether the text_cache is in sync with the active document
    cache_valid: bool,
    /// Track which tab index the cache belongs to
    cache_tab_index: usize,
    // Search state
    search_engine: SearchEngine,
    search_history: SearchHistory,
    search_bar_state: SearchBarState,
    show_search_bar: bool,
    show_replace: bool,
    show_search_results: bool,
    search_results_matches: Vec<SearchMatch>,
    current_match_index: Option<usize>,
}

impl NotepadApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let tab_manager = TabManager::new();
        let text = tab_manager.active_document().buffer.text();
        Self {
            tab_manager,
            recent_files: Self::load_recent_files(),
            show_status_bar: true,
            show_toolbar: true,
            show_line_numbers: true,
            show_whitespace: false,
            word_wrap: false,
            font_size: 14.0,
            text_cache: text,
            cache_valid: true,
            cache_tab_index: 0,
            search_engine: SearchEngine::new(),
            search_history: SearchHistory::new(50),
            search_bar_state: SearchBarState::default(),
            show_search_bar: false,
            show_replace: false,
            show_search_results: false,
            search_results_matches: Vec::new(),
            current_match_index: None,
        }
    }

    fn load_recent_files() -> RecentFiles {
        if let Some(config_dir) = dirs::config_dir() {
            let path = config_dir.join("notepadppp").join("recent_files.json");
            RecentFiles::load(&path).unwrap_or_default()
        } else {
            RecentFiles::new()
        }
    }

    fn save_recent_files(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let dir = config_dir.join("notepadppp");
            let _ = std::fs::create_dir_all(&dir);
            let path = dir.join("recent_files.json");
            let _ = self.recent_files.save(&path);
        }
    }

    fn sync_cache_from_buffer(&mut self) {
        let idx = self.tab_manager.active_index();
        if !self.cache_valid || self.cache_tab_index != idx {
            self.text_cache = self.tab_manager.active_document().buffer.text();
            self.cache_valid = true;
            self.cache_tab_index = idx;
        }
    }

    fn sync_buffer_from_cache(&mut self) {
        let doc = self.tab_manager.active_document_mut();
        let current = doc.buffer.text();
        if self.text_cache != current {
            // Replace entire buffer content with the cache
            let len = doc.buffer.len_bytes();
            if len > 0 {
                doc.buffer.delete(0, len);
            }
            if !self.text_cache.is_empty() {
                doc.buffer.insert(0, &self.text_cache);
            }
        }
    }

    fn invalidate_cache(&mut self) {
        self.cache_valid = false;
    }

    // --- Actions ---

    fn action_new(&mut self) {
        self.tab_manager.new_tab();
        self.invalidate_cache();
    }

    fn action_open(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            self.open_file_path(path);
        }
    }

    fn open_file_path(&mut self, path: PathBuf) {
        match self.tab_manager.open_file(path.clone()) {
            Ok(_) => {
                self.recent_files.add(path);
                self.save_recent_files();
                self.invalidate_cache();
            }
            Err(e) => {
                log::error!("Failed to open file: {}", e);
            }
        }
    }

    fn action_save(&mut self) {
        self.sync_buffer_from_cache();
        let idx = self.tab_manager.active_index();
        if self.tab_manager.active_document().path.is_some() {
            if let Err(e) = self.tab_manager.save_tab(idx) {
                log::error!("Failed to save: {}", e);
            }
        } else {
            self.action_save_as();
        }
    }

    fn action_save_as(&mut self) {
        self.sync_buffer_from_cache();
        if let Some(path) = rfd::FileDialog::new().save_file() {
            let idx = self.tab_manager.active_index();
            if let Err(e) = self.tab_manager.save_tab_as(idx, path.clone()) {
                log::error!("Failed to save as: {}", e);
            } else {
                self.recent_files.add(path);
                self.save_recent_files();
                self.invalidate_cache();
            }
        }
    }

    fn action_save_all(&mut self) {
        self.sync_buffer_from_cache();
        for i in 0..self.tab_manager.tab_count() {
            if let Some(doc) = self.tab_manager.get_document(i) {
                if doc.is_modified() && doc.path.is_some() {
                    if let Err(e) = self.tab_manager.save_tab(i) {
                        log::error!("Failed to save tab {}: {}", i, e);
                    }
                }
            }
        }
    }

    fn action_close_tab(&mut self, index: usize) {
        self.tab_manager.close_tab(index);
        self.invalidate_cache();
    }

    fn action_close_all(&mut self) {
        self.tab_manager.close_all();
        self.invalidate_cache();
    }

    fn action_undo(&mut self) {
        self.tab_manager.active_document_mut().buffer.undo();
        self.invalidate_cache();
    }

    fn action_redo(&mut self) {
        self.tab_manager.active_document_mut().buffer.redo();
        self.invalidate_cache();
    }

    fn action_select_all(&mut self) {
        let doc = self.tab_manager.active_document_mut();
        doc.cursor.select_all(&doc.buffer);
    }

    fn action_zoom_in(&mut self) {
        self.font_size = (self.font_size + 1.0).min(72.0);
    }

    fn action_zoom_out(&mut self) {
        self.font_size = (self.font_size - 1.0).max(6.0);
    }

    fn action_zoom_reset(&mut self) {
        self.font_size = 14.0;
    }

    fn action_set_encoding(&mut self, enc: Encoding) {
        self.tab_manager.active_document_mut().encoding = enc;
    }

    fn action_set_line_ending(&mut self, le: LineEnding) {
        self.tab_manager.active_document_mut().line_ending = le;
    }

    // --- Search Actions ---

    fn action_show_find(&mut self) {
        self.show_search_bar = true;
        self.show_replace = false;
        self.search_bar_state.request_focus = true;
    }

    fn action_show_replace(&mut self) {
        self.show_search_bar = true;
        self.show_replace = true;
        self.search_bar_state.request_focus = true;
    }

    fn action_close_search(&mut self) {
        self.show_search_bar = false;
        self.show_replace = false;
        self.show_search_results = false;
    }

    fn refresh_search_results(&mut self) {
        self.search_bar_state.apply_to_engine(&mut self.search_engine);
        self.sync_cache_from_buffer();
        self.search_results_matches = self.search_engine.find_all(&self.text_cache);
        self.show_search_results = !self.search_results_matches.is_empty();
        // Reset match index
        self.current_match_index = if self.search_results_matches.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    fn action_find_next(&mut self) {
        if self.search_results_matches.is_empty() {
            return;
        }
        self.current_match_index = Some(match self.current_match_index {
            Some(idx) => (idx + 1) % self.search_results_matches.len(),
            None => 0,
        });
    }

    fn action_find_prev(&mut self) {
        if self.search_results_matches.is_empty() {
            return;
        }
        self.current_match_index = Some(match self.current_match_index {
            Some(0) => self.search_results_matches.len() - 1,
            Some(idx) => idx - 1,
            None => self.search_results_matches.len() - 1,
        });
    }

    fn action_replace_next(&mut self) {
        self.search_bar_state.apply_to_engine(&mut self.search_engine);
        self.sync_cache_from_buffer();
        let from_pos = self.current_match_index
            .and_then(|i| self.search_results_matches.get(i))
            .map(|m| m.start)
            .unwrap_or(0);
        if let Some((new_text, _)) = self.search_engine.replace_next(&self.text_cache, from_pos) {
            self.text_cache = new_text;
            self.sync_buffer_from_cache();
            self.invalidate_cache();
            self.refresh_search_results();
        }
    }

    fn action_replace_all(&mut self) {
        self.search_bar_state.apply_to_engine(&mut self.search_engine);
        self.sync_cache_from_buffer();
        let (new_text, count) = self.search_engine.replace_all(&self.text_cache);
        if count > 0 {
            self.text_cache = new_text;
            self.sync_buffer_from_cache();
            self.invalidate_cache();
            self.refresh_search_results();
        }
    }

    // --- UI Rendering ---

    fn render_menu_bar(&mut self, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            self.render_file_menu(ui);
            self.render_edit_menu(ui);
            self.render_search_menu(ui);
            self.render_view_menu(ui);
            self.render_encoding_menu(ui);
            self.render_line_ending_menu(ui);
            self.render_help_menu(ui);
        });
    }

    fn render_file_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("File", |ui| {
            if ui.button("New                  Ctrl+N").clicked() {
                self.action_new();
                ui.close_menu();
            }
            if ui.button("Open...              Ctrl+O").clicked() {
                self.action_open();
                ui.close_menu();
            }
            if ui.button("Save                 Ctrl+S").clicked() {
                self.action_save();
                ui.close_menu();
            }
            if ui.button("Save As...     Ctrl+Shift+S").clicked() {
                self.action_save_as();
                ui.close_menu();
            }
            if ui.button("Save All").clicked() {
                self.action_save_all();
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Close              Ctrl+W").clicked() {
                let idx = self.tab_manager.active_index();
                self.action_close_tab(idx);
                ui.close_menu();
            }
            if ui.button("Close All").clicked() {
                self.action_close_all();
                ui.close_menu();
            }
            ui.separator();

            // Recent Files submenu
            ui.menu_button("Recent Files", |ui| {
                let entries: Vec<PathBuf> = self.recent_files.list().to_vec();
                if entries.is_empty() {
                    ui.label("(empty)");
                } else {
                    for path in &entries {
                        let label = path.to_string_lossy().to_string();
                        if ui.button(&label).clicked() {
                            self.open_file_path(path.clone());
                            ui.close_menu();
                        }
                    }
                    ui.separator();
                    if ui.button("Clear Recent Files").clicked() {
                        self.recent_files.clear();
                        self.save_recent_files();
                        ui.close_menu();
                    }
                }
            });

            ui.separator();
            if ui.button("Exit").clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }

    fn render_edit_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Edit", |ui| {
            if ui.button("Undo          Ctrl+Z").clicked() {
                self.action_undo();
                ui.close_menu();
            }
            if ui.button("Redo          Ctrl+Y").clicked() {
                self.action_redo();
                ui.close_menu();
            }
            ui.separator();
            // Cut/Copy/Paste/Delete operate through the text edit widget natively
            ui.label("Cut               Ctrl+X");
            ui.label("Copy              Ctrl+C");
            ui.label("Paste             Ctrl+V");
            ui.label("Delete                Del");
            ui.separator();
            if ui.button("Select All    Ctrl+A").clicked() {
                self.action_select_all();
                ui.close_menu();
            }
            ui.separator();
            ui.menu_button("Line Operations", |ui| {
                if ui.button("Duplicate Line").clicked() {
                    self.line_op_duplicate();
                    ui.close_menu();
                }
                if ui.button("Delete Line").clicked() {
                    self.line_op_delete();
                    ui.close_menu();
                }
                if ui.button("Move Line Up").clicked() {
                    self.line_op_move_up();
                    ui.close_menu();
                }
                if ui.button("Move Line Down").clicked() {
                    self.line_op_move_down();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Sort Lines Ascending").clicked() {
                    self.line_op_sort(false);
                    ui.close_menu();
                }
                if ui.button("Sort Lines Descending").clicked() {
                    self.line_op_sort(true);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Remove Empty Lines").clicked() {
                    self.line_op_remove_empty();
                    ui.close_menu();
                }
                if ui.button("Remove Duplicate Lines").clicked() {
                    self.line_op_remove_duplicates();
                    ui.close_menu();
                }
                if ui.button("Trim Trailing Whitespace").clicked() {
                    self.line_op_trim_trailing();
                    ui.close_menu();
                }
            });
        });
    }

    fn render_search_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Search", |ui| {
            if ui.button("Find...          Ctrl+F").clicked() {
                self.action_show_find();
                ui.close_menu();
            }
            if ui.button("Replace...       Ctrl+H").clicked() {
                self.action_show_replace();
                ui.close_menu();
            }
            if ui.button("Find in Files...").clicked() {
                // TODO: implement find in files dialog
                ui.close_menu();
            }
        });
    }

    fn render_view_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("View", |ui| {
            if ui.checkbox(&mut self.word_wrap, "Word Wrap").clicked() {
                ui.close_menu();
            }
            if ui.checkbox(&mut self.show_line_numbers, "Line Numbers").clicked() {
                ui.close_menu();
            }
            if ui.checkbox(&mut self.show_whitespace, "Show Whitespace").clicked() {
                ui.close_menu();
            }
            if ui.checkbox(&mut self.show_status_bar, "Status Bar").clicked() {
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Zoom In       Ctrl+=").clicked() {
                self.action_zoom_in();
                ui.close_menu();
            }
            if ui.button("Zoom Out      Ctrl+-").clicked() {
                self.action_zoom_out();
                ui.close_menu();
            }
            if ui.button("Reset Zoom    Ctrl+0").clicked() {
                self.action_zoom_reset();
                ui.close_menu();
            }
        });
    }

    fn render_encoding_menu(&mut self, ui: &mut Ui) {
        let current = self.tab_manager.active_document().encoding;
        ui.menu_button("Encoding", |ui| {
            let encs = [
                (Encoding::UTF8, "UTF-8"),
                (Encoding::UTF8BOM, "UTF-8 BOM"),
                (Encoding::UTF16LE, "UTF-16 LE"),
                (Encoding::UTF16BE, "UTF-16 BE"),
                (Encoding::ASCII, "ANSI"),
            ];
            for (enc, label) in &encs {
                let checked = current == *enc;
                let text = if checked {
                    format!("✓ {}", label)
                } else {
                    format!("   {}", label)
                };
                if ui.button(&text).clicked() {
                    self.action_set_encoding(*enc);
                    ui.close_menu();
                }
            }
        });
    }

    fn render_line_ending_menu(&mut self, ui: &mut Ui) {
        let current = self.tab_manager.active_document().line_ending;
        ui.menu_button("Line Endings", |ui| {
            let les = [
                (LineEnding::CRLF, "Windows (CRLF)"),
                (LineEnding::LF, "Unix (LF)"),
                (LineEnding::CR, "Mac (CR)"),
            ];
            for (le, label) in &les {
                let checked = current == *le;
                let text = if checked {
                    format!("✓ {}", label)
                } else {
                    format!("   {}", label)
                };
                if ui.button(&text).clicked() {
                    self.action_set_line_ending(*le);
                    ui.close_menu();
                }
            }
        });
    }

    fn render_help_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Help", |ui| {
            if ui.button("About").clicked() {
                // Simple about — just log for now
                log::info!("Notepad+++ v{}", env!("CARGO_PKG_VERSION"));
                ui.close_menu();
            }
        });
    }

    fn render_tab_bar(&mut self, ui: &mut Ui) {
        let tab_count = self.tab_manager.tab_count();
        let active = self.tab_manager.active_index();

        ui.horizontal_wrapped(|ui| {
            // Collect tab info first to avoid borrow issues
            let tab_info: Vec<(usize, String, bool)> = (0..tab_count)
                .map(|i| {
                    let title = self.tab_manager.get_tab_title(i);
                    let modified = self.tab_manager.get_document(i)
                        .map(|d| d.is_modified())
                        .unwrap_or(false);
                    (i, title, modified)
                })
                .collect();

            let mut close_idx: Option<usize> = None;
            let mut switch_idx: Option<usize> = None;
            let mut close_others_idx: Option<usize> = None;
            let mut close_all = false;

            for (i, title, modified) in &tab_info {
                let label = if *modified {
                    format!("*{}", title)
                } else {
                    title.clone()
                };

                let is_active = *i == active;
                let button_text = if is_active {
                    RichText::new(&label).strong()
                } else {
                    RichText::new(&label)
                };

                let response = ui.selectable_label(is_active, button_text);

                if response.clicked() {
                    switch_idx = Some(*i);
                }

                // Close button next to tab
                let close_resp = ui.small_button("×");
                if close_resp.clicked() {
                    close_idx = Some(*i);
                }

                // Right-click context menu
                response.context_menu(|ui| {
                    if ui.button("Close").clicked() {
                        close_idx = Some(*i);
                        ui.close_menu();
                    }
                    if ui.button("Close Others").clicked() {
                        close_others_idx = Some(*i);
                        ui.close_menu();
                    }
                    if ui.button("Close All").clicked() {
                        close_all = true;
                        ui.close_menu();
                    }
                });

                ui.separator();
            }

            // Process deferred actions
            if let Some(idx) = switch_idx {
                self.tab_manager.set_active(idx);
                self.invalidate_cache();
            }
            if close_all {
                self.action_close_all();
            } else if let Some(keep) = close_others_idx {
                // Close all tabs except the one at `keep`
                let mut i = self.tab_manager.tab_count();
                while i > 0 {
                    i -= 1;
                    if i != keep {
                        self.tab_manager.close_tab(i);
                    }
                }
                self.tab_manager.set_active(0);
                self.invalidate_cache();
            } else if let Some(idx) = close_idx {
                self.action_close_tab(idx);
            }
        });
    }

    fn render_editor(&mut self, ui: &mut Ui) {
        self.sync_cache_from_buffer();
        editor_widget(
            ui,
            &mut self.text_cache,
            self.font_size,
            self.show_line_numbers,
            self.word_wrap,
            &self.search_results_matches,
            self.current_match_index,
        );
        self.sync_buffer_from_cache();
    }

    fn render_status_bar(&self, ui: &mut Ui) {
        let doc = self.tab_manager.active_document();
        let line = doc.cursor.position.line + 1;
        let col = doc.cursor.position.col + 1;

        let encoding_str = match doc.encoding {
            Encoding::UTF8 => "UTF-8",
            Encoding::UTF8BOM => "UTF-8 BOM",
            Encoding::UTF16LE => "UTF-16 LE",
            Encoding::UTF16BE => "UTF-16 BE",
            Encoding::ASCII => "ANSI",
        };

        let eol_str = match doc.line_ending {
            LineEnding::CRLF => "CRLF",
            LineEnding::LF => "LF",
            LineEnding::CR => "CR",
        };

        let char_count = doc.buffer.len_chars();
        let tab_count = self.tab_manager.tab_count();

        ui.horizontal(|ui| {
            ui.label(format!("Ln {}, Col {}", line, col));
            ui.separator();
            ui.label(encoding_str);
            ui.separator();
            ui.label(eol_str);
            ui.separator();
            ui.label(&doc.language);
            ui.separator();
            ui.label(format!("{} tab(s)", tab_count));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(format!("{} chars", char_count));
            });
        });
    }

    // --- Line Operations ---

    fn with_text_lines<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Vec<String>, usize) -> usize,
    {
        self.sync_buffer_from_cache();
        let doc = self.tab_manager.active_document_mut();
        let text = doc.buffer.text();
        let cursor_line = doc.cursor.position.line;
        let mut lines: Vec<String> = text.lines().map(String::from).collect();
        if text.ends_with('\n') {
            lines.push(String::new());
        }
        let new_cursor_line = f(&mut lines, cursor_line);
        let new_text = lines.join("\n");

        let len = doc.buffer.len_bytes();
        if len > 0 {
            doc.buffer.delete(0, len);
        }
        if !new_text.is_empty() {
            doc.buffer.insert(0, &new_text);
        }
        doc.cursor.position.line = new_cursor_line.min(lines.len().saturating_sub(1));
        self.invalidate_cache();
    }

    fn line_op_duplicate(&mut self) {
        self.with_text_lines(|lines, cursor_line| {
            if cursor_line < lines.len() {
                let dup = lines[cursor_line].clone();
                lines.insert(cursor_line + 1, dup);
            }
            cursor_line + 1
        });
    }

    fn line_op_delete(&mut self) {
        self.with_text_lines(|lines, cursor_line| {
            if cursor_line < lines.len() && lines.len() > 1 {
                lines.remove(cursor_line);
            } else if lines.len() == 1 {
                lines[0] = String::new();
            }
            cursor_line.min(lines.len().saturating_sub(1))
        });
    }

    fn line_op_move_up(&mut self) {
        self.with_text_lines(|lines, cursor_line| {
            if cursor_line > 0 && cursor_line < lines.len() {
                lines.swap(cursor_line, cursor_line - 1);
                cursor_line - 1
            } else {
                cursor_line
            }
        });
    }

    fn line_op_move_down(&mut self) {
        self.with_text_lines(|lines, cursor_line| {
            if cursor_line + 1 < lines.len() {
                lines.swap(cursor_line, cursor_line + 1);
                cursor_line + 1
            } else {
                cursor_line
            }
        });
    }

    fn line_op_sort(&mut self, descending: bool) {
        self.with_text_lines(|lines, _cursor_line| {
            lines.sort();
            if descending {
                lines.reverse();
            }
            0
        });
    }

    fn line_op_remove_empty(&mut self) {
        self.with_text_lines(|lines, _cursor_line| {
            lines.retain(|l| !l.trim().is_empty());
            if lines.is_empty() {
                lines.push(String::new());
            }
            0
        });
    }

    fn line_op_remove_duplicates(&mut self) {
        self.with_text_lines(|lines, _cursor_line| {
            let mut seen = std::collections::HashSet::new();
            lines.retain(|l| seen.insert(l.clone()));
            if lines.is_empty() {
                lines.push(String::new());
            }
            0
        });
    }

    fn line_op_trim_trailing(&mut self) {
        self.with_text_lines(|lines, cursor_line| {
            for line in lines.iter_mut() {
                *line = line.trim_end().to_string();
            }
            cursor_line
        });
    }

    // --- Keyboard shortcuts ---

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        let modifiers = ctx.input(|i| i.modifiers);

        ctx.input(|i| {
            // Ctrl+N
            if i.key_pressed(egui::Key::N) && modifiers.ctrl && !modifiers.shift {
                // Handled after input closure
            }
        });

        // We need to check these outside the input closure to avoid borrow issues
        let ctrl_n = ctx.input(|i| i.key_pressed(egui::Key::N) && i.modifiers.ctrl && !i.modifiers.shift);
        let ctrl_o = ctx.input(|i| i.key_pressed(egui::Key::O) && i.modifiers.ctrl);
        let ctrl_s = ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl && !i.modifiers.shift);
        let ctrl_shift_s = ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl && i.modifiers.shift);
        let ctrl_w = ctx.input(|i| i.key_pressed(egui::Key::W) && i.modifiers.ctrl);
        let ctrl_z = ctx.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.ctrl && !i.modifiers.shift);
        let ctrl_y = ctx.input(|i| i.key_pressed(egui::Key::Y) && i.modifiers.ctrl);
        let ctrl_plus = ctx.input(|i| i.key_pressed(egui::Key::Equals) && i.modifiers.ctrl);
        let ctrl_minus = ctx.input(|i| i.key_pressed(egui::Key::Minus) && i.modifiers.ctrl);
        let ctrl_zero = ctx.input(|i| i.key_pressed(egui::Key::Num0) && i.modifiers.ctrl);
        let ctrl_f = ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.ctrl && !i.modifiers.shift);
        let ctrl_h = ctx.input(|i| i.key_pressed(egui::Key::H) && i.modifiers.ctrl);
        let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        let f3 = ctx.input(|i| i.key_pressed(egui::Key::F3) && !i.modifiers.shift);
        let shift_f3 = ctx.input(|i| i.key_pressed(egui::Key::F3) && i.modifiers.shift);

        if ctrl_n { self.action_new(); }
        if ctrl_o { self.action_open(); }
        if ctrl_shift_s { self.action_save_as(); }
        else if ctrl_s { self.action_save(); }
        if ctrl_w {
            let idx = self.tab_manager.active_index();
            self.action_close_tab(idx);
        }
        if ctrl_z { self.action_undo(); }
        if ctrl_y { self.action_redo(); }
        if ctrl_plus { self.action_zoom_in(); }
        if ctrl_minus { self.action_zoom_out(); }
        if ctrl_zero { self.action_zoom_reset(); }
        if ctrl_f { self.action_show_find(); }
        if ctrl_h { self.action_show_replace(); }
        if escape && self.show_search_bar { self.action_close_search(); }
        if f3 { self.action_find_next(); }
        if shift_f3 { self.action_find_prev(); }
    }
}

impl eframe::App for NotepadApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard_shortcuts(ctx);

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.render_menu_bar(ui);
        });

        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            self.render_tab_bar(ui);
        });

        // Search bar panel (below tabs, above editor)
        if self.show_search_bar {
            let match_count = self.search_results_matches.len();
            let current_idx = self.current_match_index;
            let show_replace = self.show_replace;

            egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Toggle expand button for replace mode
                    let arrow = if self.show_replace { "▼" } else { "►" };
                    if ui.small_button(arrow).on_hover_text("Toggle Replace").clicked() {
                        self.show_replace = !self.show_replace;
                    }

                    ui.vertical(|ui| {
                        let action = search_dialog::render_search_bar(
                            ui,
                            &mut self.search_bar_state,
                            show_replace,
                            match_count,
                            current_idx,
                        );

                        match action {
                            SearchAction::Close => self.action_close_search(),
                            SearchAction::FindNext => self.action_find_next(),
                            SearchAction::FindPrev => self.action_find_prev(),
                            SearchAction::ReplaceNext => self.action_replace_next(),
                            SearchAction::ReplaceAll => self.action_replace_all(),
                            SearchAction::QueryChanged => self.refresh_search_results(),
                            SearchAction::None => {}
                        }
                    });
                });
            });
        }

        if self.show_status_bar {
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                self.render_status_bar(ui);
            });
        }

        // Search results panel (above status bar, below editor)
        if self.show_search_bar && self.show_search_results && !self.search_results_matches.is_empty() {
            let matches = self.search_results_matches.clone();
            let current_idx = self.current_match_index;
            egui::TopBottomPanel::bottom("search_results").show(ctx, |ui| {
                if let Some(clicked) = search_dialog::render_search_results_panel(
                    ui,
                    &matches,
                    current_idx,
                    &self.search_history,
                ) {
                    self.current_match_index = Some(clicked);
                }
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_editor(ui);
        });
    }
}
