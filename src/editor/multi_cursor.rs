/// Multi-cursor editing support.
///
/// Tracks extra cursor positions beyond the primary cursor managed by egui's TextEdit.
/// All operations (insert, delete) are applied to every extra cursor, with byte offsets
/// adjusted to account for earlier edits.

/// A single cursor position within the text buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorPosition {
    /// Byte offset into the text.
    pub offset: usize,
    /// Optional selection range (start byte offset, end byte offset) where start <= end.
    pub selection: Option<(usize, usize)>,
}

impl CursorPosition {
    pub fn new(offset: usize) -> Self {
        Self {
            offset,
            selection: None,
        }
    }

    pub fn with_selection(offset: usize, start: usize, end: usize) -> Self {
        let (s, e) = if start <= end { (start, end) } else { (end, start) };
        Self {
            offset,
            selection: Some((s, e)),
        }
    }
}

/// State for multi-cursor editing.
#[derive(Debug, Clone)]
pub struct MultiCursorState {
    /// Extra cursors beyond the primary (which egui manages). Each is a byte offset.
    pub extra_cursors: Vec<CursorPosition>,
    /// Whether multi-cursor mode is active.
    pub active: bool,
    /// The text that was last selected via Ctrl+D (used to find next occurrence).
    pub last_selected_text: Option<String>,
}

impl Default for MultiCursorState {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiCursorState {
    pub fn new() -> Self {
        Self {
            extra_cursors: Vec::new(),
            active: false,
            last_selected_text: None,
        }
    }

    /// Add a new cursor at the given byte offset.
    pub fn add_cursor(&mut self, offset: usize) {
        // Don't add duplicates
        if self.extra_cursors.iter().any(|c| c.offset == offset && c.selection.is_none()) {
            return;
        }
        self.extra_cursors.push(CursorPosition::new(offset));
        self.active = true;
    }

    /// Add a cursor with a selection (used by Ctrl+D / select next occurrence).
    pub fn add_cursor_with_selection(&mut self, offset: usize, sel_start: usize, sel_end: usize) {
        self.extra_cursors.push(CursorPosition::with_selection(offset, sel_start, sel_end));
        self.active = true;
    }

    /// Find the next occurrence of `selection` in `text` starting after the last known cursor,
    /// and add a cursor with that selection.
    /// Returns true if a new occurrence was found.
    pub fn select_next_occurrence(&mut self, text: &str, selection: &str) -> bool {
        if selection.is_empty() {
            return false;
        }

        self.last_selected_text = Some(selection.to_string());

        // Determine search start: after the latest cursor/selection
        let search_start = self.extra_cursors.iter()
            .filter_map(|c| c.selection.map(|(_, e)| e).or(Some(c.offset)))
            .max()
            .unwrap_or(0);

        // Search from search_start, then wrap around
        if let Some(pos) = text[search_start..].find(selection) {
            let abs_start = search_start + pos;
            let abs_end = abs_start + selection.len();
            // Don't add if we already have a cursor with this exact selection
            if !self.extra_cursors.iter().any(|c| c.selection == Some((abs_start, abs_end))) {
                self.add_cursor_with_selection(abs_end, abs_start, abs_end);
                return true;
            }
        }

        // Wrap around: search from beginning
        if let Some(pos) = text[..search_start].find(selection) {
            let abs_end = pos + selection.len();
            if !self.extra_cursors.iter().any(|c| c.selection == Some((pos, abs_end))) {
                self.add_cursor_with_selection(abs_end, pos, abs_end);
                return true;
            }
        }

        false
    }

