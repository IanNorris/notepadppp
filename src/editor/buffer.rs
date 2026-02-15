use ropey::Rope;
use std::io::BufReader;

#[derive(Debug, Clone)]
pub enum EditOperation {
    Insert { pos: usize, text: String },
    Delete { pos: usize, text: String },
}

/// A text buffer backed by `ropey::Rope` with undo/redo support.
#[derive(Debug)]
pub struct TextBuffer {
    rope: Rope,
    modified: bool,
    undo_stack: Vec<EditOperation>,
    redo_stack: Vec<EditOperation>,
    /// Optional limit on undo history size (for large files).
    undo_limit: Option<usize>,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            modified: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            undo_limit: None,
        }
    }

    pub fn from_str(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
            modified: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            undo_limit: None,
        }
    }

    /// Create a buffer by streaming from a reader (efficient for large files).
    pub fn from_reader<R: std::io::Read>(reader: R) -> std::io::Result<Self> {
        let buf_reader = BufReader::new(reader);
        let rope = Rope::from_reader(buf_reader)?;
        Ok(Self {
            rope,
            modified: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            undo_limit: None,
        })
    }

    /// Set a limit on the undo history. `None` means unlimited.
    pub fn set_undo_limit(&mut self, limit: Option<usize>) {
        self.undo_limit = limit;
        if let Some(max) = limit {
            self.enforce_undo_limit(max);
        }
    }

    fn enforce_undo_limit(&mut self, max: usize) {
        if self.undo_stack.len() > max {
            let excess = self.undo_stack.len() - max;
            self.undo_stack.drain(..excess);
        }
    }

    /// Insert `text` at the given byte position.
    pub fn insert(&mut self, byte_pos: usize, text: &str) {
        let char_pos = self.rope.byte_to_char(byte_pos);
        self.rope.insert(char_pos, text);
        self.undo_stack.push(EditOperation::Insert {
            pos: byte_pos,
            text: text.to_string(),
        });
        if let Some(max) = self.undo_limit {
            self.enforce_undo_limit(max);
        }
        self.redo_stack.clear();
        self.modified = true;
    }

    /// Delete a range of bytes `[start..end)`.
    pub fn delete(&mut self, start: usize, end: usize) {
        let char_start = self.rope.byte_to_char(start);
        let char_end = self.rope.byte_to_char(end);
        let deleted: String = self.rope.slice(char_start..char_end).into();
        self.rope.remove(char_start..char_end);
        self.undo_stack.push(EditOperation::Delete {
            pos: start,
            text: deleted,
        });
        if let Some(max) = self.undo_limit {
            self.enforce_undo_limit(max);
        }
        self.redo_stack.clear();
        self.modified = true;
    }

    /// Return the full text as a `String`.
    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    /// Number of lines (always >= 1 for ropey).
    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    /// Get the text of a line by 0-based index.
    pub fn line(&self, idx: usize) -> Option<String> {
        if idx >= self.rope.len_lines() {
            return None;
        }
        Some(self.rope.line(idx).to_string())
    }

    /// Get the character at a byte position.
    pub fn char_at(&self, byte_pos: usize) -> Option<char> {
        if byte_pos >= self.rope.len_bytes() {
            return None;
        }
        let char_idx = self.rope.byte_to_char(byte_pos);
        Some(self.rope.char(char_idx))
    }

    /// Convert (line, column) to a byte offset. Column is in chars.
    pub fn byte_offset(&self, line: usize, col: usize) -> Option<usize> {
        if line >= self.rope.len_lines() {
            return None;
        }
        let line_start_char = self.rope.line_to_char(line);
        let line_len_chars = self.rope.line(line).len_chars();
        let clamped_col = col.min(line_len_chars);
        let char_idx = line_start_char + clamped_col;
        Some(self.rope.char_to_byte(char_idx))
    }

    /// Convert a byte offset to (line, column). Column is in chars.
    pub fn line_col(&self, byte_pos: usize) -> Option<(usize, usize)> {
        if byte_pos > self.rope.len_bytes() {
            return None;
        }
        let char_idx = self.rope.byte_to_char(byte_pos);
        let line = self.rope.char_to_line(char_idx);
        let line_start_char = self.rope.line_to_char(line);
        let col = char_idx - line_start_char;
        Some((line, col))
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn set_modified(&mut self, v: bool) {
        self.modified = v;
    }

    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn is_empty(&self) -> bool {
        self.rope.len_bytes() == 0
    }

    /// Get the char length of a given line (0-based index).
    pub fn line_len_chars(&self, line: usize) -> usize {
        if line >= self.rope.len_lines() {
            return 0;
        }
        self.rope.line(line).len_chars()
    }

    /// Undo the last operation. Returns `true` if an operation was undone.
    pub fn undo(&mut self) -> bool {
        let op = match self.undo_stack.pop() {
            Some(op) => op,
            None => return false,
        };
        match &op {
            EditOperation::Insert { pos, text } => {
                let char_start = self.rope.byte_to_char(*pos);
                let char_end = char_start + text.chars().count();
                self.rope.remove(char_start..char_end);
                self.redo_stack.push(op);
            }
            EditOperation::Delete { pos, text } => {
                let char_pos = self.rope.byte_to_char(*pos);
                self.rope.insert(char_pos, text);
                self.redo_stack.push(op);
            }
        }
        self.modified = true;
        true
    }

    /// Redo the last undone operation. Returns `true` if an operation was redone.
    pub fn redo(&mut self) -> bool {
        let op = match self.redo_stack.pop() {
            Some(op) => op,
            None => return false,
        };
        match &op {
            EditOperation::Insert { pos, text } => {
                let char_pos = self.rope.byte_to_char(*pos);
                self.rope.insert(char_pos, text);
                self.undo_stack.push(op);
            }
            EditOperation::Delete { pos, text } => {
                let char_start = self.rope.byte_to_char(*pos);
                let char_end = char_start + text.chars().count();
                self.rope.remove(char_start..char_end);
                self.undo_stack.push(op);
            }
        }
        self.modified = true;
        true
    }

    /// Move text from [start, end) to `dest` position.
    /// If `dest` falls within [start, end), this is a no-op.
    pub fn move_text(&mut self, start: usize, end: usize, dest: usize) {
        if start >= end || start > self.len_bytes() || end > self.len_bytes() {
            return;
        }
        if dest >= start && dest <= end {
            return;
        }
        let text = self.rope.byte_slice(start..end).to_string();
        // Delete first, then insert at adjusted position
        self.delete(start, end);
        let adjusted_dest = if dest > end { dest - (end - start) } else { dest };
        self.insert(adjusted_dest, &text);
    }

    /// Copy text from [start, end) to `dest` position (text remains at original location).
    pub fn copy_text_to(&mut self, start: usize, end: usize, dest: usize) {
        if start >= end || start > self.len_bytes() || end > self.len_bytes() {
            return;
        }
        let text = self.rope.byte_slice(start..end).to_string();
        self.insert(dest, &text);
    }
}
