use notepadppp::editor::bookmarks::BookmarkManager;

#[test]
fn test_toggle_bookmark() {
    let mut bm = BookmarkManager::default();
    assert!(!bm.is_bookmarked(5));
    bm.toggle(5);
    assert!(bm.is_bookmarked(5));
    bm.toggle(5);
    assert!(!bm.is_bookmarked(5));
}

#[test]
fn test_add_remove() {
    let mut bm = BookmarkManager::default();
    bm.add(3);
    bm.add(7);
    assert!(bm.is_bookmarked(3));
    assert!(bm.is_bookmarked(7));
    assert!(!bm.is_bookmarked(5));
    bm.remove(3);
    assert!(!bm.is_bookmarked(3));
    assert!(bm.is_bookmarked(7));
}

#[test]
fn test_count_and_clear() {
    let mut bm = BookmarkManager::default();
    assert_eq!(bm.count(), 0);
    bm.add(1);
    bm.add(5);
    bm.add(10);
    assert_eq!(bm.count(), 3);
    bm.clear();
    assert_eq!(bm.count(), 0);
    assert!(!bm.is_bookmarked(1));
}

#[test]
fn test_all_bookmarks_sorted() {
    let mut bm = BookmarkManager::default();
    bm.add(10);
    bm.add(2);
    bm.add(7);
    assert_eq!(bm.all_bookmarks(), vec![2, 7, 10]);
}

#[test]
fn test_next_bookmark_no_wrap() {
    let mut bm = BookmarkManager::default();
    bm.add(3);
    bm.add(7);
    bm.add(12);
    assert_eq!(bm.next_bookmark(0), Some(3));
    assert_eq!(bm.next_bookmark(3), Some(7));
    assert_eq!(bm.next_bookmark(7), Some(12));
}

#[test]
fn test_next_bookmark_wrap() {
    let mut bm = BookmarkManager::default();
    bm.add(3);
    bm.add(7);
    // After last bookmark, wraps to first
    assert_eq!(bm.next_bookmark(7), Some(3));
    assert_eq!(bm.next_bookmark(100), Some(3));
}

#[test]
fn test_prev_bookmark_no_wrap() {
    let mut bm = BookmarkManager::default();
    bm.add(3);
    bm.add(7);
    bm.add(12);
    assert_eq!(bm.prev_bookmark(12), Some(7));
    assert_eq!(bm.prev_bookmark(7), Some(3));
}

#[test]
fn test_prev_bookmark_wrap() {
    let mut bm = BookmarkManager::default();
    bm.add(3);
    bm.add(7);
    // Before first bookmark, wraps to last
    assert_eq!(bm.prev_bookmark(3), Some(7));
    assert_eq!(bm.prev_bookmark(0), Some(7));
}

#[test]
fn test_next_prev_empty() {
    let bm = BookmarkManager::default();
    assert_eq!(bm.next_bookmark(0), None);
    assert_eq!(bm.prev_bookmark(0), None);
}

#[test]
fn test_next_prev_single() {
    let mut bm = BookmarkManager::default();
    bm.add(5);
    assert_eq!(bm.next_bookmark(0), Some(5));
    assert_eq!(bm.next_bookmark(5), Some(5));
    assert_eq!(bm.prev_bookmark(10), Some(5));
    assert_eq!(bm.prev_bookmark(5), Some(5));
}

#[test]
fn test_bookmarked_lines() {
    let mut bm = BookmarkManager::default();
    let text = "line zero\nline one\nline two\nline three";
    bm.add(0);
    bm.add(2);
    let lines = bm.bookmarked_lines(text);
    assert_eq!(lines, vec!["line zero", "line two"]);
}

#[test]
fn test_bookmarked_lines_out_of_range() {
    let mut bm = BookmarkManager::default();
    let text = "hello\nworld";
    bm.add(0);
    bm.add(99); // out of range
    let lines = bm.bookmarked_lines(text);
    assert_eq!(lines, vec!["hello"]);
}

#[test]
fn test_remove_bookmarked_lines() {
    let mut bm = BookmarkManager::default();
    let text = "keep\nremove\nkeep2\nremove2";
    bm.add(1);
    bm.add(3);
    let result = bm.remove_bookmarked_lines(text);
    assert_eq!(result, "keep\nkeep2");
}

#[test]
fn test_remove_unbookmarked_lines() {
    let mut bm = BookmarkManager::default();
    let text = "remove\nkeep\nremove2\nkeep2";
    bm.add(1);
    bm.add(3);
    let result = bm.remove_unbookmarked_lines(text);
    assert_eq!(result, "keep\nkeep2");
}

#[test]
fn test_remove_bookmarked_lines_empty_bookmarks() {
    let bm = BookmarkManager::default();
    let text = "line1\nline2\nline3";
    let result = bm.remove_bookmarked_lines(text);
    assert_eq!(result, "line1\nline2\nline3");
}

#[test]
fn test_remove_unbookmarked_lines_all_bookmarked() {
    let mut bm = BookmarkManager::default();
    let text = "a\nb\nc";
    bm.add(0);
    bm.add(1);
    bm.add(2);
    let result = bm.remove_unbookmarked_lines(text);
    assert_eq!(result, "a\nb\nc");
}

#[test]
fn test_duplicate_add() {
    let mut bm = BookmarkManager::default();
    bm.add(5);
    bm.add(5);
    assert_eq!(bm.count(), 1);
}
