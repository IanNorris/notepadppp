// Edge case tests across all modules

// =====================================================
// search::engine edge cases
// =====================================================

use notepadppp::search::engine::SearchEngine;

#[test]
fn search_find_all_empty_text() {
    let mut engine = SearchEngine::new();
    engine.query = "hello".to_string();
    let results = engine.find_all("");
    assert!(results.is_empty());
}

#[test]
fn search_find_all_empty_query() {
    let engine = SearchEngine::new();
    // query is empty by default
    let results = engine.find_all("hello world");
    assert!(results.is_empty());
}

#[test]
fn search_find_all_invalid_regex() {
    let mut engine = SearchEngine::new();
    engine.query = "[invalid".to_string();
    engine.search_mode = notepadppp::search::SearchMode::Regex;
    let results = engine.find_all("some text");
    assert!(results.is_empty());
}

// =====================================================
// search::history edge cases
// =====================================================

use notepadppp::search::SearchHistory;

#[test]
fn search_history_add_duplicate() {
    let mut history = SearchHistory::new(10);
    history.add("test".to_string(), vec![]);
    history.add("test".to_string(), vec![]);
    assert_eq!(history.entries().len(), 2);
}

#[test]
fn search_history_max_entries_one() {
    let mut history = SearchHistory::new(1);
    history.add("first".to_string(), vec![]);
    history.add("second".to_string(), vec![]);
    // With max_entries=1, oldest is removed before adding new
    assert_eq!(history.entries().len(), 1);
    assert_eq!(history.entries()[0].query, "second");
}

// =====================================================
// tools::json_tools edge cases
// =====================================================

use notepadppp::tools::json_tools;

#[test]
fn json_format_deeply_nested() {
    let input = r#"{"a":{"b":{"c":{"d":{"e":"deep"}}}}}"#;
    let result = json_tools::format_json(input);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("deep"));
}

#[test]
fn json_format_special_chars() {
    let input = r#"{"msg":"hello\nworld\ttab"}"#;
    let result = json_tools::format_json(input);
    assert!(result.is_ok());
}

#[test]
fn json_compact_invalid_input() {
    let result = json_tools::compact_json("not json at all");
    assert!(result.is_err());
}

#[test]
fn json_sort_keys_invalid_input() {
    let result = json_tools::sort_json_keys("{invalid}");
    assert!(result.is_err());
}

#[test]
fn json_sort_keys_nested() {
    let input = r#"{"z":1,"a":{"c":2,"b":1}}"#;
    let result = json_tools::sort_json_keys(input).unwrap();
    // "a" should come before "z", "b" before "c"
    let a_pos = result.find("\"a\"").unwrap();
    let z_pos = result.find("\"z\"").unwrap();
    assert!(a_pos < z_pos);
}

// =====================================================
// tools::csv_viewer edge cases
// =====================================================

use notepadppp::tools::csv_viewer;

#[test]
fn csv_parse_unicode_emoji() {
    let csv = "name,emoji\nAlice,😀\nBob,🎉\n";
    let data = csv_viewer::parse_csv(csv, ',', true);
    assert_eq!(data.rows.len(), 2);
    assert_eq!(data.rows[0][1], "😀");
    assert_eq!(data.rows[1][1], "🎉");
}

#[test]
fn csv_parse_consecutive_delimiters() {
    let csv = "a,,b\n1,,2\n";
    let data = csv_viewer::parse_csv(csv, ',', true);
    assert_eq!(data.headers.len(), 3);
    assert_eq!(data.headers[1], "");
    assert_eq!(data.rows[0][1], "");
}

#[test]
fn csv_parse_only_headers_no_data() {
    let csv = "col1,col2,col3\n";
    let data = csv_viewer::parse_csv(csv, ',', true);
    assert_eq!(data.headers.len(), 3);
    assert!(data.rows.is_empty());
}

#[test]
fn csv_parse_empty_text() {
    let data = csv_viewer::parse_csv("", ',', true);
    assert!(data.headers.is_empty());
    assert!(data.rows.is_empty());
}

