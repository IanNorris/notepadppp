use egui::{self, Align, Color32, Layout, RichText, Ui};
use std::path::PathBuf;

use crate::editor::document::{Encoding, LineEnding};
use crate::editor::macros::MacroRecorder;
use crate::editor::syntax::SyntaxHighlighter;
use crate::editor::tab_manager::TabManager;
use crate::io::recent_files::RecentFiles;
use crate::io::session;
use crate::search::{SearchEngine, SearchHistory, SearchMatch};
use crate::tools::{csv_viewer, diff_tool, export, hex_viewer, json_tools, markdown_viewer, mime_tools};
use crate::tools::csv_viewer::CsvData;
use crate::tools::diff_tool::DiffResult;
use crate::tools::hex_viewer::HexView;

use super::editor_widget::editor_widget;
use super::search_dialog::{self, SearchAction, SearchBarState};
use super::split_view::{SplitOrientation, SplitView};

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
    // Syntax highlighting
    syntax_highlighter: SyntaxHighlighter,
    /// Cached file extension for the active document (without dot)
    file_extension: String,
    /// Whether to show the markdown preview panel
    show_markdown_preview: bool,
    /// Split view state
    split_view: SplitView,
    /// Whether the CSV viewer is enabled
    show_csv_viewer: bool,
    /// Parsed CSV data for the active document
    csv_data: Option<CsvData>,
    /// Current CSV delimiter
    csv_delimiter: char,
    /// Diff comparison result
    diff_result: Option<DiffResult>,
    /// Whether to auto-restore session on startup
    auto_restore_session: bool,
    /// Whether the hex viewer is enabled for current tab
    show_hex_viewer: bool,
    /// Hex view data for the current tab
    hex_view: Option<HexView>,
    /// Macro recorder
    macro_recorder: MacroRecorder,
    /// Last recorded macro (for quick replay)
    last_macro: Option<crate::editor::macros::Macro>,
    /// Whether to show the "play N times" dialog
    show_macro_repeat_dialog: bool,
    /// Repeat count for macro playback
    macro_repeat_count: String,
    /// Whether to show the Go to Line dialog
    show_goto_line: bool,
    /// Input for Go to Line dialog
    goto_line_input: String,
    /// Whether to show the command palette
    show_command_palette: bool,
    /// Current command palette search input
    command_palette_input: String,
    /// Filtered command indices
    command_palette_filtered: Vec<usize>,
    /// Whether to show the preferences panel
    show_preferences: bool,
    /// Application settings
    app_settings: crate::io::settings::AppSettings,
    /// Byte position of the matching bracket (if any)
    matching_bracket_pos: Option<usize>,
    /// Time of last auto-save
    last_auto_save: std::time::Instant,
    /// Track file modification times for change detection
    file_mod_times: std::collections::HashMap<std::path::PathBuf, std::time::SystemTime>,
    /// Time of last file change check
    last_file_check: std::time::Instant,
    /// Files needing reload confirmation
    files_changed_externally: Vec<std::path::PathBuf>,
}

