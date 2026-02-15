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

// =====================================================
// Additional coverage tests
// =====================================================

#[test]
fn from_reader_basic() {
    use std::io::Cursor;
    let data = "hello from reader";
    let cursor = Cursor::new(data.as_bytes());
    let buf = TextBuffer::from_reader(cursor).unwrap();
    assert_eq!(buf.text(), "hello from reader");
    assert!(!buf.is_modified());
}

#[test]
fn from_reader_multiline() {
    use std::io::Cursor;
    let data = "line1\nline2\nline3";
    let cursor = Cursor::new(data.as_bytes());
    let buf = TextBuffer::from_reader(cursor).unwrap();
    assert_eq!(buf.text(), "line1\nline2\nline3");
    assert_eq!(buf.line_count(), 3);
}

#[test]
fn from_reader_empty() {
    use std::io::Cursor;
    let cursor = Cursor::new(b"" as &[u8]);
    let buf = TextBuffer::from_reader(cursor).unwrap();
    assert_eq!(buf.text(), "");
    assert!(buf.is_empty());
}

#[test]
fn set_undo_limit_caps_stack() {
    let mut buf = TextBuffer::new();
    buf.insert(0, "a");
    buf.insert(1, "b");
    buf.insert(2, "c");
    buf.insert(3, "d");
    buf.insert(4, "e");
    // 5 edits on undo stack now
    buf.set_undo_limit(Some(2));
    // Only last 2 should remain
    assert!(buf.undo()); // undo "e"
    assert!(buf.undo()); // undo "d"
    assert!(!buf.undo()); // stack should be empty now
}

#[test]
fn set_undo_limit_enforced_on_insert() {
    let mut buf = TextBuffer::new();
    buf.set_undo_limit(Some(3));
    for i in 0..10 {
        buf.insert(i, "x");
    }
    // Only 3 undos should be possible
    let mut count = 0;
    while buf.undo() {
        count += 1;
    }
    assert_eq!(count, 3);
}

#[test]
fn set_undo_limit_none_unlimited() {
    let mut buf = TextBuffer::new();
    buf.set_undo_limit(Some(2));
    buf.set_undo_limit(None);
    for i in 0..10 {
        buf.insert(i, "x");
    }
    let mut count = 0;
    while buf.undo() {
        count += 1;
    }
    assert_eq!(count, 10);
}

#[test]
fn line_len_chars_basic() {
    let buf = TextBuffer::from_str("hello\nworld");
    assert!(buf.line_len_chars(0) > 0);
    assert!(buf.line_len_chars(1) > 0);
}

#[test]
fn line_len_chars_unicode_line() {
    let buf = TextBuffer::from_str("café\nnormal");
    // "café\n" has 5 chars (c, a, f, é, \n)
    assert_eq!(buf.line_len_chars(0), 5);
    assert_eq!(buf.line_len_chars(1), 6); // "normal" = 6 chars
}

#[test]
fn line_len_chars_out_of_bounds() {
    let buf = TextBuffer::from_str("hello");
    assert_eq!(buf.line_len_chars(100), 0);
}

#[test]
fn from_str_empty() {
    let buf = TextBuffer::from_str("");
    assert!(buf.is_empty());
    assert_eq!(buf.len_chars(), 0);
    assert_eq!(buf.len_bytes(), 0);
    assert_eq!(buf.text(), "");
}

#[test]
fn insert_empty_string() {
    let mut buf = TextBuffer::from_str("hello");
    buf.insert(0, "");
    assert_eq!(buf.text(), "hello");
    assert!(buf.is_modified()); // insert still records an operation
}

#[test]
fn delete_zero_range() {
    let mut buf = TextBuffer::from_str("hello");
    buf.delete(0, 0);
    assert_eq!(buf.text(), "hello");
}

#[test]
fn line_zero_on_empty() {
    let buf = TextBuffer::from_str("");
    // Ropey always has at least 1 line
    let line = buf.line(0);
    assert!(line.is_some());
    assert_eq!(line.unwrap(), "");
}

#[test]
fn char_at_zero_on_empty() {
    let buf = TextBuffer::from_str("");
    assert_eq!(buf.char_at(0), None);
}

#[test]
fn len_chars_on_empty() {
    let buf = TextBuffer::from_str("");
    assert_eq!(buf.len_chars(), 0);
}

#[test]
fn from_str_multi_byte_unicode() {
    let buf = TextBuffer::from_str("😀café");
    // 😀 = 4 bytes, c=1, a=1, f=1, é=2 → 9 bytes
    assert_eq!(buf.len_bytes(), 9);
    // 😀 = 1 char, café = 4 chars → 5 chars
    assert_eq!(buf.len_chars(), 5);
}

#[test]
fn char_at_multi_byte() {
    let buf = TextBuffer::from_str("😀café");
    assert_eq!(buf.char_at(0), Some('😀'));
    // after 😀 (4 bytes), 'c' at byte 4
    assert_eq!(buf.char_at(4), Some('c'));
}

#[test]
fn len_chars_emoji() {
    let buf = TextBuffer::from_str("🎉🎊🎈");
    assert_eq!(buf.len_chars(), 3);
    assert_eq!(buf.len_bytes(), 12); // 3 * 4 bytes
}

#[test]
fn from_str_crlf_line_endings() {
    let buf = TextBuffer::from_str("line1\r\nline2");
    assert_eq!(buf.text(), "line1\r\nline2");
    assert_eq!(buf.line_count(), 2);
}

