use notepadppp::io::settings::AppSettings;
use std::path::Path;

#[test]
fn test_default_settings() {
    let settings = AppSettings::default();
    assert_eq!(settings.font_size, 14.0);
    assert_eq!(settings.tab_size, 4);
    assert!(settings.use_spaces);
    assert!(!settings.word_wrap);
    assert!(settings.show_line_numbers);
    assert!(!settings.show_whitespace);
    assert!(settings.show_status_bar);
    assert!(settings.auto_indent);
    assert!(settings.auto_close_brackets);
    assert_eq!(settings.default_encoding, "UTF-8");
    assert_eq!(settings.default_line_ending, "LF");
    assert!(!settings.auto_save);
    assert_eq!(settings.auto_save_interval_secs, 300);
    assert!(!settings.remember_session);
    assert_eq!(settings.recent_files_max, 20);
    assert_eq!(settings.theme, "base16-ocean.dark");
    assert!(settings.highlight_current_line);
    assert!(settings.search_wrap_around);
    assert!(!settings.search_case_sensitive);
}

#[test]
fn test_save_and_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");

    let mut settings = AppSettings::default();
    settings.font_size = 18.0;
    settings.tab_size = 2;
    settings.use_spaces = false;
    settings.word_wrap = true;
    settings.theme = "InspiredGitHub".to_string();

    settings.save(&path).unwrap();

    let loaded = AppSettings::load(&path).unwrap();
    assert_eq!(loaded.font_size, 18.0);
    assert_eq!(loaded.tab_size, 2);
    assert!(!loaded.use_spaces);
    assert!(loaded.word_wrap);
    assert_eq!(loaded.theme, "InspiredGitHub");
    // Other fields should be defaults
    assert!(loaded.show_line_numbers);
    assert!(!loaded.auto_save);
}

#[test]
fn test_load_missing_file_returns_error() {
    let result = AppSettings::load(Path::new("/nonexistent/path/settings.json"));
    assert!(result.is_err());
}

#[test]
fn test_settings_path_not_empty() {
    let path = AppSettings::settings_path();
    assert!(!path.as_os_str().is_empty());
    assert!(path.to_string_lossy().contains("notepadppp"));
}

#[test]
fn test_save_creates_parent_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sub").join("dir").join("settings.json");

    let settings = AppSettings::default();
    settings.save(&path).unwrap();
    assert!(path.exists());
}

#[test]
fn test_serialization_format() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");

    let settings = AppSettings::default();
    settings.save(&path).unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("font_size"));
    assert!(content.contains("tab_size"));
    assert!(content.contains("theme"));
    // Verify it's valid JSON
    let _: serde_json::Value = serde_json::from_str(&content).unwrap();
}
