use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

use notepadppp::editor::document::Document;
use notepadppp::editor::tab_manager::TabManager;

// ── add_document ──

#[test]
fn add_document_appends_to_end() {
    let mut mgr = TabManager::new();
    assert_eq!(mgr.tab_count(), 1);

    let doc = Document::from_str("new doc");
    let idx = mgr.add_document(doc);
    assert_eq!(idx, 1);
    assert_eq!(mgr.tab_count(), 2);
}

#[test]
fn add_document_sets_active() {
    let mut mgr = TabManager::new();
    let doc = Document::from_str("doc");
    let idx = mgr.add_document(doc);
    assert_eq!(mgr.active_index(), idx);
}

#[test]
fn add_document_returns_correct_index() {
    let mut mgr = TabManager::new();
    let idx1 = mgr.add_document(Document::new());
    let idx2 = mgr.add_document(Document::new());
    assert_eq!(idx1, 1);
    assert_eq!(idx2, 2);
}

#[test]
fn add_document_with_path() {
    let mut mgr = TabManager::new();
    let doc = Document::new().with_path(PathBuf::from("/tmp/test.rs"));
    let idx = mgr.add_document(doc);
    assert_eq!(mgr.get_tab_title(idx), "test.rs");
}

#[test]
fn add_document_without_path_gets_untitled() {
    let mut mgr = TabManager::new();
    let doc = Document::new();
    let idx = mgr.add_document(doc);
    let title = mgr.get_tab_title(idx);
    assert!(title.starts_with("Untitled"), "Expected 'Untitled N', got '{}'", title);
}

// ── remove_document ──

#[test]
fn remove_document_returns_doc() {
    let mut mgr = TabManager::new();
    mgr.add_document(Document::from_str("content"));
    let removed = mgr.remove_document(1);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().buffer.text(), "content");
}

#[test]
fn remove_document_adjusts_active() {
    let mut mgr = TabManager::new();
    mgr.add_document(Document::new());
    mgr.add_document(Document::new());
    // active is 2
    assert_eq!(mgr.active_index(), 2);
    mgr.remove_document(2);
    // active should now be 1 (last valid index)
    assert_eq!(mgr.active_index(), 1);
}

#[test]
fn remove_document_out_of_bounds() {
    let mut mgr = TabManager::new();
    let result = mgr.remove_document(99);
    assert!(result.is_none());
}

#[test]
fn remove_document_does_not_auto_create() {
    let mut mgr = TabManager::new();
    mgr.add_document(Document::new());
    // Remove both (remove_document doesn't auto-create like close_tab)
    mgr.remove_document(1);
    mgr.remove_document(0);
    assert_eq!(mgr.tab_count(), 0);
}

// ── reload_tab ──

#[test]
fn reload_tab_updates_content() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "original content").unwrap();

    let mut mgr = TabManager::new();
    let idx = mgr.open_file(f.path().to_path_buf()).unwrap();

    // Modify the file on disk
    std::fs::write(f.path(), "updated content").unwrap();

    mgr.reload_tab(idx).unwrap();
    let doc = mgr.get_document(idx).unwrap();
    assert_eq!(doc.buffer.text(), "updated content");
}

#[test]
fn reload_tab_resets_modified() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "content").unwrap();

    let mut mgr = TabManager::new();
    let idx = mgr.open_file(f.path().to_path_buf()).unwrap();

    // Make a modification
    mgr.get_document_mut(idx).unwrap().buffer.insert(0, "edit ");
    assert!(mgr.get_document(idx).unwrap().is_modified());

    // Reload
    mgr.reload_tab(idx).unwrap();
    assert!(!mgr.get_document(idx).unwrap().is_modified());
}

#[test]
fn reload_tab_no_path_fails() {
    let mut mgr = TabManager::new();
    let result = mgr.reload_tab(0);
    assert!(result.is_err());
}

#[test]
fn reload_tab_out_of_bounds_fails() {
    let mut mgr = TabManager::new();
    let result = mgr.reload_tab(99);
    assert!(result.is_err());
}

