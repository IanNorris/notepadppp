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
use super::theme::AppColors;

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

    // Macro
    ToggleMacroRecording,
    PlayLastMacro,
    PlayMacroMultiple,

    // Settings
    ShowPreferences,
    ShowKeybindings,

    // Help
    ShowAbout,
    CloseAbout,

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
    pub split_tab_index: Option<usize>,
}

impl Default for NotepadIced {
    fn default() -> Self {
        Self {
            tab_manager: TabManager::new(),
            tab_contents: vec![TabContent::new()],
            active_menu: None,
            expanded_submenus: std::collections::HashSet::new(),
            word_wrap: false,
            show_line_numbers: true,
            show_whitespace: false,
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
            find_panel_pos: None, // None = right-aligned default
            dragging_find_panel: false,
            drag_offset: (0.0, 0.0),
            last_mouse_pos: iced::Point::ORIGIN,
            font_size: 14.0,
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
            split_tab_index: None,
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
        Message::MenuToggle(_) | Message::MenuClose | Message::SubMenuToggle(_) | Message::EditorAction(_)
            | Message::FileOpened(_) | Message::FileSaved(_)
            | Message::FindQueryChanged(_) | Message::ReplaceTextChanged(_)
            | Message::ToggleCaseSensitive | Message::ToggleWholeWord | Message::ToggleRegex
            | Message::FindNext | Message::FindPrev | Message::ReplaceNext | Message::ReplaceAll
            | Message::CloseSearch | Message::ClickSearchResult(_)
            | Message::GotoLineInputChanged(_) | Message::GotoLineConfirm | Message::GotoLineClose
            | Message::CloseAbout
            | Message::SessionFileChosen(_) | Message::ExportSaved(_) | Message::CompareFileLoaded(_)
            | Message::EscapePressed
            | Message::DragFindStart | Message::DragFindMove(_) | Message::DragFindEnd
            | Message::FifQueryChanged(_) | Message::FifDirectoryChanged(_) | Message::FifFileFilterChanged(_)
            | Message::FifToggleRecursive | Message::FifToggleCaseSensitive | Message::FifToggleRegex
            | Message::FifSearch | Message::FifSearchComplete(_) | Message::FifClickResult(_, _)
            | Message::CloseFindInFiles
            | Message::DragFifStart | Message::DragFifMove(_) | Message::DragFifEnd
    );
    if should_close_menu {
        state.active_menu = None;
    }

    match message {
        Message::EditorAction(action) => {
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
            state.tab_manager.new_tab();
            state.tab_contents.push(TabContent::new());
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
            state.split_mode = SplitMode::Horizontal;
            state.split_tab_index = Some(state.tab_manager.active_index());
            Task::none()
        }
        Message::SplitVertical => {
            state.split_mode = SplitMode::Vertical;
            state.split_tab_index = Some(state.tab_manager.active_index());
            Task::none()
        }
        Message::RemoveSplit => {
            state.split_mode = SplitMode::None;
            state.split_tab_index = None;
            Task::none()
        }
        Message::ZoomIn => {
            state.font_size = (state.font_size + 2.0).min(72.0);
            Task::none()
        }
        Message::ZoomOut => {
            state.font_size = (state.font_size - 2.0).max(6.0);
            Task::none()
        }
        Message::ZoomReset => {
            state.font_size = 14.0;
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
            state.show_markdown_preview = !state.show_markdown_preview;
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
            // We use the same goto-line logic
            let idx = state.tab_manager.active_index();
            if let Some(tc) = state.tab_contents.get(idx) {
                let text = tc.content.text();
                let mut offset = 0;
                for (i, text_line) in text.lines().enumerate() {
                    if i == line {
                        break;
                    }
                    offset += text_line.len() + 1; // +1 for newline
                }
                // Rebuild content positioned at the start (we can't move cursor directly,
                // so just log the target for now — full cursor navigation requires
                // editor action support which iced text_editor doesn't expose)
                log::info!("Navigate to symbol at line {}", line + 1);
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
            log::info!("Show preferences");
            Task::none()
        }
        Message::ShowKeybindings => {
            log::info!("Show keybindings");
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
            if state.show_find {
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
    }
}

pub fn view(state: &NotepadIced) -> Element<'_, Message> {
    let menu_bar = menu_bar::view_menu_bar(&state.active_menu);
    let tab_bar = view_tab_bar(state);

    // Build the main content area based on active viewer panels
    let main_area: Element<'_, Message> = if state.show_csv_viewer {
        super::csv_panel::view_csv_viewer(state)
    } else if state.show_hex_viewer {
        super::hex_panel::view_hex_viewer(state)
    } else {
        // Normal editor, possibly with side panels
        let editor = view_editor(state);

        // Wrap editor with split view if active
        let editor_area = if state.split_mode != SplitMode::None {
            let split_pane = view_split_pane(state);
            let divider = container(Space::new(
                if state.split_mode == SplitMode::Horizontal { Length::Fixed(2.0) } else { Length::Fill },
                if state.split_mode == SplitMode::Vertical { Length::Fixed(2.0) } else { Length::Fill },
            ))
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(AppColors::BORDER)),
                ..Default::default()
            });

            match state.split_mode {
                SplitMode::Horizontal => {
                    row![
                        container(editor).width(Length::FillPortion(1)).height(Length::Fill),
                        divider,
                        container(split_pane).width(Length::FillPortion(1)).height(Length::Fill),
                    ]
                    .height(Length::Fill)
                    .into()
                }
                SplitMode::Vertical => {
                    column![
                        container(editor).width(Length::Fill).height(Length::FillPortion(1)),
                        divider,
                        container(split_pane).width(Length::Fill).height(Length::FillPortion(1)),
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                }
                SplitMode::None => unreachable!(),
            }
        } else {
            editor
        };

        if state.show_markdown_preview {
            row![
                container(editor_area).width(Length::FillPortion(1)),
                super::markdown_panel::view_markdown_preview(state),
            ]
            .height(Length::Fill)
            .into()
        } else if state.show_function_list {
            row![
                container(editor_area).width(Length::Fill),
                super::function_list_panel::view_function_list(state),
            ]
            .height(Length::Fill)
            .into()
        } else {
            editor_area
        }
    };

