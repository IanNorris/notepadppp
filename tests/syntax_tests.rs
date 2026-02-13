use notepadppp::editor::syntax::SyntaxHighlighter;
use std::path::Path;

#[test]
fn detect_language_rust() {
    let lang = SyntaxHighlighter::detect_language(Path::new("main.rs"));
    assert_eq!(lang, "Rust");
}

#[test]
fn detect_language_python() {
    let lang = SyntaxHighlighter::detect_language(Path::new("script.py"));
    assert_eq!(lang, "Python");
}

#[test]
fn detect_language_javascript() {
    let lang = SyntaxHighlighter::detect_language(Path::new("app.js"));
    assert_eq!(lang, "JavaScript");
}

#[test]
fn detect_language_html() {
    let lang = SyntaxHighlighter::detect_language(Path::new("index.html"));
    assert_eq!(lang, "HTML");
}

#[test]
fn detect_language_css() {
    let lang = SyntaxHighlighter::detect_language(Path::new("style.css"));
    assert_eq!(lang, "CSS");
}

#[test]
fn detect_language_json() {
    let lang = SyntaxHighlighter::detect_language(Path::new("data.json"));
    assert_eq!(lang, "JSON");
}

#[test]
fn detect_language_unknown_fallback() {
    let lang = SyntaxHighlighter::detect_language(Path::new("readme.xyz123"));
    assert_eq!(lang, "Plain Text");
}

#[test]
fn detect_language_no_extension() {
    let lang = SyntaxHighlighter::detect_language(Path::new("Makefile"));
    // Makefile has no extension, should fall back
    assert_eq!(lang, "Plain Text");
}

#[test]
fn highlight_returns_spans() {
    let hl = SyntaxHighlighter::new();
    let spans = hl.highlight_line("fn main() {}\n", "rs");
    assert!(!spans.is_empty(), "highlighting should produce spans");
}

#[test]
fn highlight_plain_text() {
    let hl = SyntaxHighlighter::new();
    let spans = hl.highlight_line("hello world\n", "txt");
    assert!(!spans.is_empty(), "plain text should still produce spans");
}

#[test]
fn highlight_unknown_extension() {
    let hl = SyntaxHighlighter::new();
    let spans = hl.highlight_line("some text\n", "xyz999");
    assert!(!spans.is_empty(), "unknown extension should fall back to plain text");
}

#[test]
fn list_themes_not_empty() {
    let hl = SyntaxHighlighter::new();
    let themes = hl.list_themes();
    assert!(!themes.is_empty(), "should have at least one theme");
}

#[test]
fn list_languages_not_empty() {
    let hl = SyntaxHighlighter::new();
    let langs = hl.list_languages();
    assert!(!langs.is_empty(), "should have at least one language");
    assert!(langs.iter().any(|l| l == "Rust"), "should include Rust");
    assert!(langs.iter().any(|l| l == "Python"), "should include Python");
}

#[test]
fn set_and_get_theme() {
    let mut hl = SyntaxHighlighter::new();
    assert_eq!(hl.get_theme(), "base16-ocean.dark");
    let themes = hl.list_themes();
    if let Some(other) = themes.iter().find(|t| *t != "base16-ocean.dark") {
        hl.set_theme(other);
        assert_eq!(hl.get_theme(), other);
    }
}

#[test]
fn set_invalid_theme_ignored() {
    let mut hl = SyntaxHighlighter::new();
    let original = hl.get_theme().to_string();
    hl.set_theme("nonexistent_theme_xyz");
    assert_eq!(hl.get_theme(), original);
}

#[test]
fn get_language_name() {
    let hl = SyntaxHighlighter::new();
    assert_eq!(hl.get_language_name("rs"), "Rust");
    assert_eq!(hl.get_language_name("py"), "Python");
    assert_eq!(hl.get_language_name("xyz999"), "Plain Text");
}

#[test]
fn extension_from_path_works() {
    assert_eq!(SyntaxHighlighter::extension_from_path(Path::new("foo.rs")), "rs");
    assert_eq!(SyntaxHighlighter::extension_from_path(Path::new("bar.py")), "py");
    assert_eq!(SyntaxHighlighter::extension_from_path(Path::new("noext")), "");
}

#[test]
fn highlight_rust_multiple_spans() {
    let hl = SyntaxHighlighter::new();
    let spans = hl.highlight_line("let x = 42;\n", "rs");
    // Should produce multiple spans for keyword, variable, number etc.
    assert!(spans.len() > 1, "Rust code should produce multiple colored spans, got {}", spans.len());
}
