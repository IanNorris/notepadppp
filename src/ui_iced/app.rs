use iced::keyboard;
use iced::widget::{button, column, container, mouse_area, opaque, row, scrollable, stack, text, text_editor, Space};
use iced::advanced::text::Wrapping;
use iced::{Element, Length, Subscription, Task, Theme};

use crate::editor::document::{Encoding, LineEnding};
use crate::editor::folding::FoldManager;
use crate::editor::macros::MacroRecorder;
use crate::editor::tab_manager::TabManager;
use crate::search::{SearchEngine, SearchMatch, SearchMode};
use crate::search::find_in_files::FileSearchResult;

use super::highlighter::{SyntectHighlighter, SyntectSettings};
use super::menu_bar;
use super::theme::{AppColors, AppTheme};

#[derive(Debug, Clone, PartialEq)]
pub enum SplitMode {
    None,
    Horizontal, // side by side
    Vertical,   // top/bottom
}

/// Per-tab state that pairs an iced text_editor::Content with our Document index.
pub struct TabContent {
    pub content: text_editor::Content,
}

impl TabContent {
    fn new() -> Self {
        Self {
            content: text_editor::Content::new(),
        }
    }

    fn with_text(s: &str) -> Self {
        Self {
            content: text_editor::Content::with_text(s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    EditorAction(text_editor::Action),
    NewTab,
    OpenFile,
    FileOpened(Result<(String, std::path::PathBuf), String>),
    Save,
    SaveAs,
    FileSaved(Result<std::path::PathBuf, String>),
    CloseTab(usize),
    SelectTab(usize),
    Undo,
    Redo,

    // Menu state
    MenuToggle(String),
    MenuSwitch(String),
    MenuClose,
    SubMenuToggle(String),

    // File
    SaveAll,
    CloseAll,
    SaveSession,
    LoadSession,
    ToggleAutoRestore,
    ExportHtml,
    ExportRtf,
    Exit,

    // Edit
    SelectAll,
    ToggleComment,
    // Line operations
    DuplicateLine,
    DeleteLine,
    MoveLineUp,
    MoveLineDown,
    SortAsc,
    SortDesc,
    RemoveEmpty,
    RemoveDuplicates,
    TrimTrailing,
    JoinLines,
    SplitLine,
    InsertAbove,
    InsertBelow,
    ReverseLines,
    SortCaseInsensitive,
    SortNumeric,
    TrimLeading,
    TrimBoth,
    // Case
    UpperCase,
    LowerCase,
    TitleCase,
    SentenceCase,
    InverseCase,
    // Encoding / line ending
    SetEncoding(Encoding),
    SetLineEnding(LineEnding),

    // Search
    ShowFind,
    ShowReplace,
    ShowFindInFiles,
    SelectAllOccurrences,
    GotoLine,
    ToggleBookmark,
    NextBookmark,
    PrevBookmark,
    ClearBookmarks,
    CopyBookmarkedLines,
    RemoveBookmarkedLines,
    RemoveUnbookmarkedLines,

    // View
    ToggleWordWrap,
    ToggleLineNumbers,
    ToggleWhitespace,
    ToggleStatusBar,
    ToggleMinimap,
    ToggleFunctionList,
    SplitHorizontal,
    SplitVertical,
    RemoveSplit,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ToggleFold,
    ToggleFoldAt(usize),
    FoldAll,
    UnfoldAll,
    FoldLevel(usize),

    // Language
    SetLanguage(String),

    // Tools
    JsonFormat,
    JsonCompact,
    JsonValidate,
    JsonSortKeys,
    ToggleMarkdownPreview,
    ToggleCsvViewer,
    MimeBase64Encode,
    MimeBase64Decode,
    MimeUrlEncode,
    MimeUrlDecode,
    MimeHtmlEncode,
    MimeHtmlDecode,
    MimeHexEncode,
    MimeHexDecode,
    CompareFiles,
    ToggleHexViewer,

    // Split pane
    SplitEditorAction(text_editor::Action),
    SplitNewTab,
    SplitSelectTab(usize),
    SplitCloseTab(usize),
    SetActivePane(usize),

    // Tab management across panes
    NewTabInActivePane,
    FileDropped(std::path::PathBuf),
    MoveTabToSplit(usize),
    MoveTabFromSplit(usize),

    // Tab context menu
    TabContextMenu(usize, bool), // (tab_index, is_split_pane)
    TabContextClose(usize, bool),
    TabContextCloseOthers(usize, bool),
    TabContextCloseAll(bool),
    TabContextMoveToOtherPane(usize, bool),
    CloseTabContextMenu,

    // Macro
    ToggleMacroRecording,
    PlayLastMacro,
    PlayMacroMultiple,

    // Settings
    ShowPreferences,
    ShowKeybindings,
    SavePreferences,
    CancelPreferences,
    CloseKeybindingsDialog,
    PrefFontSizeIncrease,
    PrefFontSizeDecrease,
    PrefTabSizeChanged(String),
    PrefSetTheme(String),
    PrefToggleAutoSave(bool),
    PrefToggleWordWrap(bool),
    PrefToggleLineNumbers(bool),
    PrefToggleWhitespace(bool),

    // Help
    ShowAbout,
    CloseAbout,

    // Theme
    SetTheme(String),

    // Search panel
    FindQueryChanged(String),
    ReplaceTextChanged(String),
    ToggleCaseSensitive,
    ToggleWholeWord,
    ToggleRegex,
    FindNext,
    FindPrev,
    ReplaceNext,
    ReplaceAll,
    CloseSearch,
    ClickSearchResult(usize),

    // Go to line
    GotoLineInputChanged(String),
    GotoLineConfirm,
    GotoLineClose,

    // Async results
    SessionFileChosen(Result<std::path::PathBuf, String>),
    ExportSaved(Result<(), String>),
    CompareFileLoaded(Result<String, String>),

    // Keyboard
    EscapePressed,

    // Drag floating panels
    DragFindStart,
    DragFindMove(iced::Point),
    DragFindEnd,

    // Panel messages
    GotoSymbol(usize),
    CsvSortColumn(usize),
    CsvToggleHeaders,

    // Find in Files
    FifQueryChanged(String),
    FifDirectoryChanged(String),
    FifFileFilterChanged(String),
    FifToggleRecursive,
    FifToggleCaseSensitive,
    FifToggleRegex,
    FifSearch,
    FifSearchComplete(Result<Vec<FileSearchResult>, String>),
    FifClickResult(std::path::PathBuf, usize),
    CloseFindInFiles,
    DragFifStart,
    DragFifMove(iced::Point),
    DragFifEnd,
}

pub struct NotepadIced {
    pub tab_manager: TabManager,
    pub tab_contents: Vec<TabContent>,

    // Menu state
    pub active_menu: Option<String>,
    pub expanded_submenus: std::collections::HashSet<String>,

    // View toggles
    pub word_wrap: bool,
    pub show_line_numbers: bool,
    pub show_whitespace: bool,
    pub show_status_bar: bool,
    pub show_minimap: bool,
    pub show_function_list: bool,
    pub show_markdown_preview: bool,
    pub show_csv_viewer: bool,
    pub show_hex_viewer: bool,

    // CSV viewer state
    pub csv_sort_column: Option<usize>,
    pub csv_sort_ascending: bool,
    pub csv_has_headers: bool,

    // Session
    pub auto_restore_session: bool,

    // Search
    pub show_find: bool,
    pub show_replace: bool,
    pub search_query: String,
    pub replace_text: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub use_regex: bool,
    pub search_matches: Vec<SearchMatch>,
    pub current_match_index: Option<usize>,

    // Go to line
    pub show_goto_line: bool,
    pub goto_line_input: String,

    // About
    pub show_about: bool,

    // Preferences dialog
    pub show_preferences: bool,
    pub pref_font_size: f32,
    pub pref_tab_size: String,
    pub pref_theme: String,
    pub pref_auto_save: bool,
    pub pref_word_wrap: bool,
    pub pref_show_line_numbers: bool,
    pub pref_show_whitespace: bool,

    // Keybindings dialog
    pub show_keybindings: bool,

    // Floating panel positions (None = default position)
    pub find_panel_pos: Option<(f32, f32)>,
    pub dragging_find_panel: bool,
    pub drag_offset: (f32, f32),
    pub last_mouse_pos: iced::Point,

    // Zoom
    pub font_size: f32,

    // Macros
    pub macro_recorder: MacroRecorder,
    pub last_macro: Option<crate::editor::macros::Macro>,

    // Syntax highlighting
    pub file_extension: String,

    // Code folding
    pub fold_manager: FoldManager,

    // Find in Files
    pub show_find_in_files: bool,
    pub fif_query: String,
    pub fif_directory: String,
    pub fif_file_filter: String,
    pub fif_recursive: bool,
    pub fif_case_sensitive: bool,
    pub fif_use_regex: bool,
    pub fif_results: Vec<FileSearchResult>,
    pub fif_searching: bool,
    pub fif_panel_pos: Option<(f32, f32)>,
    pub dragging_fif_panel: bool,
    pub fif_drag_offset: (f32, f32),

    // Split view
    pub split_mode: SplitMode,
    pub split_tab_manager: TabManager,
    pub split_tab_contents: Vec<TabContent>,
    pub split_font_size: f32,
    pub split_file_extension: String,
    pub split_show_markdown_preview: bool,
    pub active_pane: usize, // 0 = primary, 1 = secondary

    // Theme
    pub theme: AppTheme,

    // Tab context menu: (tab_index, is_split_pane)
    pub tab_context_menu: Option<(usize, bool)>,
}

impl Default for NotepadIced {
    fn default() -> Self {
        let settings = crate::io::settings::AppSettings::load(
            &crate::io::settings::AppSettings::settings_path(),
        )
        .unwrap_or_default();
        let theme = AppColors::theme_by_name(&settings.theme);
        Self {
            tab_manager: TabManager::new(),
            tab_contents: vec![TabContent::new()],
            active_menu: None,
            expanded_submenus: std::collections::HashSet::new(),
            word_wrap: settings.word_wrap,
            show_line_numbers: settings.show_line_numbers,
            show_whitespace: settings.show_whitespace,
            show_status_bar: true,
            show_minimap: false,
            show_function_list: false,
            show_markdown_preview: false,
            show_csv_viewer: false,
            show_hex_viewer: false,
            csv_sort_column: None,
            csv_sort_ascending: true,
            csv_has_headers: true,
            auto_restore_session: false,
            show_find: false,
            show_replace: false,
            search_query: String::new(),
            replace_text: String::new(),
            case_sensitive: false,
            whole_word: false,
            use_regex: false,
            search_matches: Vec::new(),
            current_match_index: None,
            show_goto_line: false,
            goto_line_input: String::new(),
            show_about: false,
            show_preferences: false,
            pref_font_size: settings.font_size,
            pref_tab_size: settings.tab_size.to_string(),
            pref_theme: settings.theme.clone(),
            pref_auto_save: settings.auto_save,
            pref_word_wrap: settings.word_wrap,
            pref_show_line_numbers: settings.show_line_numbers,
            pref_show_whitespace: settings.show_whitespace,
            show_keybindings: false,
            find_panel_pos: None, // None = right-aligned default
            dragging_find_panel: false,
            drag_offset: (0.0, 0.0),
            last_mouse_pos: iced::Point::ORIGIN,
            font_size: settings.font_size,
            macro_recorder: MacroRecorder::new(),
            last_macro: None,
            file_extension: String::new(),
            fold_manager: FoldManager::new(),
            show_find_in_files: false,
            fif_query: String::new(),
            fif_directory: String::new(),
            fif_file_filter: String::from("*.*"),
            fif_recursive: true,
            fif_case_sensitive: false,
            fif_use_regex: false,
            fif_results: Vec::new(),
            fif_searching: false,
            fif_panel_pos: None,
            dragging_fif_panel: false,
            fif_drag_offset: (0.0, 0.0),
            split_mode: SplitMode::None,
            split_tab_manager: TabManager::new(),
            split_tab_contents: vec![TabContent::new()],
            split_font_size: 14.0,
            split_file_extension: String::new(),
            split_show_markdown_preview: false,
            active_pane: 0,
            theme,
            tab_context_menu: None,
        }
    }
}

pub fn title(state: &NotepadIced) -> String {
    let doc = state.tab_manager.active_document();
    let tab_title = state
        .tab_manager
        .get_tab_title(state.tab_manager.active_index());
    let modified = if doc.is_modified() { " •" } else { "" };
    format!("{}{} — Notepad+++", tab_title, modified)
}

pub fn update(state: &mut NotepadIced, message: Message) -> Task<Message> {
    // Close menu for most actions (except MenuToggle/MenuClose/EditorAction and dialog-internal messages)
    let should_close_menu = !matches!(
        message,
        Message::MenuToggle(_) | Message::MenuSwitch(_) | Message::MenuClose | Message::SubMenuToggle(_) | Message::EditorAction(_)
            | Message::FileOpened(_) | Message::FileSaved(_)
            | Message::FindQueryChanged(_) | Message::ReplaceTextChanged(_)
            | Message::ToggleCaseSensitive | Message::ToggleWholeWord | Message::ToggleRegex
            | Message::FindNext | Message::FindPrev | Message::ReplaceNext | Message::ReplaceAll
            | Message::CloseSearch | Message::ClickSearchResult(_)
            | Message::GotoLineInputChanged(_) | Message::GotoLineConfirm | Message::GotoLineClose
            | Message::CloseAbout
            | Message::SavePreferences | Message::CancelPreferences
            | Message::PrefFontSizeIncrease | Message::PrefFontSizeDecrease
            | Message::PrefTabSizeChanged(_) | Message::PrefSetTheme(_)
            | Message::PrefToggleAutoSave(_) | Message::PrefToggleWordWrap(_)
            | Message::PrefToggleLineNumbers(_) | Message::PrefToggleWhitespace(_)
            | Message::CloseKeybindingsDialog
            | Message::SessionFileChosen(_) | Message::ExportSaved(_) | Message::CompareFileLoaded(_)
            | Message::EscapePressed
            | Message::DragFindStart | Message::DragFindMove(_) | Message::DragFindEnd
            | Message::FifQueryChanged(_) | Message::FifDirectoryChanged(_) | Message::FifFileFilterChanged(_)
            | Message::FifToggleRecursive | Message::FifToggleCaseSensitive | Message::FifToggleRegex
            | Message::FifSearch | Message::FifSearchComplete(_) | Message::FifClickResult(_, _)
            | Message::CloseFindInFiles
            | Message::DragFifStart | Message::DragFifMove(_) | Message::DragFifEnd
            | Message::SplitEditorAction(_) | Message::SplitNewTab | Message::SplitSelectTab(_)
            | Message::SplitCloseTab(_) | Message::SetActivePane(_)
            | Message::NewTabInActivePane | Message::FileDropped(_)
            | Message::MoveTabToSplit(_) | Message::MoveTabFromSplit(_)
            | Message::TabContextMenu(_, _) | Message::TabContextClose(_, _)
            | Message::TabContextCloseOthers(_, _) | Message::TabContextCloseAll(_)
            | Message::TabContextMoveToOtherPane(_, _) | Message::CloseTabContextMenu
    );
    if should_close_menu {
        state.active_menu = None;
    }

    match message {
        Message::EditorAction(action) => {
            state.active_pane = 0;
            if let Some(tc) = state
                .tab_contents
                .get_mut(state.tab_manager.active_index())
            {
                tc.content.perform(action);
                let new_text = tc.content.text();
                let doc = state.tab_manager.active_document_mut();
                let old_text = doc.buffer.text();
                if new_text != old_text {
                    let len = doc.buffer.len_bytes();
                    if len > 0 {
                        doc.buffer.delete(0, len);
                    }
                    doc.buffer.insert(0, &new_text);
                    state.fold_manager.detect_regions(&new_text);
                }
            }
            Task::none()
        }
        Message::NewTab => {
            if state.active_pane == 1 && state.split_mode != SplitMode::None {
                state.split_tab_manager.new_tab();
                state.split_tab_contents.push(TabContent::new());
            } else {
                state.tab_manager.new_tab();
                state.tab_contents.push(TabContent::new());
            }
            Task::none()
        }
        Message::OpenFile => Task::perform(
            async {
                let handle = rfd::AsyncFileDialog::new()
                    .set_title("Open File")
                    .pick_file()
                    .await;
                match handle {
                    Some(h) => {
                        let path = h.path().to_path_buf();
                        match std::fs::read_to_string(&path) {
                            Ok(content) => Ok((content, path)),
                            Err(e) => Err(e.to_string()),
                        }
                    }
                    None => Err("Cancelled".to_string()),
                }
            },
            Message::FileOpened,
        ),
        Message::FileOpened(result) => {
            if let Ok((content, path)) = result {
                // Extract file extension for syntax highlighting
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    state.file_extension = ext.to_lowercase();
                }
                match state.tab_manager.open_file(path) {
                    Ok(idx) => {
                        let doc = state.tab_manager.get_document(idx).unwrap();
                        let buf_text = doc.buffer.text();
                        state.fold_manager.detect_regions(&buf_text);
                        state.tab_contents.push(TabContent::with_text(&buf_text));
                    }
                    Err(_) => {
                        state.tab_manager.new_tab();
                        state.fold_manager.detect_regions(&content);
                        state.tab_contents.push(TabContent::with_text(&content));
                    }
                }
            }
            Task::none()
        }
        Message::Save => {
            let idx = state.tab_manager.active_index();
            sync_content_to_doc(state, idx);
            let has_path = state.tab_manager.active_document().path.is_some();
            if has_path {
                if let Err(e) = state.tab_manager.save_tab(idx) {
                    log::error!("Save failed: {}", e);
                }
                Task::none()
            } else {
                save_as_dialog()
            }
        }
        Message::SaveAs => {
            let idx = state.tab_manager.active_index();
            sync_content_to_doc(state, idx);
            save_as_dialog()
        }
        Message::FileSaved(result) => {
            if let Ok(path) = result {
                let idx = state.tab_manager.active_index();
                if let Err(e) = state.tab_manager.save_tab_as(idx, path) {
                    log::error!("Save As failed: {}", e);
                }
            }
            Task::none()
        }
        Message::CloseTab(idx) => {
            let actual_idx = if idx == usize::MAX {
                state.tab_manager.active_index()
            } else {
                idx
            };
            if actual_idx < state.tab_contents.len() {
                state.tab_contents.remove(actual_idx);
                state.tab_manager.close_tab(actual_idx);
                while state.tab_contents.len() < state.tab_manager.tab_count() {
                    state.tab_contents.push(TabContent::new());
                }
            }
            Task::none()
        }
        Message::SelectTab(idx) => {
            if idx < state.tab_manager.tab_count() {
                state.tab_manager.set_active(idx);
                // Update file extension from the newly active tab's path
                if let Some(path) = &state.tab_manager.active_document().path {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        state.file_extension = ext.to_lowercase();
                    } else {
                        state.file_extension.clear();
                    }
                } else {
                    state.file_extension.clear();
                }
                // Re-detect fold regions for the newly active tab
                if let Some(tc) = state.tab_contents.get(idx) {
                    let text = tc.content.text();
                    state.fold_manager.detect_regions(&text);
                }
            }
            Task::none()
        }
        Message::Undo => {
            let doc = state.tab_manager.active_document_mut();
            doc.buffer.undo();
            let buf_text = doc.buffer.text();
            let idx = state.tab_manager.active_index();
            state.tab_contents[idx] = TabContent::with_text(&buf_text);
            Task::none()
        }
        Message::Redo => {
            let doc = state.tab_manager.active_document_mut();
            doc.buffer.redo();
            let buf_text = doc.buffer.text();
            let idx = state.tab_manager.active_index();
            state.tab_contents[idx] = TabContent::with_text(&buf_text);
            Task::none()
        }

        // ── Menu state ──
        Message::MenuToggle(name) => {
            if state.active_menu.as_ref() == Some(&name) {
                state.active_menu = None;
                state.expanded_submenus.clear();
            } else {
                state.active_menu = Some(name);
                state.expanded_submenus.clear();
            }
            Task::none()
        }
        Message::MenuSwitch(name) => {
            // Switch to a different menu when one is already open (Windows-style hover behavior)
            if state.active_menu.is_some() {
                state.active_menu = Some(name);
                state.expanded_submenus.clear();
            }
            Task::none()
        }
        Message::MenuClose => {
            state.active_menu = None;
            state.expanded_submenus.clear();
            Task::none()
        }
        Message::SubMenuToggle(name) => {
            if state.expanded_submenus.contains(&name) {
                state.expanded_submenus.remove(&name);
            } else {
                state.expanded_submenus.insert(name);
            }
            Task::none()
        }

        // ── File ──
        Message::SaveAll => {
            for i in 0..state.tab_manager.tab_count() {
                sync_content_to_doc(state, i);
                if state.tab_manager.get_document(i).map_or(false, |d| d.path.is_some()) {
                    if let Err(e) = state.tab_manager.save_tab(i) {
                        log::error!("Save All failed for tab {}: {}", i, e);
                    }
                }
            }
            Task::none()
        }
        Message::CloseAll => {
            state.tab_manager.close_all();
            state.tab_contents.clear();
            state.tab_contents.push(TabContent::new());
            Task::none()
        }
        Message::SaveSession => {
            use crate::io::session;
            let sess = session::capture_session(&state.tab_manager, "session");
            if let Some(config_dir) = dirs::config_dir() {
                let dir = config_dir.join("notepadppp").join("sessions");
                let _ = std::fs::create_dir_all(&dir);
                let path = dir.join("session.json");
                let _ = session::save_session(&sess, &path);
            }
            Task::none()
        }
        Message::LoadSession => {
            Task::perform(
                async {
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("Session", &["json"])
                        .set_title("Load Session")
                        .pick_file()
                        .await;
                    match handle {
                        Some(h) => Ok(h.path().to_path_buf()),
                        None => Err("Cancelled".to_string()),
                    }
                },
                Message::SessionFileChosen,
            )
        }
        Message::ToggleAutoRestore => {
            state.auto_restore_session = !state.auto_restore_session;
            Task::none()
        }
        Message::ExportHtml => {
            let text = get_buffer_text(state);
            let doc = state.tab_manager.active_document();
            let ext = doc
                .path
                .as_ref()
                .and_then(|p| p.extension())
                .and_then(|e| e.to_str())
                .unwrap_or("txt")
                .to_string();
            let highlighter = crate::editor::syntax::SyntaxHighlighter::new();
            let html = crate::tools::export::export_html(&text, &ext, &highlighter);
            Task::perform(
                async move {
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("HTML", &["html"])
                        .set_title("Export as HTML")
                        .save_file()
                        .await;
                    match handle {
                        Some(h) => {
                            let path = h.path().to_path_buf();
                            std::fs::write(&path, &html).map_err(|e| e.to_string())
                        }
                        None => Err("Cancelled".to_string()),
                    }
                },
                Message::ExportSaved,
            )
        }
        Message::ExportRtf => {
            let text = get_buffer_text(state);
            let rtf = crate::tools::export::export_rtf(&text);
            Task::perform(
                async move {
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("RTF", &["rtf"])
                        .set_title("Export as RTF")
                        .save_file()
                        .await;
                    match handle {
                        Some(h) => {
                            let path = h.path().to_path_buf();
                            std::fs::write(&path, &rtf).map_err(|e| e.to_string())
                        }
                        None => Err("Cancelled".to_string()),
                    }
                },
                Message::ExportSaved,
            )
        }
        Message::Exit => {
            std::process::exit(0);
        }

        // ── Edit ──
        Message::SelectAll => {
            if let Some(tc) = state.tab_contents.get_mut(state.tab_manager.active_index()) {
                tc.content.perform(text_editor::Action::SelectAll);
            }
            Task::none()
        }
        Message::ToggleComment => {
            apply_line_op(state, |lines, cursor_line| {
                let language = "Rust"; // default
                let prefix = crate::editor::comments::line_comment_prefix(language).unwrap_or("//");
                let prefix_space = format!("{} ", prefix);
                if cursor_line < lines.len() {
                    let line = &lines[cursor_line];
                    let trimmed = line.trim_start();
                    if trimmed.starts_with(&prefix_space) {
                        let indent = line.len() - trimmed.len();
                        let rest = &trimmed[prefix_space.len()..];
                        lines[cursor_line] = format!("{}{}", &line[..indent], rest);
                    } else if trimmed.starts_with(prefix) {
                        let indent = line.len() - trimmed.len();
                        let rest = &trimmed[prefix.len()..];
                        lines[cursor_line] = format!("{}{}", &line[..indent], rest);
                    } else {
                        let indent = line.len() - trimmed.len();
                        lines[cursor_line] = format!("{}{} {}", &line[..indent], prefix, trimmed);
                    }
                }
            });
            Task::none()
        }

        // Line operations
        Message::DuplicateLine => {
            apply_line_op(state, |lines, cursor_line| {
                if cursor_line < lines.len() {
                    let dup = lines[cursor_line].clone();
                    lines.insert(cursor_line + 1, dup);
                }
            });
            Task::none()
        }
        Message::DeleteLine => {
            apply_line_op(state, |lines, cursor_line| {
                if cursor_line < lines.len() && lines.len() > 1 {
                    lines.remove(cursor_line);
                } else if lines.len() == 1 {
                    lines[0] = String::new();
                }
            });
            Task::none()
        }
        Message::MoveLineUp => {
            apply_line_op(state, |lines, cursor_line| {
                if cursor_line > 0 && cursor_line < lines.len() {
                    lines.swap(cursor_line, cursor_line - 1);
                }
            });
            Task::none()
        }
        Message::MoveLineDown => {
            apply_line_op(state, |lines, cursor_line| {
                if cursor_line + 1 < lines.len() {
                    lines.swap(cursor_line, cursor_line + 1);
                }
            });
            Task::none()
        }
        Message::SortAsc => {
            apply_line_op(state, |lines, _| lines.sort());
            Task::none()
        }
        Message::SortDesc => {
            apply_line_op(state, |lines, _| {
                lines.sort();
                lines.reverse();
            });
            Task::none()
        }
        Message::RemoveEmpty => {
            apply_line_op(state, |lines, _| {
                lines.retain(|l| !l.trim().is_empty());
                if lines.is_empty() {
                    lines.push(String::new());
                }
            });
            Task::none()
        }
        Message::RemoveDuplicates => {
            apply_line_op(state, |lines, _| {
                let mut seen = std::collections::HashSet::new();
                lines.retain(|l| seen.insert(l.clone()));
                if lines.is_empty() {
                    lines.push(String::new());
                }
            });
            Task::none()
        }
        Message::TrimTrailing => {
            apply_line_op(state, |lines, _| {
                for line in lines.iter_mut() {
                    *line = line.trim_end().to_string();
                }
            });
            Task::none()
        }
        Message::JoinLines => {
            apply_line_op(state, |lines, cursor_line| {
                if cursor_line < lines.len().saturating_sub(1) {
                    let next = lines.remove(cursor_line + 1);
                    lines[cursor_line].push_str(&next);
                }
            });
            Task::none()
        }
        Message::SplitLine => {
            // Split at cursor column (simplified: split in the middle)
            apply_line_op(state, |lines, cursor_line| {
                if cursor_line < lines.len() {
                    let current = lines[cursor_line].clone();
                    let mid = current.len() / 2;
                    lines[cursor_line] = current[..mid].to_string();
                    lines.insert(cursor_line + 1, current[mid..].to_string());
                }
            });
            Task::none()
        }
        Message::InsertAbove => {
            apply_line_op(state, |lines, cursor_line| {
                lines.insert(cursor_line, String::new());
            });
            Task::none()
        }
        Message::InsertBelow => {
            apply_line_op(state, |lines, cursor_line| {
                lines.insert(cursor_line + 1, String::new());
            });
            Task::none()
        }
        Message::ReverseLines => {
            apply_line_op(state, |lines, _| lines.reverse());
            Task::none()
        }
        Message::SortCaseInsensitive => {
            apply_line_op(state, |lines, _| {
                lines.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            });
            Task::none()
        }
        Message::SortNumeric => {
            apply_line_op(state, |lines, _| {
                lines.sort_by(|a, b| {
                    let na = a.trim().parse::<f64>().ok();
                    let nb = b.trim().parse::<f64>().ok();
                    match (na, nb) {
                        (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => a.cmp(b),
                    }
                });
            });
            Task::none()
        }
        Message::TrimLeading => {
            apply_line_op(state, |lines, _| {
                for line in lines.iter_mut() {
                    *line = line.trim_start().to_string();
                }
            });
            Task::none()
        }
        Message::TrimBoth => {
            apply_line_op(state, |lines, _| {
                for line in lines.iter_mut() {
                    *line = line.trim().to_string();
                }
            });
            Task::none()
        }

        // Case
        Message::UpperCase => {
            apply_text_transform(state, |s| s.to_uppercase());
            Task::none()
        }
        Message::LowerCase => {
            apply_text_transform(state, |s| s.to_lowercase());
            Task::none()
        }
        Message::TitleCase => {
            apply_text_transform(state, |s| {
                let mut result = String::with_capacity(s.len());
                let mut cap_next = true;
                for c in s.chars() {
                    if c.is_whitespace() || c == '-' || c == '_' {
                        cap_next = true;
                        result.push(c);
                    } else if cap_next {
                        result.extend(c.to_uppercase());
                        cap_next = false;
                    } else {
                        result.extend(c.to_lowercase());
                    }
                }
                result
            });
            Task::none()
        }
        Message::SentenceCase => {
            apply_text_transform(state, |s| {
                let mut result = String::with_capacity(s.len());
                let mut cap_next = true;
                for c in s.chars() {
                    if c == '.' || c == '!' || c == '?' {
                        cap_next = true;
                        result.push(c);
                    } else if cap_next && c.is_alphabetic() {
                        result.extend(c.to_uppercase());
                        cap_next = false;
                    } else {
                        result.extend(c.to_lowercase());
                    }
                }
                result
            });
            Task::none()
        }
        Message::InverseCase => {
            apply_text_transform(state, |s| {
                s.chars()
                    .map(|c| {
                        if c.is_uppercase() {
                            c.to_lowercase().to_string()
                        } else {
                            c.to_uppercase().to_string()
                        }
                    })
                    .collect()
            });
            Task::none()
        }

        // Encoding / line ending
        Message::SetEncoding(enc) => {
            state.tab_manager.active_document_mut().encoding = enc;
            Task::none()
        }
        Message::SetLineEnding(le) => {
            state.tab_manager.active_document_mut().line_ending = le;
            Task::none()
        }

        // ── Search ──
        Message::ShowFind => {
            state.show_find = true;
            state.show_replace = false;
            state.active_menu = None;
            Task::none()
        }
        Message::ShowReplace => {
            state.show_find = true;
            state.show_replace = true;
            state.active_menu = None;
            Task::none()
        }
        Message::ShowFindInFiles => {
            state.show_find_in_files = !state.show_find_in_files;
            // Pre-fill directory from current file's parent if available
            if state.show_find_in_files && state.fif_directory.is_empty() {
                if let Some(path) = &state.tab_manager.active_document().path {
                    if let Some(parent) = path.parent() {
                        state.fif_directory = parent.display().to_string();
                    }
                }
            }
            state.active_menu = None;
            Task::none()
        }
        Message::SelectAllOccurrences => {
            if !state.search_query.is_empty() {
                run_search(state);
            }
            Task::none()
        }
        Message::GotoLine => {
            state.show_goto_line = true;
            state.goto_line_input.clear();
            state.active_menu = None;
            Task::none()
        }
        Message::ToggleBookmark => {
            let line = state.tab_manager.active_document().cursor.position.line;
            state.tab_manager.active_document_mut().bookmarks.toggle(line);
            Task::none()
        }
        Message::NextBookmark => {
            let line = state.tab_manager.active_document().cursor.position.line;
            if let Some(next) = state.tab_manager.active_document().bookmarks.next_bookmark(line) {
                state.tab_manager.active_document_mut().cursor.position.line = next;
                state.tab_manager.active_document_mut().cursor.position.col = 0;
            }
            Task::none()
        }
        Message::PrevBookmark => {
            let line = state.tab_manager.active_document().cursor.position.line;
            if let Some(prev) = state.tab_manager.active_document().bookmarks.prev_bookmark(line) {
                state.tab_manager.active_document_mut().cursor.position.line = prev;
                state.tab_manager.active_document_mut().cursor.position.col = 0;
            }
            Task::none()
        }
        Message::ClearBookmarks => {
            state.tab_manager.active_document_mut().bookmarks.clear();
            Task::none()
        }
        Message::CopyBookmarkedLines => {
            let text = get_buffer_text(state);
            let lines = state.tab_manager.active_document().bookmarks.bookmarked_lines(&text);
            if !lines.is_empty() {
                let combined = lines.join("\n");
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(combined);
                }
            }
            Task::none()
        }
        Message::RemoveBookmarkedLines => {
            let text = get_buffer_text(state);
            let result = state.tab_manager.active_document().bookmarks.remove_bookmarked_lines(&text);
            set_buffer_text(state, &result);
            state.tab_manager.active_document_mut().bookmarks.clear();
            Task::none()
        }
        Message::RemoveUnbookmarkedLines => {
            let text = get_buffer_text(state);
            let result = state.tab_manager.active_document().bookmarks.remove_unbookmarked_lines(&text);
            set_buffer_text(state, &result);
            state.tab_manager.active_document_mut().bookmarks.clear();
            Task::none()
        }

        // ── View ──
        Message::ToggleWordWrap => {
            state.word_wrap = !state.word_wrap;
            Task::none()
        }
        Message::ToggleLineNumbers => {
            state.show_line_numbers = !state.show_line_numbers;
            Task::none()
        }
        Message::ToggleWhitespace => {
            state.show_whitespace = !state.show_whitespace;
            Task::none()
        }
        Message::ToggleStatusBar => {
            state.show_status_bar = !state.show_status_bar;
            Task::none()
        }
        Message::ToggleMinimap => {
            state.show_minimap = !state.show_minimap;
            Task::none()
        }
        Message::ToggleFunctionList => {
            state.show_function_list = !state.show_function_list;
            Task::none()
        }
        Message::SplitHorizontal => {
            if state.split_mode == SplitMode::Vertical {
                // Switch from vertical to horizontal, keep existing split state
                state.split_mode = SplitMode::Horizontal;
            } else if state.split_mode == SplitMode::None {
                state.split_mode = SplitMode::Horizontal;
                state.split_tab_manager = TabManager::new();
                state.split_tab_contents = vec![TabContent::new()];
                state.split_font_size = state.font_size;
                state.split_file_extension.clear();
                state.active_pane = 1;
            }
            Task::none()
        }
        Message::SplitVertical => {
            if state.split_mode == SplitMode::Horizontal {
                // Switch from horizontal to vertical, keep existing split state
                state.split_mode = SplitMode::Vertical;
            } else if state.split_mode == SplitMode::None {
                state.split_mode = SplitMode::Vertical;
                state.split_tab_manager = TabManager::new();
                state.split_tab_contents = vec![TabContent::new()];
                state.split_font_size = state.font_size;
                state.split_file_extension.clear();
                state.active_pane = 1;
            }
            Task::none()
        }
        Message::RemoveSplit => {
            state.split_mode = SplitMode::None;
            state.split_tab_manager = TabManager::new();
            state.split_tab_contents = vec![TabContent::new()];
            state.split_font_size = 14.0;
            state.split_file_extension.clear();
            state.split_show_markdown_preview = false;
            state.active_pane = 0;
            Task::none()
        }
        Message::ZoomIn => {
            if state.active_pane == 1 && state.split_mode != SplitMode::None {
                state.split_font_size = (state.split_font_size + 2.0).min(72.0);
            } else {
                state.font_size = (state.font_size + 2.0).min(72.0);
            }
            Task::none()
        }
        Message::ZoomOut => {
            if state.active_pane == 1 && state.split_mode != SplitMode::None {
                state.split_font_size = (state.split_font_size - 2.0).max(6.0);
            } else {
                state.font_size = (state.font_size - 2.0).max(6.0);
            }
            Task::none()
        }
        Message::ZoomReset => {
            if state.active_pane == 1 && state.split_mode != SplitMode::None {
                state.split_font_size = 14.0;
            } else {
                state.font_size = 14.0;
            }
            Task::none()
        }
        Message::ToggleFold => {
            if let Some(tc) = state.tab_contents.get(state.tab_manager.active_index()) {
                let line = tc.content.cursor_position().0;
                state.fold_manager.toggle_fold(line);
            }
            Task::none()
        }
        Message::ToggleFoldAt(line) => {
            state.fold_manager.toggle_fold(line);
            Task::none()
        }
        Message::FoldAll => {
            state.fold_manager.fold_all();
            Task::none()
        }
        Message::UnfoldAll => {
            state.fold_manager.unfold_all();
            Task::none()
        }
        Message::FoldLevel(level) => {
            state.fold_manager.fold_level(level);
            Task::none()
        }

        // ── Language ──
        Message::SetLanguage(lang) => {
            state.tab_manager.active_document_mut().language = lang.clone();
            // Try to find an extension for this language in syntect
            if let Some(ext) = super::highlighter::extension_for_language(&lang) {
                state.file_extension = ext;
            }
            Task::none()
        }

        // ── Tools ──
        Message::JsonFormat => {
            let text = get_buffer_text(state);
            match crate::tools::json_tools::format_json(&text) {
                Ok(formatted) => set_buffer_text(state, &formatted),
                Err(e) => log::error!("JSON format error: {}", e),
            }
            Task::none()
        }
        Message::JsonCompact => {
            let text = get_buffer_text(state);
            match crate::tools::json_tools::compact_json(&text) {
                Ok(compacted) => set_buffer_text(state, &compacted),
                Err(e) => log::error!("JSON compact error: {}", e),
            }
            Task::none()
        }
        Message::JsonValidate => {
            let text = get_buffer_text(state);
            match crate::tools::json_tools::validate_json(&text) {
                Ok(()) => log::info!("JSON is valid"),
                Err(e) => log::error!("JSON validation error: {}", e),
            }
            Task::none()
        }
        Message::JsonSortKeys => {
            let text = get_buffer_text(state);
            match crate::tools::json_tools::sort_json_keys(&text) {
                Ok(sorted) => set_buffer_text(state, &sorted),
                Err(e) => log::error!("JSON sort keys error: {}", e),
            }
            Task::none()
        }
        Message::ToggleMarkdownPreview => {
            if state.active_pane == 0 {
                state.show_markdown_preview = !state.show_markdown_preview;
            } else {
                state.split_show_markdown_preview = !state.split_show_markdown_preview;
            }
            Task::none()
        }
        Message::ToggleCsvViewer => {
            state.show_csv_viewer = !state.show_csv_viewer;
            Task::none()
        }
        Message::MimeBase64Encode => {
            apply_text_transform(state, crate::tools::mime_tools::base64_encode);
            Task::none()
        }
        Message::MimeBase64Decode => {
            let text = get_buffer_text(state);
            match crate::tools::mime_tools::base64_decode(&text) {
                Ok(decoded) => set_buffer_text(state, &decoded),
                Err(e) => log::error!("Base64 decode error: {}", e),
            }
            Task::none()
        }
        Message::MimeUrlEncode => {
            apply_text_transform(state, crate::tools::mime_tools::url_encode);
            Task::none()
        }
        Message::MimeUrlDecode => {
            let text = get_buffer_text(state);
            match crate::tools::mime_tools::url_decode(&text) {
                Ok(decoded) => set_buffer_text(state, &decoded),
                Err(e) => log::error!("URL decode error: {}", e),
            }
            Task::none()
        }
        Message::MimeHtmlEncode => {
            apply_text_transform(state, crate::tools::mime_tools::html_entity_encode);
            Task::none()
        }
        Message::MimeHtmlDecode => {
            apply_text_transform(state, crate::tools::mime_tools::html_entity_decode);
            Task::none()
        }
        Message::MimeHexEncode => {
            apply_text_transform(state, crate::tools::mime_tools::hex_encode);
            Task::none()
        }
        Message::MimeHexDecode => {
            let text = get_buffer_text(state);
            match crate::tools::mime_tools::hex_decode(&text) {
                Ok(decoded) => set_buffer_text(state, &decoded),
                Err(e) => log::error!("Hex decode error: {}", e),
            }
            Task::none()
        }
        Message::CompareFiles => {
            Task::perform(
                async {
                    let handle = rfd::AsyncFileDialog::new()
                        .set_title("Compare With...")
                        .pick_file()
                        .await;
                    match handle {
                        Some(h) => {
                            let path = h.path().to_path_buf();
                            match std::fs::read_to_string(&path) {
                                Ok(content) => Ok(content),
                                Err(e) => Err(e.to_string()),
                            }
                        }
                        None => Err("Cancelled".to_string()),
                    }
                },
                Message::CompareFileLoaded,
            )
        }
        Message::ToggleHexViewer => {
            state.show_hex_viewer = !state.show_hex_viewer;
            Task::none()
        }

        // ── Panel messages ──
        Message::GotoSymbol(line) => {
            // Navigate to the given line in the editor
            let idx = state.tab_manager.active_index();
            if let Some(_tc) = state.tab_contents.get(idx) {
                // Set document cursor position (visual cursor movement
                // not possible with iced text_editor)
                state.tab_manager.active_document_mut().cursor.position.line = line;
                state.tab_manager.active_document_mut().cursor.position.col = 0;
            }
            Task::none()
        }
        Message::CsvSortColumn(col) => {
            if state.csv_sort_column == Some(col) {
                state.csv_sort_ascending = !state.csv_sort_ascending;
            } else {
                state.csv_sort_column = Some(col);
                state.csv_sort_ascending = true;
            }
            Task::none()
        }
        Message::CsvToggleHeaders => {
            state.csv_has_headers = !state.csv_has_headers;
            Task::none()
        }

        // ── Macro ──
        Message::ToggleMacroRecording => {
            if state.macro_recorder.is_recording() {
                state.last_macro = Some(state.macro_recorder.stop_recording("Macro"));
            } else {
                state.macro_recorder.start_recording();
            }
            Task::none()
        }
        Message::PlayLastMacro => {
            if let Some(ref m) = state.last_macro.clone() {
                let doc = state.tab_manager.active_document_mut();
                MacroRecorder::play_macro(m, doc);
                rebuild_content(state);
            }
            Task::none()
        }
        Message::PlayMacroMultiple => {
            if let Some(ref m) = state.last_macro.clone() {
                let doc = state.tab_manager.active_document_mut();
                MacroRecorder::play_macro_n_times(m, doc, 10);
                rebuild_content(state);
            }
            Task::none()
        }

        // ── Settings ──
        Message::ShowPreferences => {
            // Load current settings into pref fields
            let settings = crate::io::settings::AppSettings::load(
                &crate::io::settings::AppSettings::settings_path(),
            )
            .unwrap_or_default();
            state.pref_font_size = settings.font_size;
            state.pref_tab_size = settings.tab_size.to_string();
            state.pref_theme = state.theme.name.clone();
            state.pref_auto_save = settings.auto_save;
            state.pref_word_wrap = settings.word_wrap;
            state.pref_show_line_numbers = settings.show_line_numbers;
            state.pref_show_whitespace = settings.show_whitespace;
            state.show_preferences = true;
            Task::none()
        }
        Message::ShowKeybindings => {
            state.show_keybindings = true;
            Task::none()
        }
        Message::SavePreferences => {
            // Build settings from pref fields and save
            let mut settings = crate::io::settings::AppSettings::load(
                &crate::io::settings::AppSettings::settings_path(),
            )
            .unwrap_or_default();
            settings.font_size = state.pref_font_size;
            settings.tab_size = state.pref_tab_size.parse().unwrap_or(4);
            settings.theme = state.pref_theme.clone();
            settings.auto_save = state.pref_auto_save;
            settings.word_wrap = state.pref_word_wrap;
            settings.show_line_numbers = state.pref_show_line_numbers;
            settings.show_whitespace = state.pref_show_whitespace;
            if let Err(e) = settings.save(&crate::io::settings::AppSettings::settings_path()) {
                log::error!("Failed to save settings: {}", e);
            }
            // Apply to app state
            state.font_size = settings.font_size;
            state.word_wrap = settings.word_wrap;
            state.show_line_numbers = settings.show_line_numbers;
            state.show_whitespace = settings.show_whitespace;
            state.theme = AppColors::theme_by_name(&state.pref_theme);
            state.show_preferences = false;
            Task::none()
        }
        Message::CancelPreferences => {
            state.show_preferences = false;
            Task::none()
        }
        Message::CloseKeybindingsDialog => {
            state.show_keybindings = false;
            Task::none()
        }
        Message::PrefFontSizeIncrease => {
            state.pref_font_size = (state.pref_font_size + 1.0).min(48.0);
            Task::none()
        }
        Message::PrefFontSizeDecrease => {
            state.pref_font_size = (state.pref_font_size - 1.0).max(8.0);
            Task::none()
        }
        Message::PrefTabSizeChanged(val) => {
            state.pref_tab_size = val;
            Task::none()
        }
        Message::PrefSetTheme(name) => {
            state.pref_theme = name.clone();
            // Immediately apply the theme
            state.theme = AppColors::theme_by_name(&name);
            Task::none()
        }
        Message::PrefToggleAutoSave(v) => {
            state.pref_auto_save = v;
            Task::none()
        }
        Message::PrefToggleWordWrap(v) => {
            state.pref_word_wrap = v;
            Task::none()
        }
        Message::PrefToggleLineNumbers(v) => {
            state.pref_show_line_numbers = v;
            Task::none()
        }
        Message::PrefToggleWhitespace(v) => {
            state.pref_show_whitespace = v;
            Task::none()
        }

        // ── Help ──
        Message::ShowAbout => {
            state.show_about = true;
            Task::none()
        }
        Message::CloseAbout => {
            state.show_about = false;
            Task::none()
        }

        // ── Theme ──
        Message::SetTheme(name) => {
            state.theme = AppColors::theme_by_name(&name);
            state.pref_theme = name.clone();
            // Persist theme to settings
            let mut settings = crate::io::settings::AppSettings::load(
                &crate::io::settings::AppSettings::settings_path(),
            )
            .unwrap_or_default();
            settings.theme = name;
            let _ = settings.save(&crate::io::settings::AppSettings::settings_path());
            Task::none()
        }

        // ── Search panel ──
        Message::FindQueryChanged(query) => {
            state.search_query = query;
            run_search(state);
            Task::none()
        }
        Message::ReplaceTextChanged(text) => {
            state.replace_text = text;
            Task::none()
        }
        Message::ToggleCaseSensitive => {
            state.case_sensitive = !state.case_sensitive;
            run_search(state);
            Task::none()
        }
        Message::ToggleWholeWord => {
            state.whole_word = !state.whole_word;
            run_search(state);
            Task::none()
        }
        Message::ToggleRegex => {
            state.use_regex = !state.use_regex;
            run_search(state);
            Task::none()
        }
        Message::FindNext => {
            if !state.search_matches.is_empty() {
                let next = match state.current_match_index {
                    Some(i) => (i + 1) % state.search_matches.len(),
                    None => 0,
                };
                state.current_match_index = Some(next);
            }
            Task::none()
        }
        Message::FindPrev => {
            if !state.search_matches.is_empty() {
                let prev = match state.current_match_index {
                    Some(0) | None => state.search_matches.len() - 1,
                    Some(i) => i - 1,
                };
                state.current_match_index = Some(prev);
            }
            Task::none()
        }
        Message::ReplaceNext => {
            if let Some(idx) = state.current_match_index {
                if idx < state.search_matches.len() {
                    let m = &state.search_matches[idx];
                    let text = get_buffer_text(state);
                    let mut new_text = String::with_capacity(text.len());
                    new_text.push_str(&text[..m.start]);
                    new_text.push_str(&state.replace_text);
                    new_text.push_str(&text[m.end..]);
                    set_buffer_text(state, &new_text);
                    run_search(state);
                }
            }
            Task::none()
        }
        Message::ReplaceAll => {
            if !state.search_query.is_empty() {
                let text = get_buffer_text(state);
                let mut engine = SearchEngine::new();
                engine.query = state.search_query.clone();
                engine.replace_text = state.replace_text.clone();
                engine.case_sensitive = state.case_sensitive;
                engine.whole_word = state.whole_word;
                engine.use_regex = state.use_regex;
                engine.search_mode = if state.use_regex {
                    SearchMode::Regex
                } else {
                    SearchMode::Normal
                };
                let (new_text, count) = engine.replace_all(&text);
                if count > 0 {
                    set_buffer_text(state, &new_text);
                    run_search(state);
                }
            }
            Task::none()
        }
        Message::CloseSearch => {
            state.show_find = false;
            state.show_replace = false;
            state.search_matches.clear();
            state.current_match_index = None;
            Task::none()
        }
        Message::ClickSearchResult(idx) => {
            if idx < state.search_matches.len() {
                state.current_match_index = Some(idx);
            }
            Task::none()
        }

        // ── Go to line ──
        Message::GotoLineInputChanged(input) => {
            state.goto_line_input = input;
            Task::none()
        }
        Message::GotoLineConfirm => {
            if let Ok(line_num) = state.goto_line_input.trim().parse::<usize>() {
                if line_num > 0 {
                    let target = line_num - 1;
                    state
                        .tab_manager
                        .active_document_mut()
                        .cursor
                        .position
                        .line = target;
                    state
                        .tab_manager
                        .active_document_mut()
                        .cursor
                        .position
                        .col = 0;
                }
            }
            state.show_goto_line = false;
            Task::none()
        }
        Message::GotoLineClose => {
            state.show_goto_line = false;
            Task::none()
        }

        // ── Async results ──
        Message::SessionFileChosen(result) => {
            if let Ok(path) = result {
                if let Ok(session) = crate::io::session::load_session(&path) {
                    state.tab_manager.close_all();
                    state.tab_contents.clear();
                    for sf in &session.files {
                        match state.tab_manager.open_file(sf.path.clone()) {
                            Ok(idx) => {
                                let doc = state.tab_manager.get_document(idx).unwrap();
                                let buf_text = doc.buffer.text();
                                state.tab_contents.push(TabContent::with_text(&buf_text));
                            }
                            Err(_) => {
                                state.tab_manager.new_tab();
                                state.tab_contents.push(TabContent::new());
                            }
                        }
                    }
                    if state.tab_contents.is_empty() {
                        state.tab_contents.push(TabContent::new());
                    }
                }
            }
            Task::none()
        }
        Message::ExportSaved(result) => {
            if let Err(e) = result {
                log::error!("Export failed: {}", e);
            }
            Task::none()
        }
        Message::CompareFileLoaded(result) => {
            if let Ok(other_text) = result {
                let current_text = get_buffer_text(state);
                let diff_result = crate::tools::diff_tool::diff_texts(&current_text, &other_text);
                let mut diff_text = String::new();
                for line in &diff_result.lines {
                    match line {
                        crate::tools::diff_tool::DiffLine::Same(s) => {
                            diff_text.push_str("  ");
                            diff_text.push_str(s);
                            diff_text.push('\n');
                        }
                        crate::tools::diff_tool::DiffLine::Added(s) => {
                            diff_text.push_str("+ ");
                            diff_text.push_str(s);
                            diff_text.push('\n');
                        }
                        crate::tools::diff_tool::DiffLine::Removed(s) => {
                            diff_text.push_str("- ");
                            diff_text.push_str(s);
                            diff_text.push('\n');
                        }
                        crate::tools::diff_tool::DiffLine::Changed { old, new } => {
                            diff_text.push_str("- ");
                            diff_text.push_str(old);
                            diff_text.push('\n');
                            diff_text.push_str("+ ");
                            diff_text.push_str(new);
                            diff_text.push('\n');
                        }
                    }
                }
                state.tab_manager.new_tab();
                state.tab_contents.push(TabContent::with_text(&diff_text));
                state.tab_manager.active_document_mut().language = "Diff".to_string();
            }
            Task::none()
        }

        // ── Find in Files ──
        Message::FifQueryChanged(q) => {
            state.fif_query = q;
            Task::none()
        }
        Message::FifDirectoryChanged(d) => {
            state.fif_directory = d;
            Task::none()
        }
        Message::FifFileFilterChanged(f) => {
            state.fif_file_filter = f;
            Task::none()
        }
        Message::FifToggleRecursive => {
            state.fif_recursive = !state.fif_recursive;
            Task::none()
        }
        Message::FifToggleCaseSensitive => {
            state.fif_case_sensitive = !state.fif_case_sensitive;
            Task::none()
        }
        Message::FifToggleRegex => {
            state.fif_use_regex = !state.fif_use_regex;
            Task::none()
        }
        Message::FifSearch => {
            if state.fif_query.is_empty() || state.fif_directory.is_empty() {
                return Task::none();
            }
            state.fif_searching = true;
            state.fif_results.clear();
            let dir = std::path::PathBuf::from(&state.fif_directory);
            let query = state.fif_query.clone();
            let filter = state.fif_file_filter.clone();
            let recursive = state.fif_recursive;
            let case_sensitive = state.fif_case_sensitive;
            let use_regex = state.fif_use_regex;
            Task::perform(
                async move {
                    crate::search::FindInFiles::search(
                        &dir, &query, &filter, recursive, case_sensitive, use_regex,
                    )
                    .map_err(|e| e.to_string())
                },
                Message::FifSearchComplete,
            )
        }
        Message::FifSearchComplete(result) => {
            state.fif_searching = false;
            match result {
                Ok(results) => state.fif_results = results,
                Err(e) => log::error!("Find in Files error: {}", e),
            }
            Task::none()
        }
        Message::FifClickResult(path, line) => {
            // Open the file in a new tab and go to the line
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        state.file_extension = ext.to_lowercase();
                    }
                    match state.tab_manager.open_file(path) {
                        Ok(idx) => {
                            let doc = state.tab_manager.get_document(idx).unwrap();
                            let buf_text = doc.buffer.text();
                            state.fold_manager.detect_regions(&buf_text);
                            state.tab_contents.push(TabContent::with_text(&buf_text));
                        }
                        Err(_) => {
                            state.tab_manager.new_tab();
                            state.fold_manager.detect_regions(&content);
                            state.tab_contents.push(TabContent::with_text(&content));
                        }
                    }
                    // Go to the matched line
                    state.tab_manager.active_document_mut().cursor.position.line = line;
                    state.tab_manager.active_document_mut().cursor.position.col = 0;
                }
                Err(e) => log::error!("Failed to open file: {}", e),
            }
            Task::none()
        }
        Message::CloseFindInFiles => {
            state.show_find_in_files = false;
            Task::none()
        }

        // ── Keyboard ──
        Message::EscapePressed => {
            if state.tab_context_menu.is_some() {
                state.tab_context_menu = None;
            } else if state.show_find {
                state.show_find = false;
                state.show_replace = false;
                state.search_matches.clear();
                state.current_match_index = None;
            } else if state.show_find_in_files {
                state.show_find_in_files = false;
            } else if state.show_goto_line {
                state.show_goto_line = false;
            } else if state.show_about {
                state.show_about = false;
            } else if state.show_preferences {
                state.show_preferences = false;
            } else if state.show_keybindings {
                state.show_keybindings = false;
            } else {
                state.active_menu = None;
            }
            Task::none()
        }

        // ── Drag floating panels ──
        Message::DragFindStart => {
            state.dragging_find_panel = true;
            // Calculate offset from mouse to panel position
            let pos = state.find_panel_pos.unwrap_or((0.0, 60.0));
            state.drag_offset = (
                state.last_mouse_pos.x - pos.0,
                state.last_mouse_pos.y - pos.1,
            );
            Task::none()
        }
        Message::DragFindMove(point) => {
            state.last_mouse_pos = point;
            if state.dragging_find_panel {
                let x = (point.x - state.drag_offset.0).max(0.0);
                let y = (point.y - state.drag_offset.1).max(0.0);
                state.find_panel_pos = Some((x, y));
            }
            Task::none()
        }
        Message::DragFindEnd => {
            state.dragging_find_panel = false;
            Task::none()
        }
        Message::DragFifStart => {
            state.dragging_fif_panel = true;
            let pos = state.fif_panel_pos.unwrap_or((0.0, 60.0));
            state.fif_drag_offset = (
                state.last_mouse_pos.x - pos.0,
                state.last_mouse_pos.y - pos.1,
            );
            Task::none()
        }
        Message::DragFifMove(point) => {
            state.last_mouse_pos = point;
            if state.dragging_fif_panel {
                let x = (point.x - state.fif_drag_offset.0).max(0.0);
                let y = (point.y - state.fif_drag_offset.1).max(0.0);
                state.fif_panel_pos = Some((x, y));
            }
            Task::none()
        }
        Message::DragFifEnd => {
            state.dragging_fif_panel = false;
            Task::none()
        }

        // ── Split pane ──
        Message::SplitEditorAction(action) => {
            state.active_pane = 1;
            if let Some(tc) = state
                .split_tab_contents
                .get_mut(state.split_tab_manager.active_index())
            {
                tc.content.perform(action);
                let new_text = tc.content.text();
                let doc = state.split_tab_manager.active_document_mut();
                let old_text = doc.buffer.text();
                if new_text != old_text {
                    let len = doc.buffer.len_bytes();
                    if len > 0 {
                        doc.buffer.delete(0, len);
                    }
                    doc.buffer.insert(0, &new_text);
                }
            }
            Task::none()
        }
        Message::SplitNewTab => {
            state.active_pane = 1;
            state.split_tab_manager.new_tab();
            state.split_tab_contents.push(TabContent::new());
            Task::none()
        }
        Message::SplitSelectTab(idx) => {
            state.active_pane = 1;
            if idx < state.split_tab_manager.tab_count() {
                state.split_tab_manager.set_active(idx);
                if let Some(path) = &state.split_tab_manager.active_document().path {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        state.split_file_extension = ext.to_lowercase();
                    } else {
                        state.split_file_extension.clear();
                    }
                } else {
                    state.split_file_extension.clear();
                }
            }
            Task::none()
        }
        Message::SplitCloseTab(idx) => {
            state.active_pane = 1;
            let actual_idx = if idx == usize::MAX {
                state.split_tab_manager.active_index()
            } else {
                idx
            };
            if actual_idx < state.split_tab_contents.len() {
                state.split_tab_contents.remove(actual_idx);
                state.split_tab_manager.close_tab(actual_idx);
                while state.split_tab_contents.len() < state.split_tab_manager.tab_count() {
                    state.split_tab_contents.push(TabContent::new());
                }
            }
            // If only the auto-created empty tab remains after closing, close the split
            if state.split_tab_manager.tab_count() == 1 {
                let doc = state.split_tab_manager.active_document();
                let is_empty = doc.buffer.text().trim().is_empty() && doc.path.is_none();
                if is_empty {
                    state.split_mode = SplitMode::None;
                    state.active_pane = 0;
                }
            }
            Task::none()
        }
        Message::SetActivePane(pane) => {
            state.active_pane = pane;
            Task::none()
        }
        Message::NewTabInActivePane => {
            if state.active_pane == 1 && state.split_mode != SplitMode::None {
                state.split_tab_manager.new_tab();
                state.split_tab_contents.push(TabContent::new());
            } else {
                state.tab_manager.new_tab();
                state.tab_contents.push(TabContent::new());
            }
            Task::none()
        }
        Message::FileDropped(path) => {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                if state.active_pane == 1 && state.split_mode != SplitMode::None {
                    state.split_file_extension = ext;
                    match state.split_tab_manager.open_file(path) {
                        Ok(idx) => {
                            let doc = state.split_tab_manager.get_document(idx).unwrap();
                            let buf_text = doc.buffer.text();
                            state.split_tab_contents.push(TabContent::with_text(&buf_text));
                        }
                        Err(_) => {
                            state.split_tab_manager.new_tab();
                            state.split_tab_contents.push(TabContent::with_text(&content));
                        }
                    }
                } else {
                    state.file_extension = ext;
                    match state.tab_manager.open_file(path) {
                        Ok(idx) => {
                            let doc = state.tab_manager.get_document(idx).unwrap();
                            let buf_text = doc.buffer.text();
                            state.fold_manager.detect_regions(&buf_text);
                            state.tab_contents.push(TabContent::with_text(&buf_text));
                        }
                        Err(_) => {
                            state.tab_manager.new_tab();
                            state.fold_manager.detect_regions(&content);
                            state.tab_contents.push(TabContent::with_text(&content));
                        }
                    }
                }
            }
            Task::none()
        }
        Message::MoveTabToSplit(idx) => {
            // Move tab at idx from primary pane to split pane
            if idx >= state.tab_contents.len() || idx >= state.tab_manager.tab_count() {
                return Task::none();
            }
            // Ensure split exists
            if state.split_mode == SplitMode::None {
                state.split_mode = SplitMode::Horizontal;
                state.split_tab_manager = TabManager::new();
                state.split_tab_contents = vec![TabContent::new()];
                state.split_font_size = state.font_size;
                state.split_file_extension.clear();
            }
            // Sync content before moving
            sync_content_to_doc(state, idx);
            let tab_content = state.tab_contents.remove(idx);
            if let Some(doc) = state.tab_manager.remove_document(idx) {
                if let Some(ext) = doc.path.as_ref().and_then(|p| p.extension()).and_then(|e| e.to_str()) {
                    state.split_file_extension = ext.to_lowercase();
                }
                // Remove the default empty tab in split if it's the only one and is empty
                if state.split_tab_manager.tab_count() == 1 {
                    let split_doc = state.split_tab_manager.active_document();
                    if split_doc.buffer.text().trim().is_empty() && split_doc.path.is_none() {
                        state.split_tab_contents.remove(0);
                        state.split_tab_manager.remove_document(0);
                    }
                }
                state.split_tab_manager.add_document(doc);
                state.split_tab_contents.push(tab_content);
            }
            // If primary has no tabs left, create one
            if state.tab_manager.tab_count() == 0 {
                state.tab_manager.new_tab();
                state.tab_contents.push(TabContent::new());
            }
            state.active_pane = 1;
            Task::none()
        }
        Message::MoveTabFromSplit(idx) => {
            // Move tab at idx from split pane to primary pane
            if state.split_mode == SplitMode::None {
                return Task::none();
            }
            if idx >= state.split_tab_contents.len() || idx >= state.split_tab_manager.tab_count() {
                return Task::none();
            }
            // Sync split content before moving
            sync_split_content_to_doc(state, idx);
            let tab_content = state.split_tab_contents.remove(idx);
            if let Some(doc) = state.split_tab_manager.remove_document(idx) {
                if let Some(ext) = doc.path.as_ref().and_then(|p| p.extension()).and_then(|e| e.to_str()) {
                    state.file_extension = ext.to_lowercase();
                }
                state.tab_manager.add_document(doc);
                state.tab_contents.push(tab_content);
            }
            // If split has no tabs left, close the split
            if state.split_tab_manager.tab_count() == 0 {
                state.split_mode = SplitMode::None;
                state.split_tab_manager = TabManager::new();
                state.split_tab_contents = vec![TabContent::new()];
                state.active_pane = 0;
            } else {
                // Check if only an empty tab remains
                if state.split_tab_manager.tab_count() == 1 {
                    let split_doc = state.split_tab_manager.active_document();
                    if split_doc.buffer.text().trim().is_empty() && split_doc.path.is_none() {
                        state.split_mode = SplitMode::None;
                        state.active_pane = 0;
                    }
                }
            }
            Task::none()
        }

        // ── Tab context menu ──
        Message::TabContextMenu(idx, is_split) => {
            state.tab_context_menu = Some((idx, is_split));
            Task::none()
        }
        Message::CloseTabContextMenu => {
            state.tab_context_menu = None;
            Task::none()
        }
        Message::TabContextClose(idx, is_split) => {
            state.tab_context_menu = None;
            if is_split {
                return update(state, Message::SplitCloseTab(idx));
            } else {
                return update(state, Message::CloseTab(idx));
            }
        }
        Message::TabContextCloseOthers(idx, is_split) => {
            state.tab_context_menu = None;
            if is_split {
                // Close all split tabs except idx
                let mut i = state.split_tab_manager.tab_count();
                while i > 0 {
                    i -= 1;
                    if i != idx && i < state.split_tab_contents.len() {
                        state.split_tab_contents.remove(i);
                        state.split_tab_manager.close_tab(i);
                    }
                }
                while state.split_tab_contents.len() < state.split_tab_manager.tab_count() {
                    state.split_tab_contents.push(TabContent::new());
                }
            } else {
                let mut i = state.tab_manager.tab_count();
                while i > 0 {
                    i -= 1;
                    if i != idx && i < state.tab_contents.len() {
                        state.tab_contents.remove(i);
                        state.tab_manager.close_tab(i);
                    }
                }
                while state.tab_contents.len() < state.tab_manager.tab_count() {
                    state.tab_contents.push(TabContent::new());
                }
            }
            Task::none()
        }
        Message::TabContextCloseAll(is_split) => {
            state.tab_context_menu = None;
            if is_split {
                state.split_tab_manager.close_all();
                state.split_tab_contents.clear();
                state.split_tab_contents.push(TabContent::new());
                // Close the split entirely
                state.split_mode = SplitMode::None;
                state.active_pane = 0;
            } else {
                state.tab_manager.close_all();
                state.tab_contents.clear();
                state.tab_contents.push(TabContent::new());
            }
            Task::none()
        }
        Message::TabContextMoveToOtherPane(idx, is_split) => {
            state.tab_context_menu = None;
            if is_split {
                return update(state, Message::MoveTabFromSplit(idx));
            } else {
                return update(state, Message::MoveTabToSplit(idx));
            }
        }
    }
}

