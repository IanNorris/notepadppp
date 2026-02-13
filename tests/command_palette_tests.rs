#[test]
fn test_fuzzy_filter() {
    let commands = vec!["New File", "Open File", "Save", "Save As", "Find", "Replace"];
    let query = "fi";
    let filtered: Vec<&&str> = commands.iter()
        .filter(|c| c.to_lowercase().contains(query))
        .collect();
    assert_eq!(filtered.len(), 3); // "New File", "Open File", "Find"
}

#[test]
fn test_empty_query_shows_all() {
    let commands = vec!["New File", "Open File", "Save"];
    let query = "";
    let filtered: Vec<&&str> = commands.iter()
        .filter(|c| query.is_empty() || c.to_lowercase().contains(query))
        .collect();
    assert_eq!(filtered.len(), 3);
}

#[test]
fn test_no_match() {
    let commands = vec!["New File", "Open File", "Save"];
    let query = "xyz";
    let filtered: Vec<&&str> = commands.iter()
        .filter(|c| c.to_lowercase().contains(query))
        .collect();
    assert_eq!(filtered.len(), 0);
}
