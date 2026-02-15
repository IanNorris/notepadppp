use notepadppp::editor::line_ops;

// ── duplicate_line ──

#[test]
fn duplicate_line_middle() {
    let mut lines = vec!["a".into(), "b".into(), "c".into()];
    line_ops::duplicate_line(&mut lines, 1);
    assert_eq!(lines, vec!["a", "b", "b", "c"]);
}

#[test]
fn duplicate_line_first() {
    let mut lines = vec!["first".into(), "second".into()];
    line_ops::duplicate_line(&mut lines, 0);
    assert_eq!(lines, vec!["first", "first", "second"]);
}

#[test]
fn duplicate_line_last() {
    let mut lines = vec!["a".into(), "b".into()];
    line_ops::duplicate_line(&mut lines, 1);
    assert_eq!(lines, vec!["a", "b", "b"]);
}

#[test]
fn duplicate_line_single() {
    let mut lines = vec!["only".into()];
    line_ops::duplicate_line(&mut lines, 0);
    assert_eq!(lines, vec!["only", "only"]);
}

#[test]
fn duplicate_line_out_of_bounds() {
    let mut lines = vec!["a".into()];
    line_ops::duplicate_line(&mut lines, 5);
    assert_eq!(lines, vec!["a"]);
}

#[test]
fn duplicate_line_empty_string() {
    let mut lines = vec!["".into()];
    line_ops::duplicate_line(&mut lines, 0);
    assert_eq!(lines, vec!["", ""]);
}

#[test]
fn duplicate_line_unicode() {
    let mut lines = vec!["héllo 🌍".into()];
    line_ops::duplicate_line(&mut lines, 0);
    assert_eq!(lines, vec!["héllo 🌍", "héllo 🌍"]);
}

// ── delete_line ──

#[test]
fn delete_line_middle() {
    let mut lines = vec!["a".into(), "b".into(), "c".into()];
    line_ops::delete_line(&mut lines, 1);
    assert_eq!(lines, vec!["a", "c"]);
}

#[test]
fn delete_line_first() {
    let mut lines = vec!["a".into(), "b".into()];
    line_ops::delete_line(&mut lines, 0);
    assert_eq!(lines, vec!["b"]);
}

#[test]
fn delete_line_last_of_multiple() {
    let mut lines = vec!["a".into(), "b".into()];
    line_ops::delete_line(&mut lines, 1);
    assert_eq!(lines, vec!["a"]);
}

#[test]
fn delete_line_single_clears() {
    let mut lines = vec!["only".into()];
    line_ops::delete_line(&mut lines, 0);
    assert_eq!(lines, vec![""]);
}

#[test]
fn delete_line_out_of_bounds() {
    let mut lines = vec!["a".into(), "b".into()];
    line_ops::delete_line(&mut lines, 10);
    assert_eq!(lines, vec!["a", "b"]);
}

#[test]
fn delete_line_unicode() {
    let mut lines = vec!["日本語".into(), "中文".into()];
    line_ops::delete_line(&mut lines, 0);
    assert_eq!(lines, vec!["中文"]);
}

// ── move_line_up ──

#[test]
fn move_line_up_normal() {
    let mut lines = vec!["a".into(), "b".into(), "c".into()];
    line_ops::move_line_up(&mut lines, 1);
    assert_eq!(lines, vec!["b", "a", "c"]);
}

#[test]
fn move_line_up_at_zero() {
    let mut lines = vec!["a".into(), "b".into()];
    line_ops::move_line_up(&mut lines, 0);
    assert_eq!(lines, vec!["a", "b"]);
}

#[test]
fn move_line_up_last() {
    let mut lines = vec!["a".into(), "b".into(), "c".into()];
    line_ops::move_line_up(&mut lines, 2);
    assert_eq!(lines, vec!["a", "c", "b"]);
}

#[test]
fn move_line_up_out_of_bounds() {
    let mut lines = vec!["a".into()];
    line_ops::move_line_up(&mut lines, 5);
    assert_eq!(lines, vec!["a"]);
}

#[test]
fn move_line_up_single() {
    let mut lines = vec!["only".into()];
    line_ops::move_line_up(&mut lines, 0);
    assert_eq!(lines, vec!["only"]);
}

// ── move_line_down ──

#[test]
fn move_line_down_normal() {
    let mut lines = vec!["a".into(), "b".into(), "c".into()];
    line_ops::move_line_down(&mut lines, 0);
    assert_eq!(lines, vec!["b", "a", "c"]);
}

#[test]
fn move_line_down_at_last() {
    let mut lines = vec!["a".into(), "b".into()];
    line_ops::move_line_down(&mut lines, 1);
    assert_eq!(lines, vec!["a", "b"]);
}

