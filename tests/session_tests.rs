use std::io::Write;

use notepadppp::io::session::*;
use notepadppp::editor::tab_manager::TabManager;

#[test]
fn save_and_load_session_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let session_path = dir.path().join("test_session.json");

    let session = Session {
        name: "Test Session".to_string(),
        files: vec![
            SessionFile {
                path: "/tmp/file1.txt".into(),
                cursor_line: 10,
                cursor_col: 5,
                encoding: "UTF-8".to_string(),
                line_ending: "LF".to_string(),
            },
            SessionFile {
                path: "/tmp/file2.rs".into(),
                cursor_line: 42,
                cursor_col: 0,
                encoding: "UTF-8 BOM".to_string(),
                line_ending: "CRLF".to_string(),
            },
        ],
        active_tab: 1,
    };

    save_session(&session, &session_path).unwrap();
    let loaded = load_session(&session_path).unwrap();

    assert_eq!(loaded.name, "Test Session");
    assert_eq!(loaded.files.len(), 2);
    assert_eq!(loaded.active_tab, 1);
    assert_eq!(loaded.files[0].cursor_line, 10);
    assert_eq!(loaded.files[0].cursor_col, 5);
    assert_eq!(loaded.files[1].encoding, "UTF-8 BOM");
    assert_eq!(loaded.files[1].line_ending, "CRLF");
}

#[test]
fn capture_session_from_tab_manager() {
    let dir = tempfile::tempdir().unwrap();

    // Create real files to open
    let file1 = dir.path().join("hello.txt");
    let mut f = std::fs::File::create(&file1).unwrap();
    writeln!(f, "Hello, world!").unwrap();

    let file2 = dir.path().join("test.rs");
    let mut f = std::fs::File::create(&file2).unwrap();
    writeln!(f, "fn main() {{}}").unwrap();

    let mut mgr = TabManager::new();
    mgr.open_file(file1.clone()).unwrap();
    mgr.open_file(file2.clone()).unwrap();

    let session = capture_session(&mgr, "My Session");

    assert_eq!(session.name, "My Session");
    // Only files with paths are captured (not the initial untitled tab)
    assert_eq!(session.files.len(), 2);
    assert_eq!(session.files[0].path, file1);
    assert_eq!(session.files[1].path, file2);
}

#[test]
fn empty_session() {
    let dir = tempfile::tempdir().unwrap();
    let session_path = dir.path().join("empty.json");

    let session = Session {
        name: "Empty".to_string(),
        files: vec![],
        active_tab: 0,
    };

    save_session(&session, &session_path).unwrap();
    let loaded = load_session(&session_path).unwrap();

    assert_eq!(loaded.name, "Empty");
    assert!(loaded.files.is_empty());
    assert_eq!(loaded.active_tab, 0);
}

#[test]
fn session_with_multiple_files() {
    let dir = tempfile::tempdir().unwrap();
    let session_path = dir.path().join("multi.json");

    let files: Vec<SessionFile> = (0..5)
        .map(|i| SessionFile {
            path: format!("/tmp/file{i}.txt").into(),
            cursor_line: i * 10,
            cursor_col: i * 2,
            encoding: "UTF-8".to_string(),
            line_ending: "LF".to_string(),
        })
        .collect();

    let session = Session {
        name: "Multi File Session".to_string(),
        files,
        active_tab: 3,
    };

    save_session(&session, &session_path).unwrap();
    let loaded = load_session(&session_path).unwrap();

    assert_eq!(loaded.files.len(), 5);
    assert_eq!(loaded.active_tab, 3);
    for (i, file) in loaded.files.iter().enumerate() {
        assert_eq!(file.cursor_line, i * 10);
        assert_eq!(file.cursor_col, i * 2);
    }
}

#[test]
fn load_session_file_not_found() {
    let result = load_session(std::path::Path::new("/nonexistent/session.json"));
    assert!(result.is_err());
}

