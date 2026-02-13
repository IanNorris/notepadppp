use notepadppp::tools::csv_viewer::*;

#[test]
fn test_parse_simple_csv() {
    let text = "a,b,c\n1,2,3\n4,5,6\n";
    let data = parse_csv(text, ',', true);
    assert_eq!(data.headers, vec!["a", "b", "c"]);
    assert_eq!(data.rows.len(), 2);
    assert_eq!(data.rows[0], vec!["1", "2", "3"]);
    assert_eq!(data.rows[1], vec!["4", "5", "6"]);
}

#[test]
fn test_parse_with_headers() {
    let text = "Name,Age,City\nAlice,30,NYC\nBob,25,LA\n";
    let data = parse_csv(text, ',', true);
    assert_eq!(data.headers, vec!["Name", "Age", "City"]);
    assert_eq!(data.rows.len(), 2);
    assert_eq!(data.rows[0], vec!["Alice", "30", "NYC"]);
    assert_eq!(data.rows[1], vec!["Bob", "25", "LA"]);
}

#[test]
fn test_parse_without_headers() {
    let text = "1,2,3\n4,5,6\n";
    let data = parse_csv(text, ',', false);
    assert_eq!(data.headers, vec!["Column 1", "Column 2", "Column 3"]);
    assert_eq!(data.rows.len(), 2);
    assert_eq!(data.rows[0], vec!["1", "2", "3"]);
}

#[test]
fn test_quoted_fields_with_commas() {
    let text = "name,desc\n\"Smith, John\",\"Has a comma, here\"\n";
    let data = parse_csv(text, ',', true);
    assert_eq!(data.rows[0][0], "Smith, John");
    assert_eq!(data.rows[0][1], "Has a comma, here");
}

#[test]
fn test_escaped_quotes() {
    let text = "a,b\n\"He said \"\"hello\"\"\",world\n";
    let data = parse_csv(text, ',', true);
    assert_eq!(data.rows[0][0], "He said \"hello\"");
    assert_eq!(data.rows[0][1], "world");
}

#[test]
fn test_tab_delimiter() {
    let text = "a\tb\tc\n1\t2\t3\n";
    let data = parse_csv(text, '\t', true);
    assert_eq!(data.headers, vec!["a", "b", "c"]);
    assert_eq!(data.rows[0], vec!["1", "2", "3"]);
}

#[test]
fn test_semicolon_delimiter() {
    let text = "a;b;c\n1;2;3\n";
    let data = parse_csv(text, ';', true);
    assert_eq!(data.headers, vec!["a", "b", "c"]);
    assert_eq!(data.rows[0], vec!["1", "2", "3"]);
}

#[test]
fn test_pipe_delimiter() {
    let text = "a|b|c\n1|2|3\n";
    let data = parse_csv(text, '|', true);
    assert_eq!(data.headers, vec!["a", "b", "c"]);
    assert_eq!(data.rows[0], vec!["1", "2", "3"]);
}

#[test]
fn test_sort_by_column_ascending() {
    let text = "name,age\nCharlie,30\nAlice,25\nBob,35\n";
    let mut data = parse_csv(text, ',', true);
    sort_by_column(&mut data, 0, true);
    assert_eq!(data.rows[0][0], "Alice");
    assert_eq!(data.rows[1][0], "Bob");
    assert_eq!(data.rows[2][0], "Charlie");
}

#[test]
fn test_sort_by_column_descending() {
    let text = "name,age\nCharlie,30\nAlice,25\nBob,35\n";
    let mut data = parse_csv(text, ',', true);
    sort_by_column(&mut data, 0, false);
    assert_eq!(data.rows[0][0], "Charlie");
    assert_eq!(data.rows[1][0], "Bob");
    assert_eq!(data.rows[2][0], "Alice");
}

#[test]
fn test_sort_numeric() {
    let text = "name,age\nCharlie,30\nAlice,5\nBob,25\n";
    let mut data = parse_csv(text, ',', true);
    sort_by_column(&mut data, 1, true);
    assert_eq!(data.rows[0][1], "5");
    assert_eq!(data.rows[1][1], "25");
    assert_eq!(data.rows[2][1], "30");
}

#[test]
fn test_filter_rows() {
    let text = "name,city\nAlice,NYC\nBob,LA\nCharlie,NYC\n";
    let data = parse_csv(text, ',', true);
    let filtered = filter_rows(&data, 1, "NYC");
    assert_eq!(filtered.rows.len(), 2);
    assert_eq!(filtered.rows[0][0], "Alice");
    assert_eq!(filtered.rows[1][0], "Charlie");
}

