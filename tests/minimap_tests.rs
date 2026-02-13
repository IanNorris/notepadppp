#[test]
fn test_line_count_for_minimap() {
    let text = "line1\nline2\nline3\n";
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3);
}

#[test]
fn test_minimap_scale() {
    let total_lines = 1000;
    let available_height = 500.0f32;
    let line_height = 2.5f32;
    let total_height = total_lines as f32 * line_height;
    let scale = available_height / total_height;
    assert!(scale < 1.0);
    assert!(scale > 0.0);
}
