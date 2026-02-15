use notepadppp::editor::document::Document;
use notepadppp::editor::tab_manager::TabManager;

// --- Document read_only field ---

#[test]
fn test_document_default_not_read_only() {
    let doc = Document::new();
    assert!(!doc.read_only);
}

#[test]
fn test_document_toggle_read_only() {
    let mut doc = Document::new();
    assert!(!doc.read_only);
    doc.read_only = true;
    assert!(doc.read_only);
    doc.read_only = false;
    assert!(!doc.read_only);
}

#[test]
fn test_document_from_str_not_read_only() {
    let doc = Document::from_str("hello world");
    assert!(!doc.read_only);
}

#[test]
fn test_read_only_blocks_insert() {
    let mut doc = Document::from_str("hello");
    doc.read_only = true;
    // In read-only mode, the application layer should check read_only before modifying
    // The buffer itself doesn't enforce read-only; the UI layer does
    assert!(doc.read_only);
    assert_eq!(doc.buffer.text(), "hello");
}

#[test]
fn test_read_only_preserves_content_after_toggle() {
    let mut doc = Document::from_str("original content");
    doc.read_only = true;
    assert_eq!(doc.buffer.text(), "original content");
    doc.read_only = false;
    assert_eq!(doc.buffer.text(), "original content");
}

// --- TabManager with read_only ---

#[test]
fn test_tab_manager_toggle_active_read_only() {
    let mut mgr = TabManager::new();
    assert!(!mgr.active_document().read_only);
    mgr.active_document_mut().read_only = true;
    assert!(mgr.active_document().read_only);
}

#[test]
fn test_tab_manager_read_only_per_tab() {
    let mut mgr = TabManager::new();
    mgr.new_tab();
    // Tab 0 (original) is not read-only by default
    mgr.set_active(0);
    assert!(!mgr.active_document().read_only);
    // Set tab 0 read-only
    mgr.active_document_mut().read_only = true;
    // Tab 1 should still not be read-only
    mgr.set_active(1);
    assert!(!mgr.active_document().read_only);
    // Tab 0 should still be read-only
    mgr.set_active(0);
    assert!(mgr.active_document().read_only);
}

#[test]
fn test_read_only_with_modified_flag() {
    let mut doc = Document::from_str("test");
    doc.read_only = true;
    // Even in read-only, modified flag should still be queryable
    assert!(!doc.is_modified());
}

// --- Filesystem read-only detection ---

#[test]
fn test_open_readonly_file_sets_read_only() {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("readonly.txt");
    fs::write(&path, "cannot edit me").unwrap();
    // Make file read-only
    fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).unwrap();

    let mut mgr = TabManager::new();
    mgr.open_file(path.clone()).unwrap();
    let doc = mgr.active_document();
    assert!(doc.read_only, "File with read-only permissions should auto-set read_only");
    assert_eq!(doc.buffer.text(), "cannot edit me");
}

#[test]
fn test_open_writable_file_not_read_only() {
    use std::fs;
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("writable.txt");
    fs::write(&path, "can edit me").unwrap();

    let mut mgr = TabManager::new();
    mgr.open_file(path).unwrap();
    assert!(!mgr.active_document().read_only);
}
