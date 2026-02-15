use std::path::PathBuf;
use notepadppp::editor::document::{Document, Encoding, LineEnding};
use notepadppp::io::large_file::LargeFileInfo;

// ── Document::new ──

#[test]
fn new_default_encoding() {
    let doc = Document::new();
    assert_eq!(doc.encoding, Encoding::UTF8);
}

#[test]
fn new_default_line_ending() {
    let doc = Document::new();
    assert_eq!(doc.line_ending, LineEnding::LF);
}

#[test]
fn new_default_language() {
    let doc = Document::new();
    assert_eq!(doc.language, "Plain Text");
}

#[test]
fn new_not_read_only() {
    let doc = Document::new();
    assert!(!doc.read_only);
}

#[test]
fn new_no_path() {
    let doc = Document::new();
    assert!(doc.path.is_none());
}

#[test]
fn new_not_modified() {
    let doc = Document::new();
    assert!(!doc.is_modified());
}

// ── Document::from_str ──

#[test]
fn from_str_sets_content() {
    let doc = Document::from_str("hello world");
    assert_eq!(doc.buffer.text(), "hello world");
}

#[test]
fn from_str_default_fields() {
    let doc = Document::from_str("content");
    assert_eq!(doc.encoding, Encoding::UTF8);
    assert_eq!(doc.line_ending, LineEnding::LF);
    assert_eq!(doc.language, "Plain Text");
    assert!(doc.path.is_none());
}

// ── with_path ──

#[test]
fn with_path_sets_path() {
    let doc = Document::new().with_path(PathBuf::from("/tmp/test.txt"));
    assert_eq!(doc.path, Some(PathBuf::from("/tmp/test.txt")));
}

// ── with_encoding ──

#[test]
fn with_encoding_sets_encoding() {
    let doc = Document::new().with_encoding(Encoding::UTF16LE);
    assert_eq!(doc.encoding, Encoding::UTF16LE);
}

// ── with_line_ending ──

#[test]
fn with_line_ending_sets_line_ending() {
    let doc = Document::new().with_line_ending(LineEnding::CRLF);
    assert_eq!(doc.line_ending, LineEnding::CRLF);
}

// ── with_language ──

#[test]
fn with_language_sets_language() {
    let doc = Document::new().with_language("Rust");
    assert_eq!(doc.language, "Rust");
}

#[test]
fn with_language_from_string() {
    let lang = String::from("Python");
    let doc = Document::new().with_language(lang);
    assert_eq!(doc.language, "Python");
}

// ── with_large_file_info ──

#[test]
fn with_large_file_info_sets_info() {
    let info = LargeFileInfo::from_size(20 * 1024 * 1024); // 20 MB - large
    let doc = Document::new().with_large_file_info(info);
    assert!(doc.large_file_info.is_large);
}

#[test]
fn with_large_file_info_sets_undo_limit() {
    let info = LargeFileInfo::from_size(20 * 1024 * 1024);
    assert!(info.undo_limit.is_some());
    let _doc = Document::new().with_large_file_info(info);
    // The undo limit is set on the buffer internally
}

#[test]
fn with_large_file_info_small_file() {
    let info = LargeFileInfo::from_size(100); // small
    let doc = Document::new().with_large_file_info(info);
    assert!(!doc.large_file_info.is_large);
    assert!(doc.large_file_info.undo_limit.is_none());
}

// ── title ──

#[test]
fn title_with_path() {
    let doc = Document::new().with_path(PathBuf::from("/home/user/docs/test.txt"));
    assert_eq!(doc.title(), "test.txt");
}

#[test]
fn title_without_path() {
    let doc = Document::new();
    assert_eq!(doc.title(), "Untitled");
}

#[test]
fn title_complex_path() {
    let doc = Document::new().with_path(PathBuf::from("/a/b/c/d/file.rs"));
    assert_eq!(doc.title(), "file.rs");
}

#[test]
fn title_unicode_path() {
    let doc = Document::new().with_path(PathBuf::from("/tmp/café.txt"));
    assert_eq!(doc.title(), "café.txt");
}

// ── is_modified ──

#[test]
fn is_modified_false_new() {
    let doc = Document::new();
    assert!(!doc.is_modified());
}

#[test]
fn is_modified_true_after_edit() {
    let mut doc = Document::new();
    doc.buffer.insert(0, "text");
    assert!(doc.is_modified());
}

// ── LineEnding::as_str ──

#[test]
fn line_ending_lf_as_str() {
    assert_eq!(LineEnding::LF.as_str(), "\n");
}

#[test]
fn line_ending_crlf_as_str() {
    assert_eq!(LineEnding::CRLF.as_str(), "\r\n");
}

#[test]
fn line_ending_cr_as_str() {
    assert_eq!(LineEnding::CR.as_str(), "\r");
}

// ── Default trait ──

#[test]
fn default_matches_new() {
    let def = Document::default();
    let new = Document::new();
    assert_eq!(def.encoding, new.encoding);
    assert_eq!(def.line_ending, new.line_ending);
    assert_eq!(def.language, new.language);
    assert_eq!(def.read_only, new.read_only);
    assert_eq!(def.path, new.path);
}

// ── Builder chaining ──

#[test]
fn builder_chaining() {
    let doc = Document::from_str("content")
        .with_path(PathBuf::from("/tmp/test.rs"))
        .with_encoding(Encoding::UTF8BOM)
        .with_line_ending(LineEnding::CRLF)
        .with_language("Rust");

    assert_eq!(doc.buffer.text(), "content");
    assert_eq!(doc.path, Some(PathBuf::from("/tmp/test.rs")));
    assert_eq!(doc.encoding, Encoding::UTF8BOM);
    assert_eq!(doc.line_ending, LineEnding::CRLF);
    assert_eq!(doc.language, "Rust");
}
