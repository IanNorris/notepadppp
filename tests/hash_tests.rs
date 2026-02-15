use notepadppp::tools::hash_tools::*;
use notepadppp::tools::mime_tools;

// === SHA-256 ===

#[test]
fn sha256_empty() {
    assert_eq!(
        sha256(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn sha256_hello() {
    assert_eq!(
        sha256(b"hello"),
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

#[test]
fn sha256_hello_world() {
    assert_eq!(
        sha256(b"Hello, World!"),
        "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
    );
}

// === SHA-1 ===

#[test]
fn sha1_hello() {
    assert_eq!(sha1(b"hello"), "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
}

#[test]
fn sha1_empty() {
    assert_eq!(
        sha1(b""),
        "da39a3ee5e6b4b0d3255bfef95601890afd80709"
    );
}

// === MD5 ===

#[test]
fn md5_hello() {
    assert_eq!(md5(b"hello"), "5d41402abc4b2a76b9719d911017c592");
}

#[test]
fn md5_empty() {
    assert_eq!(md5(b""), "d41d8cd98f00b204e9800998ecf8427e");
}

// === CRC32 ===

#[test]
fn crc32_hello() {
    assert_eq!(crc32(b"hello"), "3610a686");
}

#[test]
fn crc32_empty() {
    assert_eq!(crc32(b""), "00000000");
}

// === MIME tools roundtrip tests ===

#[test]
fn base64_roundtrip() {
    let input = "Hello, World! 🌍";
    let encoded = mime_tools::base64_encode(input);
    let decoded = mime_tools::base64_decode(&encoded).unwrap();
    assert_eq!(decoded, input);
}

#[test]
fn url_roundtrip() {
    let input = "hello world & foo=bar";
    let encoded = mime_tools::url_encode(input);
    let decoded = mime_tools::url_decode(&encoded).unwrap();
    assert_eq!(decoded, input);
}

#[test]
fn hex_roundtrip() {
    let input = "Hello, World!";
    let encoded = mime_tools::hex_encode(input);
    let decoded = mime_tools::hex_decode(&encoded).unwrap();
    assert_eq!(decoded, input);
}

#[test]
fn hex_roundtrip_unicode() {
    let input = "こんにちは";
    let encoded = mime_tools::hex_encode(input);
    let decoded = mime_tools::hex_decode(&encoded).unwrap();
    assert_eq!(decoded, input);
}