pub fn view(state: &NotepadIced) -> Element<'_, Message> {
    let menu_bar = menu_bar::view_menu_bar(&state.active_menu, &state.theme);

    // Build tab bar row: if split is active, show both tab bars side by side
    let tab_bar_area: Element<'_, Message> = if state.split_mode != SplitMode::None {
        let primary_tab_bar = view_tab_bar(state);
        let split_tab_bar = view_split_tab_bar(state);
        match state.split_mode {
            SplitMode::Horizontal => {
                row![
                    container(primary_tab_bar).width(Length::FillPortion(1)),
                    container(split_tab_bar).width(Length::FillPortion(1)),
                ]
                .into()
            }
            SplitMode::Vertical => {
                column![primary_tab_bar, split_tab_bar].into()
            }
            SplitMode::None => unreachable!(),
        }
    } else {
        view_tab_bar(state)
    };

    // Build the main content area based on active viewer panels
    let main_area: Element<'_, Message> = if state.show_csv_viewer {
        super::csv_panel::view_csv_viewer(state, &state.theme)
    } else if state.show_hex_viewer {
        super::hex_panel::view_hex_viewer(state, &state.theme)
    } else {
        // Normal editor, possibly with side panels
        let editor = view_editor(state);

        // Wrap primary editor with its own markdown preview if enabled
        let primary_with_md: Element<'_, Message> = if state.show_markdown_preview {
            let primary_content = state
                .tab_contents
                .get(state.tab_manager.active_index())
                .map(|tc| tc.content.text())
                .unwrap_or_default();
            row![
                container(editor).width(Length::FillPortion(1)),
                super::markdown_panel::view_markdown_preview_for_text(&primary_content, &state.theme),
            ]
            .height(Length::Fill)
            .into()
        } else {
            editor
        };

        // Wrap primary editor in mouse_area for active pane tracking
        let primary_pane: Element<'_, Message> = mouse_area(primary_with_md)
            .on_press(Message::SetActivePane(0))
            .into();

        // Wrap editor with split view if active
        let editor_area = if state.split_mode != SplitMode::None {
            let split_editor = view_split_pane(state);

            // Wrap split pane with its own markdown preview if enabled
            let split_with_md: Element<'_, Message> = if state.split_show_markdown_preview {
                let split_content = state
                    .split_tab_contents
                    .get(state.split_tab_manager.active_index())
                    .map(|tc| tc.content.text())
                    .unwrap_or_default();
                row![
                    container(split_editor).width(Length::FillPortion(1)),
                    super::markdown_panel::view_markdown_preview_for_text(&split_content, &state.theme),
                ]
                .height(Length::Fill)
                .into()
            } else {
                split_editor
            };

            let border_color = state.theme.border;
            let divider = container(Space::new(
                if state.split_mode == SplitMode::Horizontal { Length::Fixed(2.0) } else { Length::Fill },
                if state.split_mode == SplitMode::Vertical { Length::Fixed(2.0) } else { Length::Fill },
            ))
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(border_color)),
                ..Default::default()
            });

            match state.split_mode {
                SplitMode::Horizontal => {
                    row![
                        container(primary_pane).width(Length::FillPortion(1)).height(Length::Fill),
                        divider,
                        container(split_with_md).width(Length::FillPortion(1)).height(Length::Fill),
                    ]
                    .height(Length::Fill)
                    .into()
                }
                SplitMode::Vertical => {
                    column![
                        container(primary_pane).width(Length::Fill).height(Length::FillPortion(1)),
                        divider,
                        container(split_with_md).width(Length::Fill).height(Length::FillPortion(1)),
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                }
                SplitMode::None => unreachable!(),
            }
        } else {
            primary_pane
        };

        if state.show_function_list {
            row![
                container(editor_area).width(Length::Fill),
                super::function_list_panel::view_function_list(state, &state.theme),
            ]
            .height(Length::Fill)
            .into()
        } else {
            editor_area
        }
    };

    let mut base_content = column![menu_bar, tab_bar_area, main_area];

    if state.show_status_bar {
        base_content = base_content.push(view_status_bar(state));
    }

    let base: Element<'_, Message> = container(base_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    // Check if any overlay is needed
    let has_dropdown = state.active_menu.is_some();
    let has_floating = state.show_find || state.show_goto_line || state.show_about || state.show_find_in_files || state.show_preferences || state.show_keybindings;
    let has_context_menu = state.tab_context_menu.is_some();

    if !has_dropdown && !has_floating && !has_context_menu {
        return base;
    }

    let mut layers: Vec<Element<'_, Message>> = vec![base];

    // Dropdown menu: needs a click-catcher (modal — clicking outside closes it)
    if has_dropdown {
        let click_catcher: Element<'_, Message> = mouse_area(
            container(Space::new(Length::Fill, Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Message::MenuClose)
        .into();
        layers.push(click_catcher);

        if let Some(ref menu_name) = state.active_menu {
            let dropdown = scrollable(menu_bar::view_dropdown(state, menu_name))
                .height(Length::Shrink);

            let left_offset = menu_bar::menu_x_offset(menu_name);

            let tab_bar_bg = state.theme.tab_bar_bg;
            let dropdown_border = state.theme.border;
            let dropdown_overlay: Element<'_, Message> = container(
                opaque(
                    container(dropdown)
                        .max_height(500)
                        .style(move |_theme: &Theme| container::Style {
                            background: Some(iced::Background::Color(tab_bar_bg)),
                            border: iced::Border {
                                color: dropdown_border,
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        }),
                ),
            )
            .padding(iced::Padding { top: 26.0, right: 0.0, bottom: 0.0, left: left_offset })
            .width(Length::Shrink)
            .height(Length::Shrink)
            .into();

            layers.push(dropdown_overlay);
        }
    }

    // Find/Replace — floating non-modal draggable window
    if state.show_find {
        // If dragging, add a full-window mouse tracking layer
        if state.dragging_find_panel {
            let drag_tracker: Element<'_, Message> = mouse_area(
                container(Space::new(Length::Fill, Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .on_move(Message::DragFindMove)
            .on_release(Message::DragFindEnd)
            .into();
            layers.push(drag_tracker);
        }

        let search_widget = opaque(super::search_panel::view_search_panel(state, &state.theme));

        let search_overlay: Element<'_, Message> = match state.find_panel_pos {
            Some((x, y)) => {
                // Absolute position via padding
                container(search_widget)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(iced::Padding { top: y, right: 0.0, bottom: 0.0, left: x })
                    .into()
            }
            None => {
                // Default: right-aligned, below tabs/menu
                container(search_widget)
                    .padding(iced::Padding { top: 60.0, right: 16.0, bottom: 0.0, left: 0.0 })
                    .align_right(Length::Fill)
                    .height(Length::Shrink)
                    .into()
            }
        };

        layers.push(search_overlay);
    }

    // Find in Files — floating non-modal draggable window
    if state.show_find_in_files {
        if state.dragging_fif_panel {
            let drag_tracker: Element<'_, Message> = mouse_area(
                container(Space::new(Length::Fill, Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .on_move(Message::DragFifMove)
            .on_release(Message::DragFifEnd)
            .into();
            layers.push(drag_tracker);
        }

        let fif_widget = opaque(super::find_in_files_panel::view_find_in_files_panel(state, &state.theme));

        let fif_overlay: Element<'_, Message> = match state.fif_panel_pos {
            Some((x, y)) => {
                container(fif_widget)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(iced::Padding { top: y, right: 0.0, bottom: 0.0, left: x })
                    .into()
            }
            None => {
                container(fif_widget)
                    .padding(iced::Padding { top: 60.0, right: 16.0, bottom: 0.0, left: 0.0 })
                    .align_right(Length::Fill)
                    .height(Length::Shrink)
                    .into()
            }
        };

        layers.push(fif_overlay);
    }

    // Go to Line — floating non-modal, centered
    if state.show_goto_line {
        let goto_overlay: Element<'_, Message> = container(
            opaque(super::goto_dialog::view_goto_dialog(state, &state.theme)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();

        layers.push(goto_overlay);
    }

    // About — floating non-modal, centered
    if state.show_about {
        let about_overlay: Element<'_, Message> = container(
            opaque(super::about_dialog::view_about_dialog(&state.theme)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();

        layers.push(about_overlay);
    }

    // Preferences — floating centered
    if state.show_preferences {
        let pref_overlay: Element<'_, Message> = container(
            opaque(super::preferences_dialog::view_preferences_dialog(state, &state.theme)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();

        layers.push(pref_overlay);
    }

    // Keybindings — floating centered
    if state.show_keybindings {
        let kb_overlay: Element<'_, Message> = container(
            opaque(super::keybindings_dialog::view_keybindings_dialog(&state.theme)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();

        layers.push(kb_overlay);
    }

    // Tab context menu
    if let Some((tab_idx, is_split)) = state.tab_context_menu {
        // Click catcher to close menu
        let click_catcher: Element<'_, Message> = mouse_area(
            container(Space::new(Length::Fill, Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Message::CloseTabContextMenu)
        .into();
        layers.push(click_catcher);

        let has_split_active = state.split_mode != SplitMode::None;
        let t_menu_bg = state.theme.menu_bg;
        let t_menu_hover = state.theme.menu_hover;
        let t_text = state.theme.text;
        let t_border = state.theme.border;

        let mut menu_items: Vec<Element<'_, Message>> = Vec::new();

        if has_split_active {
            let label = if is_split { "Move to Primary Pane" } else { "Move to Other Pane" };
            menu_items.push(
                button(text(label).size(13))
                    .on_press(Message::TabContextMoveToOtherPane(tab_idx, is_split))
                    .width(Length::Fill)
                    .padding([3, 8])
                    .style(move |_theme: &Theme, status| {
                        let bg = match status {
                            button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                            _ => None,
                        };
                        button::Style {
                            background: bg,
                            text_color: t_text,
                            border: iced::Border { radius: 2.0.into(), ..Default::default() },
                            ..Default::default()
                        }
                    })
                    .into(),
            );
        }

        menu_items.push(
            button(text("Close").size(13))
                .on_press(Message::TabContextClose(tab_idx, is_split))
                .width(Length::Fill)
                .padding([3, 8])
                .style(move |_theme: &Theme, status| {
                    let bg = match status {
                        button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                        _ => None,
                    };
                    button::Style {
                        background: bg,
                        text_color: t_text,
                        border: iced::Border { radius: 2.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                })
                .into(),
        );

        menu_items.push(
            button(text("Close Other Tabs").size(13))
                .on_press(Message::TabContextCloseOthers(tab_idx, is_split))
                .width(Length::Fill)
                .padding([3, 8])
                .style(move |_theme: &Theme, status| {
                    let bg = match status {
                        button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                        _ => None,
                    };
                    button::Style {
                        background: bg,
                        text_color: t_text,
                        border: iced::Border { radius: 2.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                })
                .into(),
        );

        menu_items.push(
            button(text("Close All").size(13))
                .on_press(Message::TabContextCloseAll(is_split))
                .width(Length::Fill)
                .padding([3, 8])
                .style(move |_theme: &Theme, status| {
                    let bg = match status {
                        button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                        _ => None,
                    };
                    button::Style {
                        background: bg,
                        text_color: t_text,
                        border: iced::Border { radius: 2.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                })
                .into(),
        );

        let context_menu = container(
            column(menu_items).spacing(0).padding(4).width(Length::Fixed(200.0)),
        )
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_menu_bg)),
            border: iced::Border {
                color: t_border,
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: iced::Vector::new(2.0, 2.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        });

        let context_overlay: Element<'_, Message> = container(opaque(context_menu))
            .padding(iced::Padding { top: 28.0, right: 0.0, bottom: 0.0, left: 100.0 })
            .width(Length::Shrink)
            .height(Length::Shrink)
            .into();

        layers.push(context_overlay);
    }

    stack(layers).into()
}

pub fn subscription(_state: &NotepadIced) -> Subscription<Message> {
    let keys = keyboard::on_key_press(|key, modifiers| {
        // Escape closes dialogs/menus — handled in update() which checks priority
        if matches!(key.as_ref(), keyboard::Key::Named(keyboard::key::Named::Escape)) {
            return Some(Message::EscapePressed);
        }

        // Function keys
        match key.as_ref() {
            keyboard::Key::Named(keyboard::key::Named::F3) if modifiers.shift() => {
                return Some(Message::FindPrev);
            }
            keyboard::Key::Named(keyboard::key::Named::F3) => {
                return Some(Message::FindNext);
            }
            keyboard::Key::Named(keyboard::key::Named::F2) if modifiers.shift() => {
                return Some(Message::PrevBookmark);
            }
            keyboard::Key::Named(keyboard::key::Named::F2) if modifiers.command() => {
                return Some(Message::ToggleBookmark);
            }
            keyboard::Key::Named(keyboard::key::Named::F2) => {
                return Some(Message::NextBookmark);
            }
            _ => {}
        }

        if !modifiers.command() {
            return None;
        }

        match key.as_ref() {
            keyboard::Key::Character("n") if !modifiers.shift() => Some(Message::NewTab),
            keyboard::Key::Character("t") if !modifiers.shift() => Some(Message::NewTabInActivePane),
            keyboard::Key::Character("o") => Some(Message::OpenFile),
            keyboard::Key::Character("s") if modifiers.shift() => Some(Message::SaveAs),
            keyboard::Key::Character("s") => Some(Message::Save),
            keyboard::Key::Character("w") => Some(Message::CloseTab(usize::MAX)),
            keyboard::Key::Character("z") if !modifiers.shift() => Some(Message::Undo),
            keyboard::Key::Character("y") => Some(Message::Redo),
            keyboard::Key::Character("a") => Some(Message::SelectAll),
            keyboard::Key::Character("f") if !modifiers.shift() => Some(Message::ShowFind),
            keyboard::Key::Character("h") => Some(Message::ShowReplace),
            keyboard::Key::Character("g") => Some(Message::GotoLine),
            keyboard::Key::Character("/") => Some(Message::ToggleComment),
            keyboard::Key::Character("j") if modifiers.shift() => Some(Message::JsonFormat),
            keyboard::Key::Character("=") if !modifiers.shift() => Some(Message::ZoomIn),
            keyboard::Key::Character("+") => Some(Message::ZoomIn),
            keyboard::Key::Character("-") => Some(Message::ZoomOut),
            keyboard::Key::Character("0") => Some(Message::ZoomReset),
            keyboard::Key::Character("r") if modifiers.shift() => {
                Some(Message::ToggleMacroRecording)
            }
            keyboard::Key::Character("p") if modifiers.shift() => Some(Message::PlayLastMacro),
            _ => None,
        }
    });

    let file_drops = iced::event::listen_with(|event, _status, _window| {
        match event {
            iced::Event::Window(iced::window::Event::FileDropped(path)) => {
                Some(Message::FileDropped(path))
            }
            _ => None,
        }
    });

    Subscription::batch(vec![keys, file_drops])
}

pub fn theme(state: &NotepadIced) -> Theme {
    if state.theme.is_light {
        Theme::Light
    } else {
        Theme::Dark
    }
}

// ── helpers ──

fn sync_content_to_doc(state: &mut NotepadIced, idx: usize) {
    if let Some(tc) = state.tab_contents.get(idx) {
        let new_text = tc.content.text();
        if let Some(doc) = state.tab_manager.get_document_mut(idx) {
            let len = doc.buffer.len_bytes();
            if len > 0 {
                doc.buffer.delete(0, len);
            }
            if !new_text.is_empty() {
                doc.buffer.insert(0, &new_text);
            }
        }
    }
}

fn sync_split_content_to_doc(state: &mut NotepadIced, idx: usize) {
    if let Some(tc) = state.split_tab_contents.get(idx) {
        let new_text = tc.content.text();
        if let Some(doc) = state.split_tab_manager.get_document_mut(idx) {
            let len = doc.buffer.len_bytes();
            if len > 0 {
                doc.buffer.delete(0, len);
            }
            if !new_text.is_empty() {
                doc.buffer.insert(0, &new_text);
            }
        }
    }
}

fn save_as_dialog() -> Task<Message> {
    Task::perform(
        async {
            let handle = rfd::AsyncFileDialog::new()
                .set_title("Save As")
                .save_file()
                .await;
            match handle {
                Some(h) => Ok(h.path().to_path_buf()),
                None => Err("Cancelled".to_string()),
            }
        },
        Message::FileSaved,
    )
}

fn get_buffer_text(state: &NotepadIced) -> String {
    state.tab_manager.active_document().buffer.text()
}

fn set_buffer_text(state: &mut NotepadIced, text: &str) {
    let doc = state.tab_manager.active_document_mut();
    let len = doc.buffer.len_bytes();
    if len > 0 {
        doc.buffer.delete(0, len);
    }
    if !text.is_empty() {
        doc.buffer.insert(0, text);
    }
    let idx = state.tab_manager.active_index();
    state.tab_contents[idx] = TabContent::with_text(text);
}

/// Apply a line-based transform: split text into lines, call the closure, rejoin, update buffer.
fn apply_line_op(state: &mut NotepadIced, op: impl FnOnce(&mut Vec<String>, usize)) {
    let text = get_buffer_text(state);
    let cursor_line = state
        .tab_contents
        .get(state.tab_manager.active_index())
        .map(|tc| tc.content.cursor_position().0)
        .unwrap_or(0);

    let mut lines: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();
    op(&mut lines, cursor_line);
    let new_text = lines.join("\n");
    set_buffer_text(state, &new_text);
}

/// Apply a whole-text transform.
fn apply_text_transform(state: &mut NotepadIced, transform: impl FnOnce(&str) -> String) {
    let text = get_buffer_text(state);
    let new_text = transform(&text);
    set_buffer_text(state, &new_text);
}

fn view_tab_bar<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let count = state.tab_manager.tab_count();
    let active = state.tab_manager.active_index();
    let is_active_pane = state.active_pane == 0;

    let t_tab_active = state.theme.tab_active_bg;
    let t_tab_inactive = state.theme.tab_inactive_bg;
    let t_text = state.theme.text;
    let t_text_dim = state.theme.text_dim;
    let t_accent = state.theme.accent;
    let t_tab_bar = state.theme.tab_bar_bg;

    let has_split = state.split_mode != SplitMode::None;

    let mut tabs = row![].spacing(1).padding([1, 2]);

    for i in 0..count {
        let tab_title = state.tab_manager.get_tab_title(i);
        let is_active = i == active;

        let bg_color = if is_active {
            t_tab_active
        } else {
            t_tab_inactive
        };

        let label = text(tab_title).size(13);
        let close = button(text("x").size(13))
            .on_press(Message::CloseTab(i))
            .padding(2)
            .style(move |_theme: &Theme, _status| button::Style {
                background: None,
                text_color: t_text_dim,
                ..Default::default()
            });

        let mut tab_row = row![label].spacing(6).padding([1, 6]);
        if has_split {
            let move_btn = button(text("→").size(11))
                .on_press(Message::MoveTabToSplit(i))
                .padding(2)
                .style(move |_theme: &Theme, _status| button::Style {
                    background: None,
                    text_color: t_text_dim,
                    ..Default::default()
                });
            tab_row = tab_row.push(move_btn);
        }
        tab_row = tab_row.push(close);

        let tab = button(tab_row)
            .on_press(Message::SelectTab(i))
            .style(move |_theme: &Theme, _status| button::Style {
                background: Some(iced::Background::Color(bg_color)),
                text_color: t_text,
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        let tab_with_context: Element<'a, Message> = mouse_area(tab)
            .on_right_press(Message::TabContextMenu(i, false))
            .into();

        tabs = tabs.push(tab_with_context);
    }

    // "+" add-tab button
    let add_btn = button(text("+").size(13))
        .on_press(Message::NewTab)
        .padding([1, 6])
        .style(move |_theme: &Theme, _status| button::Style {
            background: None,
            text_color: t_text_dim,
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });
    tabs = tabs.push(add_btn);

    let border_color = if is_active_pane {
        t_accent
    } else {
        t_tab_bar
    };

    container(tabs)
        .width(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_tab_bar)),
            border: iced::Border {
                color: border_color,
                width: if is_active_pane { 1.0 } else { 0.0 },
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn view_editor<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let active = state.tab_manager.active_index();
    let t_text_dim = state.theme.text_dim;
    let t_bg = state.theme.background;

    if let Some(tc) = state.tab_contents.get(active) {
        let wrapping = if state.word_wrap {
            Wrapping::Word
        } else {
            Wrapping::None
        };

        let editor = text_editor(&tc.content)
            .on_action(Message::EditorAction)
            .size(state.font_size)
            .height(Length::Fill)
            .wrapping(wrapping)
            .highlight_with::<SyntectHighlighter>(
                SyntectSettings {
                    extension: state.file_extension.clone(),
                    theme: state.theme.syntect_theme().to_string(),
                },
                |highlight, _theme| highlight.to_format(),
            );

        if state.show_line_numbers {
            let content_text = tc.content.text();
            let line_count = content_text.lines().count().max(1);
            let gutter_width = 55.0;

            let mut gutter_col = column![];
            for i in 1..=line_count {
                let line_idx = i - 1; // 0-based
                let fold_indicator = if state.fold_manager.is_folded(line_idx) {
                    "[+] "
                } else if state.fold_manager.is_fold_point(line_idx) {
                    "[-] "
                } else {
                    "    "
                };

                let line_label = text(format!("{}{}", fold_indicator, i))
                    .size(state.font_size)
                    .color(t_text_dim);

                let line_widget: Element<'_, Message> = if state.fold_manager.is_fold_point(line_idx) {
                    mouse_area(
                        container(line_label)
                            .width(Length::Fixed(gutter_width))
                            .align_x(iced::alignment::Horizontal::Right)
                            .padding([0, 4]),
                    )
                    .on_press(Message::ToggleFoldAt(line_idx))
                    .into()
                } else {
                    container(line_label)
                        .width(Length::Fixed(gutter_width))
                        .align_x(iced::alignment::Horizontal::Right)
                        .padding([0, 4])
                        .into()
                };

                gutter_col = gutter_col.push(line_widget);
            }

            let gutter = container(scrollable(gutter_col))
                .height(Length::Fill)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(t_bg)),
                    ..Default::default()
                });

            let editor_row = row![gutter, editor].height(Length::Fill);

            container(editor_row)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(t_bg)),
                    ..Default::default()
                })
                .into()
        } else {
            container(editor)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(t_bg)),
                    ..Default::default()
                })
                .into()
        }
    } else {
        container(text("No document open"))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

/// Tab bar for split pane, rendered at the same level as the primary tab bar.
fn view_split_tab_bar<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let is_active_pane = state.active_pane == 1;

    let t_tab_active = state.theme.tab_active_bg;
    let t_tab_inactive = state.theme.tab_inactive_bg;
    let t_text = state.theme.text;
    let t_text_dim = state.theme.text_dim;
    let t_accent = state.theme.accent;
    let t_tab_bar = state.theme.tab_bar_bg;

    let count = state.split_tab_manager.tab_count();
    let active = state.split_tab_manager.active_index();

    let mut tabs = row![].spacing(1).padding([1, 2]);

    for i in 0..count {
        let tab_title = state.split_tab_manager.get_tab_title(i);
        let is_active_tab = i == active;

        let bg_color = if is_active_tab {
            t_tab_active
        } else {
            t_tab_inactive
        };

        let label = text(tab_title).size(13);
        let close = button(text("x").size(13))
            .on_press(Message::SplitCloseTab(i))
            .padding(2)
            .style(move |_theme: &Theme, _status| button::Style {
                background: None,
                text_color: t_text_dim,
                ..Default::default()
            });

        let move_btn = button(text("←").size(11))
            .on_press(Message::MoveTabFromSplit(i))
            .padding(2)
            .style(move |_theme: &Theme, _status| button::Style {
                background: None,
                text_color: t_text_dim,
                ..Default::default()
            });

        let tab = button(row![label, move_btn, close].spacing(6).padding([1, 6]))
            .on_press(Message::SplitSelectTab(i))
            .style(move |_theme: &Theme, _status| button::Style {
                background: Some(iced::Background::Color(bg_color)),
                text_color: t_text,
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        let tab_with_context: Element<'a, Message> = mouse_area(tab)
            .on_right_press(Message::TabContextMenu(i, true))
            .into();

        tabs = tabs.push(tab_with_context);
    }

    // "+" add-tab button
    let add_btn = button(text("+").size(13))
        .on_press(Message::SplitNewTab)
        .padding([1, 6])
        .style(move |_theme: &Theme, _status| button::Style {
            background: None,
            text_color: t_text_dim,
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });
    tabs = tabs.push(add_btn);

    let border_color = if is_active_pane {
        t_accent
    } else {
        t_tab_bar
    };

    container(tabs)
        .width(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_tab_bar)),
            border: iced::Border {
                color: border_color,
                width: if is_active_pane { 1.0 } else { 0.0 },
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

/// Full editing secondary pane for split view (editor + status only, tab bar rendered separately).
fn view_split_pane<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let t_text_dim = state.theme.text_dim;
    let t_bg = state.theme.background;
    let t_status = state.theme.status_bar_bg;

    let active = state.split_tab_manager.active_index();

    // Editor for split pane
    let split_editor: Element<'_, Message> = if let Some(tc) = state.split_tab_contents.get(active) {
        let wrapping = if state.word_wrap {
            Wrapping::Word
        } else {
            Wrapping::None
        };

        let editor_widget = text_editor(&tc.content)
            .on_action(Message::SplitEditorAction)
            .size(state.split_font_size)
            .height(Length::Fill)
            .wrapping(wrapping)
            .highlight_with::<SyntectHighlighter>(
                SyntectSettings {
                    extension: state.split_file_extension.clone(),
                    theme: state.theme.syntect_theme().to_string(),
                },
                |highlight, _theme| highlight.to_format(),
            );

        if state.show_line_numbers {
            let content_text = tc.content.text();
            let line_count = content_text.lines().count().max(1);
            let gutter_width = 55.0;

            let mut gutter_col = column![];
            for i in 1..=line_count {
                let line_label = text(format!("    {}", i))
                    .size(state.split_font_size)
                    .color(t_text_dim);

                let line_widget: Element<'_, Message> = container(line_label)
                    .width(Length::Fixed(gutter_width))
                    .align_x(iced::alignment::Horizontal::Right)
                    .padding([0, 4])
                    .into();

                gutter_col = gutter_col.push(line_widget);
            }

            let gutter = container(scrollable(gutter_col))
                .height(Length::Fill)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(t_bg)),
                    ..Default::default()
                });

            let editor_row = row![gutter, editor_widget].height(Length::Fill);

            container(editor_row)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(t_bg)),
                    ..Default::default()
                })
                .into()
        } else {
            container(editor_widget)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(t_bg)),
                    ..Default::default()
                })
                .into()
        }
    } else {
        container(text("No document open"))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

    // Status bar for split pane
    let (line, col) = state
        .split_tab_contents
        .get(active)
        .map(|tc| {
            let (l, c) = tc.content.cursor_position();
            (l + 1, c + 1)
        })
        .unwrap_or((1, 1));

    let split_status: Element<'_, Message> = container(
        text(format!("  Ln {}, Col {}", line, col)).size(12),
    )
    .width(Length::Fill)
    .padding([2, 8])
    .style(move |_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(t_status)),
        ..Default::default()
    })
    .into();

    let pane_content = column![split_editor, split_status]
        .width(Length::Fill)
        .height(Length::Fill);

    mouse_area(pane_content)
        .on_press(Message::SetActivePane(1))
        .into()
}