impl NotepadApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::with_files(_cc, Vec::new())
    }

    pub fn with_files(_cc: &eframe::CreationContext<'_>, files: Vec<PathBuf>) -> Self {
        let mut tab_manager = TabManager::new();
        let highlighter = SyntaxHighlighter::new();
        for file in &files {
            if file.exists() {
                if let Ok(idx) = tab_manager.open_file(file.clone()) {
                    // Auto-detect language from extension
                    let lang = SyntaxHighlighter::detect_language(file);
                    if let Some(doc) = tab_manager.get_document_mut(idx) {
                        doc.language = lang;
                    }
                }
            }
        }
        // If files were opened, close the default empty tab
        if !files.is_empty() && tab_manager.tab_count() > 1 {
            tab_manager.close_tab(0);
        }
        let ext = tab_manager
            .active_document()
            .path
            .as_ref()
            .map(|p| SyntaxHighlighter::extension_from_path(p))
            .unwrap_or_default();
        let text = tab_manager.active_document().buffer.text();
        let mut app = Self {
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
            syntax_highlighter: highlighter,
            file_extension: ext,
            show_markdown_preview: false,
            split_view: SplitView::new(),
            show_csv_viewer: false,
            csv_data: None,
            csv_delimiter: ',',
            diff_result: None,
            auto_restore_session: false,
            show_hex_viewer: false,
            hex_view: None,
            macro_recorder: MacroRecorder::new(),
            last_macro: None,
            show_macro_repeat_dialog: false,
            macro_repeat_count: String::from("1"),
            show_goto_line: false,
            goto_line_input: String::new(),
            show_command_palette: false,
            command_palette_input: String::new(),
            command_palette_filtered: Vec::new(),
            show_preferences: false,
            app_settings: crate::io::settings::AppSettings::load(
                &crate::io::settings::AppSettings::settings_path(),
            )
            .unwrap_or_default(),
            matching_bracket_pos: None,
            last_auto_save: std::time::Instant::now(),
            file_mod_times: std::collections::HashMap::new(),
            last_file_check: std::time::Instant::now(),
            files_changed_externally: Vec::new(),
        };

        // Auto-restore session if enabled and no files were specified on command line
        if app.app_settings.remember_session && files.is_empty() {
            if let Some(config_dir) = dirs::config_dir() {
                let path = config_dir.join("notepadppp").join("sessions").join("session.json");
                if path.exists() {
                    if let Ok(sess) = session::load_session(&path) {
                        let _ = session::restore_session(&sess, &mut app.tab_manager);
                        app.invalidate_cache();
                    }
                }
            }
        }

        app
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
            self.file_extension = self
                .tab_manager
                .active_document()
                .path
                .as_ref()
                .map(|p| SyntaxHighlighter::extension_from_path(p))
                .unwrap_or_default();
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

    fn action_goto_line(&mut self) {
        self.show_goto_line = true;
        self.goto_line_input = String::new();
    }

    fn get_commands(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("New File", "Ctrl+N"),
            ("Open File", "Ctrl+O"),
            ("Save", "Ctrl+S"),
            ("Save As", "Ctrl+Shift+S"),
            ("Save All", ""),
            ("Close Tab", "Ctrl+W"),
            ("Close All Tabs", ""),
            ("Undo", "Ctrl+Z"),
            ("Redo", "Ctrl+Y"),
            ("Find", "Ctrl+F"),
            ("Replace", "Ctrl+H"),
            ("Find in Files", ""),
            ("Go to Line", "Ctrl+G"),
            ("Toggle Word Wrap", ""),
            ("Toggle Line Numbers", ""),
            ("Toggle Whitespace", ""),
            ("Zoom In", "Ctrl+="),
            ("Zoom Out", "Ctrl+-"),
            ("Reset Zoom", "Ctrl+0"),
            ("Split Horizontal", ""),
            ("Split Vertical", ""),
            ("Remove Split", ""),
            ("Toggle Bookmark", "Ctrl+F2"),
            ("Next Bookmark", "F2"),
            ("Previous Bookmark", "Shift+F2"),
            ("Clear Bookmarks", ""),
            ("Format JSON", "Ctrl+Shift+J"),
            ("Compact JSON", ""),
            ("Validate JSON", ""),
            ("Sort JSON Keys", ""),
            ("Toggle Markdown Preview", ""),
            ("Toggle CSV Viewer", ""),
            ("Toggle Hex Viewer", ""),
            ("Compare Files", ""),
            ("Base64 Encode", ""),
            ("Base64 Decode", ""),
            ("URL Encode", ""),
            ("URL Decode", ""),
            ("Start/Stop Macro Recording", "Ctrl+Shift+R"),
            ("Play Last Macro", "Ctrl+Shift+P"),
            ("Export as HTML", ""),
            ("Export as RTF", ""),
            ("Preferences", ""),
            ("Duplicate Line", ""),
            ("Delete Line", ""),
            ("Move Line Up", ""),
            ("Move Line Down", ""),
            ("Sort Lines Ascending", ""),
            ("Sort Lines Descending", ""),
            ("Remove Empty Lines", ""),
            ("Remove Duplicate Lines", ""),
            ("Trim Trailing Whitespace", ""),
            ("Jump to Matching Bracket", "Ctrl+]"),
        ]
    }

    fn action_show_command_palette(&mut self) {
        self.show_command_palette = true;
        self.command_palette_input = String::new();
        self.command_palette_filtered = (0..self.get_commands().len()).collect();
    }

    fn execute_command(&mut self, index: usize) {
        self.show_command_palette = false;
        match index {
            0 => self.action_new(),
            1 => self.action_open(),
            2 => self.action_save(),
            3 => self.action_save_as(),
            4 => self.action_save_all(),
            5 => { let idx = self.tab_manager.active_index(); self.action_close_tab(idx); }
            6 => self.action_close_all(),
            7 => self.action_undo(),
            8 => self.action_redo(),
            9 => self.action_show_find(),
            10 => self.action_show_replace(),
            11 => {} // find in files TODO
            12 => self.action_goto_line(),
            13 => { self.word_wrap = !self.word_wrap; }
            14 => { self.show_line_numbers = !self.show_line_numbers; }
            15 => { self.show_whitespace = !self.show_whitespace; }
            16 => self.action_zoom_in(),
            17 => self.action_zoom_out(),
            18 => self.action_zoom_reset(),
            19 => { self.split_view.enable(crate::ui::split_view::SplitOrientation::Horizontal); }
            20 => { self.split_view.enable(crate::ui::split_view::SplitOrientation::Vertical); }
            21 => { self.split_view.disable(); }
            22 => self.action_toggle_bookmark(),
            23 => self.action_next_bookmark(),
            24 => self.action_prev_bookmark(),
            25 => self.action_clear_bookmarks(),
            26 => self.action_json_format(),
            27 => self.action_json_compact(),
            28 => self.action_json_validate(),
            29 => self.action_json_sort_keys(),
            30 => { self.show_markdown_preview = !self.show_markdown_preview; }
            31 => { self.show_csv_viewer = !self.show_csv_viewer; }
            32 => { self.show_hex_viewer = !self.show_hex_viewer; }
            33 => self.action_compare_files(),
            34 => self.action_mime_transform(crate::tools::mime_tools::base64_encode),
            35 => self.action_mime_transform_result(|s| crate::tools::mime_tools::base64_decode(s)),
            36 => self.action_mime_transform(crate::tools::mime_tools::url_encode),
            37 => self.action_mime_transform_result(|s| crate::tools::mime_tools::url_decode(s)),
            38 => self.action_toggle_macro_recording(),
            39 => self.action_play_last_macro(),
            40 => self.action_export_html(),
            41 => self.action_export_rtf(),
            42 => { self.show_preferences = !self.show_preferences; }
            43 => self.line_op_duplicate(),
            44 => self.line_op_delete(),
            45 => self.line_op_move_up(),
            46 => self.line_op_move_down(),
            47 => self.line_op_sort(false),
            48 => self.line_op_sort(true),
            49 => self.line_op_remove_empty(),
            50 => self.line_op_remove_duplicates(),
            51 => self.line_op_trim_trailing(),
            52 => {
                // Jump to matching bracket
                if let Some(match_byte_pos) = self.matching_bracket_pos {
                    let doc = self.tab_manager.active_document();
                    if let Some((line, col)) = doc.buffer.line_col(match_byte_pos) {
                        self.tab_manager.active_document_mut().cursor.set_position(line, col);
                    }
                }
            }
            _ => {}
        }
    }

    fn request_repaint_if_dialog(&self, ctx: &egui::Context) {
        if self.show_goto_line || self.show_macro_repeat_dialog || self.show_preferences || self.show_command_palette {
            ctx.request_repaint();
        }
    }

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
            Ok(idx) => {
                // Auto-detect language from extension
                let lang = SyntaxHighlighter::detect_language(&path);
                let ext = SyntaxHighlighter::extension_from_path(&path);
                if let Some(doc) = self.tab_manager.get_document_mut(idx) {
                    doc.language = lang;
                }
                self.file_extension = ext;
                self.recent_files.add(path.clone());
                self.save_recent_files();
                self.invalidate_cache();
                // Track file modification time
                if let Ok(meta) = std::fs::metadata(&path) {
                    if let Ok(modified) = meta.modified() {
                        self.file_mod_times.insert(path.clone(), modified);
                    }
                }
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
                // Detect language from new path
                let lang = SyntaxHighlighter::detect_language(&path);
                let ext = SyntaxHighlighter::extension_from_path(&path);
                self.tab_manager.active_document_mut().language = lang;
                self.file_extension = ext;
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
            self.render_language_menu(ui);
            self.render_tools_menu(ui);
            self.render_settings_menu(ui);
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

            // Session Management
            if ui.button("Save Session...").clicked() {
                self.action_save_session();
                ui.close_menu();
            }
            if ui.button("Load Session...").clicked() {
                self.action_load_session();
                ui.close_menu();
            }
            let auto_label = if self.auto_restore_session {
                "✓ Auto-restore Session"
            } else {
                "  Auto-restore Session"
            };
            if ui.button(auto_label).clicked() {
                self.auto_restore_session = !self.auto_restore_session;
                ui.close_menu();
            }

            ui.separator();

            // Export submenu
            ui.menu_button("Export", |ui| {
                if ui.button("As HTML...").clicked() {
                    self.action_export_html();
                    ui.close_menu();
                }
                if ui.button("As RTF...").clicked() {
                    self.action_export_rtf();
                    ui.close_menu();
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
            ui.separator();
            if ui.button("Go to Line...    Ctrl+G").clicked() {
                self.action_goto_line();
                ui.close_menu();
            }
            ui.separator();
            ui.menu_button("Bookmarks", |ui| {
                if ui.button("Toggle Bookmark       Ctrl+F2").clicked() {
                    self.action_toggle_bookmark();
                    ui.close_menu();
                }
                if ui.button("Next Bookmark              F2").clicked() {
                    self.action_next_bookmark();
                    ui.close_menu();
                }
                if ui.button("Previous Bookmark    Shift+F2").clicked() {
                    self.action_prev_bookmark();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Clear All Bookmarks").clicked() {
                    self.action_clear_bookmarks();
                    ui.close_menu();
                }
                if ui.button("Copy Bookmarked Lines").clicked() {
                    self.action_copy_bookmarked_lines();
                    ui.close_menu();
                }
                if ui.button("Remove Bookmarked Lines").clicked() {
                    self.action_remove_bookmarked_lines();
                    ui.close_menu();
                }
                if ui.button("Remove Unbookmarked Lines").clicked() {
                    self.action_remove_unbookmarked_lines();
                    ui.close_menu();
                }
            });
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
            if ui.button("Split Horizontal").clicked() {
                self.split_view.enable(SplitOrientation::Horizontal);
                if self.split_view.second_tab_index.is_none() {
                    self.split_view.second_tab_index = Some(self.tab_manager.active_index());
                }
                ui.close_menu();
            }
            if ui.button("Split Vertical").clicked() {
                self.split_view.enable(SplitOrientation::Vertical);
                if self.split_view.second_tab_index.is_none() {
                    self.split_view.second_tab_index = Some(self.tab_manager.active_index());
                }
                ui.close_menu();
            }
            if ui.button("Remove Split").clicked() {
                self.split_view.disable();
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

    fn render_language_menu(&mut self, ui: &mut Ui) {
        let current_lang = self.tab_manager.active_document().language.clone();
        ui.menu_button("Language", |ui| {
            let mut languages = self.syntax_highlighter.list_languages();
            languages.sort();
            languages.dedup();
            // Put "Plain Text" first
            if let Some(pos) = languages.iter().position(|l| l == "Plain Text") {
                languages.remove(pos);
            }
            languages.insert(0, "Plain Text".to_string());

            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for lang in &languages {
                        let checked = current_lang == *lang;
                        let text = if checked {
                            format!("✓ {}", lang)
                        } else {
                            format!("   {}", lang)
                        };
                        if ui.button(&text).clicked() {
                            self.tab_manager.active_document_mut().language = lang.clone();
                            // Find an extension for this language to enable highlighting
                            if lang == "Plain Text" {
                                self.file_extension = String::new();
                            } else if let Some(syntax) = self
                                .syntax_highlighter
                                .syntax_set()
                                .syntaxes()
                                .iter()
                                .find(|s| s.name == *lang)
                            {
                                self.file_extension = syntax
                                    .file_extensions
                                    .first()
                                    .cloned()
                                    .unwrap_or_default();
                            }
                            ui.close_menu();
                        }
                    }
                });
        });
    }

    fn render_tools_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Tools", |ui| {
            ui.menu_button("JSON", |ui| {
                if ui.button("Format JSON  (Ctrl+Shift+J)").clicked() {
                    self.action_json_format();
                    ui.close_menu();
                }
                if ui.button("Compact JSON").clicked() {
                    self.action_json_compact();
                    ui.close_menu();
                }
                if ui.button("Validate JSON").clicked() {
                    self.action_json_validate();
                    ui.close_menu();
                }
                if ui.button("Sort JSON Keys").clicked() {
                    self.action_json_sort_keys();
                    ui.close_menu();
                }
            });
            ui.separator();
            let preview_label = if self.show_markdown_preview {
                "✓ Markdown Preview"
            } else {
                "  Markdown Preview"
            };
            if ui.button(preview_label).clicked() {
                self.show_markdown_preview = !self.show_markdown_preview;
                ui.close_menu();
            }
            ui.separator();
            let csv_label = if self.show_csv_viewer {
                "✓ CSV Viewer"
            } else {
                "  CSV Viewer"
            };
            if ui.button(csv_label).clicked() {
                self.show_csv_viewer = !self.show_csv_viewer;
                if self.show_csv_viewer {
                    // Parse current document as CSV
                    self.sync_cache_from_buffer();
                    self.csv_data = Some(csv_viewer::parse_csv(
                        &self.text_cache,
                        self.csv_delimiter,
                        true,
                    ));
                } else {
                    // Write CSV data back to buffer
                    if let Some(ref data) = self.csv_data {
                        self.text_cache = csv_viewer::to_csv(data);
                        self.sync_buffer_from_cache();
                    }
                    self.csv_data = None;
                }
                ui.close_menu();
            }

            ui.separator();
            ui.menu_button("MIME Tools", |ui| {
                if ui.button("Base64 Encode").clicked() {
                    self.action_mime_transform(mime_tools::base64_encode);
                    ui.close_menu();
                }
                if ui.button("Base64 Decode").clicked() {
                    self.action_mime_transform_result(|s| mime_tools::base64_decode(s));
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("URL Encode").clicked() {
                    self.action_mime_transform(mime_tools::url_encode);
                    ui.close_menu();
                }
                if ui.button("URL Decode").clicked() {
                    self.action_mime_transform_result(|s| mime_tools::url_decode(s));
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("HTML Entity Encode").clicked() {
                    self.action_mime_transform(mime_tools::html_entity_encode);
                    ui.close_menu();
                }
                if ui.button("HTML Entity Decode").clicked() {
                    self.action_mime_transform(mime_tools::html_entity_decode);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Hex Encode").clicked() {
                    self.action_mime_transform(mime_tools::hex_encode);
                    ui.close_menu();
                }
                if ui.button("Hex Decode").clicked() {
                    self.action_mime_transform_result(|s| mime_tools::hex_decode(s));
                    ui.close_menu();
                }
            });

            ui.separator();
            if ui.button("Compare Files...").clicked() {
                self.action_compare_files();
                ui.close_menu();
            }
            ui.separator();
            let hex_label = if self.show_hex_viewer {
                "✓ Hex Viewer"
            } else {
                "  Hex Viewer"
            };
            if ui.button(hex_label).clicked() {
                self.show_hex_viewer = !self.show_hex_viewer;
                if self.show_hex_viewer {
                    self.sync_cache_from_buffer();
                    self.hex_view = Some(HexView::from_text(&self.text_cache));
                } else {
                    self.hex_view = None;
                }
                ui.close_menu();
            }
        });

        // Macro menu
        ui.menu_button("Macro", |ui| {
            let is_recording = self.macro_recorder.is_recording();
            let rec_label = if is_recording {
                "Stop Recording  (Ctrl+Shift+R)"
            } else {
                "Start Recording  (Ctrl+Shift+R)"
            };
            if ui.button(rec_label).clicked() {
                self.action_toggle_macro_recording();
                ui.close_menu();
            }
            if ui.button("Play Last Macro  (Ctrl+Shift+P)").clicked() {
                self.action_play_last_macro();
                ui.close_menu();
            }
            if ui.button("Play Multiple Times...").clicked() {
                self.show_macro_repeat_dialog = true;
                ui.close_menu();
            }
            ui.separator();
            let saved = self.macro_recorder.saved_macros().to_vec();
            if saved.is_empty() {
                ui.label("  (no saved macros)");
            } else {
                ui.menu_button("Saved Macros", |ui| {
                    for (i, m) in saved.iter().enumerate() {
                        if ui.button(&m.name).clicked() {
                            let m = m.clone();
                            let doc = self.tab_manager.active_document_mut();
                            MacroRecorder::play_macro(&m, doc);
                            self.invalidate_cache();
                            ui.close_menu();
                            let _ = i; // used above
                        }
                    }
                });
            }
            ui.separator();
            if ui.button("Manage Macros...").clicked() {
                // Simple management: delete all for now
                ui.close_menu();
            }
        });
    }

    fn action_json_format(&mut self) {
        self.sync_buffer_from_cache();
        let text = self.tab_manager.active_document().buffer.text();
        match json_tools::format_json(&text) {
            Ok(formatted) => {
                self.text_cache = formatted;
                self.sync_buffer_from_cache();
            }
            Err(e) => log::error!("JSON format error: {}", e),
        }
    }

    fn action_json_compact(&mut self) {
        self.sync_buffer_from_cache();
        let text = self.tab_manager.active_document().buffer.text();
        match json_tools::compact_json(&text) {
            Ok(compacted) => {
                self.text_cache = compacted;
                self.sync_buffer_from_cache();
            }
            Err(e) => log::error!("JSON compact error: {}", e),
        }
    }

    fn action_json_validate(&mut self) {
        self.sync_buffer_from_cache();
        let text = self.tab_manager.active_document().buffer.text();
        match json_tools::validate_json(&text) {
            Ok(()) => log::info!("JSON is valid"),
            Err(e) => log::error!("JSON validation error: {}", e),
        }
    }

    fn action_json_sort_keys(&mut self) {
        self.sync_buffer_from_cache();
        let text = self.tab_manager.active_document().buffer.text();
        match json_tools::sort_json_keys(&text) {
            Ok(sorted) => {
                self.text_cache = sorted;
                self.sync_buffer_from_cache();
            }
            Err(e) => log::error!("JSON sort keys error: {}", e),
        }
    }

    fn action_mime_transform(&mut self, transform: fn(&str) -> String) {
        self.sync_cache_from_buffer();
        let result = transform(&self.text_cache);
        self.text_cache = result;
        self.sync_buffer_from_cache();
        self.invalidate_cache();
    }

    fn action_mime_transform_result(&mut self, transform: fn(&str) -> Result<String, String>) {
        self.sync_cache_from_buffer();
        match transform(&self.text_cache) {
            Ok(result) => {
                self.text_cache = result;
                self.sync_buffer_from_cache();
                self.invalidate_cache();
            }
            Err(e) => log::error!("MIME decode error: {}", e),
        }
    }

    fn action_compare_files(&mut self) {
        self.sync_cache_from_buffer();
        let current_text = self.text_cache.clone();

        if let Some(path) = rfd::FileDialog::new().pick_file() {
            match std::fs::read_to_string(&path) {
                Ok(other_text) => {
                    self.diff_result = Some(diff_tool::diff_texts(&current_text, &other_text));
                }
                Err(e) => log::error!("Failed to read file for comparison: {}", e),
            }
        }
    }

    fn action_toggle_macro_recording(&mut self) {
        if self.macro_recorder.is_recording() {
            let m = self.macro_recorder.stop_recording("Macro");
            self.last_macro = Some(m);
        } else {
            self.macro_recorder.start_recording();
        }
    }

    fn action_play_last_macro(&mut self) {
        if let Some(ref m) = self.last_macro.clone() {
            let doc = self.tab_manager.active_document_mut();
            MacroRecorder::play_macro(m, doc);
            self.invalidate_cache();
        }
    }

    fn action_save_session(&mut self) {
        let sess = session::capture_session(&self.tab_manager, "session");
        if let Some(config_dir) = dirs::config_dir() {
            let dir = config_dir.join("notepadppp").join("sessions");
            let _ = std::fs::create_dir_all(&dir);
            let path = dir.join("session.json");
            if let Err(e) = session::save_session(&sess, &path) {
                log::error!("Failed to save session: {}", e);
            } else {
                log::info!("Session saved to {:?}", path);
            }
        }
    }

    fn action_load_session(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Session", &["json"])
            .pick_file()
        {
            match session::load_session(&path) {
                Ok(sess) => {
                    if let Err(e) = session::restore_session(&sess, &mut self.tab_manager) {
                        log::error!("Failed to restore session: {}", e);
                    } else {
                        self.invalidate_cache();
                        log::info!("Session loaded from {:?}", path);
                    }
                }
                Err(e) => log::error!("Failed to load session: {}", e),
            }
        }
    }

    fn render_diff_panel(&mut self, ui: &mut Ui) {
        let mut close_diff = false;

        if let Some(ref result) = self.diff_result {
            ui.separator();
            let stats_text = format!(
                "Diff: {} same, {} added, {} removed, {} changed",
                result.stats.same,
                result.stats.added,
                result.stats.removed,
                result.stats.changed
            );
            ui.horizontal(|ui| {
                ui.label(RichText::new(stats_text).strong());
                if ui.button("✕ Close").clicked() {
                    close_diff = true;
                }
            });

            egui::ScrollArea::vertical()
                .id_salt("diff_panel")
                .max_height(200.0)
                .show(ui, |ui| {
                    for line in &result.lines {
                        match line {
                            diff_tool::DiffLine::Same(text) => {
                                ui.label(format!("  {text}"));
                            }
                            diff_tool::DiffLine::Added(text) => {
                                ui.label(
                                    RichText::new(format!("+ {text}"))
                                        .color(Color32::from_rgb(80, 200, 80)),
                                );
                            }
                            diff_tool::DiffLine::Removed(text) => {
                                ui.label(
                                    RichText::new(format!("- {text}"))
                                        .color(Color32::from_rgb(220, 80, 80)),
                                );
                            }
                            diff_tool::DiffLine::Changed { old, new } => {
                                ui.label(
                                    RichText::new(format!("- {old}"))
                                        .color(Color32::from_rgb(220, 180, 60)),
                                );
                                ui.label(
                                    RichText::new(format!("+ {new}"))
                                        .color(Color32::from_rgb(220, 180, 60)),
                                );
                            }
                        }
                    }
                });
        }

        if close_diff {
            self.diff_result = None;
        }
    }

    fn render_settings_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Settings", |ui| {
            if ui.button("Preferences...").clicked() {
                self.show_preferences = !self.show_preferences;
                ui.close_menu();
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
            let mut clone_to_other: Option<usize> = None;

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
                    if ui.button("Clone to Other View").clicked() {
                        clone_to_other = Some(*i);
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
            if let Some(idx) = clone_to_other {
                if !self.split_view.enabled {
                    self.split_view.enable(SplitOrientation::Horizontal);
                }
                self.split_view.second_tab_index = Some(idx);
            }
        });
    }

    fn render_editor(&mut self, ui: &mut Ui) {
        self.sync_cache_from_buffer();
        let bookmarks = self.tab_manager.active_document().bookmarks.all_bookmarks();
        editor_widget(
            ui,
            &mut self.text_cache,
            self.font_size,
            self.show_line_numbers,
            self.word_wrap,
            &self.search_results_matches,
            self.current_match_index,
            &self.syntax_highlighter,
            &self.file_extension,
            &bookmarks,
            self.matching_bracket_pos,
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
            if self.macro_recorder.is_recording() {
                ui.label(RichText::new("● REC").color(Color32::from_rgb(220, 60, 60)).strong());
                ui.separator();
            }
            if self.show_hex_viewer {
                if let Some(ref hv) = self.hex_view {
                    ui.label(format!("HEX | {} bytes | offset 0x{:08X}", hv.bytes.len(), hv.cursor_offset));
                }
            } else {
                ui.label(format!("Ln {}, Col {}", line, col));
            }
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

    // --- Bookmark actions ---

    fn action_toggle_bookmark(&mut self) {
        let line = self.tab_manager.active_document().cursor.position.line;
        self.tab_manager.active_document_mut().bookmarks.toggle(line);
    }

    fn action_next_bookmark(&mut self) {
        let line = self.tab_manager.active_document().cursor.position.line;
        if let Some(next) = self.tab_manager.active_document().bookmarks.next_bookmark(line) {
            self.tab_manager.active_document_mut().cursor.position.line = next;
            self.tab_manager.active_document_mut().cursor.position.col = 0;
        }
    }

    fn action_prev_bookmark(&mut self) {
        let line = self.tab_manager.active_document().cursor.position.line;
        if let Some(prev) = self.tab_manager.active_document().bookmarks.prev_bookmark(line) {
            self.tab_manager.active_document_mut().cursor.position.line = prev;
            self.tab_manager.active_document_mut().cursor.position.col = 0;
        }
    }

    fn action_clear_bookmarks(&mut self) {
        self.tab_manager.active_document_mut().bookmarks.clear();
    }

    fn action_copy_bookmarked_lines(&mut self) {
        self.sync_cache_from_buffer();
        let lines = self.tab_manager.active_document().bookmarks.bookmarked_lines(&self.text_cache);
        if !lines.is_empty() {
            let text = lines.join("\n");
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(text);
            }
        }
    }

    fn action_remove_bookmarked_lines(&mut self) {
        self.sync_cache_from_buffer();
        let result = self.tab_manager.active_document().bookmarks.remove_bookmarked_lines(&self.text_cache);
        self.text_cache = result;
        self.sync_buffer_from_cache();
        self.tab_manager.active_document_mut().bookmarks.clear();
    }

    fn action_remove_unbookmarked_lines(&mut self) {
        self.sync_cache_from_buffer();
        let result = self.tab_manager.active_document().bookmarks.remove_unbookmarked_lines(&self.text_cache);
        self.text_cache = result;
        self.sync_buffer_from_cache();
        self.tab_manager.active_document_mut().bookmarks.clear();
    }

    // --- Export actions ---

    fn action_export_html(&mut self) {
        self.sync_cache_from_buffer();
        let html = export::export_html(&self.text_cache, &self.file_extension, &self.syntax_highlighter);
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Export as HTML")
            .add_filter("HTML", &["html", "htm"])
            .save_file()
        {
            let _ = std::fs::write(path, html);
        }
    }

    fn action_export_rtf(&mut self) {
        self.sync_cache_from_buffer();
        let rtf = export::export_rtf(&self.text_cache);
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Export as RTF")
            .add_filter("RTF", &["rtf"])
            .save_file()
        {
            let _ = std::fs::write(path, rtf);
        }
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
        let ctrl_shift_j = ctx.input(|i| i.key_pressed(egui::Key::J) && i.modifiers.ctrl && i.modifiers.shift);
        let ctrl_shift_r = ctx.input(|i| i.key_pressed(egui::Key::R) && i.modifiers.ctrl && i.modifiers.shift);
        let ctrl_shift_p = ctx.input(|i| i.key_pressed(egui::Key::P) && i.modifiers.ctrl && i.modifiers.shift);
        let f2 = ctx.input(|i| i.key_pressed(egui::Key::F2) && !i.modifiers.ctrl && !i.modifiers.shift);
        let shift_f2 = ctx.input(|i| i.key_pressed(egui::Key::F2) && i.modifiers.shift && !i.modifiers.ctrl);
        let ctrl_f2 = ctx.input(|i| i.key_pressed(egui::Key::F2) && i.modifiers.ctrl && !i.modifiers.shift);
        let ctrl_g = ctx.input(|i| i.key_pressed(egui::Key::G) && i.modifiers.ctrl && !i.modifiers.shift);
        let ctrl_bracket = ctx.input(|i| i.key_pressed(egui::Key::CloseBracket) && i.modifiers.ctrl);
        let ctrl_p = ctx.input(|i| i.key_pressed(egui::Key::P) && i.modifiers.ctrl && !i.modifiers.shift);

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
        if ctrl_shift_j { self.action_json_format(); }
        if ctrl_shift_r { self.action_toggle_macro_recording(); }
        if ctrl_shift_p { self.action_play_last_macro(); }
        if ctrl_f2 { self.action_toggle_bookmark(); }
        if f2 { self.action_next_bookmark(); }
        if shift_f2 { self.action_prev_bookmark(); }
        if ctrl_g {
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::G));
            self.action_goto_line();
        }
        if ctrl_bracket {
            if let Some(match_byte_pos) = self.matching_bracket_pos {
                let doc = self.tab_manager.active_document();
                if let Some((line, col)) = doc.buffer.line_col(match_byte_pos) {
                    self.tab_manager.active_document_mut().cursor.set_position(line, col);
                }
            }
        }
        if ctrl_p {
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::P));
            self.action_show_command_palette();
        }
    }
}

