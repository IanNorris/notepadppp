use notepadppp::editor::text_transforms;
use notepadppp::editor::document::{Encoding, LineEnding};
use notepadppp::tools::diff_tool::{DiffResult, DiffLine, DiffStats};

// ── title_case ──

#[test]
fn title_case_normal() {
    assert_eq!(text_transforms::title_case("hello world"), "Hello World");
}

#[test]
fn title_case_all_caps() {
    assert_eq!(text_transforms::title_case("HELLO WORLD"), "Hello World");
}

#[test]
fn title_case_hyphens() {
    assert_eq!(text_transforms::title_case("well-known fact"), "Well-Known Fact");
}

#[test]
fn title_case_underscores() {
    assert_eq!(text_transforms::title_case("some_variable_name"), "Some_Variable_Name");
}

#[test]
fn title_case_empty() {
    assert_eq!(text_transforms::title_case(""), "");
}

#[test]
fn title_case_unicode() {
    assert_eq!(text_transforms::title_case("café latte"), "Café Latte");
}

#[test]
fn title_case_single_word() {
    assert_eq!(text_transforms::title_case("hello"), "Hello");
}

#[test]
fn title_case_already_title() {
    assert_eq!(text_transforms::title_case("Hello World"), "Hello World");
}

// ── sentence_case ──

#[test]
fn sentence_case_normal() {
    assert_eq!(text_transforms::sentence_case("HELLO WORLD"), "Hello world");
}

#[test]
fn sentence_case_multiple_sentences() {
    assert_eq!(
        text_transforms::sentence_case("HELLO WORLD. HOW ARE YOU? FINE!"),
        "Hello world. How are you? Fine!"
    );
}

#[test]
fn sentence_case_empty() {
    assert_eq!(text_transforms::sentence_case(""), "");
}

#[test]
fn sentence_case_unicode() {
    assert_eq!(text_transforms::sentence_case("CAFÉ LATTE"), "Café latte");
}

#[test]
fn sentence_case_single_word() {
    assert_eq!(text_transforms::sentence_case("HELLO"), "Hello");
}

#[test]
fn sentence_case_already_sentence() {
    assert_eq!(text_transforms::sentence_case("Hello world"), "Hello world");
}

// ── inverse_case ──

#[test]
fn inverse_case_mixed() {
    assert_eq!(text_transforms::inverse_case("Hello World"), "hELLO wORLD");
}

#[test]
fn inverse_case_all_upper() {
    assert_eq!(text_transforms::inverse_case("HELLO"), "hello");
}

#[test]
fn inverse_case_all_lower() {
    assert_eq!(text_transforms::inverse_case("hello"), "HELLO");
}

#[test]
fn inverse_case_empty() {
    assert_eq!(text_transforms::inverse_case(""), "");
}

#[test]
fn inverse_case_unicode() {
    assert_eq!(text_transforms::inverse_case("Café"), "cAFÉ");
}

#[test]
fn inverse_case_numbers_and_symbols() {
    assert_eq!(text_transforms::inverse_case("Hello 123!"), "hELLO 123!");
}

// ── toggle_line_comment ──

#[test]
fn toggle_line_comment_add() {
    assert_eq!(text_transforms::toggle_line_comment("hello", "//"), "// hello");
}

#[test]
fn toggle_line_comment_remove_with_space() {
    assert_eq!(text_transforms::toggle_line_comment("// hello", "//"), "hello");
}

#[test]
fn toggle_line_comment_remove_without_space() {
    assert_eq!(text_transforms::toggle_line_comment("//hello", "//"), "hello");
}

#[test]
fn toggle_line_comment_indented_add() {
    assert_eq!(text_transforms::toggle_line_comment("    hello", "//"), "    // hello");
}

#[test]
fn toggle_line_comment_indented_remove() {
    assert_eq!(text_transforms::toggle_line_comment("    // hello", "//"), "    hello");
}

#[test]
fn toggle_line_comment_empty_line() {
    assert_eq!(text_transforms::toggle_line_comment("", "//"), "// ");
}

#[test]
fn toggle_line_comment_hash_prefix() {
    assert_eq!(text_transforms::toggle_line_comment("print('hi')", "#"), "# print('hi')");
    assert_eq!(text_transforms::toggle_line_comment("# print('hi')", "#"), "print('hi')");
}

// ── format_diff_output ──

