use notepadppp::ui_iced::menu_bar::{menu_x_offset, MENU_LABELS};

// ── menu_x_offset ──

#[test]
fn file_menu_offset_is_initial() {
    let offset = menu_x_offset("File");
    // "File" is the first item, so offset should be the initial padding (4.0)
    assert!((offset - 4.0).abs() < f32::EPSILON);
}

#[test]
fn edit_menu_offset_after_file() {
    let offset = menu_x_offset("Edit");
    // File = 4 chars * 7.0 + 20.0 = 48.0, plus initial 4.0 = 52.0
    let expected = 4.0 + ("File".len() as f32) * 7.0 + 20.0;
    assert!((offset - expected).abs() < f32::EPSILON);
}

#[test]
fn later_menus_have_larger_offsets() {
    let file_offset = menu_x_offset("File");
    let edit_offset = menu_x_offset("Edit");
    let search_offset = menu_x_offset("Search");
    let view_offset = menu_x_offset("View");

    assert!(edit_offset > file_offset);
    assert!(search_offset > edit_offset);
    assert!(view_offset > search_offset);
}

#[test]
fn help_menu_is_last() {
    let help_offset = menu_x_offset("Help");
    // Help should be the last real menu, so it should have a large offset
    let file_offset = menu_x_offset("File");
    assert!(help_offset > file_offset);
}

#[test]
fn unknown_menu_returns_total_offset() {
    let unknown_offset = menu_x_offset("Unknown");
    // Should be the total offset past all menus
    let mut expected = 4.0;
    for label in MENU_LABELS {
        expected += (label.len() as f32) * 7.0 + 20.0;
    }
    assert!((unknown_offset - expected).abs() < f32::EPSILON);
}

#[test]
fn all_menu_labels_produce_valid_offsets() {
    let mut prev = 0.0f32;
    for label in MENU_LABELS {
        let offset = menu_x_offset(label);
        assert!(offset >= prev, "Offset for '{}' ({}) should be >= previous ({})", label, offset, prev);
        prev = offset;
    }
}

#[test]
fn menu_labels_has_expected_items() {
    assert!(MENU_LABELS.contains(&"File"));
    assert!(MENU_LABELS.contains(&"Edit"));
    assert!(MENU_LABELS.contains(&"Search"));
    assert!(MENU_LABELS.contains(&"View"));
    assert!(MENU_LABELS.contains(&"Language"));
    assert!(MENU_LABELS.contains(&"Tools"));
    assert!(MENU_LABELS.contains(&"Macro"));
    assert!(MENU_LABELS.contains(&"Settings"));
    assert!(MENU_LABELS.contains(&"Help"));
    assert_eq!(MENU_LABELS.len(), 9);
}
