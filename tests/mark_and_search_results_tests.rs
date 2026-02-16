use notepadppp::editor::marks::MarkManager;
use notepadppp::search::SearchEngine;
use notepadppp::editor::bookmarks::BookmarkManager;
use notepadppp::ui_iced::search_results_panel::{SearchResultsManager, SearchResultMatch};

// ── MarkManager tests ──

#[test]
fn test_mark_manager_default_empty() {
    let mm = MarkManager::default();
    assert!(mm.is_empty());
    assert_eq!(mm.count(), 0);
}

#[test]
fn test_mark_all_and_count() {
    let mut mm = MarkManager::default();
    mm.mark_all(vec![(0, 5), (10, 15), (20, 25)]);
    assert_eq!(mm.count(), 3);
    assert!(!mm.is_empty());
}

#[test]
fn test_mark_all_accumulates() {
    let mut mm = MarkManager::default();
    mm.mark_all(vec![(0, 5)]);
    mm.mark_all(vec![(10, 15)]);
    assert_eq!(mm.count(), 2);
}

#[test]
fn test_clear_marks() {
    let mut mm = MarkManager::default();
    mm.mark_all(vec![(0, 5), (10, 15)]);
    mm.clear();
    assert!(mm.is_empty());
    assert_eq!(mm.count(), 0);
}

#[test]
fn test_contains_offset() {
    let mut mm = MarkManager::default();
    mm.mark_all(vec![(5, 10), (20, 25)]);
    assert!(mm.contains(5));
    assert!(mm.contains(9));
    assert!(!mm.contains(10));
    assert!(mm.contains(20));
    assert!(mm.contains(24));
    assert!(!mm.contains(25));
    assert!(!mm.contains(0));
    assert!(!mm.contains(15));
}

#[test]
fn test_marks_returns_ranges() {
    let mut mm = MarkManager::default();
    mm.mark_all(vec![(0, 5), (10, 15)]);
    let marks = mm.marks();
    assert_eq!(marks.len(), 2);
    assert_eq!(marks[0], (0, 5));
    assert_eq!(marks[1], (10, 15));
}

// ── Integration: search engine to mark ranges ──

#[test]
fn test_search_to_mark_ranges() {
    let mut engine = SearchEngine::new();
    engine.query = "hello".to_string();
    engine.case_sensitive = false;
    let text = "hello world hello there";
    let matches = engine.find_all(text);
    let ranges: Vec<(usize, usize)> = matches.iter().map(|m| (m.start, m.end)).collect();
    
    let mut mm = MarkManager::default();
    mm.mark_all(ranges);
    assert_eq!(mm.count(), 2);
    assert!(mm.contains(0));  // "hello" at 0..5
    assert!(mm.contains(12)); // "hello" at 12..17
    assert!(!mm.contains(6)); // space
}

// ── Integration: search to bookmarks ──

#[test]
fn test_bookmark_lines_from_search() {
    let mut engine = SearchEngine::new();
    engine.query = "TODO".to_string();
    let text = "line 0\nTODO: fix this\nline 2\nTODO: another\nline 4";
    let matches = engine.find_all(text);
    
    let mut bm = BookmarkManager::default();
    for m in &matches {
        bm.add(m.line);
    }
    
    assert_eq!(bm.count(), 2);
    assert!(bm.is_bookmarked(1));
    assert!(bm.is_bookmarked(3));
    assert!(!bm.is_bookmarked(0));
    assert!(!bm.is_bookmarked(2));
    assert!(!bm.is_bookmarked(4));
}

#[test]
fn test_bookmark_lines_from_search_duplicates() {
    let mut engine = SearchEngine::new();
    engine.query = "a".to_string();
    let text = "aaa\nbbb\naxa";
    let matches = engine.find_all(text);
    
    let mut bm = BookmarkManager::default();
    for m in &matches {
        bm.add(m.line);
    }
    
    // Line 0 has 3 matches, line 2 has 2, but only 2 unique lines
    assert_eq!(bm.count(), 2);
    assert!(bm.is_bookmarked(0));
    assert!(bm.is_bookmarked(2));
}

// ── SearchResultsManager tests ──

#[test]
fn test_search_results_manager_default_empty() {
    let srm = SearchResultsManager::default();
    assert!(srm.is_empty());
    assert_eq!(srm.entry_count(), 0);
    assert_eq!(srm.total_matches(), 0);
}

#[test]
fn test_add_search_results() {
    let mut srm = SearchResultsManager::default();
    srm.add_search("hello".to_string(), vec![
        SearchResultMatch { line: 0, line_text: "hello world".to_string(), file_path: None },
        SearchResultMatch { line: 5, line_text: "say hello".to_string(), file_path: None },
    ]);
    assert_eq!(srm.entry_count(), 1);
    assert_eq!(srm.total_matches(), 2);
}

#[test]
fn test_multiple_searches_accumulate() {
    let mut srm = SearchResultsManager::default();
    srm.add_search("first".to_string(), vec![
        SearchResultMatch { line: 0, line_text: "first line".to_string(), file_path: None },
    ]);
    srm.add_search("second".to_string(), vec![
        SearchResultMatch { line: 1, line_text: "second line".to_string(), file_path: None },
        SearchResultMatch { line: 3, line_text: "also second".to_string(), file_path: None },
    ]);
    assert_eq!(srm.entry_count(), 2);
    assert_eq!(srm.total_matches(), 3);
}

#[test]
fn test_clear_search_results() {
    let mut srm = SearchResultsManager::default();
    srm.add_search("test".to_string(), vec![
        SearchResultMatch { line: 0, line_text: "test".to_string(), file_path: None },
    ]);
    srm.clear();
    assert!(srm.is_empty());
    assert_eq!(srm.entry_count(), 0);
}

#[test]
fn test_toggle_collapse() {
    let mut srm = SearchResultsManager::default();
    srm.add_search("query".to_string(), vec![
        SearchResultMatch { line: 0, line_text: "line".to_string(), file_path: None },
    ]);
    assert!(!srm.entries[0].collapsed);
    srm.toggle_collapse(0);
    assert!(srm.entries[0].collapsed);
    srm.toggle_collapse(0);
    assert!(!srm.entries[0].collapsed);
}

#[test]
fn test_toggle_collapse_out_of_bounds() {
    let mut srm = SearchResultsManager::default();
    // Should not panic
    srm.toggle_collapse(99);
}

#[test]
fn test_search_results_entries_content() {
    let mut srm = SearchResultsManager::default();
    srm.add_search("pattern".to_string(), vec![
        SearchResultMatch { line: 5, line_text: "found pattern here".to_string(), file_path: None },
        SearchResultMatch { line: 10, line_text: "another pattern".to_string(), file_path: None },
    ]);
    
    let entry = &srm.entries[0];
    assert_eq!(entry.query, "pattern");
    assert_eq!(entry.matches.len(), 2);
    assert_eq!(entry.matches[0].line, 5);
    assert_eq!(entry.matches[0].line_text, "found pattern here");
    assert_eq!(entry.matches[1].line, 10);
}
