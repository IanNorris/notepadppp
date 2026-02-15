/// A hex viewer for binary/text content.
pub struct HexView {
    pub bytes: Vec<u8>,
    pub bytes_per_row: usize,
    pub cursor_offset: usize,
    pub show_ascii: bool,
}

impl HexView {
    /// Create a hex view from text content.
    pub fn from_text(text: &str) -> HexView {
        HexView {
            bytes: text.as_bytes().to_vec(),
            bytes_per_row: 16,
            cursor_offset: 0,
            show_ascii: true,
        }
    }

    /// Create a hex view from raw bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> HexView {
        HexView {
            bytes,
            bytes_per_row: 16,
            cursor_offset: 0,
            show_ascii: true,
        }
    }

    /// Get the byte at a given offset.
    pub fn get_byte_at(&self, offset: usize) -> Option<u8> {
        self.bytes.get(offset).copied()
    }

    /// Set the byte at a given offset.
    pub fn set_byte_at(&mut self, offset: usize, value: u8) {
        if offset < self.bytes.len() {
            self.bytes[offset] = value;
        }
    }
}

/// Format bytes as a hex string (e.g. "48656C6C6F").
pub fn to_hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02X}", b)).collect()
}

/// Parse a hex string to bytes.
pub fn from_hex_string(hex: &str) -> Result<Vec<u8>, String> {
    let hex = hex.replace(' ', "");
    if hex.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte_str = &hex[i..i + 2];
        let byte = u8::from_str_radix(byte_str, 16)
            .map_err(|e| format!("Invalid hex byte '{}': {}", byte_str, e))?;
        bytes.push(byte);
    }
    Ok(bytes)
}