// =====================================================
// tools::mime_tools edge cases
// =====================================================

use notepadppp::tools::mime_tools;

#[test]
fn url_encode_empty_string() {
    assert_eq!(mime_tools::url_encode(""), "");
}

#[test]
fn url_decode_empty_string() {
    assert_eq!(mime_tools::url_decode("").unwrap(), "");
}

#[test]
fn url_encode_unicode() {
    let encoded = mime_tools::url_encode("café");
    assert!(encoded.contains("%"));
    let decoded = mime_tools::url_decode(&encoded).unwrap();
    assert_eq!(decoded, "café");
}

#[test]
fn base64_decode_invalid_utf8() {
    // Encode raw bytes that aren't valid UTF-8 via base64
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    let invalid_bytes: Vec<u8> = vec![0xFF, 0xFE, 0x80];
    let encoded = STANDARD.encode(&invalid_bytes);
    let result = mime_tools::base64_decode(&encoded);
    assert!(result.is_err());
}

#[test]
fn html_entity_decode_unterminated() {
    let result = mime_tools::html_entity_decode("hello &amp world");
    // Unterminated entity: `&amp` without `;` — should be passed through
    assert_eq!(result, "hello &amp world");
}

#[test]
fn html_entity_decode_long_unterminated() {
    let result = mime_tools::html_entity_decode("&verylongentitywithnosemicolon more text");
    // Should not loop infinitely; entity > 10 chars causes break
    assert!(result.contains("more text"));
}

// =====================================================
// tools::hex_viewer edge cases
// =====================================================

use notepadppp::tools::hex_viewer::{HexView, from_hex_string, to_hex_string};

#[test]
fn hex_from_text_with_null_bytes() {
    let hv = HexView::from_text("a\0b");
    assert_eq!(hv.bytes.len(), 3);
    assert_eq!(hv.get_byte_at(1), Some(0));
}

#[test]
fn hex_set_byte_at_offset_zero() {
    let mut hv = HexView::from_text("abc");
    hv.set_byte_at(0, b'X');
    assert_eq!(hv.bytes[0], b'X');
}

#[test]
fn hex_set_byte_at_last_valid() {
    let mut hv = HexView::from_text("abc");
    hv.set_byte_at(2, b'Z');
    assert_eq!(hv.bytes[2], b'Z');
}

#[test]
fn hex_set_byte_at_out_of_range() {
    let mut hv = HexView::from_text("abc");
    hv.set_byte_at(99, b'Z'); // no-op
    assert_eq!(hv.bytes.len(), 3);
}

#[test]
fn hex_from_hex_string_lowercase() {
    let result = from_hex_string("48656c6c6f").unwrap();
    assert_eq!(result, b"Hello");
}

#[test]
fn hex_to_hex_string_roundtrip() {
    let original = b"Test123";
    let hex = to_hex_string(original);
    let decoded = from_hex_string(&hex).unwrap();
    assert_eq!(decoded, original);
}

// =====================================================
// tools::diff_tool edge cases
// =====================================================

use notepadppp::tools::diff_tool;

#[test]
fn diff_texts_unicode_emoji() {
    let left = "hello 😀\nworld";
    let right = "hello 🎉\nworld";
    let result = diff_tool::diff_texts(left, right);
    assert!(result.stats.changed > 0 || result.stats.removed > 0 || result.stats.added > 0);
    assert_eq!(result.stats.same, 1); // "world" is the same
}

#[test]
fn diff_texts_trailing_newline_difference() {
    let left = "hello\n";
    let right = "hello";
    let result = diff_tool::diff_texts(left, right);
    // Both should have "hello" as same
    assert_eq!(result.stats.same, 1);
}

#[test]
fn diff_texts_both_empty() {
    let result = diff_tool::diff_texts("", "");
    assert!(result.lines.is_empty());
    assert_eq!(result.stats.same, 0);
}