impl eframe::App for NotepadApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-save session on close request
        if ctx.input(|i| i.viewport().close_requested()) && self.app_settings.remember_session {
            self.action_save_session();
        }

        self.handle_keyboard_shortcuts(ctx);

        // Update bracket matching
        {
            let doc = self.tab_manager.active_document();
            let text = doc.buffer.text();
            if let Some(byte_pos) = doc.buffer.byte_offset(doc.cursor.position.line, doc.cursor.position.col) {
                self.matching_bracket_pos = crate::editor::brackets::find_matching_bracket(&text, byte_pos);
            } else {
                self.matching_bracket_pos = None;
            }
        }

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
            let is_md = self
                .tab_manager
                .active_document()
                .path
                .as_ref()
                .map(|p| {
                    p.extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("markdown"))
                        .unwrap_or(false)
                })
                .unwrap_or(false);

            let is_csv = self
                .tab_manager
                .active_document()
                .path
                .as_ref()
                .map(|p| {
                    p.extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.eq_ignore_ascii_case("csv") || e.eq_ignore_ascii_case("tsv"))
                        .unwrap_or(false)
                })
                .unwrap_or(false);

            if self.show_hex_viewer {
                // Hex viewer mode
                if let Some(ref mut hv) = self.hex_view {
                    hex_viewer::render_hex(ui, hv);
                }
            } else if self.show_csv_viewer && is_csv {
                // CSV viewer mode
                ui.horizontal(|ui| {
                    ui.label("Delimiter:");
                    let delimiters = [(',', "Comma"), ('\t', "Tab"), (';', "Semicolon"), ('|', "Pipe")];
                    for (d, label) in &delimiters {
                        if ui.selectable_label(self.csv_delimiter == *d, *label).clicked() {
                            self.csv_delimiter = *d;
                            self.sync_cache_from_buffer();
                            self.csv_data = Some(csv_viewer::parse_csv(
                                &self.text_cache,
                                self.csv_delimiter,
                                true,
                            ));
                        }
                    }
                    ui.separator();
                    if ui.button("Switch to Text View").clicked() {
                        if let Some(ref data) = self.csv_data {
                            self.text_cache = csv_viewer::to_csv(data);
                            self.sync_buffer_from_cache();
                        }
                        self.show_csv_viewer = false;
                        self.csv_data = None;
                    }
                });
                ui.separator();
                if let Some(ref mut data) = self.csv_data {
                    egui::ScrollArea::both().show(ui, |ui| {
                        csv_viewer::render_csv_table(ui, data);
                    });
                }
            } else if self.show_markdown_preview && is_md {
                self.sync_cache_from_buffer();
                let md_text = self.text_cache.clone();
                ui.columns(2, |columns| {
                    columns[0].push_id("editor_col", |ui| {
                        self.render_editor(ui);
                    });
                    columns[1].push_id("preview_col", |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            markdown_viewer::render_markdown(ui, &md_text);
                        });
                    });
                });
            } else if self.split_view.enabled {
                // Split view mode
                let second_idx = self.split_view.second_tab_index.unwrap_or(0);
                let orientation = self.split_view.orientation;
                let ratio = self.split_view.ratio;
                let available = ui.available_rect_before_wrap();
                let divider_width = 6.0;

                match orientation {
                    SplitOrientation::Horizontal => {
                        let total_width = available.width();
                        let left_width = (total_width * ratio - divider_width / 2.0).max(50.0);
                        let right_width = (total_width - left_width - divider_width).max(50.0);

                        ui.horizontal(|ui| {
                            ui.allocate_ui(egui::vec2(left_width, available.height()), |ui| {
                                self.render_editor(ui);
                            });

                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(divider_width, available.height()),
                                egui::Sense::drag(),
                            );
                            ui.painter().rect_filled(
                                rect,
                                0.0,
                                if response.hovered() || response.dragged() {
                                    egui::Color32::from_gray(120)
                                } else {
                                    egui::Color32::from_gray(80)
                                },
                            );
                            if response.dragged() {
                                let delta = response.drag_delta().x;
                                self.split_view.ratio = ((ratio * total_width + delta) / total_width)
                                    .clamp(0.1, 0.9);
                            }

                            ui.allocate_ui(egui::vec2(right_width, available.height()), |ui| {
                                // Render second panel with the second tab
                                let second_text = self.tab_manager
                                    .get_document(second_idx)
                                    .map(|d| d.buffer.text())
                                    .unwrap_or_default();
                                let mut text = second_text;
                                ui.add(
                                    egui::TextEdit::multiline(&mut text)
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(f32::INFINITY)
                                );
                            });
                        });
                    }
                    SplitOrientation::Vertical => {
                        let total_height = available.height();
                        let top_height = (total_height * ratio - divider_width / 2.0).max(50.0);
                        let bottom_height = (total_height - top_height - divider_width).max(50.0);

                        ui.vertical(|ui| {
                            ui.allocate_ui(egui::vec2(available.width(), top_height), |ui| {
                                self.render_editor(ui);
                            });

                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(available.width(), divider_width),
                                egui::Sense::drag(),
                            );
                            ui.painter().rect_filled(
                                rect,
                                0.0,
                                if response.hovered() || response.dragged() {
                                    egui::Color32::from_gray(120)
                                } else {
                                    egui::Color32::from_gray(80)
                                },
                            );
                            if response.dragged() {
                                let delta = response.drag_delta().y;
                                self.split_view.ratio = ((ratio * total_height + delta) / total_height)
                                    .clamp(0.1, 0.9);
                            }

                            ui.allocate_ui(egui::vec2(available.width(), bottom_height), |ui| {
                                let second_text = self.tab_manager
                                    .get_document(second_idx)
                                    .map(|d| d.buffer.text())
                                    .unwrap_or_default();
                                let mut text = second_text;
                                ui.add(
                                    egui::TextEdit::multiline(&mut text)
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(f32::INFINITY)
                                );
                            });
                        });
                    }
                }
            } else {
                self.render_editor(ui);
            }

            // Diff panel at the bottom
            self.render_diff_panel(ui);
        });

        // Macro repeat dialog
        if self.show_macro_repeat_dialog {
            egui::Window::new("Play Macro Multiple Times")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Repeat count:");
                        ui.text_edit_singleline(&mut self.macro_repeat_count);
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Play").clicked() {
                            if let Ok(n) = self.macro_repeat_count.parse::<usize>() {
                                if let Some(ref m) = self.last_macro.clone() {
                                    let doc = self.tab_manager.active_document_mut();
                                    MacroRecorder::play_macro_n_times(m, doc, n);
                                    self.invalidate_cache();
                                }
                            }
                            self.show_macro_repeat_dialog = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_macro_repeat_dialog = false;
                        }
                    });
                });
        }

        // Go to Line dialog
        if self.show_goto_line {
            let line_count = self.tab_manager.active_document().buffer.line_count();
            let mut goto_target: Option<usize> = None;
            let mut close = false;
            egui::Window::new("Go to Line")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("Line (1-{}):", line_count));
                        let response = ui.text_edit_singleline(&mut self.goto_line_input);
                        if !response.has_focus() {
                            response.request_focus();
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Go").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if let Ok(line_num) = self.goto_line_input.parse::<usize>() {
                                goto_target = Some(line_num.saturating_sub(1).min(line_count.saturating_sub(1)));
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            if let Some(target) = goto_target {
                self.tab_manager.active_document_mut().cursor.set_position(target, 0);
                self.show_goto_line = false;
            } else if close {
                self.show_goto_line = false;
            }
        }

        // Command Palette
        if self.show_command_palette {
            let commands = self.get_commands();
            let mut selected: Option<usize> = None;
            let mut close = false;

            egui::Window::new("Command Palette")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .anchor(egui::Align2::CENTER_TOP, [0.0, 50.0])
                .fixed_size([400.0, 300.0])
                .show(ctx, |ui| {
                    let response = ui.text_edit_singleline(&mut self.command_palette_input);
                    if !response.has_focus() {
                        response.request_focus();
                    }

                    // Filter commands
                    let query = self.command_palette_input.to_lowercase();
                    let filtered: Vec<usize> = (0..commands.len())
                        .filter(|&i| {
                            query.is_empty() || commands[i].0.to_lowercase().contains(&query)
                        })
                        .collect();

                    ui.separator();

                    egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                        for &idx in &filtered {
                            let (name, shortcut) = commands[idx];
                            let label = if shortcut.is_empty() {
                                name.to_string()
                            } else {
                                format!("{}  ({})", name, shortcut)
                            };
                            if ui.selectable_label(false, &label).clicked() {
                                selected = Some(idx);
                            }
                        }
                    });

                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        close = true;
                    }
                });

            if let Some(idx) = selected {
                self.execute_command(idx);
            } else if close {
                self.show_command_palette = false;
            }
        }

        // File changed externally dialog
        if !self.files_changed_externally.is_empty() {
            let files = self.files_changed_externally.clone();
            let mut reload_all = false;
            let mut dismiss = false;
            egui::Window::new("Files Changed Externally")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("The following files have been modified outside the editor:");
                    for f in &files {
                        ui.label(format!("  • {}", f.display()));
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Reload All").clicked() {
                            reload_all = true;
                        }
                        if ui.button("Ignore").clicked() {
                            dismiss = true;
                        }
                    });
                });
            if reload_all {
                for path in &files {
                    for i in 0..self.tab_manager.tab_count() {
                        if let Some(doc) = self.tab_manager.get_document(i) {
                            if doc.path.as_ref() == Some(path) {
                                let _ = self.tab_manager.reload_tab(i);
                                if let Ok(meta) = std::fs::metadata(path) {
                                    if let Ok(modified) = meta.modified() {
                                        self.file_mod_times.insert(path.clone(), modified);
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
                self.files_changed_externally.clear();
                self.invalidate_cache();
            } else if dismiss {
                // Update mod times to current
                for path in &files {
                    if let Ok(meta) = std::fs::metadata(path) {
                        if let Ok(modified) = meta.modified() {
                            self.file_mod_times.insert(path.clone(), modified);
                        }
                    }
                }
                self.files_changed_externally.clear();
            }
        }

        // Preferences window
        if self.show_preferences {
            let mut open = self.show_preferences;
            egui::Window::new("Preferences")
                .open(&mut open)
                .resizable(true)
                .default_width(400.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.heading("Editor");
                        ui.horizontal(|ui| {
                            ui.label("Font size:");
                            ui.add(egui::DragValue::new(&mut self.app_settings.font_size).range(8.0..=48.0).speed(0.5));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Tab size:");
                            let mut tab = self.app_settings.tab_size as f32;
                            if ui.add(egui::DragValue::new(&mut tab).range(1.0..=16.0).speed(0.5)).changed() {
                                self.app_settings.tab_size = tab as usize;
                            }
                        });
                        ui.checkbox(&mut self.app_settings.use_spaces, "Use spaces for tabs");
                        ui.checkbox(&mut self.app_settings.word_wrap, "Word wrap");
                        ui.checkbox(&mut self.app_settings.show_line_numbers, "Show line numbers");
                        ui.checkbox(&mut self.app_settings.show_whitespace, "Show whitespace");
                        ui.checkbox(&mut self.app_settings.show_status_bar, "Show status bar");
                        ui.checkbox(&mut self.app_settings.auto_indent, "Auto indent");
                        ui.checkbox(&mut self.app_settings.auto_close_brackets, "Auto close brackets");

                        ui.separator();
                        ui.heading("Files");
                        ui.checkbox(&mut self.app_settings.auto_save, "Auto save");
                        if self.app_settings.auto_save {
                            ui.horizontal(|ui| {
                                ui.label("Interval (secs):");
                                let mut interval = self.app_settings.auto_save_interval_secs as f32;
                                if ui.add(egui::DragValue::new(&mut interval).range(10.0..=3600.0).speed(5.0)).changed() {
                                    self.app_settings.auto_save_interval_secs = interval as u64;
                                }
                            });
                        }
                        ui.checkbox(&mut self.app_settings.remember_session, "Remember session");

                        ui.separator();
                        ui.heading("Appearance");
                        ui.checkbox(&mut self.app_settings.highlight_current_line, "Highlight current line");

                        ui.separator();
                        ui.heading("Search");
                        ui.checkbox(&mut self.app_settings.search_wrap_around, "Wrap around");
                        ui.checkbox(&mut self.app_settings.search_case_sensitive, "Case sensitive");

                        ui.separator();
                        if ui.button("Save Settings").clicked() {
                            // Apply settings to app state
                            self.font_size = self.app_settings.font_size;
                            self.word_wrap = self.app_settings.word_wrap;
                            self.show_line_numbers = self.app_settings.show_line_numbers;
                            self.show_whitespace = self.app_settings.show_whitespace;
                            self.show_status_bar = self.app_settings.show_status_bar;
                            let _ = self.app_settings.save(&crate::io::settings::AppSettings::settings_path());
                        }
                    });
                });
            self.show_preferences = open;
        }

        // File change detection
        if self.last_file_check.elapsed().as_secs() >= 2 {
            self.last_file_check = std::time::Instant::now();
            let mut changed = Vec::new();
            for i in 0..self.tab_manager.tab_count() {
                if let Some(doc) = self.tab_manager.get_document(i) {
                    if let Some(ref path) = doc.path {
                        if let Ok(meta) = std::fs::metadata(path) {
                            if let Ok(modified) = meta.modified() {
                                if let Some(prev) = self.file_mod_times.get(path) {
                                    if modified > *prev {
                                        changed.push(path.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !changed.is_empty() {
                self.files_changed_externally = changed;
            }
        }

        // Auto-save
        if self.app_settings.auto_save {
            let elapsed = self.last_auto_save.elapsed().as_secs();
            if elapsed >= self.app_settings.auto_save_interval_secs {
                self.last_auto_save = std::time::Instant::now();
                for i in 0..self.tab_manager.tab_count() {
                    if let Some(doc) = self.tab_manager.get_document(i) {
                        if doc.is_modified() && doc.path.is_some() {
                            let _ = self.tab_manager.save_tab(i);
                        }
                    }
                }
            }
        }

        self.request_repaint_if_dialog(ctx);
    }
}