#[test]
fn test_filter_case_insensitive() {
    let text = "name,city\nAlice,NYC\nBob,la\n";
    let data = parse_csv(text, ',', true);
    let filtered = filter_rows(&data, 1, "LA");
    assert_eq!(filtered.rows.len(), 1);
    assert_eq!(filtered.rows[0][0], "Bob");
}

#[test]
fn test_add_column() {
    let text = "a,b\n1,2\n3,4\n";
    let mut data = parse_csv(text, ',', true);
    add_column(&mut data, "c");
    assert_eq!(data.headers, vec!["a", "b", "c"]);
    assert_eq!(data.rows[0].len(), 3);
    assert_eq!(data.rows[0][2], "");
}

#[test]
fn test_remove_column() {
    let text = "a,b,c\n1,2,3\n4,5,6\n";
    let mut data = parse_csv(text, ',', true);
    remove_column(&mut data, 1);
    assert_eq!(data.headers, vec!["a", "c"]);
    assert_eq!(data.rows[0], vec!["1", "3"]);
    assert_eq!(data.rows[1], vec!["4", "6"]);
}

#[test]
fn test_round_trip() {
    let text = "name,age,city\nAlice,30,NYC\nBob,25,LA\n";
    let data = parse_csv(text, ',', true);
    let output = to_csv(&data);
    let reparsed = parse_csv(&output, ',', true);
    assert_eq!(reparsed.headers, data.headers);
    assert_eq!(reparsed.rows, data.rows);
}

#[test]
fn test_round_trip_quoted_fields() {
    let text = "a,b\n\"hello, world\",\"she said \"\"hi\"\"\"\n";
    let data = parse_csv(text, ',', true);
    assert_eq!(data.rows[0][0], "hello, world");
    assert_eq!(data.rows[0][1], "she said \"hi\"");
    let output = to_csv(&data);
    let reparsed = parse_csv(&output, ',', true);
    assert_eq!(reparsed.rows[0][0], "hello, world");
    assert_eq!(reparsed.rows[0][1], "she said \"hi\"");
}

#[test]
fn test_empty_csv() {
    let text = "";
    let data = parse_csv(text, ',', true);
    assert!(data.headers.is_empty());
    assert!(data.rows.is_empty());
}

#[test]
fn test_single_column() {
    let text = "name\nAlice\nBob\n";
    let data = parse_csv(text, ',', true);
    assert_eq!(data.headers, vec!["name"]);
    assert_eq!(data.rows.len(), 2);
    assert_eq!(data.rows[0], vec!["Alice"]);
}

#[test]
fn test_single_row() {
    let text = "a,b,c\n1,2,3\n";
    let data = parse_csv(text, ',', true);
    assert_eq!(data.headers, vec!["a", "b", "c"]);
    assert_eq!(data.rows.len(), 1);
    assert_eq!(data.rows[0], vec!["1", "2", "3"]);
}

#[test]
fn test_newline_in_quoted_field() {
    let text = "a,b\n\"line1\nline2\",value\n";
    let data = parse_csv(text, ',', true);
    assert_eq!(data.rows[0][0], "line1\nline2");
    assert_eq!(data.rows[0][1], "value");
}

#[test]
fn test_remove_column_out_of_bounds() {
    let text = "a,b\n1,2\n";
    let mut data = parse_csv(text, ',', true);
    remove_column(&mut data, 5);
    assert_eq!(data.headers, vec!["a", "b"]);
    assert_eq!(data.rows[0], vec!["1", "2"]);
}

#[test]
fn test_sort_out_of_bounds() {
    let text = "a,b\n1,2\n";
    let mut data = parse_csv(text, ',', true);
    sort_by_column(&mut data, 10, true);
    // Should not panic, rows unchanged
    assert_eq!(data.rows[0], vec!["1", "2"]);
}

#[test]
fn test_to_csv_no_headers() {
    let data = CsvData {
        headers: vec!["Column 1".to_string(), "Column 2".to_string()],
        rows: vec![vec!["a".to_string(), "b".to_string()]],
        delimiter: ',',
        has_headers: false,
    };
    let output = to_csv(&data);
    assert_eq!(output, "a,b\n");
}

#[test]
fn test_crlf_line_endings() {
    let text = "a,b\r\n1,2\r\n3,4\r\n";
    let data = parse_csv(text, ',', true);
    assert_eq!(data.headers, vec!["a", "b"]);
    assert_eq!(data.rows.len(), 2);
    assert_eq!(data.rows[0], vec!["1", "2"]);
}