#[test]
fn format_diff_output_same() {
    let result = DiffResult {
        lines: vec![DiffLine::Same("hello".into())],
        stats: DiffStats { added: 0, removed: 0, changed: 0, same: 1 },
    };
    assert_eq!(text_transforms::format_diff_output(&result), "  hello\n");
}

#[test]
fn format_diff_output_added() {
    let result = DiffResult {
        lines: vec![DiffLine::Added("new line".into())],
        stats: DiffStats { added: 1, removed: 0, changed: 0, same: 0 },
    };
    assert_eq!(text_transforms::format_diff_output(&result), "+ new line\n");
}

#[test]
fn format_diff_output_removed() {
    let result = DiffResult {
        lines: vec![DiffLine::Removed("old line".into())],
        stats: DiffStats { added: 0, removed: 1, changed: 0, same: 0 },
    };
    assert_eq!(text_transforms::format_diff_output(&result), "- old line\n");
}

#[test]
fn format_diff_output_changed() {
    let result = DiffResult {
        lines: vec![DiffLine::Changed { old: "old".into(), new: "new".into() }],
        stats: DiffStats { added: 0, removed: 0, changed: 1, same: 0 },
    };
    assert_eq!(text_transforms::format_diff_output(&result), "- old\n+ new\n");
}

#[test]
fn format_diff_output_empty() {
    let result = DiffResult {
        lines: vec![],
        stats: DiffStats { added: 0, removed: 0, changed: 0, same: 0 },
    };
    assert_eq!(text_transforms::format_diff_output(&result), "");
}

#[test]
fn format_diff_output_mixed() {
    let result = DiffResult {
        lines: vec![
            DiffLine::Same("same".into()),
            DiffLine::Added("added".into()),
            DiffLine::Removed("removed".into()),
        ],
        stats: DiffStats { added: 1, removed: 1, changed: 0, same: 1 },
    };
    let output = text_transforms::format_diff_output(&result);
    assert_eq!(output, "  same\n+ added\n- removed\n");
}

// ── parse_goto_line ──

#[test]
fn parse_goto_line_valid() {
    assert_eq!(text_transforms::parse_goto_line("5"), Some(4));
}

#[test]
fn parse_goto_line_one() {
    assert_eq!(text_transforms::parse_goto_line("1"), Some(0));
}

#[test]
fn parse_goto_line_zero() {
    assert_eq!(text_transforms::parse_goto_line("0"), None);
}

#[test]
fn parse_goto_line_invalid() {
    assert_eq!(text_transforms::parse_goto_line("abc"), None);
}

#[test]
fn parse_goto_line_negative() {
    assert_eq!(text_transforms::parse_goto_line("-1"), None);
}

#[test]
fn parse_goto_line_whitespace() {
    assert_eq!(text_transforms::parse_goto_line("  10  "), Some(9));
}

#[test]
fn parse_goto_line_empty() {
    assert_eq!(text_transforms::parse_goto_line(""), None);
}

// ── clamp_zoom ──

#[test]
fn clamp_zoom_within_range() {
    assert_eq!(text_transforms::clamp_zoom(14.0, 2.0), 16.0);
}

#[test]
fn clamp_zoom_at_min() {
    assert_eq!(text_transforms::clamp_zoom(6.0, 0.0), 6.0);
}

#[test]
fn clamp_zoom_at_max() {
    assert_eq!(text_transforms::clamp_zoom(72.0, 0.0), 72.0);
}

#[test]
fn clamp_zoom_below_min() {
    assert_eq!(text_transforms::clamp_zoom(6.0, -5.0), 6.0);
}

#[test]
fn clamp_zoom_above_max() {
    assert_eq!(text_transforms::clamp_zoom(70.0, 10.0), 72.0);
}

// ── clamp_pref_font_size ──

#[test]
fn clamp_pref_font_size_within() {
    assert_eq!(text_transforms::clamp_pref_font_size(14.0, 2.0), 16.0);
}

#[test]
fn clamp_pref_font_size_at_min() {
    assert_eq!(text_transforms::clamp_pref_font_size(8.0, 0.0), 8.0);
}

#[test]
fn clamp_pref_font_size_at_max() {
    assert_eq!(text_transforms::clamp_pref_font_size(48.0, 0.0), 48.0);
}

#[test]
fn clamp_pref_font_size_below_min() {
    assert_eq!(text_transforms::clamp_pref_font_size(8.0, -5.0), 8.0);
}

