use notepadppp::editor::column_select::ColumnSelection;

#[test]
fn test_new_column_selection() {
    let cs = ColumnSelection::new();
    assert!(!cs.active);
    assert_eq!(cs.start_row, 0);
    assert_eq!(cs.start_col, 0);
}

#[test]
fn test_start_and_extend() {
    let mut cs = ColumnSelection::new();
    cs.start(2, 3);
    assert!(cs.active);
    assert_eq!(cs.start_row, 2);
    assert_eq!(cs.start_col, 3);
    assert_eq!(cs.end_row, 2);
    assert_eq!(cs.end_col, 3);

    cs.extend(5, 7);
    assert_eq!(cs.end_row, 5);
    assert_eq!(cs.end_col, 7);
}

#[test]
fn test_clear() {
    let mut cs = ColumnSelection::new();
    cs.start(1, 1);
    cs.extend(3, 5);
    cs.clear();
    assert!(!cs.active);
}

#[test]
fn test_rows_and_cols() {
    let mut cs = ColumnSelection::new();
    cs.start(5, 8);
    cs.extend(2, 3);
    assert_eq!(cs.rows(), (2, 5));
    assert_eq!(cs.cols(), (3, 8));
}

#[test]
fn test_extract_text() {
    let text = "Hello World\nFoo Bar Baz\nRust Lang!\n";
    let mut cs = ColumnSelection::new();
    cs.start(0, 0);
    cs.extend(2, 5);
    let extracted = cs.extract_text(text);
    assert_eq!(extracted, "Hello\nFoo B\nRust ");
}

#[test]
fn test_extract_text_short_lines() {
    let text = "AB\nCDEF\nGH";
    let mut cs = ColumnSelection::new();
    cs.start(0, 0);
    cs.extend(2, 3);
    let extracted = cs.extract_text(text);
    assert_eq!(extracted, "AB\nCDE\nGH");
}

#[test]
fn test_extract_text_inactive() {
    let cs = ColumnSelection::new();
    assert_eq!(cs.extract_text("hello"), "");
}

#[test]
fn test_delete_selection() {
    let text = "Hello World\nFoo Bar Baz\nRust Lang!";
    let mut cs = ColumnSelection::new();
    cs.start(0, 5);
    cs.extend(2, 9);
    let result = cs.delete_selection(text);
    // Deletes chars at columns [5..9) on each line
    assert_eq!(result, "Hellold\nFoo Baz\nRust !");
}

#[test]
fn test_delete_preserves_trailing_newline() {
    let text = "ABCD\nEFGH\n";
    let mut cs = ColumnSelection::new();
    cs.start(0, 1);
    cs.extend(1, 3);
    let result = cs.delete_selection(text);
    assert_eq!(result, "AD\nEH\n");
}

#[test]
fn test_insert_at_selection_single_char() {
    let text = "ABCD\nEFGH\nIJKL";
    let mut cs = ColumnSelection::new();
    cs.start(0, 2);
    cs.extend(2, 2);
    // Zero-width selection, inserts "X" at col 2 on each row
    let result = cs.insert_at_selection(text, "X");
    assert_eq!(result, "ABXCD\nEFXGH\nIJXKL");
}

#[test]
fn test_insert_replaces_selection() {
    let text = "ABCD\nEFGH\nIJKL";
    let mut cs = ColumnSelection::new();
    cs.start(0, 1);
    cs.extend(2, 3);
    let result = cs.insert_at_selection(text, "XX");
    assert_eq!(result, "AXXD\nEXXH\nIXXL");
}

#[test]
fn test_insert_multi_line_replacement() {
    let text = "ABCD\nEFGH\nIJKL";
    let mut cs = ColumnSelection::new();
    cs.start(0, 1);
    cs.extend(2, 3);
    let result = cs.insert_at_selection(text, "11\n22\n33");
    assert_eq!(result, "A11D\nE22H\nI33L");
}

#[test]
fn test_rows_single_row() {
    let mut cs = ColumnSelection::new();
    cs.start(3, 1);
    cs.extend(3, 5);
    assert_eq!(cs.rows(), (3, 3));
    assert_eq!(cs.cols(), (1, 5));
}

#[test]
fn test_delete_inactive() {
    let cs = ColumnSelection::new();
    assert_eq!(cs.delete_selection("hello"), "hello");
}

#[test]
fn test_insert_inactive() {
    let cs = ColumnSelection::new();
    assert_eq!(cs.insert_at_selection("hello", "X"), "hello");
}