#[test]
fn move_line_down_middle() {
    let mut lines = vec!["a".into(), "b".into(), "c".into()];
    line_ops::move_line_down(&mut lines, 1);
    assert_eq!(lines, vec!["a", "c", "b"]);
}

#[test]
fn move_line_down_out_of_bounds() {
    let mut lines = vec!["a".into()];
    line_ops::move_line_down(&mut lines, 5);
    assert_eq!(lines, vec!["a"]);
}

#[test]
fn move_line_down_single() {
    let mut lines = vec!["only".into()];
    line_ops::move_line_down(&mut lines, 0);
    assert_eq!(lines, vec!["only"]);
}

// ── sort_asc ──

#[test]
fn sort_asc_normal() {
    let mut lines = vec!["banana".into(), "apple".into(), "cherry".into()];
    line_ops::sort_asc(&mut lines);
    assert_eq!(lines, vec!["apple", "banana", "cherry"]);
}

#[test]
fn sort_asc_empty_vec() {
    let mut lines: Vec<String> = vec![];
    line_ops::sort_asc(&mut lines);
    assert!(lines.is_empty());
}

#[test]
fn sort_asc_single() {
    let mut lines = vec!["only".into()];
    line_ops::sort_asc(&mut lines);
    assert_eq!(lines, vec!["only"]);
}

#[test]
fn sort_asc_with_empty_strings() {
    let mut lines = vec!["b".into(), "".into(), "a".into()];
    line_ops::sort_asc(&mut lines);
    assert_eq!(lines, vec!["", "a", "b"]);
}

#[test]
fn sort_asc_unicode() {
    let mut lines = vec!["ñ".into(), "á".into(), "z".into()];
    line_ops::sort_asc(&mut lines);
    // Unicode sort: 'z' < 'á' < 'ñ' by byte value
    assert_eq!(lines[0], "z");
}

// ── sort_desc ──

#[test]
fn sort_desc_normal() {
    let mut lines = vec!["apple".into(), "cherry".into(), "banana".into()];
    line_ops::sort_desc(&mut lines);
    assert_eq!(lines, vec!["cherry", "banana", "apple"]);
}

#[test]
fn sort_desc_empty_vec() {
    let mut lines: Vec<String> = vec![];
    line_ops::sort_desc(&mut lines);
    assert!(lines.is_empty());
}

#[test]
fn sort_desc_single() {
    let mut lines = vec!["only".into()];
    line_ops::sort_desc(&mut lines);
    assert_eq!(lines, vec!["only"]);
}

// ── sort_case_insensitive ──

#[test]
fn sort_case_insensitive_normal() {
    let mut lines = vec!["Banana".into(), "apple".into(), "Cherry".into()];
    line_ops::sort_case_insensitive(&mut lines);
    assert_eq!(lines, vec!["apple", "Banana", "Cherry"]);
}

#[test]
fn sort_case_insensitive_empty() {
    let mut lines: Vec<String> = vec![];
    line_ops::sort_case_insensitive(&mut lines);
    assert!(lines.is_empty());
}

#[test]
fn sort_case_insensitive_mixed_case() {
    let mut lines = vec!["Z".into(), "a".into(), "B".into()];
    line_ops::sort_case_insensitive(&mut lines);
    assert_eq!(lines, vec!["a", "B", "Z"]);
}

// ── sort_numeric ──

#[test]
fn sort_numeric_normal() {
    let mut lines = vec!["10".into(), "2".into(), "30".into(), "1".into()];
    line_ops::sort_numeric(&mut lines);
    assert_eq!(lines, vec!["1", "2", "10", "30"]);
}

#[test]
fn sort_numeric_with_non_numeric() {
    let mut lines = vec!["10".into(), "abc".into(), "2".into()];
    line_ops::sort_numeric(&mut lines);
    // Numbers come before non-numbers
    assert_eq!(lines, vec!["2", "10", "abc"]);
}

#[test]
fn sort_numeric_floats() {
    let mut lines = vec!["3.14".into(), "1.5".into(), "2.7".into()];
    line_ops::sort_numeric(&mut lines);
    assert_eq!(lines, vec!["1.5", "2.7", "3.14"]);
}

#[test]
fn sort_numeric_empty() {
    let mut lines: Vec<String> = vec![];
    line_ops::sort_numeric(&mut lines);
    assert!(lines.is_empty());
}

#[test]
fn sort_numeric_with_whitespace() {
    let mut lines = vec!["  5".into(), "3  ".into(), " 1 ".into()];
    line_ops::sort_numeric(&mut lines);
    assert_eq!(lines, vec![" 1 ", "3  ", "  5"]);
}

