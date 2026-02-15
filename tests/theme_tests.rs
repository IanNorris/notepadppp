use notepadppp::ui_iced::theme::{AppColors, AppTheme};

// ── Theme constructors ──

#[test]
fn dark_theme_name_and_is_light() {
    let t = AppTheme::dark();
    assert_eq!(t.name, "Dark");
    assert!(!t.is_light);
}

#[test]
fn light_theme_name_and_is_light() {
    let t = AppTheme::light();
    assert_eq!(t.name, "Light");
    assert!(t.is_light);
}

#[test]
fn white_theme_name_and_is_light() {
    let t = AppTheme::white();
    assert_eq!(t.name, "White");
    assert!(t.is_light);
}

#[test]
fn high_contrast_name_and_is_light() {
    let t = AppTheme::high_contrast();
    assert_eq!(t.name, "High Contrast");
    assert!(!t.is_light);
}

#[test]
fn solarized_dark_name_and_is_light() {
    let t = AppTheme::solarized_dark();
    assert_eq!(t.name, "Solarized Dark");
    assert!(!t.is_light);
}

#[test]
fn solarized_light_name_and_is_light() {
    let t = AppTheme::solarized_light();
    assert_eq!(t.name, "Solarized Light");
    assert!(t.is_light);
}

// ── syntect_theme ──

#[test]
fn syntect_theme_dark() {
    let t = AppTheme::dark();
    assert_eq!(t.syntect_theme(), "base16-ocean.dark");
}

#[test]
fn syntect_theme_light() {
    let t = AppTheme::light();
    assert_eq!(t.syntect_theme(), "base16-ocean.light");
}

#[test]
fn syntect_theme_high_contrast_is_dark() {
    let t = AppTheme::high_contrast();
    assert_eq!(t.syntect_theme(), "base16-ocean.dark");
}

#[test]
fn syntect_theme_solarized_light() {
    let t = AppTheme::solarized_light();
    assert_eq!(t.syntect_theme(), "base16-ocean.light");
}

// ── available_themes ──

#[test]
fn available_themes_count() {
    let themes = AppColors::available_themes();
    assert_eq!(themes.len(), 6);
}

#[test]
fn available_themes_names() {
    let themes = AppColors::available_themes();
    let names: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"Dark"));
    assert!(names.contains(&"Light"));
    assert!(names.contains(&"White"));
    assert!(names.contains(&"High Contrast"));
    assert!(names.contains(&"Solarized Dark"));
    assert!(names.contains(&"Solarized Light"));
}

// ── theme_by_name ──

#[test]
fn theme_by_name_dark() {
    let t = AppColors::theme_by_name("Dark");
    assert_eq!(t.name, "Dark");
}

#[test]
fn theme_by_name_light() {
    let t = AppColors::theme_by_name("Light");
    assert_eq!(t.name, "Light");
}

#[test]
fn theme_by_name_white() {
    let t = AppColors::theme_by_name("White");
    assert_eq!(t.name, "White");
}

#[test]
fn theme_by_name_high_contrast() {
    let t = AppColors::theme_by_name("High Contrast");
    assert_eq!(t.name, "High Contrast");
}

#[test]
fn theme_by_name_solarized_dark() {
    let t = AppColors::theme_by_name("Solarized Dark");
    assert_eq!(t.name, "Solarized Dark");
}

#[test]
fn theme_by_name_solarized_light() {
    let t = AppColors::theme_by_name("Solarized Light");
    assert_eq!(t.name, "Solarized Light");
}

#[test]
fn theme_by_name_unknown_falls_back_to_dark() {
    let t = AppColors::theme_by_name("NonExistent");
    assert_eq!(t.name, "Dark");
}

// ── Theme fields are non-default ──

fn color_has_nonzero_component(c: iced::Color) -> bool {
    c.r > 0.0 || c.g > 0.0 || c.b > 0.0 || c.a > 0.0
}

