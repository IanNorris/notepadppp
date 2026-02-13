use notepadppp::editor::buffer::TextBuffer;
use notepadppp::editor::cursor::{CursorState, Position};

#[test]
fn initial_position() {
    let cursor = CursorState::new();
    assert_eq!(cursor.position, Position::zero());
    assert!(!cursor.has_selection());
}

#[test]
fn move_right_basic() {
    let buf = TextBuffer::from_str("hello");
    let mut cur = CursorState::new();
    cur.move_right(&buf);
    assert_eq!(cur.position, Position::new(0, 1));
}

#[test]
fn move_left_basic() {
    let buf = TextBuffer::from_str("hello");
    let mut cur = CursorState::new();
    cur.set_position(0, 3);
    cur.move_left(&buf);
    assert_eq!(cur.position, Position::new(0, 2));
}

#[test]
fn move_left_at_start_of_line_wraps() {
    let buf = TextBuffer::from_str("ab\ncd");
    let mut cur = CursorState::new();
    cur.set_position(1, 0);
    cur.move_left(&buf);
    assert_eq!(cur.position, Position::new(0, 2));
}

#[test]
fn move_right_at_end_of_line_wraps() {
    let buf = TextBuffer::from_str("ab\ncd");
    let mut cur = CursorState::new();
    // line 0 has "ab\n", visible chars = 2, but the \n is at col 2
    cur.set_position(0, 2);
    // After the visible chars, next step should be col 3 (\n position) if line has \n
    // Actually line_char_len strips trailing \n, so col 2 is the end.
    // move_right: col < line_len(raw - trailing) => if col(2) < line_len(2) is false
    // So it wraps to next line
    cur.move_right(&buf);
    assert_eq!(cur.position, Position::new(1, 0));
}

#[test]
fn move_left_at_buffer_start() {
    let buf = TextBuffer::from_str("hello");
    let mut cur = CursorState::new();
    cur.move_left(&buf);
    assert_eq!(cur.position, Position::zero());
}

#[test]
fn move_right_at_buffer_end() {
    let buf = TextBuffer::from_str("hi");
    let mut cur = CursorState::new();
    cur.set_position(0, 2);
    cur.move_right(&buf);
    // Should stay at (0, 2), no more lines
    assert_eq!(cur.position, Position::new(0, 2));
}

#[test]
fn move_up() {
    let buf = TextBuffer::from_str("aaa\nbbb\nccc");
    let mut cur = CursorState::new();
    cur.set_position(2, 1);
    cur.move_up(&buf);
    assert_eq!(cur.position, Position::new(1, 1));
}

#[test]
fn move_up_at_top() {
    let buf = TextBuffer::from_str("hello");
    let mut cur = CursorState::new();
    cur.move_up(&buf);
    assert_eq!(cur.position, Position::zero());
}

#[test]
fn move_down() {
    let buf = TextBuffer::from_str("aaa\nbbb\nccc");
    let mut cur = CursorState::new();
    cur.move_down(&buf);
    assert_eq!(cur.position, Position::new(1, 0));
}

#[test]
fn move_down_at_bottom() {
    let buf = TextBuffer::from_str("hello");
    let mut cur = CursorState::new();
    cur.move_down(&buf);
    // only one line, stays
    assert_eq!(cur.position, Position::new(0, 0));
}

#[test]
fn move_down_clamps_col() {
    let buf = TextBuffer::from_str("longline\nab");
    let mut cur = CursorState::new();
    cur.set_position(0, 8);
    cur.move_down(&buf);
    // line 1 has 2 chars, col should clamp to 2
    assert_eq!(cur.position, Position::new(1, 2));
}

#[test]
fn move_home() {
    let buf = TextBuffer::from_str("hello");
    let mut cur = CursorState::new();
    let _ = &buf;
    cur.set_position(0, 3);
    cur.move_home();
    assert_eq!(cur.position, Position::new(0, 0));
}

