use egui::{self, Color32, FontId, FontFamily, RichText, Ui};

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

/// Render a hex view in an egui UI.
pub fn render_hex(ui: &mut Ui, hex_view: &mut HexView) {
    let mono = FontId::new(14.0, FontFamily::Monospace);
    let offset_color = Color32::from_gray(140);
    let hex_color = Color32::from_gray(230);
    let ascii_color = Color32::from_rgb(100, 210, 210);
    let selected_bg = Color32::from_rgb(60, 80, 120);

    let row_count = if hex_view.bytes.is_empty() {
        0
    } else {
        (hex_view.bytes.len() + hex_view.bytes_per_row - 1) / hex_view.bytes_per_row
    };

    egui::ScrollArea::vertical().show(ui, |ui| {
        for row in 0..row_count {
            let row_start = row * hex_view.bytes_per_row;
            let row_end = (row_start + hex_view.bytes_per_row).min(hex_view.bytes.len());

            ui.horizontal(|ui| {
                // Offset column
                ui.label(
                    RichText::new(format!("{:08X}", row_start))
                        .font(mono.clone())
                        .color(offset_color),
                );

                ui.add_space(8.0);

                // Hex bytes
                for i in row_start..row_start + hex_view.bytes_per_row {
                    if i == row_start + 8 {
                        ui.add_space(6.0);
                    }
                    if i < row_end {
                        let byte = hex_view.bytes[i];
                        let is_selected = i == hex_view.cursor_offset;
                        let text = RichText::new(format!("{:02X}", byte))
                            .font(mono.clone())
                            .color(hex_color);

                        let label = if is_selected {
                            text.background_color(selected_bg)
                        } else {
                            text
                        };
                        let resp = ui.add(egui::Label::new(label).sense(egui::Sense::click()));
                        if resp.clicked() {
                            hex_view.cursor_offset = i;
                        }
                    } else {
                        ui.label(RichText::new("  ").font(mono.clone()));
                    }
                    ui.add_space(2.0);
                }

                if hex_view.show_ascii {
                    ui.add_space(8.0);
                    // ASCII sidebar
                    let mut ascii = String::with_capacity(hex_view.bytes_per_row);
                    for i in row_start..row_start + hex_view.bytes_per_row {
                        if i < row_end {
                            let b = hex_view.bytes[i];
                            if b.is_ascii_graphic() || b == b' ' {
                                ascii.push(b as char);
                            } else {
                                ascii.push('.');
                            }
                        }
                    }
                    ui.label(
                        RichText::new(ascii)
                            .font(mono.clone())
                            .color(ascii_color),
                    );
                }
            });
        }
    });
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
