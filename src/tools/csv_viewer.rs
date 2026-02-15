/// Parsed CSV data with headers, rows, and configuration.
pub struct CsvData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub delimiter: char,
    pub has_headers: bool,
}

/// Parse CSV text into a CsvData struct.
///
/// Handles quoted fields (with escaped quotes ""), different delimiters,
/// and newlines within quoted fields.
pub fn parse_csv(text: &str, delimiter: char, has_headers: bool) -> CsvData {
    let records = parse_records(text, delimiter);

    if records.is_empty() {
        return CsvData {
            headers: Vec::new(),
            rows: Vec::new(),
            delimiter,
            has_headers,
        };
    }

    let (headers, rows) = if has_headers && !records.is_empty() {
        let headers = records[0].clone();
        let rows = records[1..].to_vec();
        (headers, rows)
    } else {
        let col_count = records.iter().map(|r| r.len()).max().unwrap_or(0);
        let headers: Vec<String> = (0..col_count).map(|i| format!("Column {}", i + 1)).collect();
        (headers, records)
    };

    // Normalize row lengths to match header count
    let col_count = headers.len();
    let rows = rows
        .into_iter()
        .map(|mut row| {
            row.resize(col_count, String::new());
            row
        })
        .collect();

    CsvData {
        headers,
        rows,
        delimiter,
        has_headers,
    }
}

/// Parse raw text into a list of records (each record is a list of fields).
fn parse_records(text: &str, delimiter: char) -> Vec<Vec<String>> {
    let mut records: Vec<Vec<String>> = Vec::new();
    let mut current_field = String::new();
    let mut current_record: Vec<String> = Vec::new();
    let mut in_quotes = false;
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    // Escaped quote
                    current_field.push('"');
                    chars.next();
                } else {
                    // End of quoted field
                    in_quotes = false;
                }
            } else {
                current_field.push(c);
            }
        } else if c == '"' {
            in_quotes = true;
        } else if c == delimiter {
            current_record.push(current_field.clone());
            current_field.clear();
        } else if c == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            current_record.push(current_field.clone());
            current_field.clear();
            records.push(current_record.clone());
            current_record.clear();
        } else if c == '\n' {
            current_record.push(current_field.clone());
            current_field.clear();
            records.push(current_record.clone());
            current_record.clear();
        } else {
            current_field.push(c);
        }
    }

    // Push the last field/record if there's content
    if !current_field.is_empty() || !current_record.is_empty() {
        current_record.push(current_field);
        records.push(current_record);
    }

    records
}

/// Convert CsvData back to CSV text.
pub fn to_csv(data: &CsvData) -> String {
    let mut output = String::new();

    if data.has_headers {
        let header_line: Vec<String> = data.headers.iter().map(|h| escape_field(h, data.delimiter)).collect();
        output.push_str(&header_line.join(&data.delimiter.to_string()));
        output.push('\n');
    }

    for row in &data.rows {
        let line: Vec<String> = row.iter().map(|f| escape_field(f, data.delimiter)).collect();
        output.push_str(&line.join(&data.delimiter.to_string()));
        output.push('\n');
    }

    output
}

/// Escape a field for CSV output, quoting if necessary.
fn escape_field(field: &str, delimiter: char) -> String {
    if field.contains(delimiter) || field.contains('"') || field.contains('\n') || field.contains('\r') {
        let escaped = field.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        field.to_string()
    }
}

/// Sort rows by the given column index.
pub fn sort_by_column(data: &mut CsvData, col: usize, ascending: bool) {
    if col >= data.headers.len() {
        return;
    }
    data.rows.sort_by(|a, b| {
        let a_val = a.get(col).map(|s| s.as_str()).unwrap_or("");
        let b_val = b.get(col).map(|s| s.as_str()).unwrap_or("");

        // Try numeric comparison first
        let cmp = match (a_val.parse::<f64>(), b_val.parse::<f64>()) {
            (Ok(a_num), Ok(b_num)) => a_num.partial_cmp(&b_num).unwrap_or(std::cmp::Ordering::Equal),
            _ => a_val.cmp(b_val),
        };

        if ascending { cmp } else { cmp.reverse() }
    });
}

/// Filter rows where the given column matches the pattern (case-insensitive substring match).
pub fn filter_rows(data: &CsvData, col: usize, pattern: &str) -> CsvData {
    let pattern_lower = pattern.to_lowercase();
    let rows: Vec<Vec<String>> = data
        .rows
        .iter()
        .filter(|row| {
            row.get(col)
                .map(|val| val.to_lowercase().contains(&pattern_lower))
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    CsvData {
        headers: data.headers.clone(),
        rows,
        delimiter: data.delimiter,
        has_headers: data.has_headers,
    }
}

/// Add an empty column with the given name.
pub fn add_column(data: &mut CsvData, name: &str) {
    data.headers.push(name.to_string());
    for row in &mut data.rows {
        row.push(String::new());
    }
}

/// Remove a column by index.
pub fn remove_column(data: &mut CsvData, col: usize) {
    if col >= data.headers.len() {
        return;
    }
    data.headers.remove(col);
    for row in &mut data.rows {
        if col < row.len() {
            row.remove(col);
        }
    }
}