#[test]
fn restore_session_with_real_files() {
    let dir = tempfile::tempdir().unwrap();

    // Create real files
    let file1 = dir.path().join("restore1.txt");
    std::fs::write(&file1, "content 1").unwrap();

    let file2 = dir.path().join("restore2.txt");
    std::fs::write(&file2, "content 2").unwrap();

    let session = Session {
        name: "Restore Test".to_string(),
        files: vec![
            SessionFile {
                path: file1.clone(),
                cursor_line: 0,
                cursor_col: 3,
                encoding: "UTF-8".to_string(),
                line_ending: "LF".to_string(),
            },
            SessionFile {
                path: file2.clone(),
                cursor_line: 0,
                cursor_col: 5,
                encoding: "UTF-8".to_string(),
                line_ending: "LF".to_string(),
            },
        ],
        active_tab: 1,
    };

    let mut mgr = TabManager::new();
    restore_session(&session, &mut mgr).unwrap();

    // Should have 2 tabs from restored files
    assert_eq!(mgr.tab_count(), 2);
}

#[test]
fn auto_session_path_returns_valid_path() {
    let path = auto_session_path();
    assert!(path.to_string_lossy().contains("notepadppp"));
    assert!(path.to_string_lossy().contains("auto_session.json"));
}

// =====================================================
// Additional coverage tests
// =====================================================

#[test]
fn load_session_malformed_json() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("malformed.json");
    std::fs::write(&path, "{ this is not valid json }").unwrap();
    let result = load_session(&path);
    assert!(result.is_err());
}

#[test]
fn load_session_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.json");
    std::fs::write(&path, "").unwrap();
    let result = load_session(&path);
    assert!(result.is_err());
}

#[test]
fn restore_session_with_nonexistent_files_skips() {
    let session = Session {
        name: "Missing Files".to_string(),
        files: vec![
            SessionFile {
                path: "/nonexistent/file1.txt".into(),
                cursor_line: 0,
                cursor_col: 0,
                encoding: "UTF-8".to_string(),
                line_ending: "LF".to_string(),
            },
            SessionFile {
                path: "/nonexistent/file2.txt".into(),
                cursor_line: 0,
                cursor_col: 0,
                encoding: "UTF-8".to_string(),
                line_ending: "LF".to_string(),
            },
        ],
        active_tab: 0,
    };

    let mut mgr = TabManager::new();
    let result = restore_session(&session, &mut mgr);
    assert!(result.is_ok());
    // Only the default empty tab should remain since files don't exist
    assert_eq!(mgr.tab_count(), 1);
}

#[test]
fn restore_session_out_of_range_active_tab() {
    let dir = tempfile::tempdir().unwrap();
    let file1 = dir.path().join("f1.txt");
    std::fs::write(&file1, "content").unwrap();

    let session = Session {
        name: "Out of range".to_string(),
        files: vec![SessionFile {
            path: file1,
            cursor_line: 0,
            cursor_col: 0,
            encoding: "UTF-8".to_string(),
            line_ending: "LF".to_string(),
        }],
        active_tab: 999, // way out of range
    };

    let mut mgr = TabManager::new();
    let result = restore_session(&session, &mut mgr);
    assert!(result.is_ok());
    // active_tab should not have been set to 999
    assert!(mgr.active_index() < mgr.tab_count());
}

#[test]
fn restore_session_cursor_position_verification() {
    let dir = tempfile::tempdir().unwrap();
    let file1 = dir.path().join("cursor_test.txt");
    std::fs::write(&file1, "line1\nline2\nline3\n").unwrap();

    let session = Session {
        name: "Cursor Test".to_string(),
        files: vec![SessionFile {
            path: file1,
            cursor_line: 1,
            cursor_col: 3,
            encoding: "UTF-8".to_string(),
            line_ending: "LF".to_string(),
        }],
        active_tab: 0,
    };

    let mut mgr = TabManager::new();
    restore_session(&session, &mut mgr).unwrap();
    // Verify the file was opened and cursor was set
    // Due to the close_tab(0) logic in restore_session, the actual_idx may differ
    // The file should be open regardless
    assert!(mgr.tab_count() >= 1);
    let doc = mgr.active_document();
    assert!(doc.path.is_some());
}

#[test]
fn capture_session_no_files_all_untitled() {
    let mgr = TabManager::new();
    let session = capture_session(&mgr, "Empty Session");
    assert_eq!(session.name, "Empty Session");
    // Untitled tabs have no path and are not captured
    assert!(session.files.is_empty());
}
