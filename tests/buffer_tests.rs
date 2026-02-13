use notepadppp::editor::buffer::TextBuffer;

#[test]
fn create_empty() {
    let buf = TextBuffer::new();
    assert_eq!(buf.text(), "");
    assert!(buf.is_empty());
    assert!(!buf.is_modified());
}

#[test]
fn create_from_string() {
    let buf = TextBuffer::from_str("hello world");
    assert_eq!(buf.text(), "hello world");
    assert!(!buf.is_modified());
}

#[test]
fn insert_at_beginning() {
    let mut buf = TextBuffer::new();
    buf.insert(0, "hello");
    assert_eq!(buf.text(), "hello");
    assert!(buf.is_modified());
}

#[test]
fn insert_at_end() {
    let mut buf = TextBuffer::from_str("hello");
    buf.insert(5, " world");
    assert_eq!(buf.text(), "hello world");
}

#[test]
fn insert_at_middle() {
    let mut buf = TextBuffer::from_str("helo");
    buf.insert(2, "l");
    assert_eq!(buf.text(), "hello");
}

#[test]
fn delete_from_beginning() {
    let mut buf = TextBuffer::from_str("hello world");
    buf.delete(0, 6);
    assert_eq!(buf.text(), "world");
}

#[test]
fn delete_from_end() {
    let mut buf = TextBuffer::from_str("hello world");
    buf.delete(5, 11);
    assert_eq!(buf.text(), "hello");
}

#[test]
fn delete_from_middle() {
    let mut buf = TextBuffer::from_str("hello world");
    buf.delete(5, 6);
    assert_eq!(buf.text(), "helloworld");
}

#[test]
fn line_count_single_line() {
    let buf = TextBuffer::from_str("hello");
    assert_eq!(buf.line_count(), 1);
}

#[test]
fn line_count_multiple_lines() {
    let buf = TextBuffer::from_str("line1\nline2\nline3\n");
    // ropey counts the trailing empty line
    assert_eq!(buf.line_count(), 4);
}

#[test]
fn line_count_after_insert() {
    let mut buf = TextBuffer::from_str("hello");
    buf.insert(5, "\nworld");
    assert_eq!(buf.line_count(), 2);
}

#[test]
fn get_line_by_index() {
    let buf = TextBuffer::from_str("alpha\nbeta\ngamma");
    assert_eq!(buf.line(0).unwrap(), "alpha\n");
    assert_eq!(buf.line(1).unwrap(), "beta\n");
    assert_eq!(buf.line(2).unwrap(), "gamma");
    assert!(buf.line(3).is_none());
}

#[test]
fn char_at_position() {
    let buf = TextBuffer::from_str("abc");
    assert_eq!(buf.char_at(0), Some('a'));
    assert_eq!(buf.char_at(1), Some('b'));
    assert_eq!(buf.char_at(2), Some('c'));
    assert_eq!(buf.char_at(3), None);
}

#[test]
fn byte_offset_from_line_col() {
    let buf = TextBuffer::from_str("hello\nworld");
    assert_eq!(buf.byte_offset(0, 0), Some(0));
    assert_eq!(buf.byte_offset(0, 5), Some(5));
    assert_eq!(buf.byte_offset(1, 0), Some(6));
    assert_eq!(buf.byte_offset(1, 5), Some(11));
}

#[test]
fn line_col_from_byte_offset() {
    let buf = TextBuffer::from_str("hello\nworld");
    assert_eq!(buf.line_col(0), Some((0, 0)));
    assert_eq!(buf.line_col(5), Some((0, 5)));
    assert_eq!(buf.line_col(6), Some((1, 0)));
    assert_eq!(buf.line_col(11), Some((1, 5)));
}

#[test]
fn undo_single_insert() {
    let mut buf = TextBuffer::new();
    buf.insert(0, "hello");
    assert_eq!(buf.text(), "hello");
    assert!(buf.undo());
    assert_eq!(buf.text(), "");
}

#[test]
fn undo_single_delete() {
    let mut buf = TextBuffer::from_str("hello");
    buf.delete(0, 5);
    assert_eq!(buf.text(), "");
    assert!(buf.undo());
    assert_eq!(buf.text(), "hello");
}