// ── remove_empty ──

#[test]
fn remove_empty_normal() {
    let mut lines = vec!["a".into(), "".into(), "b".into(), "  ".into(), "c".into()];
    line_ops::remove_empty(&mut lines);
    assert_eq!(lines, vec!["a", "b", "c"]);
}

#[test]
fn remove_empty_all_empty() {
    let mut lines = vec!["".into(), "  ".into(), "\t".into()];
    line_ops::remove_empty(&mut lines);
    assert_eq!(lines, vec![""]);
}

#[test]
fn remove_empty_no_empty() {
    let mut lines = vec!["a".into(), "b".into()];
    line_ops::remove_empty(&mut lines);
    assert_eq!(lines, vec!["a", "b"]);
}

#[test]
fn remove_empty_single_empty() {
    let mut lines = vec!["".into()];
    line_ops::remove_empty(&mut lines);
    assert_eq!(lines, vec![""]);
}

// ── remove_duplicates ──

#[test]
fn remove_duplicates_normal() {
    let mut lines = vec!["a".into(), "b".into(), "a".into(), "c".into(), "b".into()];
    line_ops::remove_duplicates(&mut lines);
    assert_eq!(lines, vec!["a", "b", "c"]);
}

#[test]
fn remove_duplicates_no_dups() {
    let mut lines = vec!["x".into(), "y".into(), "z".into()];
    line_ops::remove_duplicates(&mut lines);
    assert_eq!(lines, vec!["x", "y", "z"]);
}

#[test]
fn remove_duplicates_all_same() {
    let mut lines = vec!["same".into(), "same".into(), "same".into()];
    line_ops::remove_duplicates(&mut lines);
    assert_eq!(lines, vec!["same"]);
}

#[test]
fn remove_duplicates_empty_strings() {
    let mut lines = vec!["".into(), "".into(), "a".into()];
    line_ops::remove_duplicates(&mut lines);
    assert_eq!(lines, vec!["", "a"]);
}

#[test]
fn remove_duplicates_unicode() {
    let mut lines = vec!["café".into(), "café".into(), "naïve".into()];
    line_ops::remove_duplicates(&mut lines);
    assert_eq!(lines, vec!["café", "naïve"]);
}

// ── trim_trailing ──

#[test]
fn trim_trailing_normal() {
    let mut lines = vec!["hello   ".into(), "world\t".into(), "no_trail".into()];
    line_ops::trim_trailing(&mut lines);
    assert_eq!(lines, vec!["hello", "world", "no_trail"]);
}

#[test]
fn trim_trailing_empty() {
    let mut lines = vec!["".into()];
    line_ops::trim_trailing(&mut lines);
    assert_eq!(lines, vec![""]);
}

#[test]
fn trim_trailing_preserves_leading() {
    let mut lines = vec!["  hello  ".into()];
    line_ops::trim_trailing(&mut lines);
    assert_eq!(lines, vec!["  hello"]);
}

// ── trim_leading ──

#[test]
fn trim_leading_normal() {
    let mut lines = vec!["  hello".into(), "\tworld".into(), "no_lead".into()];
    line_ops::trim_leading(&mut lines);
    assert_eq!(lines, vec!["hello", "world", "no_lead"]);
}

#[test]
fn trim_leading_empty() {
    let mut lines = vec!["".into()];
    line_ops::trim_leading(&mut lines);
    assert_eq!(lines, vec![""]);
}

#[test]
fn trim_leading_preserves_trailing() {
    let mut lines = vec!["  hello  ".into()];
    line_ops::trim_leading(&mut lines);
    assert_eq!(lines, vec!["hello  "]);
}

// ── trim_both ──

#[test]
fn trim_both_normal() {
    let mut lines = vec!["  hello  ".into(), "\tworld\t".into()];
    line_ops::trim_both(&mut lines);
    assert_eq!(lines, vec!["hello", "world"]);
}

#[test]
fn trim_both_empty() {
    let mut lines = vec!["".into(), "   ".into()];
    line_ops::trim_both(&mut lines);
    assert_eq!(lines, vec!["", ""]);
}

#[test]
fn trim_both_unicode() {
    let mut lines = vec!["  héllo  ".into()];
    line_ops::trim_both(&mut lines);
    assert_eq!(lines, vec!["héllo"]);
}

// ── join_lines ──

#[test]
fn join_lines_normal() {
    let mut lines = vec!["hello".into(), " world".into(), "!".into()];
    line_ops::join_lines(&mut lines, 0);
    assert_eq!(lines, vec!["hello world", "!"]);
}