#[test]
fn undo_with_unicode() {
    let mut buf = TextBuffer::new();
    buf.insert(0, "café");
    assert_eq!(buf.text(), "café");
    buf.undo();
    assert_eq!(buf.text(), "");
}

#[test]
fn undo_unicode_delete() {
    let mut buf = TextBuffer::from_str("héllo");
    // Delete é (bytes 1..3)
    buf.delete(1, 3);
    assert_eq!(buf.text(), "hllo");
    buf.undo();
    assert_eq!(buf.text(), "héllo");
}

#[test]
fn redo_delete_operation() {
    let mut buf = TextBuffer::from_str("hello world");
    buf.delete(5, 11);
    assert_eq!(buf.text(), "hello");
    buf.undo();
    assert_eq!(buf.text(), "hello world");
    buf.redo();
    assert_eq!(buf.text(), "hello");
}

#[test]
fn redo_multiple_deletes() {
    let mut buf = TextBuffer::from_str("abcdef");
    buf.delete(0, 2); // "cdef"
    buf.delete(0, 2); // "ef"
    buf.undo();
    buf.undo();
    assert_eq!(buf.text(), "abcdef");
    buf.redo();
    assert_eq!(buf.text(), "cdef");
    buf.redo();
    assert_eq!(buf.text(), "ef");
}

#[test]
fn byte_offset_with_unicode_content() {
    let buf = TextBuffer::from_str("café\nworld");
    // line 0: c(1) a(1) f(1) é(2) \n(1) = 6 bytes
    // byte_offset(1, 0) should be 6
    assert_eq!(buf.byte_offset(1, 0), Some(6));
    // col 2 on line 0 → byte 2
    assert_eq!(buf.byte_offset(0, 2), Some(2));
}

#[test]
fn byte_offset_column_beyond_line_length() {
    let buf = TextBuffer::from_str("hi\nworld");
    // line 0 = "hi\n", col beyond line length should be clamped
    let result = buf.byte_offset(0, 100);
    assert!(result.is_some());
    // Clamped to end of line including newline
}

#[test]
fn line_col_at_boundary_len_bytes() {
    let buf = TextBuffer::from_str("hello");
    // byte_pos == len_bytes (5) should be valid (end of buffer)
    let result = buf.line_col(5);
    assert!(result.is_some());
    assert_eq!(result, Some((0, 5)));
}

#[test]
fn line_col_at_boundary_multiline() {
    let buf = TextBuffer::from_str("ab\ncd");
    // len_bytes = 5, should return (1, 2)
    assert_eq!(buf.line_col(5), Some((1, 2)));
}

// ── move_text / copy_text_to ──

#[test]
fn move_text_forward() {
    let mut buf = TextBuffer::from_str("abcdef");
    buf.move_text(0, 2, 4); // move "ab" to position 4
    assert_eq!(buf.text(), "cdabef");
}

#[test]
fn move_text_backward() {
    let mut buf = TextBuffer::from_str("abcdef");
    buf.move_text(4, 6, 0); // move "ef" to position 0
    assert_eq!(buf.text(), "efabcd");
}

#[test]
fn move_text_within_range_noop() {
    let mut buf = TextBuffer::from_str("abcdef");
    buf.move_text(1, 4, 2); // dest inside [1, 4) => no-op
    assert_eq!(buf.text(), "abcdef");
}

#[test]
fn move_text_same_position() {
    let mut buf = TextBuffer::from_str("abcdef");
    buf.move_text(2, 4, 2); // dest == start => no-op
    assert_eq!(buf.text(), "abcdef");
}

#[test]
fn move_text_out_of_bounds() {
    let mut buf = TextBuffer::from_str("abc");
    buf.move_text(0, 10, 0); // end > len => no-op
    assert_eq!(buf.text(), "abc");
}

#[test]
fn copy_text_to_position() {
    let mut buf = TextBuffer::from_str("abcdef");
    buf.copy_text_to(0, 2, 4); // copy "ab" to position 4
    assert_eq!(buf.text(), "abcdabef");
}

#[test]
fn copy_text_to_start() {
    let mut buf = TextBuffer::from_str("abcdef");
    buf.copy_text_to(4, 6, 0); // copy "ef" to position 0
    assert_eq!(buf.text(), "efabcdef");
}

#[test]
fn copy_text_to_out_of_bounds() {
    let mut buf = TextBuffer::from_str("abc");
    buf.copy_text_to(0, 10, 0); // end > len => no-op
    assert_eq!(buf.text(), "abc");
}

// ── Tab reorder with move_tab ──

#[test]
fn move_tab_active_follows_non_active() {
    use notepadppp::editor::tab_manager::TabManager;
    let mut mgr = TabManager::new();
    mgr.new_tab();
    mgr.new_tab();
    // 3 tabs: 0, 1, 2. Active is 2.
    mgr.set_active(1);
    // Move tab 0 to position 2
    mgr.move_tab(0, 2);
    // Active was 1, which should shift to 0 (since tab before it was removed)
    assert_eq!(mgr.active_index(), 0);
}

#[test]
fn move_tab_out_of_bounds() {
    use notepadppp::editor::tab_manager::TabManager;
    let mut mgr = TabManager::new();
    mgr.new_tab();
    let t0 = mgr.get_tab_title(0);
    let t1 = mgr.get_tab_title(1);
    mgr.move_tab(0, 99); // out of bounds => no-op
    assert_eq!(mgr.get_tab_title(0), t0);
    assert_eq!(mgr.get_tab_title(1), t1);
}
