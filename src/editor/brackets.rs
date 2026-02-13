/// Bracket matching utilities
const OPEN_BRACKETS: &[char] = &['(', '[', '{'];
const CLOSE_BRACKETS: &[char] = &[')', ']', '}'];

/// Given a byte position in text, find the matching bracket position.
/// Returns None if the character at `pos` is not a bracket, or if no match is found.
pub fn find_matching_bracket(text: &str, byte_pos: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if byte_pos >= bytes.len() {
        return None;
    }
    let ch = text[byte_pos..].chars().next()?;

    if let Some(idx) = OPEN_BRACKETS.iter().position(|&b| b == ch) {
        // Search forward for matching close bracket
        find_forward(text, byte_pos, OPEN_BRACKETS[idx], CLOSE_BRACKETS[idx])
    } else if let Some(idx) = CLOSE_BRACKETS.iter().position(|&b| b == ch) {
        // Search backward for matching open bracket
        find_backward(text, byte_pos, OPEN_BRACKETS[idx], CLOSE_BRACKETS[idx])
    } else {
        None
    }
}

fn find_forward(text: &str, start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0i32;
    for (i, ch) in text[start..].char_indices() {
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Some(start + i);
            }
        }
    }
    None
}

fn find_backward(text: &str, start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0i32;
    for (i, ch) in text[..=start].char_indices().rev() {
        if ch == close {
            depth += 1;
        } else if ch == open {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

/// Check if a character at the given byte position is a bracket
pub fn is_bracket(text: &str, byte_pos: usize) -> bool {
    if byte_pos >= text.len() {
        return false;
    }
    if let Some(ch) = text[byte_pos..].chars().next() {
        OPEN_BRACKETS.contains(&ch) || CLOSE_BRACKETS.contains(&ch)
    } else {
        false
    }
}
