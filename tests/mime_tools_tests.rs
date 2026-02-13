use notepadppp::tools::mime_tools::*;

// === Base64 ===

#[test]
fn base64_encode_simple() {
    assert_eq!(base64_encode("Hello, World!"), "SGVsbG8sIFdvcmxkIQ==");
}

#[test]
fn base64_decode_simple() {
    assert_eq!(
        base64_decode("SGVsbG8sIFdvcmxkIQ==").unwrap(),
        "Hello, World!"
    );
}

#[test]
fn base64_round_trip() {
    let input = "The quick brown fox jumps over the lazy dog 🦊";
    assert_eq!(base64_decode(&base64_encode(input)).unwrap(), input);
}

#[test]
fn base64_empty() {
    assert_eq!(base64_encode(""), "");
    assert_eq!(base64_decode("").unwrap(), "");
}

#[test]
fn base64_decode_invalid() {
    assert!(base64_decode("!!!not-valid-base64!!!").is_err());
}

// === URL ===

#[test]
fn url_encode_special_chars() {
    assert_eq!(url_encode("hello world"), "hello%20world");
    assert_eq!(url_encode("a=1&b=2"), "a%3D1%26b%3D2");
}

#[test]
fn url_encode_preserves_unreserved() {
    assert_eq!(url_encode("hello-world_123.test~ok"), "hello-world_123.test~ok");
}

#[test]
fn url_decode_special_chars() {
    assert_eq!(url_decode("hello%20world").unwrap(), "hello world");
    assert_eq!(url_decode("a%3D1%26b%3D2").unwrap(), "a=1&b=2");
}

#[test]
fn url_decode_plus_as_space() {
    assert_eq!(url_decode("hello+world").unwrap(), "hello world");
}

#[test]
fn url_round_trip() {
    let input = "key=value with spaces&special=<tag>";
    assert_eq!(url_decode(&url_encode(input)).unwrap(), input);
}

#[test]
fn url_decode_invalid() {
    assert!(url_decode("hello%GZ").is_err());
    assert!(url_decode("hello%2").is_err());
}

// === HTML Entity ===

#[test]
fn html_entity_encode_all_special() {
    assert_eq!(
        html_entity_encode("<div class=\"test\">&'hello'</div>"),
        "&lt;div class=&quot;test&quot;&gt;&amp;&apos;hello&apos;&lt;/div&gt;"
    );
}

#[test]
fn html_entity_encode_no_special() {
    assert_eq!(html_entity_encode("plain text"), "plain text");
}

#[test]
fn html_entity_decode_named() {
    assert_eq!(
        html_entity_decode("&lt;b&gt;bold&lt;/b&gt;"),
        "<b>bold</b>"
    );
    assert_eq!(html_entity_decode("&amp;&quot;&apos;"), "&\"'");
}

#[test]
fn html_entity_decode_numeric() {
    assert_eq!(html_entity_decode("&#65;"), "A");
    assert_eq!(html_entity_decode("&#x41;"), "A");
}

#[test]
fn html_entity_round_trip() {
    let input = "<script>alert('XSS');</script>";
    assert_eq!(
        html_entity_decode(&html_entity_encode(input)),
        input
    );
}

#[test]
fn html_entity_decode_unknown_entity() {
    assert_eq!(html_entity_decode("&unknown;"), "&unknown;");
}

// === Hex ===

#[test]
fn hex_encode_simple() {
    assert_eq!(hex_encode("ABC"), "414243");
}

#[test]
fn hex_encode_empty() {
    assert_eq!(hex_encode(""), "");
}

#[test]
fn hex_decode_simple() {
    assert_eq!(hex_decode("414243").unwrap(), "ABC");
}

#[test]
fn hex_decode_with_whitespace() {
    assert_eq!(hex_decode("41 42 43").unwrap(), "ABC");
}

#[test]
fn hex_round_trip() {
    let input = "Hello, World! 🌍";
    assert_eq!(hex_decode(&hex_encode(input)).unwrap(), input);
}

#[test]
fn hex_decode_odd_length() {
    assert!(hex_decode("414").is_err());
}

#[test]
fn hex_decode_invalid_chars() {
    assert!(hex_decode("ZZZZ").is_err());
}
