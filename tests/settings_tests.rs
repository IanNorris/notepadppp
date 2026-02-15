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

// ── New field tests ──

#[test]
fn test_default_new_fields() {
    let settings = AppSettings::default();
    assert_eq!(settings.font_family, "monospace");
    assert_eq!(settings.line_spacing, 1.3);
    assert!(!settings.show_indent_guides);
    assert!(!settings.show_line_endings);
    assert!(settings.color_background.is_none());
    assert!(settings.color_foreground.is_none());
    assert!(settings.color_selection.is_none());
    assert!(settings.color_caret.is_none());
}

#[test]
fn test_new_fields_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");

    let mut settings = AppSettings::default();
    settings.font_family = "JetBrains Mono".to_string();
    settings.line_spacing = 1.5;
    settings.show_indent_guides = true;
    settings.show_line_endings = true;
    settings.color_background = Some("#1E1E2E".to_string());
    settings.color_foreground = Some("#CDD6F4".to_string());
    settings.color_selection = Some("#585B70".to_string());
    settings.color_caret = Some("#F5E0DC".to_string());
    settings.save(&path).unwrap();

    let loaded = AppSettings::load(&path).unwrap();
    assert_eq!(loaded.font_family, "JetBrains Mono");
    assert_eq!(loaded.line_spacing, 1.5);
    assert!(loaded.show_indent_guides);
    assert!(loaded.show_line_endings);
    assert_eq!(loaded.color_background.as_deref(), Some("#1E1E2E"));
    assert_eq!(loaded.color_foreground.as_deref(), Some("#CDD6F4"));
    assert_eq!(loaded.color_selection.as_deref(), Some("#585B70"));
    assert_eq!(loaded.color_caret.as_deref(), Some("#F5E0DC"));
}

#[test]
fn test_backwards_compatibility_missing_new_fields() {
    // Simulate loading settings saved by an older version without the new fields
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");

    let old_json = r#"{
        "font_size": 14.0,
        "font_family": "monospace",
        "tab_size": 4,
        "use_spaces": true,
        "word_wrap": false,
        "show_line_numbers": true,
        "show_whitespace": false,
        "show_status_bar": true,
        "auto_indent": true,
        "auto_close_brackets": true,
        "default_encoding": "UTF-8",
        "default_line_ending": "LF",
        "auto_save": false,
        "auto_save_interval_secs": 300,
        "remember_session": false,
        "recent_files_max": 20,
        "theme": "base16-ocean.dark",
        "highlight_current_line": true,
        "search_wrap_around": true,
        "search_case_sensitive": false
    }"#;
    std::fs::write(&path, old_json).unwrap();

    let loaded = AppSettings::load(&path).unwrap();
    // New fields should get their defaults
    assert_eq!(loaded.line_spacing, 1.3);
    assert!(!loaded.show_indent_guides);
    assert!(!loaded.show_line_endings);
    assert!(loaded.color_background.is_none());
    assert!(loaded.color_foreground.is_none());
    assert!(loaded.color_selection.is_none());
    assert!(loaded.color_caret.is_none());
}