    /// Insert text at all extra cursor positions.
    /// Cursors are sorted by offset and processed from last to first to avoid offset invalidation.
    pub fn apply_insert(&mut self, text: &mut String, insert: &str) {
        if self.extra_cursors.is_empty() || insert.is_empty() {
            return;
        }

        // Sort cursors by offset (ascending) for processing
        self.extra_cursors.sort_by_key(|c| c.offset);

        // Process from last to first so earlier offsets remain valid
        let insert_len = insert.len();
        for i in (0..self.extra_cursors.len()).rev() {
            let cursor = &self.extra_cursors[i];
            if let Some((sel_start, sel_end)) = cursor.selection {
                // Replace the selected range
                let clamped_start = sel_start.min(text.len());
                let clamped_end = sel_end.min(text.len());
                text.replace_range(clamped_start..clamped_end, insert);
                let old_len = clamped_end - clamped_start;
                self.extra_cursors[i].offset = clamped_start + insert_len;
                self.extra_cursors[i].selection = None;
                // Adjust earlier cursors
                self.adjust_offsets_after_edit(i, clamped_start, old_len, insert_len);
            } else {
                let pos = cursor.offset.min(text.len());
                text.insert_str(pos, insert);
                self.extra_cursors[i].offset = pos + insert_len;
                self.adjust_offsets_after_edit(i, pos, 0, insert_len);
            }
        }
    }

    /// Delete `count` bytes before each extra cursor position (like Backspace).
    /// Cursors are processed from last to first.
    pub fn apply_backspace(&mut self, text: &mut String, count: usize) {
        if self.extra_cursors.is_empty() || count == 0 {
            return;
        }

        self.extra_cursors.sort_by_key(|c| c.offset);

        for i in (0..self.extra_cursors.len()).rev() {
            let cursor = &self.extra_cursors[i];
            if let Some((sel_start, sel_end)) = cursor.selection {
                // Delete the selection
                let clamped_start = sel_start.min(text.len());
                let clamped_end = sel_end.min(text.len());
                if clamped_start < clamped_end {
                    text.replace_range(clamped_start..clamped_end, "");
                    let old_len = clamped_end - clamped_start;
                    self.extra_cursors[i].offset = clamped_start;
                    self.extra_cursors[i].selection = None;
                    self.adjust_offsets_after_edit(i, clamped_start, old_len, 0);
                }
            } else {
                let pos = cursor.offset.min(text.len());
                // Find the start position for deletion (handle multi-byte chars)
                let delete_start = find_char_boundary_back(text, pos, count);
                if delete_start < pos {
                    text.replace_range(delete_start..pos, "");
                    let deleted = pos - delete_start;
                    self.extra_cursors[i].offset = delete_start;
                    self.adjust_offsets_after_edit(i, delete_start, deleted, 0);
                }
            }
        }
    }

    /// Delete `count` bytes after each extra cursor position (like Delete key).
    pub fn apply_delete(&mut self, text: &mut String, count: usize) {
        if self.extra_cursors.is_empty() || count == 0 {
            return;
        }

        self.extra_cursors.sort_by_key(|c| c.offset);

        for i in (0..self.extra_cursors.len()).rev() {
            let cursor = &self.extra_cursors[i];
            if let Some((sel_start, sel_end)) = cursor.selection {
                let clamped_start = sel_start.min(text.len());
                let clamped_end = sel_end.min(text.len());
                if clamped_start < clamped_end {
                    text.replace_range(clamped_start..clamped_end, "");
                    let old_len = clamped_end - clamped_start;
                    self.extra_cursors[i].offset = clamped_start;
                    self.extra_cursors[i].selection = None;
                    self.adjust_offsets_after_edit(i, clamped_start, old_len, 0);
                }
            } else {
                let pos = cursor.offset.min(text.len());
                let delete_end = find_char_boundary_forward(text, pos, count);
                if delete_end > pos {
                    text.replace_range(pos..delete_end, "");
                    let deleted = delete_end - pos;
                    self.adjust_offsets_after_edit(i, pos, deleted, 0);
                }
            }
        }
    }

    /// Clear all extra cursors and deactivate multi-cursor mode.
    pub fn clear(&mut self) {
        self.extra_cursors.clear();
        self.active = false;
        self.last_selected_text = None;
    }

