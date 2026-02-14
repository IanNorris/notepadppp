use iced::keyboard;
use iced::widget::{button, column, container, row, scrollable, text, text_editor};
use iced::{Element, Length, Subscription, Task, Theme};

use crate::editor::document::{Encoding, LineEnding};
use crate::editor::tab_manager::TabManager;

use super::menu_bar;
use super::theme::AppColors;

/// Per-tab state that pairs an iced text_editor::Content with our Document index.
struct TabContent {
    content: text_editor::Content,
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
}

pub struct NotepadIced {
    pub tab_manager: TabManager,
    tab_contents: Vec<TabContent>,

    // Menu state
    pub active_menu: Option<String>,

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

    // Session
    pub auto_restore_session: bool,
}

impl Default for NotepadIced {
    fn default() -> Self {
        Self {
            tab_manager: TabManager::new(),
            tab_contents: vec![TabContent::new()],
            active_menu: None,
            word_wrap: false,
            show_line_numbers: true,
            show_whitespace: false,
            show_status_bar: true,
            show_minimap: false,
            show_function_list: false,
            show_markdown_preview: false,
            show_csv_viewer: false,
            show_hex_viewer: false,
            auto_restore_session: false,
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
    // Close menu for most actions (except MenuToggle/MenuClose/EditorAction)
    let should_close_menu = !matches!(
        message,
        Message::MenuToggle(_) | Message::MenuClose | Message::EditorAction(_)
            | Message::FileOpened(_) | Message::FileSaved(_)
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
                match state.tab_manager.open_file(path) {
                    Ok(idx) => {
                        let doc = state.tab_manager.get_document(idx).unwrap();
                        let buf_text = doc.buffer.text();
                        state.tab_contents.push(TabContent::with_text(&buf_text));
                    }
                    Err(_) => {
                        state.tab_manager.new_tab();
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
            } else {
                state.active_menu = Some(name);
            }
            Task::none()
        }
        Message::MenuClose => {
            state.active_menu = None;
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
            log::info!("Save session requested (not yet wired)");
            Task::none()
        }
        Message::LoadSession => {
            log::info!("Load session requested (not yet wired)");
            Task::none()
        }
        Message::ToggleAutoRestore => {
            state.auto_restore_session = !state.auto_restore_session;
            Task::none()
        }
        Message::ExportHtml => {
            log::info!("Export HTML requested");
            Task::none()
        }
        Message::ExportRtf => {
            log::info!("Export RTF requested");
            Task::none()
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
            log::info!("Show Find dialog");
            Task::none()
        }
        Message::ShowReplace => {
            log::info!("Show Replace dialog");
            Task::none()
        }
        Message::ShowFindInFiles => {
            log::info!("Show Find in Files dialog");
            Task::none()
        }
        Message::SelectAllOccurrences => {
            log::info!("Select all occurrences");
            Task::none()
        }
        Message::GotoLine => {
            log::info!("Go to line dialog");
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
            log::info!("Split horizontal");
            Task::none()
        }
        Message::SplitVertical => {
            log::info!("Split vertical");
            Task::none()
        }
        Message::RemoveSplit => {
            log::info!("Remove split");
            Task::none()
        }
        Message::ZoomIn => {
            log::info!("Zoom in");
            Task::none()
        }
        Message::ZoomOut => {
            log::info!("Zoom out");
            Task::none()
        }
        Message::ZoomReset => {
            log::info!("Zoom reset");
            Task::none()
        }
        Message::ToggleFold => {
            log::info!("Toggle fold");
            Task::none()
        }
        Message::FoldAll => {
            log::info!("Fold all");
            Task::none()
        }
        Message::UnfoldAll => {
            log::info!("Unfold all");
            Task::none()
        }
        Message::FoldLevel(level) => {
            log::info!("Fold level {}", level);
            Task::none()
        }

        // ── Language ──
        Message::SetLanguage(lang) => {
            state.tab_manager.active_document_mut().language = lang;
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
            log::info!("Compare files requested");
            Task::none()
        }
        Message::ToggleHexViewer => {
            state.show_hex_viewer = !state.show_hex_viewer;
            Task::none()
        }

        // ── Macro ──
        Message::ToggleMacroRecording => {
            log::info!("Toggle macro recording");
            Task::none()
        }
        Message::PlayLastMacro => {
            log::info!("Play last macro");
            Task::none()
        }
        Message::PlayMacroMultiple => {
            log::info!("Play macro multiple times");
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
            log::info!("Notepad+++ v{}", env!("CARGO_PKG_VERSION"));
            Task::none()
        }
    }
}

pub fn view(state: &NotepadIced) -> Element<'_, Message> {
    let menu_bar = menu_bar::view_menu_bar(&state.active_menu);

    let dropdown: Element<'_, Message> = if let Some(ref menu_name) = state.active_menu {
        scrollable(menu_bar::view_dropdown(state, menu_name))
            .height(Length::Shrink)
            .into()
    } else {
        column![].into()
    };

    let tab_bar = view_tab_bar(state);
    let editor = view_editor(state);

    let mut content = column![menu_bar, dropdown, tab_bar, editor];

    if state.show_status_bar {
        content = content.push(view_status_bar(state));
    }

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn subscription(_state: &NotepadIced) -> Subscription<Message> {
    keyboard::on_key_press(|key, modifiers| {
        // Escape closes menus
        if matches!(key.as_ref(), keyboard::Key::Named(keyboard::key::Named::Escape)) {
            return Some(Message::MenuClose);
        }

        // Function keys
        match key.as_ref() {
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
        let editor = text_editor(&tc.content)
            .on_action(Message::EditorAction)
            .height(Length::Fill);

        container(editor)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(AppColors::BACKGROUND)),
                ..Default::default()
            })
            .into()
    } else {
        container(text("No document open"))
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
        "  Ln {}, Col {}    {}    {}    {}",
        line, col, encoding_str, line_ending_str, language
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


