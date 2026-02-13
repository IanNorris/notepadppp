use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub fn base64_encode(input: &str) -> String {
    BASE64.encode(input.as_bytes())
}

pub fn base64_decode(input: &str) -> Result<String, String> {
    let bytes = BASE64.decode(input.trim()).map_err(|e| format!("Base64 decode error: {e}"))?;
    String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 in decoded data: {e}"))
}

pub fn url_encode(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 3);
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(b as char);
            }
            _ => {
                result.push('%');
                result.push_str(&format!("{b:02X}"));
            }
        }
    }
    result
}

pub fn url_decode(input: &str) -> Result<String, String> {
    let mut bytes = Vec::with_capacity(input.len());
    let mut chars = input.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let hi = chars.next().ok_or("Incomplete percent-encoding")?;
            let lo = chars.next().ok_or("Incomplete percent-encoding")?;
            let hex = format!("{}{}", hi as char, lo as char);
            let val = u8::from_str_radix(&hex, 16)
                .map_err(|_| format!("Invalid percent-encoding: %{hex}"))?;
            bytes.push(val);
        } else if b == b'+' {
            bytes.push(b' ');
        } else {
            bytes.push(b);
        }
    }
    String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 in decoded data: {e}"))
}

pub fn html_entity_encode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(ch),
        }
    }
    result
}

pub fn html_entity_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '&' {
            let mut entity = String::new();
            let mut found_semicolon = false;
            for c in chars.by_ref() {
                if c == ';' {
                    found_semicolon = true;
                    break;
                }
                entity.push(c);
                if entity.len() > 10 {
                    break;
                }
            }
            if found_semicolon {
                match entity.as_str() {
                    "amp" => result.push('&'),
                    "lt" => result.push('<'),
                    "gt" => result.push('>'),
                    "quot" => result.push('"'),
                    "apos" => result.push('\''),
                    s if s.starts_with('#') => {
                        let num_str = &s[1..];
                        let code = if let Some(hex) = num_str.strip_prefix('x') {
                            u32::from_str_radix(hex, 16).ok()
                        } else {
                            num_str.parse::<u32>().ok()
                        };
                        if let Some(c) = code.and_then(char::from_u32) {
                            result.push(c);
                        } else {
                            result.push('&');
                            result.push_str(&entity);
                            result.push(';');
                        }
                    }
                    _ => {
                        result.push('&');
                        result.push_str(&entity);
                        result.push(';');
                    }
                }
            } else {
                result.push('&');
                result.push_str(&entity);
            }
        } else {
            result.push(ch);
        }
    }
    result
}

pub fn hex_encode(input: &str) -> String {
    input.bytes().map(|b| format!("{b:02x}")).collect()
}

pub fn hex_decode(input: &str) -> Result<String, String> {
    let clean: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    if clean.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }
    let bytes: Result<Vec<u8>, _> = (0..clean.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&clean[i..i + 2], 16)
                .map_err(|_| format!("Invalid hex at position {i}: {}", &clean[i..i + 2]))
        })
        .collect();
    String::from_utf8(bytes?).map_err(|e| format!("Invalid UTF-8 in decoded data: {e}"))
}
