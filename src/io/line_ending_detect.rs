/// Detect mixed line endings in text.
/// Returns Some with counts if mixed, None if consistent.
pub fn detect_mixed_line_endings(text: &str) -> Option<MixedLineEndingReport> {
    let mut crlf = 0usize;
    let mut lf = 0usize;
    let mut cr = 0usize;

    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes[i] == b'\r' {
            if i + 1 < len && bytes[i + 1] == b'\n' {
                crlf += 1;
                i += 2;
            } else {
                cr += 1;
                i += 1;
            }
        } else if bytes[i] == b'\n' {
            lf += 1;
            i += 1;
        } else {
            i += 1;
        }
    }

    let kinds_present = (crlf > 0) as u8 + (lf > 0) as u8 + (cr > 0) as u8;
    if kinds_present > 1 {
        Some(MixedLineEndingReport { crlf, lf, cr })
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub struct MixedLineEndingReport {
    pub crlf: usize,
    pub lf: usize,
    pub cr: usize,
}

/// Normalize all line endings to the specified type
pub fn normalize_line_endings(text: &str, target: &str) -> String {
    // First normalize all to \n, then convert to target
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    if target == "\r\n" {
        normalized.replace('\n', "\r\n")
    } else if target == "\r" {
        normalized.replace('\n', "\r")
    } else {
        normalized
    }
}