// ── get_document out of range ──

#[test]
fn get_document_out_of_range() {
    let mgr = TabManager::new();
    assert!(mgr.get_document(99).is_none());
}

#[test]
fn get_document_valid() {
    let mgr = TabManager::new();
    assert!(mgr.get_document(0).is_some());
}

// ── get_document_mut out of range ──

#[test]
fn get_document_mut_out_of_range() {
    let mut mgr = TabManager::new();
    assert!(mgr.get_document_mut(99).is_none());
}

#[test]
fn get_document_mut_valid() {
    let mut mgr = TabManager::new();
    assert!(mgr.get_document_mut(0).is_some());
}

#[test]
fn get_document_mut_can_modify() {
    let mut mgr = TabManager::new();
    if let Some(doc) = mgr.get_document_mut(0) {
        doc.buffer.insert(0, "hello");
    }
    assert_eq!(mgr.get_document(0).unwrap().buffer.text(), "hello");
}

// ── close_tabs_to_left ──

#[test]
fn close_tabs_to_left_removes_tabs_before_index() {
    let mut mgr = TabManager::new();
    mgr.new_tab(); // idx 1
    mgr.new_tab(); // idx 2
    mgr.new_tab(); // idx 3
    assert_eq!(mgr.tab_count(), 4);
    mgr.close_tabs_to_left(2);
    assert_eq!(mgr.tab_count(), 2);
}

#[test]
fn close_tabs_to_left_at_zero_does_nothing() {
    let mut mgr = TabManager::new();
    mgr.new_tab();
    mgr.new_tab();
    assert_eq!(mgr.tab_count(), 3);
    mgr.close_tabs_to_left(0);
    assert_eq!(mgr.tab_count(), 3);
}

#[test]
fn close_tabs_to_left_adjusts_active_tab() {
    let mut mgr = TabManager::new();
    mgr.new_tab();
    mgr.new_tab();
    mgr.set_active(0); // active is before the kept index
    mgr.close_tabs_to_left(2);
    assert_eq!(mgr.active_index(), 0); // snaps to 0
}

#[test]
fn close_tabs_to_left_active_after_index_adjusts() {
    let mut mgr = TabManager::new();
    mgr.new_tab(); // 1
    mgr.new_tab(); // 2
    mgr.new_tab(); // 3
    mgr.set_active(3);
    mgr.close_tabs_to_left(2);
    assert_eq!(mgr.active_index(), 1); // 3 - 2 = 1
}

#[test]
fn close_tabs_to_left_out_of_bounds_does_nothing() {
    let mut mgr = TabManager::new();
    mgr.new_tab();
    assert_eq!(mgr.tab_count(), 2);
    mgr.close_tabs_to_left(10);
    assert_eq!(mgr.tab_count(), 2);
}

// ── close_unmodified ──

#[test]
fn close_unmodified_removes_unmodified_tabs() {
    let mut mgr = TabManager::new();
    mgr.new_tab();
    mgr.new_tab();
    // All tabs are unmodified, so should end up with 1 new untitled tab
    mgr.close_unmodified();
    assert_eq!(mgr.tab_count(), 1);
}

#[test]
fn close_unmodified_keeps_modified_tabs() {
    let mut mgr = TabManager::new();
    mgr.new_tab();
    // Modify tab 1
    mgr.get_document_mut(1).unwrap().buffer.insert(0, "changed");
    assert_eq!(mgr.tab_count(), 2);
    mgr.close_unmodified();
    assert_eq!(mgr.tab_count(), 1);
    assert!(mgr.get_document(0).unwrap().is_modified());
}

#[test]
fn close_unmodified_all_modified_keeps_all() {
    let mut mgr = TabManager::new();
    mgr.get_document_mut(0).unwrap().buffer.insert(0, "a");
    mgr.new_tab();
    mgr.get_document_mut(1).unwrap().buffer.insert(0, "b");
    assert_eq!(mgr.tab_count(), 2);
    mgr.close_unmodified();
    assert_eq!(mgr.tab_count(), 2);
}