#[test]
fn clamp_pref_font_size_above_max() {
    assert_eq!(text_transforms::clamp_pref_font_size(46.0, 10.0), 48.0);
}

// ── format_title ──

#[test]
fn format_title_unmodified() {
    assert_eq!(
        text_transforms::format_title("test.txt", false),
        "test.txt — Notepad+++"
    );
}

#[test]
fn format_title_modified() {
    assert_eq!(
        text_transforms::format_title("test.txt", true),
        "test.txt • — Notepad+++"
    );
}

#[test]
fn format_title_empty() {
    assert_eq!(
        text_transforms::format_title("", false),
        " — Notepad+++"
    );
}

// ── encoding_display_name ──

#[test]
fn encoding_display_name_utf8() {
    assert_eq!(text_transforms::encoding_display_name(&Encoding::UTF8), "UTF-8");
}

#[test]
fn encoding_display_name_utf8bom() {
    assert_eq!(text_transforms::encoding_display_name(&Encoding::UTF8BOM), "UTF-8 BOM");
}

#[test]
fn encoding_display_name_utf16le() {
    assert_eq!(text_transforms::encoding_display_name(&Encoding::UTF16LE), "UTF-16 LE");
}

#[test]
fn encoding_display_name_utf16be() {
    assert_eq!(text_transforms::encoding_display_name(&Encoding::UTF16BE), "UTF-16 BE");
}

#[test]
fn encoding_display_name_ascii() {
    assert_eq!(text_transforms::encoding_display_name(&Encoding::ASCII), "ASCII");
}

// ── line_ending_display_name ──

#[test]
fn line_ending_display_name_lf() {
    assert_eq!(text_transforms::line_ending_display_name(&LineEnding::LF), "LF");
}

#[test]
fn line_ending_display_name_crlf() {
    assert_eq!(text_transforms::line_ending_display_name(&LineEnding::CRLF), "CRLF");
}

#[test]
fn line_ending_display_name_cr() {
    assert_eq!(text_transforms::line_ending_display_name(&LineEnding::CR), "CR");
}

// ── format_status_bar ──

#[test]
fn format_status_bar_without_whitespace() {
    let result = text_transforms::format_status_bar(10, 5, "UTF-8", "LF", "Rust", false);
    assert_eq!(result, "  Ln 10, Col 5    UTF-8    LF    Rust");
}

#[test]
fn format_status_bar_with_whitespace() {
    let result = text_transforms::format_status_bar(1, 1, "UTF-8", "CRLF", "Python", true);
    assert_eq!(result, "  Ln 1, Col 1    UTF-8    CRLF    Python    WS");
}

// ── next_match_index ──

#[test]
fn next_match_index_from_none() {
    assert_eq!(text_transforms::next_match_index(None, 5), Some(0));
}

#[test]
fn next_match_index_normal() {
    assert_eq!(text_transforms::next_match_index(Some(2), 5), Some(3));
}

#[test]
fn next_match_index_wrap_around() {
    assert_eq!(text_transforms::next_match_index(Some(4), 5), Some(0));
}

#[test]
fn next_match_index_empty() {
    assert_eq!(text_transforms::next_match_index(Some(0), 0), None);
}

#[test]
fn next_match_index_single() {
    assert_eq!(text_transforms::next_match_index(Some(0), 1), Some(0));
}

#[test]
fn next_match_index_none_empty() {
    assert_eq!(text_transforms::next_match_index(None, 0), None);
}

// ── prev_match_index ──

#[test]
fn prev_match_index_from_none() {
    assert_eq!(text_transforms::prev_match_index(None, 5), Some(4));
}

#[test]
fn prev_match_index_normal() {
    assert_eq!(text_transforms::prev_match_index(Some(3), 5), Some(2));
}

#[test]
fn prev_match_index_wrap_around() {
    assert_eq!(text_transforms::prev_match_index(Some(0), 5), Some(4));
}

#[test]
fn prev_match_index_empty() {
    assert_eq!(text_transforms::prev_match_index(Some(0), 0), None);
}

#[test]
fn prev_match_index_single() {
    assert_eq!(text_transforms::prev_match_index(Some(0), 1), Some(0));
}

#[test]
fn prev_match_index_none_empty() {
    assert_eq!(text_transforms::prev_match_index(None, 0), None);
}