#[test]
fn move_end() {
    let buf = TextBuffer::from_str("hello\nworld");
    let mut cur = CursorState::new();
    cur.move_end(&buf);
    assert_eq!(cur.position, Position::new(0, 5));
}

#[test]
fn move_word_right() {
    let buf = TextBuffer::from_str("hello world foo");
    let mut cur = CursorState::new();
    cur.move_word_right(&buf);
    assert_eq!(cur.position, Position::new(0, 6)); // start of "world"
    cur.move_word_right(&buf);
    assert_eq!(cur.position, Position::new(0, 12)); // start of "foo"
}

#[test]
fn move_word_left() {
    let buf = TextBuffer::from_str("hello world");
    let mut cur = CursorState::new();
    cur.set_position(0, 11);
    cur.move_word_left(&buf);
    assert_eq!(cur.position, Position::new(0, 6)); // start of "world"
    cur.move_word_left(&buf);
    assert_eq!(cur.position, Position::new(0, 0)); // start of "hello"
}

#[test]
fn selection_creation() {
    let mut cur = CursorState::new();
    cur.set_position(0, 2);
    cur.start_selection();
    assert!(cur.has_selection());
    assert_eq!(cur.anchor, Some(Position::new(0, 2)));
}

#[test]
fn selected_range_ordered() {
    let mut cur = CursorState::new();
    // anchor at (1, 5), cursor at (0, 2) → start should be (0,2)
    cur.anchor = Some(Position::new(1, 5));
    cur.position = Position::new(0, 2);
    let (start, end) = cur.selected_range().unwrap();
    assert_eq!(start, Position::new(0, 2));
    assert_eq!(end, Position::new(1, 5));
}

#[test]
fn clear_selection() {
    let mut cur = CursorState::new();
    cur.start_selection();
    cur.clear_selection();
    assert!(!cur.has_selection());
}

#[test]
fn move_clears_selection() {
    let buf = TextBuffer::from_str("hello");
    let mut cur = CursorState::new();
    cur.start_selection();
    cur.move_right(&buf);
    assert!(!cur.has_selection());
}

#[test]
fn select_word() {
    let buf = TextBuffer::from_str("hello world");
    let mut cur = CursorState::new();
    cur.set_position(0, 1); // inside "hello"
    cur.select_word(&buf);
    let (start, end) = cur.selected_range().unwrap();
    assert_eq!(start, Position::new(0, 0));
    assert_eq!(end, Position::new(0, 5));
}

#[test]
fn select_word_at_second_word() {
    let buf = TextBuffer::from_str("hello world");
    let mut cur = CursorState::new();
    cur.set_position(0, 7); // inside "world"
    cur.select_word(&buf);
    let (start, end) = cur.selected_range().unwrap();
    assert_eq!(start, Position::new(0, 6));
    assert_eq!(end, Position::new(0, 11));
}

#[test]
fn select_line() {
    let buf = TextBuffer::from_str("aaa\nbbb\nccc");
    let mut cur = CursorState::new();
    cur.set_position(1, 1);
    cur.select_line(&buf);
    let (start, end) = cur.selected_range().unwrap();
    assert_eq!(start, Position::new(1, 0));
    // line 1 is "bbb\n", line_len_chars=4 (includes \n)
    assert_eq!(end.line, 1);
}

#[test]
fn select_all() {
    let buf = TextBuffer::from_str("aaa\nbbb\nccc");
    let mut cur = CursorState::new();
    cur.select_all(&buf);
    let (start, end) = cur.selected_range().unwrap();
    assert_eq!(start, Position::zero());
    assert_eq!(end, Position::new(2, 3)); // last line "ccc" has 3 chars
}

#[test]
fn select_all_empty() {
    let buf = TextBuffer::new();
    let mut cur = CursorState::new();
    cur.select_all(&buf);
    let (start, end) = cur.selected_range().unwrap();
    assert_eq!(start, Position::zero());
    assert_eq!(end, Position::zero());
}