#[test]
fn join_lines_at_last() {
    let mut lines = vec!["a".into(), "b".into()];
    line_ops::join_lines(&mut lines, 1);
    assert_eq!(lines, vec!["a", "b"]);
}

#[test]
fn join_lines_single() {
    let mut lines = vec!["only".into()];
    line_ops::join_lines(&mut lines, 0);
    assert_eq!(lines, vec!["only"]);
}

#[test]
fn join_lines_empty_next() {
    let mut lines = vec!["a".into(), "".into(), "c".into()];
    line_ops::join_lines(&mut lines, 0);
    assert_eq!(lines, vec!["a", "c"]);
}

#[test]
fn join_lines_out_of_bounds() {
    let mut lines = vec!["a".into()];
    line_ops::join_lines(&mut lines, 5);
    assert_eq!(lines, vec!["a"]);
}

// ── split_line ──

#[test]
fn split_line_even_length() {
    let mut lines = vec!["abcd".into()];
    line_ops::split_line(&mut lines, 0);
    assert_eq!(lines, vec!["ab", "cd"]);
}

#[test]
fn split_line_odd_length() {
    let mut lines = vec!["abcde".into()];
    line_ops::split_line(&mut lines, 0);
    assert_eq!(lines, vec!["ab", "cde"]);
}

#[test]
fn split_line_single_char() {
    let mut lines = vec!["x".into()];
    line_ops::split_line(&mut lines, 0);
    // mid = 0, so first part is empty
    assert_eq!(lines, vec!["", "x"]);
}

#[test]
fn split_line_empty() {
    let mut lines = vec!["".into()];
    line_ops::split_line(&mut lines, 0);
    assert_eq!(lines, vec!["", ""]);
}

#[test]
fn split_line_out_of_bounds() {
    let mut lines = vec!["abc".into()];
    line_ops::split_line(&mut lines, 5);
    assert_eq!(lines, vec!["abc"]);
}

#[test]
fn split_line_in_context() {
    let mut lines = vec!["first".into(), "abcdef".into(), "last".into()];
    line_ops::split_line(&mut lines, 1);
    assert_eq!(lines, vec!["first", "abc", "def", "last"]);
}

// ── insert_above ──

#[test]
fn insert_above_first() {
    let mut lines = vec!["a".into(), "b".into()];
    line_ops::insert_above(&mut lines, 0);
    assert_eq!(lines, vec!["", "a", "b"]);
}

#[test]
fn insert_above_middle() {
    let mut lines = vec!["a".into(), "b".into(), "c".into()];
    line_ops::insert_above(&mut lines, 1);
    assert_eq!(lines, vec!["a", "", "b", "c"]);
}

#[test]
fn insert_above_single() {
    let mut lines = vec!["only".into()];
    line_ops::insert_above(&mut lines, 0);
    assert_eq!(lines, vec!["", "only"]);
}

// ── insert_below ──

#[test]
fn insert_below_first() {
    let mut lines = vec!["a".into(), "b".into()];
    line_ops::insert_below(&mut lines, 0);
    assert_eq!(lines, vec!["a", "", "b"]);
}

#[test]
fn insert_below_last() {
    let mut lines = vec!["a".into(), "b".into()];
    line_ops::insert_below(&mut lines, 1);
    assert_eq!(lines, vec!["a", "b", ""]);
}

#[test]
fn insert_below_single() {
    let mut lines = vec!["only".into()];
    line_ops::insert_below(&mut lines, 0);
    assert_eq!(lines, vec!["only", ""]);
}

// ── reverse_lines ──

#[test]
fn reverse_lines_normal() {
    let mut lines = vec!["a".into(), "b".into(), "c".into()];
    line_ops::reverse_lines(&mut lines);
    assert_eq!(lines, vec!["c", "b", "a"]);
}

#[test]
fn reverse_lines_single() {
    let mut lines = vec!["only".into()];
    line_ops::reverse_lines(&mut lines);
    assert_eq!(lines, vec!["only"]);
}

#[test]
fn reverse_lines_empty() {
    let mut lines: Vec<String> = vec![];
    line_ops::reverse_lines(&mut lines);
    assert!(lines.is_empty());
}

#[test]
fn reverse_lines_two() {
    let mut lines = vec!["first".into(), "second".into()];
    line_ops::reverse_lines(&mut lines);
    assert_eq!(lines, vec!["second", "first"]);
}

#[test]
fn reverse_lines_unicode() {
    let mut lines = vec!["αβγ".into(), "δεζ".into()];
    line_ops::reverse_lines(&mut lines);
    assert_eq!(lines, vec!["δεζ", "αβγ"]);
}
