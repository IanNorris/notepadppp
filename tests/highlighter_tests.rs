use notepadppp::ui_iced::highlighter::extension_for_language;

// ── Known languages ──

#[test]
fn extension_for_rust() {
    assert_eq!(extension_for_language("Rust"), Some("rs".into()));
}

#[test]
fn extension_for_python() {
    assert_eq!(extension_for_language("Python"), Some("py".into()));
}

#[test]
fn extension_for_javascript() {
    assert_eq!(extension_for_language("JavaScript"), Some("js".into()));
}

#[test]
fn extension_for_html() {
    assert_eq!(extension_for_language("HTML"), Some("html".into()));
}

#[test]
fn extension_for_css() {
    assert_eq!(extension_for_language("CSS"), Some("css".into()));
}

#[test]
fn extension_for_json() {
    assert_eq!(extension_for_language("JSON"), Some("json".into()));
}

#[test]
fn extension_for_java() {
    assert_eq!(extension_for_language("Java"), Some("java".into()));
}

#[test]
fn extension_for_c() {
    assert_eq!(extension_for_language("C"), Some("c".into()));
}

#[test]
fn extension_for_go() {
    assert_eq!(extension_for_language("Go"), Some("go".into()));
}

#[test]
fn extension_for_ruby() {
    assert_eq!(extension_for_language("Ruby"), Some("rb".into()));
}

#[test]
fn extension_for_markdown() {
    assert_eq!(extension_for_language("Markdown"), Some("md".into()));
}

// ── Unknown language ──

#[test]
fn extension_for_unknown() {
    assert_eq!(extension_for_language("FooBarLang"), None);
}

// ── Empty string ──

#[test]
fn extension_for_empty() {
    assert_eq!(extension_for_language(""), None);
}
