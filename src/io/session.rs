use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::editor::document::{Encoding, LineEnding};
use crate::editor::tab_manager::TabManager;

/// A saved editor session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub name: String,
    pub files: Vec<SessionFile>,
    pub active_tab: usize,
}

/// A file entry within a saved session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFile {
    pub path: PathBuf,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub encoding: String,
    pub line_ending: String,
}

/// Save a session to a JSON file.
pub fn save_session(session: &Session, path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(session)
        .map_err(|e| format!("Failed to serialize session: {e}"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create session directory: {e}"))?;
    }
    std::fs::write(path, json).map_err(|e| format!("Failed to write session file: {e}"))
}

/// Load a session from a JSON file.
pub fn load_session(path: &Path) -> Result<Session, String> {
    let json =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read session file: {e}"))?;
    serde_json::from_str(&json).map_err(|e| format!("Failed to parse session file: {e}"))
}

/// Capture the current editor state into a Session.
pub fn capture_session(tab_manager: &TabManager, name: &str) -> Session {
    let count = tab_manager.tab_count();
    let mut files = Vec::new();

    for i in 0..count {
        if let Some(doc) = tab_manager.get_document(i) {
            if let Some(ref path) = doc.path {
                files.push(SessionFile {
                    path: path.clone(),
                    cursor_line: doc.cursor.position.line,
                    cursor_col: doc.cursor.position.col,
                    encoding: encoding_to_string(doc.encoding),
                    line_ending: line_ending_to_string(doc.line_ending),
                });
            }
        }
    }

    Session {
        name: name.to_string(),
        files,
        active_tab: tab_manager.active_index(),
    }
}

/// Restore tabs from a session into the tab manager.
pub fn restore_session(session: &Session, tab_manager: &mut TabManager) -> Result<(), String> {
    tab_manager.close_all();

    let mut first = true;
    for file in &session.files {
        if !file.path.exists() {
            continue;
        }
        match tab_manager.open_file(file.path.clone()) {
            Ok(idx) => {
                if first {
                    // Close the default empty tab created by close_all
                    if tab_manager.tab_count() > 1 {
                        tab_manager.close_tab(0);
                    }
                    first = false;
                }
                // Restore cursor position - recalculate idx after possible close
                let actual_idx = if !first { idx - 0 } else { idx };
                if let Some(doc) = tab_manager.get_document_mut(actual_idx) {
                    doc.cursor.set_position(file.cursor_line, file.cursor_col);
                    doc.encoding = parse_encoding(&file.encoding);
                    doc.line_ending = parse_line_ending(&file.line_ending);
                }
            }
            Err(e) => {
                log::warn!("Failed to restore file {:?}: {}", file.path, e);
            }
        }
    }

    let tab_count = tab_manager.tab_count();
    if session.active_tab < tab_count {
        tab_manager.set_active(session.active_tab);
    }

    Ok(())
}

/// Returns the default auto-session file path in the config directory.
pub fn auto_session_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("notepadppp")
        .join("auto_session.json")
}

/// List all named sessions saved in the config directory.
pub fn list_saved_sessions() -> Vec<(String, PathBuf)> {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("notepadppp")
        .join("sessions");

    if !dir.exists() {
        return Vec::new();
    }

    let mut sessions = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Ok(session) = load_session(&path) {
                    sessions.push((session.name, path));
                }
            }
        }
    }
    sessions.sort_by(|a, b| a.0.cmp(&b.0));
    sessions
}

fn encoding_to_string(enc: Encoding) -> String {
    match enc {
        Encoding::UTF8 => "UTF-8".to_string(),
        Encoding::UTF8BOM => "UTF-8 BOM".to_string(),
        Encoding::UTF16LE => "UTF-16 LE".to_string(),
        Encoding::UTF16BE => "UTF-16 BE".to_string(),
        Encoding::ASCII => "ASCII".to_string(),
    }
}

fn line_ending_to_string(le: LineEnding) -> String {
    match le {
        LineEnding::CRLF => "CRLF".to_string(),
        LineEnding::LF => "LF".to_string(),
        LineEnding::CR => "CR".to_string(),
    }
}

fn parse_encoding(s: &str) -> Encoding {
    match s {
        "UTF-8" => Encoding::UTF8,
        "UTF-8 BOM" => Encoding::UTF8BOM,
        "UTF-16 LE" => Encoding::UTF16LE,
        "UTF-16 BE" => Encoding::UTF16BE,
        "ASCII" => Encoding::ASCII,
        _ => Encoding::UTF8,
    }
}

fn parse_line_ending(s: &str) -> LineEnding {
    match s {
        "CRLF" => LineEnding::CRLF,
        "LF" => LineEnding::LF,
        "CR" => LineEnding::CR,
        _ => LineEnding::LF,
    }
}
