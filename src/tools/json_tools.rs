use serde_json::Value;

/// Pretty-print JSON with 2-space indentation.
pub fn format_json(input: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(input).map_err(|e| format!("Invalid JSON: {}", e))?;
    serde_json::to_string_pretty(&value).map_err(|e| format!("Formatting error: {}", e))
}

/// Minify JSON to a single line with no extra whitespace.
pub fn compact_json(input: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(input).map_err(|e| format!("Invalid JSON: {}", e))?;
    serde_json::to_string(&value).map_err(|e| format!("Compacting error: {}", e))
}

/// Validate JSON and return an error with position info if invalid.
pub fn validate_json(input: &str) -> Result<(), String> {
    serde_json::from_str::<Value>(input).map_err(|e| format!("Invalid JSON at {}: {}", format_position(&e), e))?;
    Ok(())
}

fn format_position(e: &serde_json::Error) -> String {
    format!("line {} column {}", e.line(), e.column())
}

/// Sort all object keys alphabetically (recursively).
pub fn sort_json_keys(input: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(input).map_err(|e| format!("Invalid JSON: {}", e))?;
    let sorted = sort_value(value);
    serde_json::to_string_pretty(&sorted).map_err(|e| format!("Formatting error: {}", e))
}

fn sort_value(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let sorted: serde_json::Map<String, Value> = map
                .into_iter()
                .map(|(k, v)| (k, sort_value(v)))
                .collect::<Vec<_>>()
                .into_iter()
                .collect::<std::collections::BTreeMap<_, _>>()
                .into_iter()
                .collect();
            Value::Object(sorted)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(sort_value).collect()),
        other => other,
    }
}

/// Extract value at a JSON path using simple dot notation (e.g. "data.users.0.name").
pub fn json_to_path(input: &str, path: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(input).map_err(|e| format!("Invalid JSON: {}", e))?;
    let result = resolve_path(&value, path)?;
    serde_json::to_string_pretty(&result).map_err(|e| format!("Formatting error: {}", e))
}

fn resolve_path<'a>(value: &'a Value, path: &str) -> Result<&'a Value, String> {
    if path.is_empty() {
        return Ok(value);
    }

    let mut current = value;
    for segment in path.split('.') {
        current = match current {
            Value::Object(map) => map
                .get(segment)
                .ok_or_else(|| format!("Key '{}' not found", segment))?,
            Value::Array(arr) => {
                let index: usize = segment
                    .parse()
                    .map_err(|_| format!("Invalid array index '{}'", segment))?;
                arr.get(index)
                    .ok_or_else(|| format!("Array index {} out of bounds (length {})", index, arr.len()))?
            }
            _ => return Err(format!("Cannot index into {:?} with '{}'", current, segment)),
        };
    }
    Ok(current)
}
