use notepadppp::editor::syntax::SyntaxHighlighter;
use notepadppp::tools::export;

#[test]
fn test_html_export_contains_expected_tags() {
    let highlighter = SyntaxHighlighter::new();
    let text = "fn main() {}";
    let html = export::export_html(text, "rs", &highlighter);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<html>"));
    assert!(html.contains("</html>"));
    assert!(html.contains("<head>"));
    assert!(html.contains("<body>"));
    assert!(html.contains("</body>"));
    assert!(html.contains("<style>"));
}

#[test]
fn test_html_export_contains_code_text() {
    let highlighter = SyntaxHighlighter::new();
    let text = "let x = 42;";
    let html = export::export_html(text, "rs", &highlighter);
    // The text content should appear in the output (possibly split across spans)
    assert!(html.contains("42"));
    assert!(html.contains("let"));
}

#[test]
fn test_html_export_plain_text() {
    let highlighter = SyntaxHighlighter::new();
    let text = "Hello, world!";
    let html = export::export_html(text, "", &highlighter);
    assert!(html.contains("Hello, world!"));
    assert!(html.contains("<span"));
}

#[test]
fn test_html_export_multiline() {
    let highlighter = SyntaxHighlighter::new();
    let text = "line1\nline2\nline3";
    let html = export::export_html(text, "", &highlighter);
    assert!(html.contains("line1"));
    assert!(html.contains("line2"));
    assert!(html.contains("line3"));
    // Each line should produce a <br>
    let br_count = html.matches("<br>").count();
    assert_eq!(br_count, 3);
}

#[test]
fn test_html_export_escapes_html_chars() {
    let highlighter = SyntaxHighlighter::new();
    let text = "<script>alert('xss')</script>";
    let html = export::export_html(text, "", &highlighter);
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn test_rtf_export_valid_header() {
    let rtf = export::export_rtf("Hello world");
    assert!(rtf.starts_with("{\\rtf1"));
    assert!(rtf.contains("\\ansi"));
    assert!(rtf.contains("\\fonttbl"));
    assert!(rtf.ends_with('}'));
}

#[test]
fn test_rtf_export_contains_text() {
    let rtf = export::export_rtf("Hello world");
    assert!(rtf.contains("Hello world"));
}

#[test]
fn test_rtf_export_multiline() {
    let rtf = export::export_rtf("line1\nline2");
    assert!(rtf.contains("line1"));
    assert!(rtf.contains("line2"));
    assert!(rtf.contains("\\par"));
}

#[test]
fn test_rtf_export_escapes_special_chars() {
    let rtf = export::export_rtf("a{b}c\\d");
    assert!(rtf.contains("\\{"));
    assert!(rtf.contains("\\}"));
    assert!(rtf.contains("\\\\"));
}

#[test]
fn test_rtf_export_empty_text() {
    let rtf = export::export_rtf("");
    assert!(rtf.starts_with("{\\rtf1"));
    assert!(rtf.ends_with('}'));
}
