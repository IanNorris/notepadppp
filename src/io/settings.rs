use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    // Editor
    pub font_size: f32,
    pub font_family: String,
    pub tab_size: usize,
    pub use_spaces: bool,
    pub word_wrap: bool,
    pub show_line_numbers: bool,
    pub show_whitespace: bool,
    pub show_status_bar: bool,
    pub auto_indent: bool,
    pub auto_close_brackets: bool,
    #[serde(default = "default_line_spacing")]
    pub line_spacing: f32,
    #[serde(default)]
    pub show_indent_guides: bool,
    #[serde(default)]
    pub show_line_endings: bool,

    // Files
    pub default_encoding: String,
    pub default_line_ending: String,
    pub auto_save: bool,
    pub auto_save_interval_secs: u64,
    pub remember_session: bool,
    pub recent_files_max: usize,

    // Appearance
    pub theme: String,
    pub highlight_current_line: bool,
    #[serde(default)]
    pub color_background: Option<String>,
    #[serde(default)]
    pub color_foreground: Option<String>,
    #[serde(default)]
    pub color_selection: Option<String>,
    #[serde(default)]
    pub color_caret: Option<String>,

    // Search
    pub search_wrap_around: bool,
    pub search_case_sensitive: bool,
}

fn default_line_spacing() -> f32 {
    1.3
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            font_family: String::from("monospace"),
            tab_size: 4,
            use_spaces: true,
            word_wrap: false,
            show_line_numbers: true,
            show_whitespace: false,
            show_status_bar: true,
            auto_indent: true,
            auto_close_brackets: true,
            line_spacing: 1.3,
            show_indent_guides: false,
            show_line_endings: false,
            default_encoding: String::from("UTF-8"),
            default_line_ending: String::from("LF"),
            auto_save: false,
            auto_save_interval_secs: 300,
            remember_session: false,
            recent_files_max: 20,
            theme: String::from("base16-ocean.dark"),
            highlight_current_line: true,
            color_background: None,
            color_foreground: None,
            color_selection: None,
            color_caret: None,
            search_wrap_around: true,
            search_case_sensitive: false,
        }
    }
}

impl AppSettings {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read_to_string(path)?;
        let settings: AppSettings = serde_json::from_str(&data)?;
        Ok(settings)
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn settings_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("notepadppp")
            .join("settings.json")
    }
}
