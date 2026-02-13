use notepadppp::io::line_ending_detect::{detect_mixed_line_endings, normalize_line_endings};

#[test]
fn test_consistent_lf() {
    assert!(detect_mixed_line_endings("line1\nline2\nline3").is_none());
}

#[test]
fn test_consistent_crlf() {
    assert!(detect_mixed_line_endings("line1\r\nline2\r\nline3").is_none());
}

#[test]
fn test_mixed_lf_crlf() {
    let report = detect_mixed_line_endings("line1\nline2\r\nline3").unwrap();
    assert_eq!(report.lf, 1);
    assert_eq!(report.crlf, 1);
    assert_eq!(report.cr, 0);
}

#[test]
fn test_mixed_all_three() {
    let report = detect_mixed_line_endings("a\nb\r\nc\rd").unwrap();
    assert_eq!(report.lf, 1);
    assert_eq!(report.crlf, 1);
    assert_eq!(report.cr, 1);
}

#[test]
fn test_normalize_to_lf() {
    let text = "a\r\nb\rc\n";
    assert_eq!(normalize_line_endings(text, "\n"), "a\nb\nc\n");
}

#[test]
fn test_normalize_to_crlf() {
    let text = "a\nb\rc\r\n";
    assert_eq!(normalize_line_endings(text, "\r\n"), "a\r\nb\r\nc\r\n");
}

#[test]
fn test_no_line_endings() {
    assert!(detect_mixed_line_endings("hello world").is_none());
}

#[test]
fn test_empty() {
    assert!(detect_mixed_line_endings("").is_none());
}