#[test]
fn dark_theme_has_colors() {
    let t = AppTheme::dark();
    assert!(color_has_nonzero_component(t.text));
    assert!(color_has_nonzero_component(t.accent));
    assert!(color_has_nonzero_component(t.background));
}

#[test]
fn light_theme_has_colors() {
    let t = AppTheme::light();
    assert!(color_has_nonzero_component(t.text));
    assert!(color_has_nonzero_component(t.accent));
    assert!(color_has_nonzero_component(t.background));
}

#[test]
fn high_contrast_has_full_white_text() {
    let t = AppTheme::high_contrast();
    assert_eq!(t.text.r, 1.0);
    assert_eq!(t.text.g, 1.0);
    assert_eq!(t.text.b, 1.0);
}

// ── parse_hex_color ──

use notepadppp::ui_iced::theme::{parse_hex_color, color_to_hex};

#[test]
fn parse_hex_color_6_digit() {
    let c = parse_hex_color("#FF8800").unwrap();
    assert_eq!((c.r * 255.0).round() as u8, 255);
    assert_eq!((c.g * 255.0).round() as u8, 136);
    assert_eq!((c.b * 255.0).round() as u8, 0);
}

#[test]
fn parse_hex_color_3_digit() {
    let c = parse_hex_color("#F80").unwrap();
    assert_eq!((c.r * 255.0).round() as u8, 255);
    assert_eq!((c.g * 255.0).round() as u8, 136);
    assert_eq!((c.b * 255.0).round() as u8, 0);
}

#[test]
fn parse_hex_color_no_hash() {
    let c = parse_hex_color("AABBCC").unwrap();
    assert_eq!((c.r * 255.0).round() as u8, 0xAA);
    assert_eq!((c.g * 255.0).round() as u8, 0xBB);
    assert_eq!((c.b * 255.0).round() as u8, 0xCC);
}

#[test]
fn parse_hex_color_black() {
    let c = parse_hex_color("#000000").unwrap();
    assert_eq!(c.r, 0.0);
    assert_eq!(c.g, 0.0);
    assert_eq!(c.b, 0.0);
}

#[test]
fn parse_hex_color_white() {
    let c = parse_hex_color("#FFFFFF").unwrap();
    assert_eq!((c.r * 255.0).round() as u8, 255);
    assert_eq!((c.g * 255.0).round() as u8, 255);
    assert_eq!((c.b * 255.0).round() as u8, 255);
}

#[test]
fn parse_hex_color_invalid_returns_none() {
    assert!(parse_hex_color("").is_none());
    assert!(parse_hex_color("#GG0000").is_none());
    assert!(parse_hex_color("#1234").is_none());
    assert!(parse_hex_color("xyz").is_none());
}

#[test]
fn color_to_hex_black() {
    let c = iced::Color::from_rgb(0.0, 0.0, 0.0);
    assert_eq!(color_to_hex(&c), "#000000");
}

#[test]
fn color_to_hex_white() {
    let c = iced::Color::from_rgb(1.0, 1.0, 1.0);
    assert_eq!(color_to_hex(&c), "#FFFFFF");
}

#[test]
fn color_to_hex_roundtrip() {
    let original = "#3A7BCD";
    let color = parse_hex_color(original).unwrap();
    let hex = color_to_hex(&color);
    assert_eq!(hex, original);
}

// ── Theme export/import ──

use notepadppp::ui_iced::theme::ThemeColors;

#[test]
fn theme_export_roundtrip() {
    let original = AppTheme::dark();
    let tc = original.to_theme_colors();
    let json = serde_json::to_string_pretty(&tc).unwrap();
    let loaded: ThemeColors = serde_json::from_str(&json).unwrap();
    let restored = AppTheme::from_theme_colors(&loaded).unwrap();

    assert_eq!(restored.name, original.name);
    assert_eq!(restored.is_light, original.is_light);
    // Verify colors are close (float rounding)
    let orig_hex = color_to_hex(&original.background);
    let rest_hex = color_to_hex(&restored.background);
    assert_eq!(orig_hex, rest_hex);
}

