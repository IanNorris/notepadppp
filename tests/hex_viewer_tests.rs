use notepadppp::tools::hex_viewer::*;

#[test]
fn test_from_text() {
    let hv = HexView::from_text("Hello");
    assert_eq!(hv.bytes, b"Hello");
    assert_eq!(hv.bytes_per_row, 16);
    assert_eq!(hv.cursor_offset, 0);
    assert!(hv.show_ascii);
}

#[test]
fn test_from_bytes() {
    let hv = HexView::from_bytes(vec![0x00, 0xFF, 0x42]);
    assert_eq!(hv.bytes, vec![0x00, 0xFF, 0x42]);
}

#[test]
fn test_get_byte_at() {
    let hv = HexView::from_text("ABC");
    assert_eq!(hv.get_byte_at(0), Some(b'A'));
    assert_eq!(hv.get_byte_at(1), Some(b'B'));
    assert_eq!(hv.get_byte_at(2), Some(b'C'));
    assert_eq!(hv.get_byte_at(3), None);
}

#[test]
fn test_set_byte_at() {
    let mut hv = HexView::from_text("ABC");
    hv.set_byte_at(1, b'X');
    assert_eq!(hv.bytes, b"AXC");
    // Out of bounds does nothing
    hv.set_byte_at(100, b'Z');
    assert_eq!(hv.bytes.len(), 3);
}

#[test]
fn test_hex_string_roundtrip() {
    let original = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F];
    let hex_str = to_hex_string(&original);
    assert_eq!(hex_str, "48656C6C6F");
    let decoded = from_hex_string(&hex_str).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn test_empty_input() {
    let hv = HexView::from_text("");
    assert!(hv.bytes.is_empty());
    assert_eq!(hv.get_byte_at(0), None);

    let hex_str = to_hex_string(&[]);
    assert_eq!(hex_str, "");
    let decoded = from_hex_string("").unwrap();
    assert!(decoded.is_empty());
}

#[test]
fn test_non_ascii_bytes() {
    let bytes = vec![0x00, 0x01, 0x7F, 0x80, 0xFE, 0xFF];
    let hv = HexView::from_bytes(bytes.clone());
    assert_eq!(hv.get_byte_at(0), Some(0x00));
    assert_eq!(hv.get_byte_at(3), Some(0x80));
    assert_eq!(hv.get_byte_at(5), Some(0xFF));

    let hex_str = to_hex_string(&bytes);
    assert_eq!(hex_str, "00017F80FEFF");
    let decoded = from_hex_string(&hex_str).unwrap();
    assert_eq!(decoded, bytes);
}

#[test]
fn test_from_hex_string_invalid() {
    // Odd length
    assert!(from_hex_string("ABC").is_err());
    // Invalid hex chars
    assert!(from_hex_string("ZZZZ").is_err());
}

#[test]
fn test_from_hex_string_with_spaces() {
    let result = from_hex_string("48 65 6C 6C 6F").unwrap();
    assert_eq!(result, b"Hello");
}
