use notepadppp::tools::json_tools::*;

#[test]
fn test_format_valid_json() {
    let input = r#"{"name":"Alice","age":30}"#;
    let result = format_json(input).unwrap();
    assert!(result.contains("  \"name\": \"Alice\""));
    assert!(result.contains("  \"age\": 30"));
}

#[test]
fn test_format_invalid_json() {
    let input = "not json at all";
    let result = format_json(input);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid JSON"));
}

#[test]
fn test_compact_json() {
    let input = r#"{
  "name": "Alice",
  "age": 30
}"#;
    let result = compact_json(input).unwrap();
    assert!(!result.contains('\n'));
    assert!(!result.contains("  "));
    assert!(result.contains("\"name\":\"Alice\""));
}

#[test]
fn test_validate_valid_json() {
    let input = r#"{"valid": true}"#;
    assert!(validate_json(input).is_ok());
}

#[test]
fn test_validate_invalid_json() {
    let input = r#"{"missing": }"#;
    let result = validate_json(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("line"));
    assert!(err.contains("column"));
}

#[test]
fn test_sort_json_keys() {
    let input = r#"{"z": 1, "a": 2, "m": 3}"#;
    let result = sort_json_keys(input).unwrap();
    let _lines: Vec<&str> = result.lines().collect();
    // Keys should appear in alphabetical order
    let a_pos = result.find("\"a\"").unwrap();
    let m_pos = result.find("\"m\"").unwrap();
    let z_pos = result.find("\"z\"").unwrap();
    assert!(a_pos < m_pos);
    assert!(m_pos < z_pos);
}

#[test]
fn test_sort_json_keys_nested() {
    let input = r#"{"b": {"z": 1, "a": 2}, "a": 1}"#;
    let result = sort_json_keys(input).unwrap();
    // Top-level: "a" before "b"
    let a_pos = result.find("\"a\": 1").unwrap();
    let b_pos = result.find("\"b\"").unwrap();
    assert!(a_pos < b_pos);
}

#[test]
fn test_json_path_simple() {
    let input = r#"{"data": {"name": "Alice"}}"#;
    let result = json_to_path(input, "data.name").unwrap();
    assert_eq!(result.trim(), "\"Alice\"");
}

#[test]
fn test_json_path_array_index() {
    let input = r#"{"users": ["Alice", "Bob", "Charlie"]}"#;
    let result = json_to_path(input, "users.1").unwrap();
    assert_eq!(result.trim(), "\"Bob\"");
}

#[test]
fn test_json_path_nested_object_in_array() {
    let input = r#"{"data": {"users": [{"name": "Alice"}, {"name": "Bob"}]}}"#;
    let result = json_to_path(input, "data.users.0.name").unwrap();
    assert_eq!(result.trim(), "\"Alice\"");
}

#[test]
fn test_json_path_not_found() {
    let input = r#"{"data": {"name": "Alice"}}"#;
    let result = json_to_path(input, "data.missing");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_json_path_empty() {
    let input = r#"{"a": 1}"#;
    let result = json_to_path(input, "").unwrap();
    assert!(result.contains("\"a\""));
}

#[test]
fn test_format_empty_object() {
    let result = format_json("{}").unwrap();
    assert_eq!(result, "{}");
}

#[test]
fn test_format_empty_array() {
    let result = format_json("[]").unwrap();
    assert_eq!(result, "[]");
}

#[test]
fn test_compact_nested() {
    let input = r#"{"a": {"b": [1, 2, {"c": true}]}}"#;
    let result = compact_json(input).unwrap();
    assert!(!result.contains(' '));
}

#[test]
fn test_unicode_json() {
    let input = r#"{"emoji": "🎉", "japanese": "日本語"}"#;
    let result = format_json(input).unwrap();
    assert!(result.contains("🎉"));
    assert!(result.contains("日本語"));
}

#[test]
fn test_sort_preserves_array_order() {
    let input = r#"{"arr": [3, 1, 2]}"#;
    let result = sort_json_keys(input).unwrap();
    // Array order must be preserved
    let three_pos = result.find('3').unwrap();
    let one_pos = result.find('1').unwrap();
    let two_pos = result.find('2').unwrap();
    assert!(three_pos < one_pos);
    assert!(one_pos < two_pos);
}

#[test]
fn test_json_path_array_out_of_bounds() {
    let input = r#"{"items": [1, 2, 3]}"#;
    let result = json_to_path(input, "items.5");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("out of bounds"));
}

#[test]
fn test_validate_array() {
    assert!(validate_json("[1, 2, 3]").is_ok());
}

#[test]
fn test_validate_string() {
    assert!(validate_json("\"hello\"").is_ok());
}

#[test]
fn test_validate_number() {
    assert!(validate_json("42").is_ok());
}