#[test]
fn diff_texts_one_empty() {
    let result = diff_tool::diff_texts("hello\nworld", "");
    assert!(result.stats.removed > 0 || result.stats.changed > 0);
}

// =====================================================
// io::file_io edge cases
// =====================================================

use notepadppp::io::file_io;

#[test]
fn detect_encoding_empty_input() {
    let enc = file_io::detect_encoding(&[]);
    assert_eq!(enc, notepadppp::editor::document::Encoding::ASCII);
}

#[test]
fn detect_encoding_single_byte() {
    let enc = file_io::detect_encoding(&[0x41]); // 'A'
    assert_eq!(enc, notepadppp::editor::document::Encoding::ASCII);
}

#[test]
fn detect_encoding_single_high_byte() {
    let enc = file_io::detect_encoding(&[0x80]);
    assert_eq!(enc, notepadppp::editor::document::Encoding::UTF8);
}

#[test]
fn detect_line_ending_mixed() {
    // Mixed endings - CRLF dominant
    let text = "line1\r\nline2\r\nline3\n";
    let le = file_io::detect_line_ending(text);
    assert_eq!(le, notepadppp::editor::document::LineEnding::CRLF);
}

#[test]
fn detect_line_ending_no_newlines() {
    let le = file_io::detect_line_ending("no newlines here");
    assert_eq!(le, notepadppp::editor::document::LineEnding::LF);
}

#[test]
fn read_file_nonexistent() {
    let result = file_io::read_file(std::path::Path::new("/nonexistent/file.txt"));
    assert!(result.is_err());
}

// =====================================================
// io::keybindings edge cases
// =====================================================

use notepadppp::io::keybindings::KeyBindings;

#[test]
fn keybindings_defaults_has_all_actions() {
    let kb = KeyBindings::defaults();
    let actions = [
        "new_file", "open_file", "save", "save_as", "close_tab",
        "undo", "redo", "zoom_in", "zoom_out", "zoom_reset",
        "find", "replace", "find_next", "find_prev",
        "goto_line", "command_palette", "bracket_jump",
        "toggle_bookmark", "next_bookmark", "prev_bookmark",
        "format_json", "toggle_macro_recording", "play_last_macro",
        "select_next", "toggle_hex_view", "escape",
        "delete_line", "join_lines", "case_upper", "case_lower",
        "toggle_comment", "select_all_occurrences", "toggle_fold",
    ];
    for action in &actions {
        assert!(kb.bindings.contains_key(*action), "Missing action: {}", action);
    }
}

#[test]
fn keybindings_load_malformed_json() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad_kb.json");
    std::fs::write(&path, "not json").unwrap();
    let result = KeyBindings::load(&path);
    assert!(result.is_err());
}

// =====================================================
// io::settings edge cases
// =====================================================

use notepadppp::io::settings::AppSettings;

#[test]
fn settings_load_malformed_json() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad_settings.json");
    std::fs::write(&path, "{{invalid").unwrap();
    let result = AppSettings::load(&path);
    assert!(result.is_err());
}

#[test]
fn settings_load_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty_settings.json");
    std::fs::write(&path, "").unwrap();
    let result = AppSettings::load(&path);
    assert!(result.is_err());
}

#[test]
fn settings_save_and_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let settings = AppSettings::default();
    settings.save(&path).unwrap();
    let loaded = AppSettings::load(&path).unwrap();
    assert_eq!(loaded.font_size, 14.0);
    assert_eq!(loaded.tab_size, 4);
}

// =====================================================
// editor::cursor edge cases
// =====================================================

use notepadppp::editor::cursor::CursorState;
use notepadppp::editor::buffer::TextBuffer;

#[test]
fn cursor_move_left_on_empty_buffer() {
    let buf = TextBuffer::new();
    let mut cursor = CursorState::new();
    cursor.move_left(&buf);
    assert_eq!(cursor.position.line, 0);
    assert_eq!(cursor.position.col, 0);
}

