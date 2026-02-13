use std::io::Write;
use tempfile::NamedTempFile;

use notepadppp::editor::tab_manager::TabManager;

#[test]
fn new_manager_has_one_tab() {
    let mgr = TabManager::new();
    assert_eq!(mgr.tab_count(), 1);
    assert_eq!(mgr.active_index(), 0);
}

#[test]
fn create_new_tabs() {
    let mut mgr = TabManager::new();
    assert_eq!(mgr.tab_count(), 1);
    let idx = mgr.new_tab();
    assert_eq!(mgr.tab_count(), 2);
    assert_eq!(mgr.active_index(), idx);
}

#[test]
fn tab_titles_untitled() {
    let mut mgr = TabManager::new();
    let title0 = mgr.get_tab_title(0);
    assert!(title0.starts_with("Untitled"), "Expected 'Untitled N', got '{}'", title0);

    mgr.new_tab();
    let title1 = mgr.get_tab_title(1);
    assert!(title1.starts_with("Untitled"));
    assert_ne!(title0, title1);
}

#[test]
fn open_file() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "File content here\n").unwrap();

    let mut mgr = TabManager::new();
    let idx = mgr.open_file(f.path().to_path_buf()).unwrap();
    assert_eq!(mgr.tab_count(), 2);
    assert_eq!(mgr.active_index(), idx);

    let doc = mgr.active_document();
    assert_eq!(doc.buffer.text(), "File content here\n");
    assert!(doc.path.is_some());
}

#[test]
fn open_file_tab_title() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("myfile.txt");
    std::fs::write(&path, "hello").unwrap();

    let mut mgr = TabManager::new();
    let idx = mgr.open_file(path).unwrap();
    assert_eq!(mgr.get_tab_title(idx), "myfile.txt");
}

#[test]
fn close_unmodified_tab() {
    let mut mgr = TabManager::new();
    mgr.new_tab();
    assert_eq!(mgr.tab_count(), 2);

    let result = mgr.close_tab(0);
    assert!(result.is_none(), "Unmodified doc should return None");
    assert_eq!(mgr.tab_count(), 1);
}

#[test]
fn close_modified_tab_returns_document() {
    let mut mgr = TabManager::new();
    mgr.active_document_mut().buffer.insert(0, "modified");
    assert!(mgr.active_document().is_modified());

    let result = mgr.close_tab(0);
    assert!(result.is_some(), "Modified doc should be returned");
}

#[test]
fn close_last_tab_creates_new() {
    let mut mgr = TabManager::new();
    assert_eq!(mgr.tab_count(), 1);
    mgr.close_tab(0);
    assert_eq!(mgr.tab_count(), 1, "Closing last tab should create a new one");
}

#[test]
fn close_tab_adjusts_active_before() {
    let mut mgr = TabManager::new();
    mgr.new_tab();
    mgr.new_tab();
    // active is 2 (last tab)
    assert_eq!(mgr.active_index(), 2);

    // Close tab 0, active should shift from 2 to 1
    mgr.close_tab(0);
    assert_eq!(mgr.active_index(), 1);
}

#[test]
fn close_tab_adjusts_active_at_end() {
    let mut mgr = TabManager::new();
    mgr.new_tab();
    // active is 1
    mgr.close_tab(1);
    assert_eq!(mgr.active_index(), 0);
}

#[test]
fn close_all() {
    let mut mgr = TabManager::new();
    mgr.new_tab();
    mgr.new_tab();
    assert_eq!(mgr.tab_count(), 3);

    mgr.close_all();
    assert_eq!(mgr.tab_count(), 1);
    assert_eq!(mgr.active_index(), 0);
}

#[test]
fn save_and_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("save_test.txt");

    let mut mgr = TabManager::new();
    mgr.active_document_mut().buffer.insert(0, "saved content");
    mgr.save_tab_as(0, path.clone()).unwrap();

    assert!(!mgr.active_document().is_modified());
    assert_eq!(mgr.get_tab_title(0), "save_test.txt");

    // Re-open and verify
    let idx = mgr.open_file(path).unwrap();
    assert_eq!(mgr.get_document(idx).unwrap().buffer.text(), "saved content");
}

#[test]
fn save_tab_to_existing_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("existing.txt");
    std::fs::write(&path, "original").unwrap();

    let mut mgr = TabManager::new();
    let idx = mgr.open_file(path.clone()).unwrap();
    mgr.get_document_mut(idx).unwrap().buffer.insert(8, " updated");
    mgr.save_tab(idx).unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content, "original updated");
}

#[test]
fn save_tab_no_path_fails() {
    let mut mgr = TabManager::new();
    let result = mgr.save_tab(0);
    assert!(result.is_err());
}

#[test]
fn set_active_tab() {
    let mut mgr = TabManager::new();
    mgr.new_tab();
    mgr.new_tab();

    mgr.set_active(0);
    assert_eq!(mgr.active_index(), 0);
    mgr.set_active(2);
    assert_eq!(mgr.active_index(), 2);
}

#[test]
fn set_active_out_of_range() {
    let mut mgr = TabManager::new();
    mgr.set_active(99);
    assert_eq!(mgr.active_index(), 0);
}

#[test]
fn move_tab_forward() {
    let mut mgr = TabManager::new();
    mgr.new_tab();
    mgr.new_tab();

    // Get titles before
    let t0 = mgr.get_tab_title(0);
    let t1 = mgr.get_tab_title(1);

    mgr.set_active(0);
    mgr.move_tab(0, 1);

    assert_eq!(mgr.get_tab_title(0), t1);
    assert_eq!(mgr.get_tab_title(1), t0);
    assert_eq!(mgr.active_index(), 1); // active followed the moved tab
}

#[test]
fn move_tab_backward() {
    let mut mgr = TabManager::new();
    mgr.new_tab();
    mgr.new_tab();

    let t2 = mgr.get_tab_title(2);
    mgr.set_active(2);
    mgr.move_tab(2, 0);

    assert_eq!(mgr.get_tab_title(0), t2);
    assert_eq!(mgr.active_index(), 0);
}

#[test]
fn is_any_modified_false() {
    let mgr = TabManager::new();
    assert!(!mgr.is_any_modified());
}

#[test]
fn is_any_modified_true() {
    let mut mgr = TabManager::new();
    mgr.new_tab();
    mgr.active_document_mut().buffer.insert(0, "text");
    assert!(mgr.is_any_modified());
}

#[test]
fn move_tab_same_position() {
    let mut mgr = TabManager::new();
    mgr.new_tab();
    let t0 = mgr.get_tab_title(0);
    let t1 = mgr.get_tab_title(1);
    mgr.move_tab(0, 0);
    assert_eq!(mgr.get_tab_title(0), t0);
    assert_eq!(mgr.get_tab_title(1), t1);
}

#[test]
fn get_tab_title_out_of_range() {
    let mgr = TabManager::new();
    assert_eq!(mgr.get_tab_title(99), "");
}
