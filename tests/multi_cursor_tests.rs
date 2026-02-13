use notepadppp::editor::multi_cursor::{CursorPosition, MultiCursorState, word_at_offset};

#[test]
fn test_add_and_remove_cursors() {
    let mut state = MultiCursorState::new();
    assert!(!state.active);

    state.add_cursor(10);
    state.add_cursor(20);
    state.add_cursor(30);
    assert!(state.active);
    assert_eq!(state.extra_cursors.len(), 3);

    state.clear();
    assert!(!state.active);
    assert!(state.extra_cursors.is_empty());
}

#[test]
fn test_insert_at_multiple_positions() {
    let mut state = MultiCursorState::new();
    let mut text = String::from("let x = 1;\nlet y = 2;\nlet z = 3;");
    // Add cursors after each "let" (at offset 3, 14, 25)
    state.add_cursor(3);
    state.add_cursor(14);
    state.add_cursor(25);

    state.apply_insert(&mut text, "XXX");
    assert_eq!(text, "letXXX x = 1;\nletXXX y = 2;\nletXXX z = 3;");
}

#[test]
fn test_delete_at_multiple_positions() {
    let mut state = MultiCursorState::new();
    let mut text = String::from("aXb cXd eXf");
    // Cursors right after each X to delete forward
    state.add_cursor(1);  // before X
    state.add_cursor(5);  // before X
    state.add_cursor(9);  // before X

    state.apply_delete(&mut text, 1);
    assert_eq!(text, "ab cd ef");
}

#[test]
fn test_select_next_occurrence_wraps() {
    let mut state = MultiCursorState::new();
    let text = "abc def abc";

    // Find first occurrence
    assert!(state.select_next_occurrence(text, "abc"));
    assert_eq!(state.extra_cursors[0].selection, Some((0, 3)));

    // Find second occurrence
    assert!(state.select_next_occurrence(text, "abc"));
    assert_eq!(state.extra_cursors[1].selection, Some((8, 11)));

    // No more — shouldn't add
    assert!(!state.select_next_occurrence(text, "abc"));
    assert_eq!(state.extra_cursors.len(), 2);
}

#[test]
fn test_offset_adjustment_after_multi_insert() {
    let mut state = MultiCursorState::new();
    let mut text = String::from("____");
    state.add_cursor(0);
    state.add_cursor(2);
    state.add_cursor(4);

    state.apply_insert(&mut text, "X");
    assert_eq!(text, "X__X__X");

    // Cursors should be at 1, 4, 7
    let offsets: Vec<usize> = state.extra_cursors.iter().map(|c| c.offset).collect();
    assert_eq!(offsets, vec![1, 4, 7]);
}

#[test]
fn test_backspace_at_multiple_positions() {
    let mut state = MultiCursorState::new();
    let mut text = String::from("1a2b3c");
    state.add_cursor(2); // after "1a"
    state.add_cursor(4); // after "2b"
    state.add_cursor(6); // after "3c"

    state.apply_backspace(&mut text, 1);
    // Each backspace removes the char before cursor: 'a', 'b', 'c'
    assert_eq!(text, "123");
}

#[test]
fn test_backspace_at_multiple_positions_simple() {
    let mut state = MultiCursorState::new();
    let mut text = String::from("XaXbXc");
    state.add_cursor(1);  // after first X
    state.add_cursor(3);  // after second X
    state.add_cursor(5);  // after third X

    state.apply_backspace(&mut text, 1);
    assert_eq!(text, "abc");
}

#[test]
fn test_cursor_position_with_selection() {
    let cp = CursorPosition::with_selection(10, 5, 10);
    assert_eq!(cp.offset, 10);
    assert_eq!(cp.selection, Some((5, 10)));

    let cp2 = CursorPosition::with_selection(5, 10, 5);
    assert_eq!(cp2.selection, Some((5, 10)));  // should be normalized
}

#[test]
fn test_word_at_offset_various() {
    assert_eq!(word_at_offset("hello_world", 0), Some(("hello_world".to_string(), 0, 11)));
    assert_eq!(word_at_offset("hello world", 5), None); // space
    assert_eq!(word_at_offset("", 0), None);
    assert_eq!(word_at_offset("a", 0), Some(("a".to_string(), 0, 1)));
}

#[test]
fn test_selections_and_offsets() {
    let mut state = MultiCursorState::new();
    state.add_cursor(5);
    state.add_cursor_with_selection(10, 8, 10);

    let offsets = state.cursor_offsets();
    assert_eq!(offsets, vec![5, 10]);

    let sels = state.selections();
    assert_eq!(sels, vec![(8, 10)]);
}
