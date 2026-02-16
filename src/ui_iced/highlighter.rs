use iced::advanced::text::highlighter;
use std::ops::Range;
use std::sync::LazyLock;

use syntect::highlighting::{
    HighlightState, Highlighter as SyntectHL, RangedHighlightIterator, Theme, ThemeSet,
};
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet};

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(|| {
    SyntaxSet::load_defaults_newlines()
});

static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(|| {
    ThemeSet::load_defaults()
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
    syntax: &'static SyntaxReference,
    theme: &'static Theme,
    // Cached parse + highlight states at the START of each line
    parse_states: Vec<ParseState>,
    highlight_states: Vec<HighlightState>,
    // Current working states
    parse_state: ParseState,
    highlight_state: HighlightState,
    settings: SyntectSettings,
}

impl SyntectHighlighter {
    fn get_syntax(ext: &str) -> &'static SyntaxReference {
        SYNTAX_SET
            .find_syntax_by_extension(ext)
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text())
    }

    fn get_theme(name: &str) -> &'static Theme {
        THEME_SET
            .themes
            .get(name)
            .unwrap_or_else(|| THEME_SET.themes.values().next().expect("no themes available"))
    }

    fn initial_states(syntax: &'static SyntaxReference, theme: &'static Theme) -> (ParseState, HighlightState) {
        let parse_state = ParseState::new(syntax);
        let highlighter = SyntectHL::new(theme);
        let highlight_state = HighlightState::new(&highlighter, ScopeStack::new());
        (parse_state, highlight_state)
    }
}

impl iced::advanced::text::Highlighter for SyntectHighlighter {
    type Settings = SyntectSettings;
    type Highlight = SyntectHighlight;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, SyntectHighlight)>;

    fn new(settings: &Self::Settings) -> Self {
        let syntax = Self::get_syntax(&settings.extension);
        let theme = Self::get_theme(&settings.theme);
        let (parse_state, highlight_state) = Self::initial_states(syntax, theme);
        Self {
            current_line: 0,
            syntax,
            theme,
            parse_states: vec![parse_state.clone()],
            highlight_states: vec![highlight_state.clone()],
            parse_state,
            highlight_state,
            settings: settings.clone(),
        }
    }

    fn update(&mut self, new_settings: &Self::Settings) {
        if *new_settings != self.settings {
            self.settings = new_settings.clone();
            self.syntax = Self::get_syntax(&new_settings.extension);
            self.theme = Self::get_theme(&new_settings.theme);
            let (ps, hs) = Self::initial_states(self.syntax, self.theme);
            self.parse_state = ps.clone();
            self.highlight_state = hs.clone();
            self.parse_states = vec![ps];
            self.highlight_states = vec![hs];
            self.current_line = 0;
        }
    }

    fn change_line(&mut self, line: usize) {
        if line < self.current_line {
            if line < self.parse_states.len() {
                self.parse_state = self.parse_states[line].clone();
                self.highlight_state = self.highlight_states[line].clone();
            } else {
                let (ps, hs) = Self::initial_states(self.syntax, self.theme);
                self.parse_state = ps;
                self.highlight_state = hs;
            }
            self.current_line = line;
            self.parse_states.truncate(line + 1);
            self.highlight_states.truncate(line + 1);
        }
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        self.current_line += 1;

        let line_nl = format!("{}\n", line);
        let ops = self.parse_state.parse_line(&line_nl, &SYNTAX_SET)
            .unwrap_or_default();

        let highlighter = SyntectHL::new(self.theme);
        let iter = RangedHighlightIterator::new(
            &mut self.highlight_state, &ops, &line_nl, &highlighter,
        );

        let line_len = line.len();
        let mut spans = Vec::new();
        let mut offset = 0;
        for (style, _range, text) in iter {
            let len = text.len();
            if len > 0 && offset < line_len {
                let end = (offset + len).min(line_len);
                let fg = style.foreground;
                spans.push((
                    offset..end,
                    SyntectHighlight {
                        color: iced::Color::from_rgba8(fg.r, fg.g, fg.b, fg.a as f32 / 255.0),
                    },
                ));
            }
            offset += len;
        }

        // Cache state at the start of the next line
        if self.current_line >= self.parse_states.len() {
            self.parse_states.push(self.parse_state.clone());
            self.highlight_states.push(self.highlight_state.clone());
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
