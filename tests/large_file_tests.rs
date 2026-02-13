use notepadppp::io::large_file::{LargeFileInfo, LARGE_FILE_THRESHOLD, VERY_LARGE_FILE_THRESHOLD, LARGE_FILE_UNDO_LIMIT};

#[test]
fn small_file_not_flagged() {
    let info = LargeFileInfo::from_size(1024); // 1 KB
    assert!(!info.is_large);
    assert!(!info.is_very_large);
    assert!(!info.syntax_disabled);
    assert!(!info.minimap_disabled);
    assert!(!info.function_list_disabled);
    assert!(info.undo_limit.is_none());
}

#[test]
fn large_file_detected() {
    let info = LargeFileInfo::from_size(LARGE_FILE_THRESHOLD as u64);
    assert!(info.is_large);
    assert!(!info.is_very_large);
    assert!(info.syntax_disabled);
    assert!(!info.minimap_disabled);
    assert!(!info.function_list_disabled);
    assert_eq!(info.undo_limit, Some(LARGE_FILE_UNDO_LIMIT));
}

#[test]
fn very_large_file_detected() {
    let info = LargeFileInfo::from_size(VERY_LARGE_FILE_THRESHOLD as u64);
    assert!(info.is_large);
    assert!(info.is_very_large);
    assert!(info.syntax_disabled);
    assert!(info.minimap_disabled);
    assert!(info.function_list_disabled);
    assert_eq!(info.undo_limit, Some(LARGE_FILE_UNDO_LIMIT));
}

#[test]
fn just_below_threshold_not_large() {
    let info = LargeFileInfo::from_size((LARGE_FILE_THRESHOLD - 1) as u64);
    assert!(!info.is_large);
    assert!(!info.is_very_large);
    assert!(info.undo_limit.is_none());
}

#[test]
fn from_path_works_for_small_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("small.txt");
    std::fs::write(&path, "hello").unwrap();
    let info = LargeFileInfo::from_path(&path).unwrap();
    assert!(!info.is_large);
    assert_eq!(info.file_size, 5);
}

#[test]
fn from_path_nonexistent_returns_error() {
    let result = LargeFileInfo::from_path(std::path::Path::new("/nonexistent/file.txt"));
    assert!(result.is_err());
}

#[test]
fn size_display_mb() {
    let info = LargeFileInfo::from_size(15 * 1024 * 1024);
    assert_eq!(info.size_display(), "15.0 MB");
}

#[test]
fn size_display_kb() {
    let info = LargeFileInfo::from_size(512 * 1024);
    assert_eq!(info.size_display(), "512.0 KB");
}

#[test]
fn default_is_not_large() {
    let info = LargeFileInfo::default();
    assert!(!info.is_large);
    assert!(!info.is_very_large);
    assert_eq!(info.file_size, 0);
}

#[test]
fn undo_limit_enforced_in_buffer() {
    use notepadppp::editor::buffer::TextBuffer;

    let mut buf = TextBuffer::from_str("hello world");
    buf.set_undo_limit(Some(3));

    // Perform more than 3 edits
    for i in 0..10 {
        buf.insert(0, &format!("{}", i));
    }

    // Undo should only succeed 3 times (the limit)
    let mut undo_count = 0;
    while buf.undo() {
        undo_count += 1;
    }
    assert_eq!(undo_count, 3);
}
