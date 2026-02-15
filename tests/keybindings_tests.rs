use notepadppp::io::keybindings::{KeyBinding, KeyBindings};
use tempfile::TempDir;

#[test]
fn test_defaults_have_all_expected_actions() {
    let kb = KeyBindings::defaults();
    let expected = [
        "new_file", "open_file", "save", "save_as", "close_tab",
        "undo", "redo", "zoom_in", "zoom_out", "zoom_reset",
        "find", "replace", "find_next", "find_prev",
        "goto_line", "command_palette", "bracket_jump",
        "toggle_bookmark", "next_bookmark", "prev_bookmark",
        "format_json", "toggle_macro_recording", "play_last_macro",
        "select_next", "escape",
    ];
    for action in &expected {
        assert!(kb.bindings.contains_key(*action), "Missing action: {}", action);
    }
}

#[test]
fn test_default_new_file_is_ctrl_n() {
    let kb = KeyBindings::defaults();
    let b = &kb.bindings["new_file"];
    assert_eq!(b.key, "N");
    assert!(b.ctrl);
    assert!(!b.shift);
    assert!(!b.alt);
}

#[test]
fn test_default_save_as_is_ctrl_shift_s() {
    let kb = KeyBindings::defaults();
    let b = &kb.bindings["save_as"];
    assert_eq!(b.key, "S");
    assert!(b.ctrl);
    assert!(b.shift);
    assert!(!b.alt);
}

#[test]
fn test_save_load_roundtrip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("keybindings.json");
    let kb = KeyBindings::defaults();
    kb.save(&path).unwrap();

    let loaded = KeyBindings::load(&path).unwrap();
    assert_eq!(loaded.bindings.len(), kb.bindings.len());
    for (action, binding) in &kb.bindings {
        let loaded_binding = loaded.bindings.get(action).unwrap();
        assert_eq!(binding, loaded_binding, "Mismatch for action: {}", action);
    }
}

#[test]
fn test_save_load_custom_binding() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("keybindings.json");
    let mut kb = KeyBindings::defaults();
    // Change save to Ctrl+Shift+X
    kb.bindings.insert("save".into(), KeyBinding::new("X", true, true, false));
    kb.save(&path).unwrap();

    let loaded = KeyBindings::load(&path).unwrap();
    let b = &loaded.bindings["save"];
    assert_eq!(b.key, "X");
    assert!(b.ctrl);
    assert!(b.shift);
}

#[test]
fn test_display_shortcut() {
    let kb = KeyBindings::defaults();
    assert_eq!(kb.display_shortcut("new_file"), "Ctrl+N");
    assert_eq!(kb.display_shortcut("save_as"), "Ctrl+Shift+S");
    assert_eq!(kb.display_shortcut("escape"), "Escape");
    assert_eq!(kb.display_shortcut("find_next"), "F3");
    assert_eq!(kb.display_shortcut("find_prev"), "Shift+F3");
    assert_eq!(kb.display_shortcut("format_json"), "Ctrl+Shift+J");
}

#[test]
fn test_display_shortcut_unknown_action() {
    let kb = KeyBindings::defaults();
    assert_eq!(kb.display_shortcut("nonexistent_action"), "");
}

#[test]
fn test_keybinding_display() {
    assert_eq!(KeyBinding::new("S", true, false, false).display(), "Ctrl+S");
    assert_eq!(KeyBinding::new("S", true, true, false).display(), "Ctrl+Shift+S");
    assert_eq!(KeyBinding::new("F2", false, false, false).display(), "F2");
    assert_eq!(KeyBinding::new("A", true, true, true).display(), "Ctrl+Shift+Alt+A");
    assert_eq!(KeyBinding::new("Z", false, false, true).display(), "Alt+Z");
}

#[test]
fn test_matches_ctrl_n_new_file() {
    let kb = KeyBindings::defaults();
    assert!(kb.matches("new_file", "N", true, false, false));
    assert!(kb.matches("new_file", "n", true, false, false));
}

#[test]
fn test_matches_ctrl_shift_s_save_as() {
    let kb = KeyBindings::defaults();
    assert!(kb.matches("save_as", "S", true, true, false));
}

#[test]
fn test_matches_ctrl_s_does_not_match_save_as() {
    let kb = KeyBindings::defaults();
    assert!(!kb.matches("save_as", "S", true, false, false));
}

#[test]
fn test_matches_wrong_key_does_not_match() {
    let kb = KeyBindings::defaults();
    assert!(!kb.matches("new_file", "M", true, false, false));
}

#[test]
fn test_matches_unknown_action_does_not_match() {
    let kb = KeyBindings::defaults();
    assert!(!kb.matches("unknown_action", "N", true, false, false));
}

#[test]
fn test_matches_f3_find_next() {
    let kb = KeyBindings::defaults();
    assert!(kb.matches("find_next", "F3", false, false, false));
}

#[test]
fn test_matches_shift_f3_find_prev() {
    let kb = KeyBindings::defaults();
    assert!(kb.matches("find_prev", "F3", false, true, false));
}

#[test]
fn test_matches_escape() {
    let kb = KeyBindings::defaults();
    assert!(kb.matches("escape", "Escape", false, false, false));
}

#[test]
fn test_all_actions_sorted() {
    let kb = KeyBindings::defaults();
    let actions = kb.all_actions_sorted();
    assert!(!actions.is_empty());
    // Check that they're sorted by display name
    for i in 1..actions.len() {
        assert!(actions[i - 1].1 <= actions[i].1, "Not sorted: {} > {}", actions[i-1].1, actions[i].1);
    }
}

#[test]
fn test_load_nonexistent_file() {
    let result = KeyBindings::load(std::path::Path::new("/tmp/nonexistent_keybindings_12345.json"));
    assert!(result.is_err());
}

#[test]
fn test_save_creates_parent_dirs() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("subdir").join("keybindings.json");
    let kb = KeyBindings::defaults();
    kb.save(&path).unwrap();
    assert!(path.exists());
}
