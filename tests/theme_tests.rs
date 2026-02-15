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
