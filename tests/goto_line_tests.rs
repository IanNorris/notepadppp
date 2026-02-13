use notepadppp::editor::buffer::TextBuffer;
use notepadppp::editor::cursor::CursorState;

#[test]
fn test_goto_line_basic() {
    let _buf = TextBuffer::from_str("line one\nline two\nline three\nline four\n");
    let mut cursor = CursorState::new();

    // Go to line 3 (0-indexed: 2)
    cursor.set_position(2, 0);
    assert_eq!(cursor.position.line, 2);
    assert_eq!(cursor.position.col, 0);
}

#[test]
fn test_goto_line_clamped() {
    let buf = TextBuffer::from_str("line one\nline two\n");
    let mut cursor = CursorState::new();

    // Attempt to go beyond last line, should be clamped by the UI logic
    let line_count = buf.line_count();
    let target = 100_usize.saturating_sub(1).min(line_count.saturating_sub(1));
    cursor.set_position(target, 0);
    assert!(cursor.position.line < line_count);
}

#[test]
fn test_goto_line_first() {
    let buf = TextBuffer::from_str("a\nb\nc\n");
    let mut cursor = CursorState::new();
    cursor.set_position(5, 3); // some position

    // Go to line 1 (0-indexed: 0)
    let target = 1_usize.saturating_sub(1).min(buf.line_count().saturating_sub(1));
    cursor.set_position(target, 0);
    assert_eq!(cursor.position.line, 0);
    assert_eq!(cursor.position.col, 0);
}