#[test]
fn cursor_move_right_on_empty_buffer() {
    let buf = TextBuffer::new();
    let mut cursor = CursorState::new();
    cursor.move_right(&buf);
    assert_eq!(cursor.position.line, 0);
    assert_eq!(cursor.position.col, 0);
}

#[test]
fn cursor_move_up_on_empty_buffer() {
    let buf = TextBuffer::new();
    let mut cursor = CursorState::new();
    cursor.move_up(&buf);
    assert_eq!(cursor.position.line, 0);
}

#[test]
fn cursor_move_down_on_empty_buffer() {
    let buf = TextBuffer::new();
    let mut cursor = CursorState::new();
    cursor.move_down(&buf);
    assert_eq!(cursor.position.line, 0);
}

#[test]
fn cursor_select_line_last_line_no_trailing_newline() {
    let buf = TextBuffer::from_str("first\nsecond");
    let mut cursor = CursorState::new();
    cursor.set_position(1, 0);
    cursor.select_line(&buf);
    assert!(cursor.has_selection());
    let (start, end) = cursor.selected_range().unwrap();
    assert_eq!(start.line, 1);
    assert_eq!(start.col, 0);
    assert_eq!(end.col, buf.line_len_chars(1));
}

#[test]
fn cursor_set_position_clears_anchor() {
    let buf = TextBuffer::from_str("hello");
    let mut cursor = CursorState::new();
    cursor.start_selection();
    cursor.move_right(&buf);
    assert!(cursor.anchor.is_none()); // move_right clears anchor
    // Use start_selection + set_position
    cursor.start_selection();
    assert!(cursor.anchor.is_some());
    cursor.set_position(0, 3);
    assert!(cursor.anchor.is_none());
}

// =====================================================
// editor::column_select edge cases
// =====================================================

use notepadppp::editor::column_select::ColumnSelection;

#[test]
fn column_select_extract_empty_text() {
    let mut sel = ColumnSelection::new();
    sel.start(0, 0);
    sel.extend(0, 5);
    let result = sel.extract_text("");
    assert_eq!(result, "");
}

#[test]
fn column_select_extract_unicode() {
    let mut sel = ColumnSelection::new();
    sel.start(0, 0);
    sel.extend(0, 4);
    let result = sel.extract_text("café");
    assert_eq!(result, "café");
}

#[test]
fn column_select_delete_empty_text() {
    let mut sel = ColumnSelection::new();
    sel.start(0, 0);
    sel.extend(0, 5);
    let result = sel.delete_selection("");
    assert_eq!(result, "");
}

#[test]
fn column_select_insert_empty_text() {
    let mut sel = ColumnSelection::new();
    sel.start(0, 0);
    sel.extend(0, 0);
    let result = sel.insert_at_selection("", "hello");
    assert_eq!(result, "");
}

#[test]
fn column_select_inactive_extract() {
    let sel = ColumnSelection::new();
    assert!(!sel.active);
    let result = sel.extract_text("some text");
    assert_eq!(result, "");
}

// =====================================================
// editor::bookmarks edge cases
// =====================================================

use notepadppp::editor::bookmarks::BookmarkManager;

#[test]
fn bookmarks_remove_non_existent() {
    let mut bm = BookmarkManager::default();
    bm.add(5);
    bm.remove(99); // non-existent, should be no-op
    assert!(bm.is_bookmarked(5));
    assert_eq!(bm.count(), 1);
}

#[test]
fn bookmarks_bookmarked_lines_empty_text() {
    let mut bm = BookmarkManager::default();
    bm.add(0);
    let lines = bm.bookmarked_lines("");
    assert!(lines.is_empty());
}

#[test]
fn bookmarks_remove_bookmarked_lines_all_bookmarked() {
    let mut bm = BookmarkManager::default();
    bm.add(0);
    bm.add(1);
    bm.add(2);
    let result = bm.remove_bookmarked_lines("line1\nline2\nline3");
    assert_eq!(result, "");
}

