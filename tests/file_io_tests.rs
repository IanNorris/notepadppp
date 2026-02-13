use std::io::Write;
use tempfile::NamedTempFile;

use notepadppp::io::file_io;
use notepadppp::editor::document::{Encoding, LineEnding};

#[test]
fn read_utf8_file() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "Hello, world!\nSecond line\n").unwrap();
    let (content, enc, le) = file_io::read_file(f.path()).unwrap();
    assert_eq!(content, "Hello, world!\nSecond line\n");
    assert!(matches!(enc, Encoding::UTF8 | Encoding::ASCII));
    assert_eq!(le, LineEnding::LF);
}

#[test]
fn read_utf8_bom_file() {
    let mut f = NamedTempFile::new().unwrap();
    // Write BOM + content
    f.write_all(&[0xEF, 0xBB, 0xBF]).unwrap();
    f.write_all(b"BOM content\n").unwrap();
    let (content, enc, _le) = file_io::read_file(f.path()).unwrap();
    assert_eq!(content, "BOM content\n");
    assert_eq!(enc, Encoding::UTF8BOM);
}

#[test]
fn detect_lf_line_ending() {
    let le = file_io::detect_line_ending("line1\nline2\nline3\n");
    assert_eq!(le, LineEnding::LF);
}

#[test]
fn detect_crlf_line_ending() {
    let le = file_io::detect_line_ending("line1\r\nline2\r\nline3\r\n");
    assert_eq!(le, LineEnding::CRLF);
}

#[test]
fn detect_cr_line_ending() {
    let le = file_io::detect_line_ending("line1\rline2\rline3\r");
    assert_eq!(le, LineEnding::CR);
}

#[test]
fn write_and_read_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.txt");
    let content = "Hello\nWorld\n";
    file_io::write_file(&path, content, Encoding::UTF8, LineEnding::LF).unwrap();
    let (read_back, enc, le) = file_io::read_file(&path).unwrap();
    assert_eq!(read_back, content);
    assert!(matches!(enc, Encoding::UTF8 | Encoding::ASCII));
    assert_eq!(le, LineEnding::LF);
}

#[test]
fn write_and_read_roundtrip_crlf() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_crlf.txt");
    let content = "Hello\nWorld\n";
    file_io::write_file(&path, content, Encoding::UTF8, LineEnding::CRLF).unwrap();
    let (read_back, _enc, le) = file_io::read_file(&path).unwrap();
    assert_eq!(read_back, "Hello\r\nWorld\r\n");
    assert_eq!(le, LineEnding::CRLF);
}

#[test]
fn write_and_read_roundtrip_bom() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_bom.txt");
    let content = "BOM test\n";
    file_io::write_file(&path, content, Encoding::UTF8BOM, LineEnding::LF).unwrap();
    let (read_back, enc, _le) = file_io::read_file(&path).unwrap();
    assert_eq!(read_back, "BOM test\n");
    assert_eq!(enc, Encoding::UTF8BOM);
}

#[test]
fn line_ending_conversion_lf_to_crlf() {
    let result = file_io::convert_line_endings("a\nb\nc\n", LineEnding::CRLF);
    assert_eq!(result, "a\r\nb\r\nc\r\n");
}

#[test]
fn line_ending_conversion_crlf_to_lf() {
    let result = file_io::convert_line_endings("a\r\nb\r\nc\r\n", LineEnding::LF);
    assert_eq!(result, "a\nb\nc\n");
}

#[test]
fn line_ending_conversion_cr_to_lf() {
    let result = file_io::convert_line_endings("a\rb\rc\r", LineEnding::LF);
    assert_eq!(result, "a\nb\nc\n");
}

#[test]
fn empty_file_handling() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "").unwrap();
    let (content, _enc, le) = file_io::read_file(f.path()).unwrap();
    assert_eq!(content, "");
    // Default to LF when no line endings present
    assert_eq!(le, LineEnding::LF);
}

#[test]
fn detect_encoding_ascii() {
    let bytes = b"Hello ASCII";
    assert_eq!(file_io::detect_encoding(bytes), Encoding::ASCII);
}

#[test]
fn detect_encoding_utf8_non_ascii() {
    let bytes = "Héllo".as_bytes();
    assert_eq!(file_io::detect_encoding(bytes), Encoding::UTF8);
}

#[test]
fn detect_encoding_utf16le_bom() {
    let bytes = &[0xFF, 0xFE, 0x48, 0x00];
    assert_eq!(file_io::detect_encoding(bytes), Encoding::UTF16LE);
}

#[test]
fn detect_encoding_utf16be_bom() {
    let bytes = &[0xFE, 0xFF, 0x00, 0x48];
    assert_eq!(file_io::detect_encoding(bytes), Encoding::UTF16BE);
}
