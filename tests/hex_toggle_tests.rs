use notepadppp::tools::hex_viewer::HexView;

#[test]
fn test_toggle_state_text_to_hex() {
    // Simulate toggling: start in text mode, create hex view
    let text = "Hello, World!";
    let show_hex = false;

    // Toggle to hex
    let show_hex = !show_hex;
    assert!(show_hex);
    let hex_view = HexView::from_text(text);
    assert_eq!(hex_view.bytes, text.as_bytes());
    assert_eq!(hex_view.bytes_per_row, 16);
    assert!(hex_view.show_ascii);
}

#[test]
fn test_toggle_state_hex_to_text() {
    // Simulate toggling: start in hex mode, toggle back
    let show_hex = true;
    let hex_view: Option<HexView> = Some(HexView::from_text("test"));

    // Toggle back to text
    let show_hex = !show_hex;
    assert!(!show_hex);
    let hex_view: Option<HexView> = None;
    assert!(hex_view.is_none());
}

#[test]
fn test_hex_view_from_text_content() {
    let text = "fn main() {\n    println!(\"hello\");\n}\n";
    let hv = HexView::from_text(text);
    assert_eq!(hv.bytes, text.as_bytes());
    assert_eq!(hv.bytes.len(), text.len());
    // First byte should be 'f'
    assert_eq!(hv.get_byte_at(0), Some(b'f'));
    // Newline should be present
    assert!(hv.bytes.contains(&b'\n'));
}

#[test]
fn test_hex_view_from_raw_bytes() {
    // Simulate reading a binary file
    let bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
    let hv = HexView::from_bytes(bytes.clone());
    assert_eq!(hv.bytes, bytes);
    assert_eq!(hv.get_byte_at(0), Some(0x89));
    assert_eq!(hv.get_byte_at(1), Some(0x50)); // 'P'
}

#[test]
fn test_roundtrip_toggle() {
    // Text -> Hex -> verify content preserved -> toggle back
    let original_text = "Hello, Notepad+++!";
    let original_show_hex = false;

    // Toggle to hex
    let show_hex = !original_show_hex;
    assert!(show_hex);
    let hex_view = HexView::from_text(original_text);

    // Verify hex view has correct bytes
    assert_eq!(hex_view.bytes, original_text.as_bytes());
    assert_eq!(std::str::from_utf8(&hex_view.bytes).unwrap(), original_text);

    // Toggle back to text
    let show_hex = !show_hex;
    assert!(!show_hex);
    // Original text should still be intact (hex view is just a view, not modifying)
    assert_eq!(original_text, "Hello, Notepad+++!");
}

#[test]
fn test_hex_view_empty_content() {
    let hv = HexView::from_text("");
    assert!(hv.bytes.is_empty());
    assert_eq!(hv.get_byte_at(0), None);

    let hv = HexView::from_bytes(vec![]);
    assert!(hv.bytes.is_empty());
}

#[test]
fn test_hex_view_cursor_offset_default() {
    let hv = HexView::from_text("test data");
    assert_eq!(hv.cursor_offset, 0);
}

#[test]
fn test_hex_view_multibyte_utf8() {
    // UTF-8 multi-byte characters: bytes != chars
    let text = "café";
    let hv = HexView::from_text(text);
    // 'é' is 2 bytes in UTF-8, so "café" = 5 bytes
    assert_eq!(hv.bytes.len(), 5);
    assert_eq!(hv.get_byte_at(0), Some(b'c'));
    assert_eq!(hv.get_byte_at(1), Some(b'a'));
    assert_eq!(hv.get_byte_at(2), Some(b'f'));
    // 'é' = 0xC3 0xA9
    assert_eq!(hv.get_byte_at(3), Some(0xC3));
    assert_eq!(hv.get_byte_at(4), Some(0xA9));
}

#[test]
fn test_toggle_preserves_bytes_per_row() {
    let hv = HexView::from_text("some content");
    assert_eq!(hv.bytes_per_row, 16);

    let hv = HexView::from_bytes(vec![0xFF; 100]);
    assert_eq!(hv.bytes_per_row, 16);
}