// =====================================================
// editor::autoclose edge cases
// =====================================================

use notepadppp::editor::autoclose;

#[test]
fn closing_pair_with_closing_bracket() {
    assert_eq!(autoclose::closing_pair(')'), None);
    assert_eq!(autoclose::closing_pair(']'), None);
    assert_eq!(autoclose::closing_pair('}'), None);
}

#[test]
fn should_autoclose_with_digit_next() {
    assert!(!autoclose::should_autoclose('(', Some('5')));
}

#[test]
fn should_autoclose_with_opening_bracket_next() {
    assert!(!autoclose::should_autoclose('(', Some('(')));
}

// =====================================================
// editor::indent edge cases
// =====================================================

use notepadppp::editor::indent;

#[test]
fn get_indent_mixed_whitespace() {
    let result = indent::get_indent("\t  hello");
    assert_eq!(result, "\t  ");
}

#[test]
fn compute_indent_with_paren_line() {
    let result = indent::compute_indent("fn foo(", "    ");
    assert_eq!(result, "    ");
}

#[test]
fn compute_indent_with_bracket_line() {
    let result = indent::compute_indent("items = [", "    ");
    assert_eq!(result, "    ");
}

#[test]
fn compute_indent_with_colon_line() {
    let result = indent::compute_indent("def foo():", "    ");
    assert_eq!(result, "    ");
}

#[test]
fn reindent_paste_empty_string() {
    let result = indent::reindent_paste("", "    ");
    assert_eq!(result, "");
}

// =====================================================
// editor::brackets edge cases
// =====================================================

use notepadppp::editor::brackets;

#[test]
fn find_matching_bracket_empty_string() {
    assert_eq!(brackets::find_matching_bracket("", 0), None);
}

#[test]
fn find_matching_bracket_pos_beyond_length() {
    assert_eq!(brackets::find_matching_bracket("hello", 10), None);
}

#[test]
fn find_matching_bracket_non_bracket_char() {
    assert_eq!(brackets::find_matching_bracket("hello", 0), None);
}

#[test]
fn is_bracket_empty_string() {
    assert!(!brackets::is_bracket("", 0));
}

// =====================================================
// editor::syntax edge cases
// =====================================================

use notepadppp::editor::syntax::SyntaxHighlighter;

#[test]
fn highlight_line_empty_string() {
    let sh = SyntaxHighlighter::new();
    let result = sh.highlight_line("", "rs");
    // Empty line should produce empty or minimal results
    assert!(result.is_empty() || result.iter().all(|(_, t)| t.is_empty()));
}

#[test]
fn detect_language_dotfile() {
    let lang = SyntaxHighlighter::detect_language(std::path::Path::new(".gitignore"));
    // No extension -> Plain Text
    assert_eq!(lang, "Plain Text");
}

#[test]
fn detect_language_multiple_dots() {
    let lang = SyntaxHighlighter::detect_language(std::path::Path::new("file.test.js"));
    // Should detect "js" extension
    assert_ne!(lang, "Plain Text");
}

#[test]
fn extension_from_path_no_extension() {
    let ext = SyntaxHighlighter::extension_from_path(std::path::Path::new("Makefile"));
    assert_eq!(ext, "");
}

// =====================================================
// editor::multi_cursor edge cases
// =====================================================

use notepadppp::editor::multi_cursor::word_at_offset;

#[test]
fn word_at_offset_beyond_length() {
    assert_eq!(word_at_offset("hello", 100), None);
}

#[test]
fn word_at_offset_on_space() {
    assert_eq!(word_at_offset("hello world", 5), None);
}

#[test]
fn word_at_offset_unicode_ascii_part() {
    // word_at_offset works on ASCII bytes; on ASCII part of Unicode text
    let text = "café";
    // 'c' at offset 0 is ASCII
    let result = word_at_offset(text, 0);
    assert!(result.is_some());
    assert_eq!(result.unwrap().0, "caf");
}