fn view_status_bar<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let active = state.tab_manager.active_index();
    let t_status = state.theme.status_bar_bg;
    let (line, col, encoding_str, line_ending_str, language) =
        if let Some(tc) = state.tab_contents.get(active) {
            let (cursor_line, cursor_col) = tc.content.cursor_position();
            let doc = state.tab_manager.active_document();
            let enc = match doc.encoding {
                Encoding::UTF8 => "UTF-8",
                Encoding::UTF8BOM => "UTF-8 BOM",
                Encoding::UTF16LE => "UTF-16 LE",
                Encoding::UTF16BE => "UTF-16 BE",
                Encoding::ASCII => "ASCII",
            };
            let le = match doc.line_ending {
                LineEnding::LF => "LF",
                LineEnding::CRLF => "CRLF",
                LineEnding::CR => "CR",
            };
            let lang = if doc.language.is_empty() {
                "Plain Text"
            } else {
                &doc.language
            };
            (
                cursor_line + 1,
                cursor_col + 1,
                enc.to_string(),
                le.to_string(),
                lang.to_string(),
            )
        } else {
            (
                1,
                1,
                "UTF-8".to_string(),
                "LF".to_string(),
                "Plain Text".to_string(),
            )
        };

    let status_text = text(format!(
        "  Ln {}, Col {}    {}    {}    {}{}",
        line, col, encoding_str, line_ending_str, language,
        if state.show_whitespace { "    WS" } else { "" }
    ))
    .size(12);

    container(status_text)
        .width(Length::Fill)
        .padding([2, 8])
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_status)),
            ..Default::default()
        })
        .into()
}


fn run_search(state: &mut NotepadIced) {
    if state.search_query.is_empty() {
        state.search_matches.clear();
        state.current_match_index = None;
        return;
    }

    let mut engine = SearchEngine::new();
    engine.query = state.search_query.clone();
    engine.case_sensitive = state.case_sensitive;
    engine.whole_word = state.whole_word;
    engine.use_regex = state.use_regex;
    engine.search_mode = if state.use_regex {
        SearchMode::Regex
    } else {
        SearchMode::Normal
    };

    let text = get_buffer_text(state);
    state.search_matches = engine.find_all(&text);

    // Keep current_match_index in bounds
    if state.search_matches.is_empty() {
        state.current_match_index = None;
    } else if let Some(idx) = state.current_match_index {
        if idx >= state.search_matches.len() {
            state.current_match_index = Some(0);
        }
    } else {
        state.current_match_index = Some(0);
    }
}

fn rebuild_content(state: &mut NotepadIced) {
    let idx = state.tab_manager.active_index();
    let buf_text = state.tab_manager.active_document().buffer.text();
    if idx < state.tab_contents.len() {
        state.tab_contents[idx] = TabContent::with_text(&buf_text);
    }
}
