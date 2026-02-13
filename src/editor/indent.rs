/// Smart indentation utilities

/// Get the leading whitespace of a line
pub fn get_indent(line: &str) -> &str {
    let trimmed = line.trim_start();
    &line[..line.len() - trimmed.len()]
}

/// Compute the indent for a new line based on the previous line.
/// If the previous line ends with `{`, `(`, `[`, or `:` increase indent.
pub fn compute_indent(prev_line: &str, tab_str: &str) -> String {
    let base = get_indent(prev_line);
    let trimmed = prev_line.trim_end();

    if trimmed.ends_with('{')
        || trimmed.ends_with('(')
        || trimmed.ends_with('[')
        || trimmed.ends_with(':')
    {
        format!("{}{}", base, tab_str)
    } else {
        base.to_string()
    }
}

/// Re-indent pasted text to match the current indentation level.
/// Takes the pasted text and the indent at the cursor position.
pub fn reindent_paste(text: &str, target_indent: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return text.to_string();
    }

    // Find the common indent of the pasted block (from line 2 onwards, or line 1 if only 1 line)
    let source_indent = if lines.len() > 1 {
        lines[1..]
            .iter()
            .filter(|l| !l.trim().is_empty())
            .map(|l| get_indent(l))
            .min_by_key(|i| i.len())
            .unwrap_or("")
    } else {
        get_indent(lines[0])
    };

    let mut result = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            // First line keeps as-is (it's at cursor position)
            result.push(line.to_string());
        } else if line.trim().is_empty() {
            result.push(String::new());
        } else if line.starts_with(source_indent) {
            // Re-indent: replace source indent with target indent
            result.push(format!("{}{}", target_indent, &line[source_indent.len()..]));
        } else {
            result.push(line.to_string());
        }
    }

    // Preserve trailing newline if original had one
    let joined = result.join("\n");
    if text.ends_with('\n') {
        format!("{}\n", joined)
    } else {
        joined
    }
}
