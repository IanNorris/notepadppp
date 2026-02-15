use std::path::PathBuf;
use notepadppp::io::recent_files::RecentFiles;

// ── new ──

#[test]
fn new_has_empty_list() {
    let rf = RecentFiles::new();
    assert!(rf.list().is_empty());
}

// ── with_max ──

#[test]
fn with_max_custom() {
    let rf = RecentFiles::with_max(5);
    assert!(rf.list().is_empty());
}

// ── add ──

#[test]
fn add_single_entry() {
    let mut rf = RecentFiles::new();
    rf.add(PathBuf::from("/tmp/test.txt"));
    assert_eq!(rf.list().len(), 1);
    assert_eq!(rf.list()[0], PathBuf::from("/tmp/test.txt"));
}

#[test]
fn add_multiple_entries() {
    let mut rf = RecentFiles::new();
    rf.add(PathBuf::from("/a"));
    rf.add(PathBuf::from("/b"));
    rf.add(PathBuf::from("/c"));
    assert_eq!(rf.list().len(), 3);
    // Most recent first
    assert_eq!(rf.list()[0], PathBuf::from("/c"));
    assert_eq!(rf.list()[1], PathBuf::from("/b"));
    assert_eq!(rf.list()[2], PathBuf::from("/a"));
}

#[test]
fn add_dedup_moves_to_front() {
    let mut rf = RecentFiles::new();
    rf.add(PathBuf::from("/a"));
    rf.add(PathBuf::from("/b"));
    rf.add(PathBuf::from("/a")); // re-add
    assert_eq!(rf.list().len(), 2);
    assert_eq!(rf.list()[0], PathBuf::from("/a"));
    assert_eq!(rf.list()[1], PathBuf::from("/b"));
}

#[test]
fn add_truncates_at_max() {
    let mut rf = RecentFiles::with_max(3);
    rf.add(PathBuf::from("/a"));
    rf.add(PathBuf::from("/b"));
    rf.add(PathBuf::from("/c"));
    rf.add(PathBuf::from("/d"));
    assert_eq!(rf.list().len(), 3);
    assert_eq!(rf.list()[0], PathBuf::from("/d"));
    // /a should have been evicted
    assert!(!rf.list().contains(&PathBuf::from("/a")));
}

// ── remove ──

#[test]
fn remove_existing() {
    let mut rf = RecentFiles::new();
    rf.add(PathBuf::from("/a"));
    rf.add(PathBuf::from("/b"));
    rf.remove(&PathBuf::from("/a"));
    assert_eq!(rf.list().len(), 1);
    assert_eq!(rf.list()[0], PathBuf::from("/b"));
}

#[test]
fn remove_non_existing() {
    let mut rf = RecentFiles::new();
    rf.add(PathBuf::from("/a"));
    rf.remove(&PathBuf::from("/nonexistent"));
    assert_eq!(rf.list().len(), 1);
}

// ── list ──

#[test]
fn list_empty() {
    let rf = RecentFiles::new();
    assert!(rf.list().is_empty());
}

#[test]
fn list_after_adds() {
    let mut rf = RecentFiles::new();
    rf.add(PathBuf::from("/x"));
    rf.add(PathBuf::from("/y"));
    let list = rf.list();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0], PathBuf::from("/y"));
}

// ── clear ──

#[test]
fn clear_empties_list() {
    let mut rf = RecentFiles::new();
    rf.add(PathBuf::from("/a"));
    rf.add(PathBuf::from("/b"));
    rf.clear();
    assert!(rf.list().is_empty());
}

// ── save / load roundtrip ──

#[test]
fn save_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("recent.json");

    let mut rf = RecentFiles::new();
    rf.add(PathBuf::from("/path/one.txt"));
    rf.add(PathBuf::from("/path/two.txt"));
    rf.save(&config_path).unwrap();

    let loaded = RecentFiles::load(&config_path).unwrap();
    assert_eq!(loaded.list().len(), 2);
    assert_eq!(loaded.list()[0], PathBuf::from("/path/two.txt"));
    assert_eq!(loaded.list()[1], PathBuf::from("/path/one.txt"));
}

#[test]
fn save_load_empty() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("recent_empty.json");

    let rf = RecentFiles::new();
    rf.save(&config_path).unwrap();

    let loaded = RecentFiles::load(&config_path).unwrap();
    assert!(loaded.list().is_empty());
}

#[test]
fn load_missing_file_returns_error() {
    let result = RecentFiles::load(std::path::Path::new("/nonexistent/path.json"));
    assert!(result.is_err());
}

// ── Unicode paths ──

#[test]
fn unicode_paths() {
    let mut rf = RecentFiles::new();
    rf.add(PathBuf::from("/tmp/日本語/テスト.txt"));
    rf.add(PathBuf::from("/tmp/café/naïve.rs"));
    assert_eq!(rf.list().len(), 2);
    assert_eq!(rf.list()[0], PathBuf::from("/tmp/café/naïve.rs"));

    // Roundtrip with save/load
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("unicode_recent.json");
    rf.save(&config_path).unwrap();
    let loaded = RecentFiles::load(&config_path).unwrap();
    assert_eq!(loaded.list().len(), 2);
    assert_eq!(loaded.list()[0], PathBuf::from("/tmp/café/naïve.rs"));
}

// ── Default trait ──

#[test]
fn default_matches_new() {
    let rf = RecentFiles::default();
    assert!(rf.list().is_empty());
}
