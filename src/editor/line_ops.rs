use std::collections::HashSet;

/// Duplicate the line at cursor_line
pub fn duplicate_line(lines: &mut Vec<String>, cursor_line: usize) {
    if cursor_line < lines.len() {
        let dup = lines[cursor_line].clone();
        lines.insert(cursor_line + 1, dup);
    }
}

/// Delete the line at cursor_line
pub fn delete_line(lines: &mut Vec<String>, cursor_line: usize) {
    if cursor_line < lines.len() && lines.len() > 1 {
        lines.remove(cursor_line);
    } else if lines.len() == 1 {
        lines[0] = String::new();
    }
}

/// Move line at cursor_line up by one
pub fn move_line_up(lines: &mut Vec<String>, cursor_line: usize) {
    if cursor_line > 0 && cursor_line < lines.len() {
        lines.swap(cursor_line, cursor_line - 1);
    }
}

/// Move line at cursor_line down by one
pub fn move_line_down(lines: &mut Vec<String>, cursor_line: usize) {
    if cursor_line + 1 < lines.len() {
        lines.swap(cursor_line, cursor_line + 1);
    }
}

/// Sort lines ascending (alphabetical)
pub fn sort_asc(lines: &mut Vec<String>) {
    lines.sort();
}

/// Sort lines descending
pub fn sort_desc(lines: &mut Vec<String>) {
    lines.sort();
    lines.reverse();
}

/// Sort lines case-insensitively
pub fn sort_case_insensitive(lines: &mut Vec<String>) {
    lines.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
}

/// Sort lines numerically
pub fn sort_numeric(lines: &mut Vec<String>) {
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
}

/// Remove empty/whitespace-only lines
pub fn remove_empty(lines: &mut Vec<String>) {
    lines.retain(|l| !l.trim().is_empty());
    if lines.is_empty() {
        lines.push(String::new());
    }
}

/// Remove duplicate lines, keeping first occurrence
pub fn remove_duplicates(lines: &mut Vec<String>) {
    let mut seen = HashSet::new();
    lines.retain(|l| seen.insert(l.clone()));
    if lines.is_empty() {
        lines.push(String::new());
    }
}

/// Trim trailing whitespace from all lines
pub fn trim_trailing(lines: &mut Vec<String>) {
    for line in lines.iter_mut() {
        *line = line.trim_end().to_string();
    }
}

/// Trim leading whitespace from all lines
pub fn trim_leading(lines: &mut Vec<String>) {
    for line in lines.iter_mut() {
        *line = line.trim_start().to_string();
    }
}

/// Trim both leading and trailing whitespace
pub fn trim_both(lines: &mut Vec<String>) {
    for line in lines.iter_mut() {
        *line = line.trim().to_string();
    }
}

/// Join current line with next line
pub fn join_lines(lines: &mut Vec<String>, cursor_line: usize) {
    if cursor_line < lines.len().saturating_sub(1) {
        let next = lines.remove(cursor_line + 1);
        lines[cursor_line].push_str(&next);
    }
}

/// Split current line at midpoint
pub fn split_line(lines: &mut Vec<String>, cursor_line: usize) {
    if cursor_line < lines.len() {
        let current = lines[cursor_line].clone();
        let mid = current.len() / 2;
        lines[cursor_line] = current[..mid].to_string();
        lines.insert(cursor_line + 1, current[mid..].to_string());
    }
}

/// Insert empty line above cursor
pub fn insert_above(lines: &mut Vec<String>, cursor_line: usize) {
    lines.insert(cursor_line, String::new());
}

/// Insert empty line below cursor
pub fn insert_below(lines: &mut Vec<String>, cursor_line: usize) {
    lines.insert(cursor_line + 1, String::new());
}

/// Reverse all lines
pub fn reverse_lines(lines: &mut Vec<String>) {
    lines.reverse();
}