    let mut base_content = column![menu_bar, tab_bar, main_area];

    if state.show_status_bar {
        base_content = base_content.push(view_status_bar(state));
    }

    let base: Element<'_, Message> = container(base_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    // Check if any overlay is needed
    let has_dropdown = state.active_menu.is_some();
    let has_floating = state.show_find || state.show_goto_line || state.show_about || state.show_find_in_files;

    if !has_dropdown && !has_floating {
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

            let dropdown_overlay: Element<'_, Message> = container(
                opaque(
                    container(dropdown)
                        .max_height(500)
                        .style(|_theme: &Theme| container::Style {
                            background: Some(iced::Background::Color(super::theme::AppColors::TAB_BAR_BG)),
                            border: iced::Border {
                                color: iced::Color::from_rgb(0.3, 0.3, 0.35),
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        }),
                ),
            )
            .padding(iced::Padding { top: 26.0, right: 0.0, bottom: 0.0, left: 0.0 })
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

        let search_widget = opaque(super::search_panel::view_search_panel(state));

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

        let fif_widget = opaque(super::find_in_files_panel::view_find_in_files_panel(state));

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
            opaque(super::goto_dialog::view_goto_dialog(state)),
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
            opaque(super::about_dialog::view_about_dialog()),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();

        layers.push(about_overlay);
    }

    stack(layers).into()
}

pub fn subscription(_state: &NotepadIced) -> Subscription<Message> {
    keyboard::on_key_press(|key, modifiers| {
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
    })
}