    /// Adjust offsets of all other cursors after an edit at `edit_offset`.
    /// `old_len` bytes were replaced by `new_len` bytes.
    /// Since we process back-to-front, we need to adjust cursors at indices != edited_idx
    /// that have offsets > edit_offset.
    fn adjust_offsets_after_edit(&mut self, edited_idx: usize, edit_offset: usize, old_len: usize, new_len: usize) {
        let delta = new_len as isize - old_len as isize;
        if delta == 0 {
            return;
        }
        for j in 0..self.extra_cursors.len() {
            if j == edited_idx {
                continue;
            }
            let c = &mut self.extra_cursors[j];
            if c.offset > edit_offset {
                c.offset = (c.offset as isize + delta).max(0) as usize;
            }
            if let Some((ref mut s, ref mut e)) = c.selection {
                if *s > edit_offset {
                    *s = (*s as isize + delta).max(0) as usize;
                }
                if *e > edit_offset {
                    *e = (*e as isize + delta).max(0) as usize;
                }
            }
        }
    }

    /// Get all extra cursor byte offsets (for rendering).
    pub fn cursor_offsets(&self) -> Vec<usize> {
        self.extra_cursors.iter().map(|c| c.offset).collect()
    }

    /// Get all extra cursor selections (for rendering).
    pub fn selections(&self) -> Vec<(usize, usize)> {
        self.extra_cursors.iter().filter_map(|c| c.selection).collect()
    }
}

/// Find the byte position `count` characters before `pos`, respecting char boundaries.
fn find_char_boundary_back(text: &str, pos: usize, count: usize) -> usize {
    let mut chars_back = 0;
    let mut idx = pos;
    while chars_back < count && idx > 0 {
        idx -= 1;
        while idx > 0 && !text.is_char_boundary(idx) {
            idx -= 1;
        }
        chars_back += 1;
    }
    idx
}

/// Find the byte position `count` characters after `pos`, respecting char boundaries.
fn find_char_boundary_forward(text: &str, pos: usize, count: usize) -> usize {
    let mut chars_fwd = 0;
    let mut idx = pos;
    let len = text.len();
    while chars_fwd < count && idx < len {
        idx += 1;
        while idx < len && !text.is_char_boundary(idx) {
            idx += 1;
        }
        chars_fwd += 1;
    }
    idx
}

/// Extract the word at a given byte offset in text.
/// Returns (word, start_offset, end_offset) or None if not on a word character.
pub fn word_at_offset(text: &str, offset: usize) -> Option<(String, usize, usize)> {
    let bytes = text.as_bytes();
    if offset >= bytes.len() || !is_word_byte(bytes[offset]) {
        return None;
    }

    let mut start = offset;
    while start > 0 && is_word_byte(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = offset;
    while end < bytes.len() && is_word_byte(bytes[end]) {
        end += 1;
    }

    Some((text[start..end].to_string(), start, end))
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_cursor() {
        let mut state = MultiCursorState::new();
        assert!(!state.active);
        state.add_cursor(5);
        assert!(state.active);
        assert_eq!(state.extra_cursors.len(), 1);
        assert_eq!(state.extra_cursors[0].offset, 5);
    }

    #[test]
    fn test_no_duplicate_cursors() {
        let mut state = MultiCursorState::new();
        state.add_cursor(5);
        state.add_cursor(5);
        assert_eq!(state.extra_cursors.len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut state = MultiCursorState::new();
        state.add_cursor(5);
        state.add_cursor(10);
        state.clear();
        assert!(!state.active);
        assert!(state.extra_cursors.is_empty());
    }

    #[test]
    fn test_insert_single_cursor() {
        let mut state = MultiCursorState::new();
        let mut text = String::from("hello world");
        state.add_cursor(5);
        state.apply_insert(&mut text, "X");
        assert_eq!(text, "helloX world");
        assert_eq!(state.extra_cursors[0].offset, 6);
    }

    #[test]
    fn test_insert_multiple_cursors() {
        let mut state = MultiCursorState::new();
        let mut text = String::from("aaa bbb ccc");
        state.add_cursor(3);  // after "aaa"
        state.add_cursor(7);  // after "bbb"
        state.apply_insert(&mut text, "X");
        assert_eq!(text, "aaaX bbbX ccc");
        // After insert, cursors should be at 4 and 9
        let offsets: Vec<usize> = state.extra_cursors.iter().map(|c| c.offset).collect();
        assert_eq!(offsets, vec![4, 9]);
    }

    #[test]
    fn test_backspace_single_cursor() {
        let mut state = MultiCursorState::new();
        let mut text = String::from("hello world");
        state.add_cursor(5);
        state.apply_backspace(&mut text, 1);
        assert_eq!(text, "hell world");
        assert_eq!(state.extra_cursors[0].offset, 4);
    }

    #[test]
    fn test_backspace_multiple_cursors() {
        let mut state = MultiCursorState::new();
        let mut text = String::from("aXa bXb ccc");
        state.add_cursor(2);  // after "aX"
        state.add_cursor(6);  // after "bX"
        state.apply_backspace(&mut text, 1);
        assert_eq!(text, "aa bb ccc");
        let offsets: Vec<usize> = state.extra_cursors.iter().map(|c| c.offset).collect();
        assert_eq!(offsets, vec![1, 4]);
    }

    #[test]
    fn test_delete_single_cursor() {
        let mut state = MultiCursorState::new();
        let mut text = String::from("hello world");
        state.add_cursor(5);
        state.apply_delete(&mut text, 1);
        assert_eq!(text, "helloworld");
        assert_eq!(state.extra_cursors[0].offset, 5);
    }

    #[test]
    fn test_select_next_occurrence() {
        let mut state = MultiCursorState::new();
        let text = "foo bar foo baz foo";
        assert!(state.select_next_occurrence(text, "foo"));
        assert_eq!(state.extra_cursors.len(), 1);
        assert_eq!(state.extra_cursors[0].selection, Some((0, 3)));

        assert!(state.select_next_occurrence(text, "foo"));
        assert_eq!(state.extra_cursors.len(), 2);
        assert_eq!(state.extra_cursors[1].selection, Some((8, 11)));

        assert!(state.select_next_occurrence(text, "foo"));
        assert_eq!(state.extra_cursors.len(), 3);
        assert_eq!(state.extra_cursors[2].selection, Some((16, 19)));

        // No more occurrences
        assert!(!state.select_next_occurrence(text, "foo"));
    }

    #[test]
    fn test_insert_replaces_selection() {
        let mut state = MultiCursorState::new();
        let mut text = String::from("foo bar foo");
        state.add_cursor_with_selection(3, 0, 3);  // select first "foo"
        state.add_cursor_with_selection(11, 8, 11); // select second "foo"
        state.apply_insert(&mut text, "baz");
        assert_eq!(text, "baz bar baz");
    }

    #[test]
    fn test_word_at_offset() {
        let text = "hello world_test 123";
        assert_eq!(word_at_offset(text, 0), Some(("hello".to_string(), 0, 5)));
        assert_eq!(word_at_offset(text, 6), Some(("world_test".to_string(), 6, 16)));
        assert_eq!(word_at_offset(text, 5), None); // space
        assert_eq!(word_at_offset(text, 17), Some(("123".to_string(), 17, 20)));
    }

    #[test]
    fn test_backspace_at_start() {
        let mut state = MultiCursorState::new();
        let mut text = String::from("hello");
        state.add_cursor(0);
        state.apply_backspace(&mut text, 1);
        assert_eq!(text, "hello"); // nothing deleted
        assert_eq!(state.extra_cursors[0].offset, 0);
    }

    #[test]
    fn test_delete_at_end() {
        let mut state = MultiCursorState::new();
        let mut text = String::from("hello");
        state.add_cursor(5);
        state.apply_delete(&mut text, 1);
        assert_eq!(text, "hello"); // nothing deleted
        assert_eq!(state.extra_cursors[0].offset, 5);
    }
}
