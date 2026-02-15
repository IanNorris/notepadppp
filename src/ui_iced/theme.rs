use iced::Color;
use serde::{Deserialize, Serialize};

/// Serializable theme colors for import/export.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ThemeColors {
    pub name: String,
    pub is_light: bool,
    pub background: String,
    pub tab_bar_bg: String,
    pub tab_active_bg: String,
    pub tab_inactive_bg: String,
    pub status_bar_bg: String,
    pub text: String,
    pub text_dim: String,
    pub accent: String,
    pub close_hover: String,
    pub border: String,
    pub menu_bg: String,
    pub menu_hover: String,
    pub separator: String,
    pub dialog_bg: String,
    pub button_bg: String,
    pub editor_bg: String,
    pub gutter_bg: String,
    pub selection_bg: String,
    pub search_highlight: String,
}

/// Parse a hex color string (#RRGGBB or #RGB) into an iced Color.
pub fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim().trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::from_rgb8(r, g, b))
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
            Some(Color::from_rgb8(r * 17, g * 17, b * 17))
        }
        _ => None,
    }
}

/// Convert an iced Color to a hex string (#RRGGBB).
pub fn color_to_hex(c: &Color) -> String {
    let r = (c.r * 255.0).round() as u8;
    let g = (c.g * 255.0).round() as u8;
    let b = (c.b * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

#[derive(Debug, Clone)]
pub struct AppTheme {
    pub name: String,
    pub is_light: bool,
    pub background: Color,
    pub tab_bar_bg: Color,
    pub tab_active_bg: Color,
    pub tab_inactive_bg: Color,
    pub status_bar_bg: Color,
    pub text: Color,
    pub text_dim: Color,
    pub accent: Color,
    pub close_hover: Color,
    pub border: Color,
    pub menu_bg: Color,
    pub menu_hover: Color,
    pub separator: Color,
    pub dialog_bg: Color,
    pub button_bg: Color,
    pub editor_bg: Color,
    pub gutter_bg: Color,
    pub selection_bg: Color,
    pub search_highlight: Color,
}

impl AppTheme {
    /// The syntect theme name that matches this app theme.
    pub fn syntect_theme(&self) -> &str {
        if self.is_light {
            "base16-ocean.light"
        } else {
            "base16-ocean.dark"
        }
    }

    pub fn dark() -> Self {
        Self {
            name: "Dark".into(),
            is_light: false,
            background: Color::from_rgb(0.12, 0.12, 0.14),
            tab_bar_bg: Color::from_rgb(0.15, 0.15, 0.18),
            tab_active_bg: Color::from_rgb(0.22, 0.22, 0.26),
            tab_inactive_bg: Color::from_rgb(0.15, 0.15, 0.18),
            status_bar_bg: Color::from_rgb(0.15, 0.15, 0.18),
            text: Color::from_rgb(0.85, 0.85, 0.85),
            text_dim: Color::from_rgb(0.55, 0.55, 0.55),
            accent: Color::from_rgb(0.0, 0.47, 0.84),
            close_hover: Color::from_rgb(0.9, 0.2, 0.2),
            border: Color::from_rgb(0.3, 0.3, 0.35),
            menu_bg: Color::from_rgb(0.18, 0.18, 0.22),
            menu_hover: Color::from_rgb(0.25, 0.25, 0.30),
            separator: Color::from_rgb(0.30, 0.30, 0.35),
            dialog_bg: Color::from_rgb(0.18, 0.18, 0.22),
            button_bg: Color::from_rgb(0.25, 0.25, 0.30),
            editor_bg: Color::from_rgb(0.12, 0.12, 0.14),
            gutter_bg: Color::from_rgb(0.12, 0.12, 0.14),
            selection_bg: Color::from_rgb(0.22, 0.22, 0.26),
            search_highlight: Color::from_rgb(0.50, 0.50, 0.0),
        }
    }

    pub fn light() -> Self {
        Self {
            name: "Light".into(),
            is_light: true,
            background: Color::from_rgb(0.96, 0.96, 0.96),
            tab_bar_bg: Color::from_rgb(0.90, 0.90, 0.90),
            tab_active_bg: Color::from_rgb(1.0, 1.0, 1.0),
            tab_inactive_bg: Color::from_rgb(0.88, 0.88, 0.88),
            status_bar_bg: Color::from_rgb(0.90, 0.90, 0.92),
            text: Color::from_rgb(0.10, 0.10, 0.10),
            text_dim: Color::from_rgb(0.45, 0.45, 0.45),
            accent: Color::from_rgb(0.0, 0.47, 0.84),
            close_hover: Color::from_rgb(0.9, 0.2, 0.2),
            border: Color::from_rgb(0.75, 0.75, 0.78),
            menu_bg: Color::from_rgb(0.94, 0.94, 0.94),
            menu_hover: Color::from_rgb(0.85, 0.85, 0.88),
            separator: Color::from_rgb(0.78, 0.78, 0.80),
            dialog_bg: Color::from_rgb(0.94, 0.94, 0.94),
            button_bg: Color::from_rgb(0.82, 0.82, 0.85),
            editor_bg: Color::from_rgb(1.0, 1.0, 1.0),
            gutter_bg: Color::from_rgb(0.94, 0.94, 0.94),
            selection_bg: Color::from_rgb(0.70, 0.83, 0.97),
            search_highlight: Color::from_rgb(1.0, 1.0, 0.0),
        }
    }

    pub fn white() -> Self {
        Self {
            name: "White".into(),
            is_light: true,
            background: Color::from_rgb(1.0, 1.0, 1.0),
            tab_bar_bg: Color::from_rgb(0.96, 0.96, 0.96),
            tab_active_bg: Color::from_rgb(1.0, 1.0, 1.0),
            tab_inactive_bg: Color::from_rgb(0.94, 0.94, 0.94),
            status_bar_bg: Color::from_rgb(0.96, 0.96, 0.96),
            text: Color::from_rgb(0.12, 0.12, 0.12),
            text_dim: Color::from_rgb(0.50, 0.50, 0.50),
            accent: Color::from_rgb(0.0, 0.45, 0.80),
            close_hover: Color::from_rgb(0.9, 0.2, 0.2),
            border: Color::from_rgb(0.85, 0.85, 0.85),
            menu_bg: Color::from_rgb(0.98, 0.98, 0.98),
            menu_hover: Color::from_rgb(0.90, 0.90, 0.92),
            separator: Color::from_rgb(0.88, 0.88, 0.88),
            dialog_bg: Color::from_rgb(0.98, 0.98, 0.98),
            button_bg: Color::from_rgb(0.90, 0.90, 0.90),
            editor_bg: Color::from_rgb(1.0, 1.0, 1.0),
            gutter_bg: Color::from_rgb(0.97, 0.97, 0.97),
            selection_bg: Color::from_rgb(0.73, 0.85, 0.98),
            search_highlight: Color::from_rgb(1.0, 1.0, 0.0),
        }
    }

    pub fn high_contrast() -> Self {
        Self {
            name: "High Contrast".into(),
            is_light: false,
            background: Color::from_rgb(0.0, 0.0, 0.0),
            tab_bar_bg: Color::from_rgb(0.05, 0.05, 0.05),
            tab_active_bg: Color::from_rgb(0.15, 0.15, 0.15),
            tab_inactive_bg: Color::from_rgb(0.05, 0.05, 0.05),
            status_bar_bg: Color::from_rgb(0.05, 0.05, 0.05),
            text: Color::from_rgb(1.0, 1.0, 1.0),
            text_dim: Color::from_rgb(0.70, 0.70, 0.70),
            accent: Color::from_rgb(0.0, 0.60, 1.0),
            close_hover: Color::from_rgb(1.0, 0.0, 0.0),
            border: Color::from_rgb(0.50, 0.50, 0.50),
            menu_bg: Color::from_rgb(0.08, 0.08, 0.08),
            menu_hover: Color::from_rgb(0.20, 0.20, 0.20),
            separator: Color::from_rgb(0.40, 0.40, 0.40),
            dialog_bg: Color::from_rgb(0.08, 0.08, 0.08),
            button_bg: Color::from_rgb(0.20, 0.20, 0.20),
            editor_bg: Color::from_rgb(0.0, 0.0, 0.0),
            gutter_bg: Color::from_rgb(0.0, 0.0, 0.0),
            selection_bg: Color::from_rgb(0.0, 0.40, 0.80),
            search_highlight: Color::from_rgb(1.0, 1.0, 0.0),
        }
    }

    pub fn solarized_dark() -> Self {
        Self {
            name: "Solarized Dark".into(),
            is_light: false,
            background: Color::from_rgb(0.0, 0.169, 0.212),
            tab_bar_bg: Color::from_rgb(0.027, 0.212, 0.259),
            tab_active_bg: Color::from_rgb(0.035, 0.247, 0.298),
            tab_inactive_bg: Color::from_rgb(0.027, 0.212, 0.259),
            status_bar_bg: Color::from_rgb(0.027, 0.212, 0.259),
            text: Color::from_rgb(0.514, 0.580, 0.588),
            text_dim: Color::from_rgb(0.396, 0.482, 0.514),
            accent: Color::from_rgb(0.149, 0.545, 0.824),
            close_hover: Color::from_rgb(0.863, 0.196, 0.184),
            border: Color::from_rgb(0.035, 0.247, 0.298),
            menu_bg: Color::from_rgb(0.027, 0.212, 0.259),
            menu_hover: Color::from_rgb(0.035, 0.247, 0.298),
            separator: Color::from_rgb(0.035, 0.247, 0.298),
            dialog_bg: Color::from_rgb(0.027, 0.212, 0.259),
            button_bg: Color::from_rgb(0.035, 0.247, 0.298),
            editor_bg: Color::from_rgb(0.0, 0.169, 0.212),
            gutter_bg: Color::from_rgb(0.0, 0.169, 0.212),
            selection_bg: Color::from_rgb(0.035, 0.247, 0.298),
            search_highlight: Color::from_rgb(0.710, 0.537, 0.0),
        }
    }

    pub fn solarized_light() -> Self {
        Self {
            name: "Solarized Light".into(),
            is_light: true,
            background: Color::from_rgb(0.992, 0.965, 0.890),
            tab_bar_bg: Color::from_rgb(0.933, 0.910, 0.835),
            tab_active_bg: Color::from_rgb(0.992, 0.965, 0.890),
            tab_inactive_bg: Color::from_rgb(0.933, 0.910, 0.835),
            status_bar_bg: Color::from_rgb(0.933, 0.910, 0.835),
            text: Color::from_rgb(0.396, 0.482, 0.514),
            text_dim: Color::from_rgb(0.514, 0.580, 0.588),
            accent: Color::from_rgb(0.149, 0.545, 0.824),
            close_hover: Color::from_rgb(0.863, 0.196, 0.184),
            border: Color::from_rgb(0.933, 0.910, 0.835),
            menu_bg: Color::from_rgb(0.933, 0.910, 0.835),
            menu_hover: Color::from_rgb(0.898, 0.878, 0.812),
            separator: Color::from_rgb(0.898, 0.878, 0.812),
            dialog_bg: Color::from_rgb(0.933, 0.910, 0.835),
            button_bg: Color::from_rgb(0.898, 0.878, 0.812),
            editor_bg: Color::from_rgb(0.992, 0.965, 0.890),
            gutter_bg: Color::from_rgb(0.933, 0.910, 0.835),
            selection_bg: Color::from_rgb(0.898, 0.878, 0.812),
            search_highlight: Color::from_rgb(0.710, 0.537, 0.0),
        }
    }

    /// Export theme to a serializable ThemeColors.
    pub fn to_theme_colors(&self) -> ThemeColors {
        ThemeColors {
            name: self.name.clone(),
            is_light: self.is_light,
            background: color_to_hex(&self.background),
            tab_bar_bg: color_to_hex(&self.tab_bar_bg),
            tab_active_bg: color_to_hex(&self.tab_active_bg),
            tab_inactive_bg: color_to_hex(&self.tab_inactive_bg),
            status_bar_bg: color_to_hex(&self.status_bar_bg),
            text: color_to_hex(&self.text),
            text_dim: color_to_hex(&self.text_dim),
            accent: color_to_hex(&self.accent),
            close_hover: color_to_hex(&self.close_hover),
            border: color_to_hex(&self.border),
            menu_bg: color_to_hex(&self.menu_bg),
            menu_hover: color_to_hex(&self.menu_hover),
            separator: color_to_hex(&self.separator),
            dialog_bg: color_to_hex(&self.dialog_bg),
            button_bg: color_to_hex(&self.button_bg),
            editor_bg: color_to_hex(&self.editor_bg),
            gutter_bg: color_to_hex(&self.gutter_bg),
            selection_bg: color_to_hex(&self.selection_bg),
            search_highlight: color_to_hex(&self.search_highlight),
        }
    }

    /// Import theme from ThemeColors.
    pub fn from_theme_colors(tc: &ThemeColors) -> Option<Self> {
        Some(Self {
            name: tc.name.clone(),
            is_light: tc.is_light,
            background: parse_hex_color(&tc.background)?,
            tab_bar_bg: parse_hex_color(&tc.tab_bar_bg)?,
            tab_active_bg: parse_hex_color(&tc.tab_active_bg)?,
            tab_inactive_bg: parse_hex_color(&tc.tab_inactive_bg)?,
            status_bar_bg: parse_hex_color(&tc.status_bar_bg)?,
            text: parse_hex_color(&tc.text)?,
            text_dim: parse_hex_color(&tc.text_dim)?,
            accent: parse_hex_color(&tc.accent)?,
            close_hover: parse_hex_color(&tc.close_hover)?,
            border: parse_hex_color(&tc.border)?,
            menu_bg: parse_hex_color(&tc.menu_bg)?,
            menu_hover: parse_hex_color(&tc.menu_hover)?,
            separator: parse_hex_color(&tc.separator)?,
            dialog_bg: parse_hex_color(&tc.dialog_bg)?,
            button_bg: parse_hex_color(&tc.button_bg)?,
            editor_bg: parse_hex_color(&tc.editor_bg)?,
            gutter_bg: parse_hex_color(&tc.gutter_bg)?,
            selection_bg: parse_hex_color(&tc.selection_bg)?,
            search_highlight: parse_hex_color(&tc.search_highlight)?,
        })
    }

    /// Apply color overrides from settings.
    pub fn apply_color_overrides(&mut self, settings: &crate::io::settings::AppSettings) {
        if let Some(ref hex) = settings.color_background {
            if let Some(c) = parse_hex_color(hex) {
                self.editor_bg = c;
                self.background = c;
            }
        }
        if let Some(ref hex) = settings.color_foreground {
            if let Some(c) = parse_hex_color(hex) {
                self.text = c;
            }
        }
        if let Some(ref hex) = settings.color_selection {
            if let Some(c) = parse_hex_color(hex) {
                self.selection_bg = c;
            }
        }
    }
}

pub struct AppColors;

impl AppColors {
    pub fn available_themes() -> Vec<AppTheme> {
        vec![
            AppTheme::dark(),
            AppTheme::light(),
            AppTheme::white(),
            AppTheme::high_contrast(),
            AppTheme::solarized_dark(),
            AppTheme::solarized_light(),
        ]
    }

    pub fn theme_by_name(name: &str) -> AppTheme {
        Self::available_themes()
            .into_iter()
            .find(|t| t.name == name)
            .unwrap_or_else(AppTheme::dark)
    }
}
