use std::fs;
use std::io;
use std::path::Path;

use crate::editor::document::{Encoding, LineEnding};

/// Detect encoding from a BOM (byte order mark) at the start of the data.
pub fn detect_encoding(bytes: &[u8]) -> Encoding {
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        Encoding::UTF8BOM
    } else if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        Encoding::UTF16LE
    } else if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        Encoding::UTF16BE
    } else {
        // Check if pure ASCII
        if bytes.iter().all(|&b| b < 128) {
            Encoding::ASCII
        } else {
            Encoding::UTF8
        }
    }
}

/// Detect the dominant line ending in a text string.
pub fn detect_line_ending(text: &str) -> LineEnding {
    let crlf_count = text.matches("\r\n").count();
    let cr_only = text.matches('\r').count() - crlf_count;
    let lf_only = text.matches('\n').count() - crlf_count;

    if crlf_count >= lf_only && crlf_count >= cr_only && crlf_count > 0 {
        LineEnding::CRLF
    } else if cr_only > lf_only && cr_only > 0 {
        LineEnding::CR
    } else {
        LineEnding::LF
    }
}

/// Convert all line endings in `text` to the specified `target` line ending.
pub fn convert_line_endings(text: &str, target: LineEnding) -> String {
    // Normalize to LF first, then convert to target
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    match target {
        LineEnding::LF => normalized,
        LineEnding::CRLF => normalized.replace('\n', "\r\n"),
        LineEnding::CR => normalized.replace('\n', "\r"),
    }
}

/// Read a file, detect its encoding and line ending.
/// Returns `(content, encoding, line_ending)`.
pub fn read_file(path: &Path) -> io::Result<(String, Encoding, LineEnding)> {
    let bytes = fs::read(path)?;
    let encoding = detect_encoding(&bytes);

    let (text, skip) = match encoding {
        Encoding::UTF8BOM => {
            let s = String::from_utf8(bytes[3..].to_vec())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            (s, true)
        }
        Encoding::UTF16LE | Encoding::UTF16BE => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "UTF-16 reading is not yet supported",
            ));
        }
        _ => {
            let s = String::from_utf8(bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            (s, false)
        }
    };
    let _ = skip;

    let line_ending = detect_line_ending(&text);
    Ok((text, encoding, line_ending))
}

/// Write content to a file with the specified encoding and line ending.
pub fn write_file(
    path: &Path,
    content: &str,
    encoding: Encoding,
    line_ending: LineEnding,
) -> io::Result<()> {
    let converted = convert_line_endings(content, line_ending);

    let bytes = match encoding {
        Encoding::UTF8BOM => {
            let mut v = vec![0xEF, 0xBB, 0xBF];
            v.extend_from_slice(converted.as_bytes());
            v
        }
        Encoding::UTF16LE | Encoding::UTF16BE => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "UTF-16 writing is not yet supported",
            ));
        }
        _ => converted.into_bytes(),
    };

    fs::write(path, bytes)
}
