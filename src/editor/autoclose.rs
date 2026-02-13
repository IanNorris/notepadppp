/// Given a character that was just typed, return its closing pair if applicable
pub fn closing_pair(ch: char) -> Option<char> {
    match ch {
        '(' => Some(')'),
        '[' => Some(']'),
        '{' => Some('}'),
        '"' => Some('"'),
        '\'' => Some('\''),
        '`' => Some('`'),
        _ => None,
    }
}

/// Check if a character is an opening bracket/quote that should auto-close
pub fn should_autoclose(ch: char, next_char: Option<char>) -> bool {
    if closing_pair(ch).is_none() {
        return false;
    }
    // Only auto-close if next char is whitespace, closing bracket, or end of line
    match next_char {
        None => true,  // End of line/file
        Some(c) => c.is_whitespace() || c == ')' || c == ']' || c == '}' || c == ',' || c == ';',
    }
}
