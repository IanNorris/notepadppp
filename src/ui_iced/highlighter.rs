use iced::advanced::text::highlighter;
use std::ops::Range;
use std::sync::LazyLock;

static SYNTAX_SET: LazyLock<syntect::parsing::SyntaxSet> = LazyLock::new(|| {
    syntect::parsing::SyntaxSet::load_defaults_newlines()
});

static THEME_SET: LazyLock<syntect::highlighting::ThemeSet> = LazyLock::new(|| {
    syntect::highlighting::ThemeSet::load_defaults()
});

#[derive(Clone, PartialEq)]
pub struct SyntectSettings {
    pub extension: String,
    pub theme: String,
}

#[derive(Clone)]
pub struct SyntectHighlight {
    pub color: iced::Color,
}

impl SyntectHighlight {
    pub fn to_format<Font>(&self) -> highlighter::Format<Font> {
        highlighter::Format {
            color: Some(self.color),
            font: None,
        }
    }
}

pub struct SyntectHighlighter {
    current_line: usize,
    highlight_lines: syntect::easy::HighlightLines<'static>,
    settings: SyntectSettings,
}

impl SyntectHighlighter {
    fn create_highlight_lines(settings: &SyntectSettings) -> syntect::easy::HighlightLines<'static> {
        let syntax = SYNTAX_SET
            .find_syntax_by_extension(&settings.extension)
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

        let theme = THEME_SET
            .themes
            .get(&settings.theme)
            .unwrap_or_else(|| {
                THEME_SET.themes.values().next().expect("no themes available")
            });

        syntect::easy::HighlightLines::new(syntax, theme)
    }
}

impl iced::advanced::text::Highlighter for SyntectHighlighter {
    type Settings = SyntectSettings;
    type Highlight = SyntectHighlight;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, SyntectHighlight)>;

    fn new(settings: &Self::Settings) -> Self {
        Self {
            current_line: 0,
            highlight_lines: Self::create_highlight_lines(settings),
            settings: settings.clone(),
        }
    }

    fn update(&mut self, new_settings: &Self::Settings) {
        if *new_settings != self.settings {
            self.settings = new_settings.clone();
            self.highlight_lines = Self::create_highlight_lines(new_settings);
            self.current_line = 0;
        }
    }

    fn change_line(&mut self, line: usize) {
        if line < self.current_line {
            self.current_line = 0;
            self.highlight_lines = Self::create_highlight_lines(&self.settings);
        }
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        self.current_line += 1;

        let regions = self
            .highlight_lines
            .highlight_line(line, &SYNTAX_SET)
            .unwrap_or_default();

        let mut spans = Vec::with_capacity(regions.len());
        let mut offset = 0;
        for (style, text) in regions {
            let len = text.len();
            if len > 0 {
                let fg = style.foreground;
                spans.push((
                    offset..offset + len,
                    SyntectHighlight {
                        color: iced::Color::from_rgba8(fg.r, fg.g, fg.b, fg.a as f32 / 255.0),
                    },
                ));
                offset += len;
            }
        }

        spans.into_iter()
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}

/// Look up a syntect syntax by language name and return its first file extension.
pub fn extension_for_language(lang: &str) -> Option<String> {
    for syntax in SYNTAX_SET.syntaxes() {
        if syntax.name == lang {
            return syntax.file_extensions.first().cloned();
        }
    }
    None
}

/// Detect the syntect language name for a file extension.
///
/// This wraps `find_syntax_by_extension` but fixes ambiguous extensions like
/// `.h` which syntect maps to Objective-C instead of C/C++.
pub fn language_for_extension(ext: &str) -> String {
    // Header files: prefer C++ over Objective-C
    if ext == "h" || ext == "H" {
        return "C++".to_string();
    }
    SYNTAX_SET
        .find_syntax_by_extension(ext)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "Plain Text".to_string())
}
