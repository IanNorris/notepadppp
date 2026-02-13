use super::buffer::TextBuffer;

/// A cursor position within the buffer (0-based line and column).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

impl Position {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }

    pub fn zero() -> Self {
        Self { line: 0, col: 0 }
    }
}

/// Cursor state including optional selection anchor.
#[derive(Debug, Clone)]
pub struct CursorState {
    pub position: Position,
    pub anchor: Option<Position>,
}

impl Default for CursorState {
    fn default() -> Self {
        Self::new()
    }
}

impl CursorState {
    pub fn new() -> Self {
        Self {
            position: Position::zero(),
            anchor: None,
        }
    }

    pub fn has_selection(&self) -> bool {
        self.anchor.is_some()
    }

    /// Returns the selected range as (start, end) positions ordered so start <= end.
    pub fn selected_range(&self) -> Option<(Position, Position)> {
        let anchor = self.anchor?;
        let (start, end) = if (anchor.line, anchor.col) <= (self.position.line, self.position.col) {
            (anchor, self.position)
        } else {
            (self.position, anchor)
        };
        Some((start, end))
    }

    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }

    pub fn start_selection(&mut self) {
        if self.anchor.is_none() {
            self.anchor = Some(self.position);
        }
    }

    // --- Movement ---

    pub fn move_left(&mut self, buf: &TextBuffer) {
        self.anchor = None;
        if self.position.col > 0 {
            self.position.col -= 1;
        } else if self.position.line > 0 {
            self.position.line -= 1;
            self.position.col = line_char_len(buf, self.position.line);
        }
    }

    pub fn move_right(&mut self, buf: &TextBuffer) {
        self.anchor = None;
        let line_len = line_char_len(buf, self.position.line);
        if self.position.col < line_len {
            self.position.col += 1;
        } else if self.position.line + 1 < buf.line_count() {
            self.position.line += 1;
            self.position.col = 0;
        }
    }

    pub fn move_up(&mut self, buf: &TextBuffer) {
        self.anchor = None;
        if self.position.line > 0 {
            self.position.line -= 1;
            let line_len = line_char_len(buf, self.position.line);
            self.position.col = self.position.col.min(line_len);
        }
    }

    pub fn move_down(&mut self, buf: &TextBuffer) {
        self.anchor = None;
        if self.position.line + 1 < buf.line_count() {
            self.position.line += 1;
            let line_len = line_char_len(buf, self.position.line);
            self.position.col = self.position.col.min(line_len);
        }
    }

    pub fn move_home(&mut self) {
        self.anchor = None;
        self.position.col = 0;
    }

    pub fn move_end(&mut self, buf: &TextBuffer) {
        self.anchor = None;
        self.position.col = line_char_len(buf, self.position.line);
    }

    /// Move cursor to the start of the next word, or end of buffer.
    pub fn move_word_right(&mut self, buf: &TextBuffer) {
        self.anchor = None;
        let text = buf.text();
        let byte_off = match buf.byte_offset(self.position.line, self.position.col) {
            Some(o) => o,
            None => return,
        };
        let bytes = text.as_bytes();
        let len = bytes.len();
        let mut i = byte_off;

        // Skip current word chars
        while i < len && is_word_byte(bytes[i]) {
            i += 1;
        }
        // Skip non-word chars
        while i < len && !is_word_byte(bytes[i]) {
            i += 1;
        }

        if let Some((l, c)) = buf.line_col(i) {
            self.position.line = l;
            self.position.col = c;
        }
    }

    /// Move cursor to the start of the previous word, or start of buffer.
    pub fn move_word_left(&mut self, buf: &TextBuffer) {
        self.anchor = None;
        let text = buf.text();
        let byte_off = match buf.byte_offset(self.position.line, self.position.col) {
            Some(o) => o,
            None => return,
        };
        let bytes = text.as_bytes();
        if byte_off == 0 {
            return;
        }
        let mut i = byte_off;

        // Skip non-word chars backwards
        while i > 0 && !is_word_byte(bytes[i - 1]) {
            i -= 1;
        }
        // Skip word chars backwards
        while i > 0 && is_word_byte(bytes[i - 1]) {
            i -= 1;
        }

        if let Some((l, c)) = buf.line_col(i) {
            self.position.line = l;
            self.position.col = c;
        }
    }

    // --- Selections ---

    /// Select the word at the current cursor position.
    pub fn select_word(&mut self, buf: &TextBuffer) {
        let text = buf.text();
        let byte_off = match buf.byte_offset(self.position.line, self.position.col) {
            Some(o) => o,
            None => return,
        };
        let bytes = text.as_bytes();
        let len = bytes.len();

        if byte_off >= len || !is_word_byte(bytes[byte_off]) {
            return;
        }

        let mut start = byte_off;
        while start > 0 && is_word_byte(bytes[start - 1]) {
            start -= 1;
        }
        let mut end = byte_off;
        while end < len && is_word_byte(bytes[end]) {
            end += 1;
        }

        if let (Some((sl, sc)), Some((el, ec))) = (buf.line_col(start), buf.line_col(end)) {
            self.anchor = Some(Position::new(sl, sc));
            self.position = Position::new(el, ec);
        }
    }

    /// Select the entire line at the current cursor position.
    pub fn select_line(&mut self, buf: &TextBuffer) {
        let line = self.position.line;
        self.anchor = Some(Position::new(line, 0));
        let line_len = buf.line_len_chars(line);
        self.position = Position::new(line, line_len);
    }

    /// Select all text in the buffer.
    pub fn select_all(&mut self, buf: &TextBuffer) {
        self.anchor = Some(Position::zero());
        let last_line = if buf.line_count() > 0 {
            buf.line_count() - 1
        } else {
            0
        };
        let col = line_char_len(buf, last_line);
        self.position = Position::new(last_line, col);
    }

    pub fn set_position(&mut self, line: usize, col: usize) {
        self.position = Position::new(line, col);
        self.anchor = None;
    }
}

/// Returns the character length of a line, excluding trailing newline chars.
fn line_char_len(buf: &TextBuffer, line: usize) -> usize {
    let raw = buf.line_len_chars(line);
    let text = match buf.line(line) {
        Some(t) => t,
        None => return 0,
    };
    // Strip trailing \n and \r
    let trimmed = text.trim_end_matches(&['\n', '\r'][..]);
    let trailing = text.len() - trimmed.len();
    // trailing is byte count of newline chars; for ASCII newlines byte == char count
    if raw >= trailing {
        raw - trailing
    } else {
        0
    }
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}
