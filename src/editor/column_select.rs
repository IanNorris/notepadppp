/// Column (rectangular) selection support.
///
/// Tracks a rectangular region defined by start/end row and column.
/// All row/col values are 0-indexed.

#[derive(Debug, Clone)]
pub struct ColumnSelection {
    pub active: bool,
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
}

impl Default for ColumnSelection {
    fn default() -> Self {
        Self::new()
    }
}

impl ColumnSelection {
    pub fn new() -> Self {
        Self {
            active: false,
            start_row: 0,
            start_col: 0,
            end_row: 0,
            end_col: 0,
        }
    }

    /// Begin a column selection at the given row/col.
    pub fn start(&mut self, row: usize, col: usize) {
        self.active = true;
        self.start_row = row;
        self.start_col = col;
        self.end_row = row;
        self.end_col = col;
    }

    /// Extend the column selection to the given row/col.
    pub fn extend(&mut self, row: usize, col: usize) {
        self.end_row = row;
        self.end_col = col;
    }

    /// Clear the column selection.
    pub fn clear(&mut self) {
        self.active = false;
        self.start_row = 0;
        self.start_col = 0;
        self.end_row = 0;
        self.end_col = 0;
    }

    /// Returns (min_row, max_row) of the selection rectangle.
    pub fn rows(&self) -> (usize, usize) {
        let min = self.start_row.min(self.end_row);
        let max = self.start_row.max(self.end_row);
        (min, max)
    }

    /// Returns (min_col, max_col) of the selection rectangle.
    pub fn cols(&self) -> (usize, usize) {
        let min = self.start_col.min(self.end_col);
        let max = self.start_col.max(self.end_col);
        (min, max)
    }

    /// Extract the selected rectangular text. Each row's selected portion is
    /// joined with newlines.
    pub fn extract_text(&self, text: &str) -> String {
        if !self.active {
            return String::new();
        }
        let (min_row, max_row) = self.rows();
        let (min_col, max_col) = self.cols();
        let lines: Vec<&str> = text.lines().collect();
        let mut result = Vec::new();
        for row in min_row..=max_row {
            if row < lines.len() {
                let chars: Vec<char> = lines[row].chars().collect();
                let start = min_col.min(chars.len());
                let end = max_col.min(chars.len());
                let selected: String = chars[start..end].iter().collect();
                result.push(selected);
            } else {
                result.push(String::new());
            }
        }
        result.join("\n")
    }

    /// Delete the selected rectangular region from the text and return the new text.
    pub fn delete_selection(&self, text: &str) -> String {
        if !self.active {
            return text.to_string();
        }
        let (min_row, max_row) = self.rows();
        let (min_col, max_col) = self.cols();
        let lines: Vec<&str> = text.lines().collect();
        let trailing_newline = text.ends_with('\n');
        let mut result = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if i >= min_row && i <= max_row {
                let chars: Vec<char> = line.chars().collect();
                let start = min_col.min(chars.len());
                let end = max_col.min(chars.len());
                let mut new_line: String = chars[..start].iter().collect();
                new_line.extend(chars[end..].iter());
                result.push(new_line);
            } else {
                result.push(line.to_string());
            }
        }
        let mut out = result.join("\n");
        if trailing_newline {
            out.push('\n');
        }
        out
    }

    /// Insert text at each row in the column selection. Each line of `insert`
    /// is placed at the corresponding row; if `insert` is a single line, it is
    /// repeated for every selected row. Inserts at `min_col` after deleting
    /// the selected rectangle.
    pub fn insert_at_selection(&self, text: &str, insert: &str) -> String {
        if !self.active {
            return text.to_string();
        }
        let (min_row, max_row) = self.rows();
        let (min_col, max_col) = self.cols();
        let lines: Vec<&str> = text.lines().collect();
        let trailing_newline = text.ends_with('\n');
        let insert_lines: Vec<&str> = insert.lines().collect();
        let mut result = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if i >= min_row && i <= max_row {
                let chars: Vec<char> = line.chars().collect();
                let start = min_col.min(chars.len());
                let end = max_col.min(chars.len());
                let row_offset = i - min_row;
                let ins = if insert_lines.len() == 1 {
                    insert_lines[0]
                } else if row_offset < insert_lines.len() {
                    insert_lines[row_offset]
                } else {
                    ""
                };
                let mut new_line: String = chars[..start].iter().collect();
                new_line.push_str(ins);
                new_line.extend(chars[end..].iter());
                result.push(new_line);
            } else {
                result.push(line.to_string());
            }
        }
        let mut out = result.join("\n");
        if trailing_newline {
            out.push('\n');
        }
        out
    }
}