#[test]
fn undo_multiple_operations() {
    let mut buf = TextBuffer::new();
    buf.insert(0, "aaa");
    buf.insert(3, "bbb");
    buf.insert(6, "ccc");
    assert_eq!(buf.text(), "aaabbbccc");

    assert!(buf.undo());
    assert_eq!(buf.text(), "aaabbb");

    assert!(buf.undo());
    assert_eq!(buf.text(), "aaa");

    assert!(buf.undo());
    assert_eq!(buf.text(), "");

    // no more to undo
    assert!(!buf.undo());
}

#[test]
fn redo_after_undo() {
    let mut buf = TextBuffer::new();
    buf.insert(0, "hello");
    buf.undo();
    assert_eq!(buf.text(), "");
    assert!(buf.redo());
    assert_eq!(buf.text(), "hello");
}

#[test]
fn redo_cleared_after_new_edit() {
    let mut buf = TextBuffer::new();
    buf.insert(0, "hello");
    buf.undo();
    // New edit should clear redo stack
    buf.insert(0, "world");
    assert!(!buf.redo());
    assert_eq!(buf.text(), "world");
}

#[test]
fn redo_multiple() {
    let mut buf = TextBuffer::new();
    buf.insert(0, "aaa");
    buf.insert(3, "bbb");
    buf.undo();
    buf.undo();
    assert_eq!(buf.text(), "");

    buf.redo();
    assert_eq!(buf.text(), "aaa");
    buf.redo();
    assert_eq!(buf.text(), "aaabbb");
    // nothing more to redo
    assert!(!buf.redo());
}

#[test]
fn large_text_operations() {
    let big = "x".repeat(100_000);
    let mut buf = TextBuffer::from_str(&big);
    assert_eq!(buf.len_bytes(), 100_000);
    buf.insert(50_000, "INSERTED");
    assert_eq!(buf.len_bytes(), 100_008);
    buf.delete(50_000, 50_008);
    assert_eq!(buf.len_bytes(), 100_000);
    assert_eq!(buf.text(), big);
}

#[test]
fn empty_buffer_operations() {
    let mut buf = TextBuffer::new();
    assert_eq!(buf.line_count(), 1); // ropey always has at least 1 line
    assert_eq!(buf.char_at(0), None);
    assert!(!buf.undo());
    assert!(!buf.redo());
}

#[test]
fn delete_entire_contents() {
    let mut buf = TextBuffer::from_str("hello");
    buf.delete(0, 5);
    assert_eq!(buf.text(), "");
    assert!(buf.is_empty());
}

#[test]
fn modified_flag_tracking() {
    let mut buf = TextBuffer::new();
    assert!(!buf.is_modified());
    buf.insert(0, "x");
    assert!(buf.is_modified());
    buf.set_modified(false);
    assert!(!buf.is_modified());
    buf.delete(0, 1);
    assert!(buf.is_modified());
}

#[test]
fn modified_stays_true_after_undo() {
    let mut buf = TextBuffer::new();
    buf.insert(0, "x");
    buf.set_modified(false);
    buf.undo();
    // undo itself marks as modified
    assert!(buf.is_modified());
}

#[test]
fn byte_offset_out_of_range() {
    let buf = TextBuffer::from_str("hi");
    assert!(buf.byte_offset(5, 0).is_none());
}

#[test]
fn line_col_out_of_range() {
    let buf = TextBuffer::from_str("hi");
    assert!(buf.line_col(100).is_none());
}

#[test]
fn insert_and_delete_unicode() {
    let mut buf = TextBuffer::new();
    buf.insert(0, "café");
    assert_eq!(buf.text(), "café");
    assert_eq!(buf.len_chars(), 4);
    // 'é' is 2 bytes in UTF-8, total = 5 bytes
    assert_eq!(buf.len_bytes(), 5);
    // Delete the 'é' (bytes 3..5)
    buf.delete(3, 5);
    assert_eq!(buf.text(), "caf");
}