#[test]
fn theme_export_all_themes_roundtrip() {
    for theme in AppColors::available_themes() {
        let tc = theme.to_theme_colors();
        let json = serde_json::to_string(&tc).unwrap();
        let loaded: ThemeColors = serde_json::from_str(&json).unwrap();
        let restored = AppTheme::from_theme_colors(&loaded);
        assert!(restored.is_some(), "Failed to restore theme: {}", theme.name);
        assert_eq!(restored.unwrap().name, theme.name);
    }
}

#[test]
fn theme_import_invalid_color_returns_none() {
    let tc = ThemeColors {
        name: "Bad".into(),
        is_light: false,
        background: "INVALID".into(),
        tab_bar_bg: "#000000".into(),
        tab_active_bg: "#000000".into(),
        tab_inactive_bg: "#000000".into(),
        status_bar_bg: "#000000".into(),
        text: "#000000".into(),
        text_dim: "#000000".into(),
        accent: "#000000".into(),
        close_hover: "#000000".into(),
        border: "#000000".into(),
        menu_bg: "#000000".into(),
        menu_hover: "#000000".into(),
        separator: "#000000".into(),
        dialog_bg: "#000000".into(),
        button_bg: "#000000".into(),
        editor_bg: "#000000".into(),
        gutter_bg: "#000000".into(),
        selection_bg: "#000000".into(),
        search_highlight: "#000000".into(),
    };
    assert!(AppTheme::from_theme_colors(&tc).is_none());
}

#[test]
fn theme_colors_json_has_all_fields() {
    let tc = AppTheme::dark().to_theme_colors();
    let json = serde_json::to_string(&tc).unwrap();
    assert!(json.contains("background"));
    assert!(json.contains("text"));
    assert!(json.contains("editor_bg"));
    assert!(json.contains("selection_bg"));
    assert!(json.contains("search_highlight"));
}

// ── apply_color_overrides ──

#[test]
fn apply_color_overrides_background() {
    let mut theme = AppTheme::dark();
    let mut settings = notepadppp::io::settings::AppSettings::default();
    settings.color_background = Some("#FF0000".into());
    theme.apply_color_overrides(&settings);
    assert_eq!((theme.editor_bg.r * 255.0).round() as u8, 255);
    assert_eq!((theme.editor_bg.g * 255.0).round() as u8, 0);
}

#[test]
fn apply_color_overrides_foreground() {
    let mut theme = AppTheme::dark();
    let mut settings = notepadppp::io::settings::AppSettings::default();
    settings.color_foreground = Some("#00FF00".into());
    theme.apply_color_overrides(&settings);
    assert_eq!((theme.text.g * 255.0).round() as u8, 255);
}

#[test]
fn apply_color_overrides_selection() {
    let mut theme = AppTheme::dark();
    let mut settings = notepadppp::io::settings::AppSettings::default();
    settings.color_selection = Some("#0000FF".into());
    theme.apply_color_overrides(&settings);
    assert_eq!((theme.selection_bg.b * 255.0).round() as u8, 255);
}

#[test]
fn apply_color_overrides_invalid_hex_ignored() {
    let mut theme = AppTheme::dark();
    let original_bg = theme.editor_bg;
    let mut settings = notepadppp::io::settings::AppSettings::default();
    settings.color_background = Some("not-a-color".into());
    theme.apply_color_overrides(&settings);
    assert_eq!(color_to_hex(&theme.editor_bg), color_to_hex(&original_bg));
}

#[test]
fn apply_color_overrides_none_leaves_defaults() {
    let mut theme = AppTheme::dark();
    let original_bg = theme.editor_bg;
    let settings = notepadppp::io::settings::AppSettings::default();
    theme.apply_color_overrides(&settings);
    assert_eq!(color_to_hex(&theme.editor_bg), color_to_hex(&original_bg));
}
