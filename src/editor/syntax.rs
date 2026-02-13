use std::path::Path;

use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::easy::HighlightLines;

/// Syntax highlighting engine backed by syntect.
pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_theme: String,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            current_theme: String::from("base16-ocean.dark"),
        }
    }

    /// Highlight a single line, returning colored spans.
    pub fn highlight_line(&self, line: &str, extension: &str) -> Vec<(Style, String)> {
        let syntax = self
            .syntax_set
            .find_syntax_by_extension(extension)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let theme = self
            .theme_set
            .themes
            .get(&self.current_theme)
            .unwrap_or_else(|| self.theme_set.themes.values().next().unwrap());
        let mut h = HighlightLines::new(syntax, theme);
        h.highlight_line(line, &self.syntax_set)
            .unwrap_or_default()
            .into_iter()
            .map(|(style, text)| (style, text.to_string()))
            .collect()
    }

    /// Get the display language name for a file extension.
    pub fn get_language_name(&self, extension: &str) -> String {
        self.syntax_set
            .find_syntax_by_extension(extension)
            .map(|s| s.name.clone())
            .unwrap_or_else(|| String::from("Plain Text"))
    }

    /// List all supported language names.
    pub fn list_languages(&self) -> Vec<String> {
        self.syntax_set
            .syntaxes()
            .iter()
            .map(|s| s.name.clone())
            .collect()
    }

    /// List all available theme names.
    pub fn list_themes(&self) -> Vec<String> {
        self.theme_set.themes.keys().cloned().collect()
    }

    /// Set the current theme by name.
    pub fn set_theme(&mut self, name: &str) {
        if self.theme_set.themes.contains_key(name) {
            self.current_theme = name.to_string();
        }
    }

    /// Get the current theme name.
    pub fn get_theme(&self) -> &str {
        &self.current_theme
    }

    /// Detect language name from a file path's extension.
    pub fn detect_language(path: &Path) -> String {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| {
                let ss = SyntaxSet::load_defaults_newlines();
                ss.find_syntax_by_extension(ext)
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|| String::from("Plain Text"))
            })
            .unwrap_or_else(|| String::from("Plain Text"))
    }

    /// Extract extension from a file path (without the dot).
    pub fn extension_from_path(path: &Path) -> String {
        path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string()
    }

    /// Get the SyntaxSet reference (for advanced use).
    pub fn syntax_set(&self) -> &SyntaxSet {
        &self.syntax_set
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}
