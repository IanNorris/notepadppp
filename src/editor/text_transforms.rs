use crate::editor::document::{Encoding, LineEnding};
use crate::tools::diff_tool::DiffResult;

/// Convert text to TITLE CASE (capitalize after whitespace/hyphen/underscore)
pub fn title_case(s: &str) -> String {
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
}

/// Convert text to Sentence case (capitalize after . ! ?)
pub fn sentence_case(s: &str) -> String {
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
}

/// Invert case of each character
pub fn inverse_case(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_uppercase() {
                c.to_lowercase().to_string()
            } else {
                c.to_uppercase().to_string()
            }
        })
        .collect()
}

/// Toggle line comment for a specific line
/// Returns the modified line
pub fn toggle_line_comment(line: &str, comment_prefix: &str) -> String {
    let prefix_space = format!("{} ", comment_prefix);
    let trimmed = line.trim_start();
    if trimmed.starts_with(&prefix_space) {
        let indent = line.len() - trimmed.len();
        let rest = &trimmed[prefix_space.len()..];
        format!("{}{}", &line[..indent], rest)
    } else if trimmed.starts_with(comment_prefix) {
        let indent = line.len() - trimmed.len();
        let rest = &trimmed[comment_prefix.len()..];
        format!("{}{}", &line[..indent], rest)
    } else {
        let indent = line.len() - trimmed.len();
        format!("{}{} {}", &line[..indent], comment_prefix, trimmed)
    }
}

/// Format diff result lines with prefix markers
pub fn format_diff_output(diff_result: &DiffResult) -> String {
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
    diff_text
}

/// Parse goto-line input: returns 0-indexed target line, or None
pub fn parse_goto_line(input: &str) -> Option<usize> {
    input.trim().parse::<usize>().ok().and_then(|n| if n > 0 { Some(n - 1) } else { None })
}

/// Clamp font size for zoom operations
pub fn clamp_zoom(current: f32, delta: f32) -> f32 {
    (current + delta).clamp(6.0, 72.0)
}

/// Clamp font size for preferences
pub fn clamp_pref_font_size(current: f32, delta: f32) -> f32 {
    (current + delta).clamp(8.0, 48.0)
}

/// Format the window title
pub fn format_title(tab_title: &str, is_modified: bool) -> String {
    let modified = if is_modified { " •" } else { "" };
    format!("{}{} — Notepad+++", tab_title, modified)
}

/// Format encoding for display
pub fn encoding_display_name(encoding: &Encoding) -> &'static str {
    match encoding {
        Encoding::UTF8 => "UTF-8",
        Encoding::UTF8BOM => "UTF-8 BOM",
        Encoding::UTF16LE => "UTF-16 LE",
        Encoding::UTF16BE => "UTF-16 BE",
        Encoding::ASCII => "ASCII",
    }
}

/// Format line ending for display
pub fn line_ending_display_name(le: &LineEnding) -> &'static str {
    match le {
        LineEnding::LF => "LF",
        LineEnding::CRLF => "CRLF",
        LineEnding::CR => "CR",
    }
}

/// Format status bar text
pub fn format_status_bar(line: usize, col: usize, encoding: &str, line_ending: &str, language: &str, show_whitespace: bool) -> String {
    format!(
        "  Ln {}, Col {}    {}    {}    {}{}",
        line, col, encoding, line_ending, language,
        if show_whitespace { "    WS" } else { "" }
    )
}

/// Navigate to next match index (wrapping)
pub fn next_match_index(current: Option<usize>, total: usize) -> Option<usize> {
    if total == 0 { return None; }
    Some(match current {
        Some(i) => (i + 1) % total,
        None => 0,
    })
}

/// Navigate to previous match index (wrapping)
pub fn prev_match_index(current: Option<usize>, total: usize) -> Option<usize> {
    if total == 0 { return None; }
    Some(match current {
        Some(0) | None => total - 1,
        Some(i) => i - 1,
    })
}
