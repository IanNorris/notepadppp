use std::path::Path;

use serde::{Deserialize, Serialize};

use super::document::Document;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CursorMove {
    Left(usize),
    Right(usize),
    Up(usize),
    Down(usize),
    Home,
    End,
    DocumentStart,
    DocumentEnd,
    WordLeft,
    WordRight,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MacroAction {
    InsertText(String),
    DeleteForward(usize),
    DeleteBackward(usize),
    MoveCursor(CursorMove),
    SelectText(CursorMove),
    Cut,
    Copy,
    Paste(String),
    Undo,
    Redo,
    FindNext(String),
    Replace(String, String),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Macro {
    pub name: String,
    pub actions: Vec<MacroAction>,
}

pub struct MacroRecorder {
    recording: bool,
    current_recording: Vec<MacroAction>,
    saved_macros: Vec<Macro>,
}

impl Default for MacroRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl MacroRecorder {
    pub fn new() -> Self {
        Self {
            recording: false,
            current_recording: Vec::new(),
            saved_macros: Vec::new(),
        }
    }

    pub fn start_recording(&mut self) {
        self.recording = true;
        self.current_recording.clear();
    }

    pub fn stop_recording(&mut self, name: &str) -> Macro {
        self.recording = false;
        let macro_def = Macro {
            name: name.to_string(),
            actions: std::mem::take(&mut self.current_recording),
        };
        self.saved_macros.push(macro_def.clone());
        macro_def
    }

    pub fn is_recording(&self) -> bool {
        self.recording
    }

    pub fn record_action(&mut self, action: MacroAction) {
        if self.recording {
            self.current_recording.push(action);
        }
    }

    pub fn play_macro(macro_def: &Macro, document: &mut Document) {
        for action in &macro_def.actions {
            apply_action(action, document);
        }
    }

    pub fn play_macro_n_times(macro_def: &Macro, document: &mut Document, n: usize) {
        for _ in 0..n {
            Self::play_macro(macro_def, document);
        }
    }

    pub fn save_macros(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.saved_macros)
            .map_err(|e| format!("Failed to serialize macros: {}", e))?;
        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write macros file: {}", e))?;
        Ok(())
    }

    pub fn load_macros(&mut self, path: &Path) -> Result<(), String> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read macros file: {}", e))?;
        self.saved_macros = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse macros: {}", e))?;
        Ok(())
    }

    pub fn saved_macros(&self) -> &[Macro] {
        &self.saved_macros
    }

    pub fn delete_macro(&mut self, index: usize) {
        if index < self.saved_macros.len() {
            self.saved_macros.remove(index);
        }
    }
}

fn apply_action(action: &MacroAction, document: &mut Document) {
    match action {
        MacroAction::InsertText(text) => {
            if let Some(offset) = document.buffer.byte_offset(
                document.cursor.position.line,
                document.cursor.position.col,
            ) {
                document.buffer.insert(offset, text);
                // Advance cursor by the inserted text length (in chars)
                let added_chars = text.chars().count();
                // Simple: count newlines for line advancement
                let newlines = text.chars().filter(|c| *c == '\n').count();
                if newlines > 0 {
                    document.cursor.position.line += newlines;
                    let last_line_len = text.rsplit('\n').next().map(|s| s.chars().count()).unwrap_or(0);
                    document.cursor.position.col = last_line_len;
                } else {
                    document.cursor.position.col += added_chars;
                }
            }
        }
        MacroAction::DeleteForward(n) => {
            if let Some(start) = document.buffer.byte_offset(
                document.cursor.position.line,
                document.cursor.position.col,
            ) {
                let text = document.buffer.text();
                let remaining = &text[start..];
                let char_end: usize = remaining.chars().take(*n).map(|c| c.len_utf8()).sum();
                let end = start + char_end;
                if end > start && end <= document.buffer.len_bytes() {
                    document.buffer.delete(start, end);
                }
            }
        }
        MacroAction::DeleteBackward(n) => {
            if let Some(end) = document.buffer.byte_offset(
                document.cursor.position.line,
                document.cursor.position.col,
            ) {
                let text = document.buffer.text();
                let before = &text[..end];
                let char_start: usize = before.chars().rev().take(*n).map(|c| c.len_utf8()).sum();
                let start = end - char_start;
                if start < end {
                    document.buffer.delete(start, end);
                    // Move cursor back
                    if let Some((line, col)) = document.buffer.line_col(start) {
                        document.cursor.position.line = line;
                        document.cursor.position.col = col;
                    }
                }
            }
        }
        MacroAction::MoveCursor(mv) => {
            apply_cursor_move(mv, document, false);
        }
        MacroAction::SelectText(mv) => {
            apply_cursor_move(mv, document, true);
        }
        MacroAction::Undo => {
            document.buffer.undo();
        }
        MacroAction::Redo => {
            document.buffer.redo();
        }
        MacroAction::Cut | MacroAction::Copy | MacroAction::Paste(_) |
        MacroAction::FindNext(_) | MacroAction::Replace(_, _) => {
            // These require clipboard/search integration; no-op in headless playback
        }
    }
}

fn apply_cursor_move(mv: &CursorMove, document: &mut Document, select: bool) {
    if select {
        document.cursor.start_selection();
    }
    match mv {
        CursorMove::Left(n) => {
            for _ in 0..*n {
                document.cursor.move_left(&document.buffer);
            }
        }
        CursorMove::Right(n) => {
            for _ in 0..*n {
                document.cursor.move_right(&document.buffer);
            }
        }
        CursorMove::Up(n) => {
            for _ in 0..*n {
                document.cursor.move_up(&document.buffer);
            }
        }
        CursorMove::Down(n) => {
            for _ in 0..*n {
                document.cursor.move_down(&document.buffer);
            }
        }
        CursorMove::Home => {
            document.cursor.move_home();
        }
        CursorMove::End => {
            document.cursor.move_end(&document.buffer);
        }
        CursorMove::DocumentStart => {
            document.cursor.set_position(0, 0);
        }
        CursorMove::DocumentEnd => {
            let last_line = document.buffer.line_count().saturating_sub(1);
            let col = document.buffer.line_len_chars(last_line);
            document.cursor.set_position(last_line, col);
        }
        CursorMove::WordLeft => {
            document.cursor.move_word_left(&document.buffer);
        }
        CursorMove::WordRight => {
            document.cursor.move_word_right(&document.buffer);
        }
    }
}