pub fn theme(_state: &NotepadIced) -> Theme {
    Theme::Dark
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

    let mut tabs = row![].spacing(2).padding(4);

    for i in 0..count {
        let tab_title = state.tab_manager.get_tab_title(i);
        let is_active = i == active;

        let bg_color = if is_active {
            AppColors::TAB_ACTIVE_BG
        } else {
            AppColors::TAB_INACTIVE_BG
        };

        let label = text(tab_title).size(13);
        let close = button(text("×").size(13))
            .on_press(Message::CloseTab(i))
            .padding(2)
            .style(|_theme: &Theme, _status| button::Style {
                background: None,
                text_color: AppColors::TEXT_DIM,
                ..Default::default()
            });

        let tab = button(row![label, close].spacing(6).padding([4, 8]))
            .on_press(Message::SelectTab(i))
            .style(move |_theme: &Theme, _status| button::Style {
                background: Some(iced::Background::Color(bg_color)),
                text_color: AppColors::TEXT,
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        tabs = tabs.push(tab);
    }

    container(tabs)
        .width(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(AppColors::TAB_BAR_BG)),
            ..Default::default()
        })
        .into()
}

fn view_editor<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let active = state.tab_manager.active_index();

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
                    theme: "base16-ocean.dark".to_string(),
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
                    "[−] "
                } else {
                    "    "
                };

                let line_label = text(format!("{}{}", fold_indicator, i))
                    .size(state.font_size)
                    .color(AppColors::TEXT_DIM);

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
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(AppColors::BACKGROUND)),
                    ..Default::default()
                });

            let editor_row = row![gutter, editor].height(Length::Fill);

            container(editor_row)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(AppColors::BACKGROUND)),
                    ..Default::default()
                })
                .into()
        } else {
            container(editor)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(AppColors::BACKGROUND)),
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

/// Read-only secondary pane for split view, showing syntax-highlighted content.
fn view_split_pane<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let tab_idx = state.split_tab_index.unwrap_or(state.tab_manager.active_index());

    if let Some(tc) = state.tab_contents.get(tab_idx) {
        let content_text = tc.content.text();
        let lines: Vec<String> = content_text.lines().map(|l| l.to_string()).collect();
        let gutter_width = 55.0;

        let mut lines_col = column![].spacing(0);

        if state.show_line_numbers {
            let mut gutter_col = column![].spacing(0);
            for (i, line) in lines.iter().enumerate() {
                gutter_col = gutter_col.push(
                    container(
                        text(format!("{}", i + 1))
                            .size(state.font_size)
                            .color(AppColors::TEXT_DIM),
                    )
                    .width(Length::Fixed(gutter_width))
                    .align_x(iced::alignment::Horizontal::Right)
                    .padding([0, 4]),
                );
                lines_col = lines_col.push(
                    text(if line.is_empty() { String::from(" ") } else { line.clone() })
                        .size(state.font_size)
                        .color(AppColors::TEXT)
                        .font(iced::Font::MONOSPACE),
                );
            }
            if lines.is_empty() {
                gutter_col = gutter_col.push(
                    container(
                        text("1").size(state.font_size).color(AppColors::TEXT_DIM),
                    )
                    .width(Length::Fixed(gutter_width))
                    .align_x(iced::alignment::Horizontal::Right)
                    .padding([0, 4]),
                );
                lines_col = lines_col.push(
                    text(" ")
                        .size(state.font_size)
                        .color(AppColors::TEXT)
                        .font(iced::Font::MONOSPACE),
                );
            }

            let gutter = container(scrollable(gutter_col))
                .height(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(AppColors::BACKGROUND)),
                    ..Default::default()
                });

            let text_area = container(scrollable(lines_col))
                .width(Length::Fill)
                .height(Length::Fill);

            container(row![gutter, text_area].height(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(AppColors::BACKGROUND)),
                    ..Default::default()
                })
                .into()
        } else {
            for line in &lines {
                lines_col = lines_col.push(
                    text(if line.is_empty() { String::from(" ") } else { line.clone() })
                        .size(state.font_size)
                        .color(AppColors::TEXT)
                        .font(iced::Font::MONOSPACE),
                );
            }
            if lines.is_empty() {
                lines_col = lines_col.push(
                    text(" ")
                        .size(state.font_size)
                        .color(AppColors::TEXT)
                        .font(iced::Font::MONOSPACE),
                );
            }

            container(scrollable(lines_col))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(AppColors::BACKGROUND)),
                    ..Default::default()
                })
                .into()
        }
    } else {
        container(text("No document"))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn view_status_bar<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let active = state.tab_manager.active_index();
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
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(AppColors::STATUS_BAR_BG)),
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
