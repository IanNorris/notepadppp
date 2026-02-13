use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    Normal,
    Extended,
    Regex,
}

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub line_text: String,
    pub match_text: String,
}

pub struct SearchEngine {
    pub query: String,
    pub replace_text: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub use_regex: bool,
    pub wrap_around: bool,
    pub search_mode: SearchMode,
    results: Vec<SearchMatch>,
    results_valid: bool,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            replace_text: String::new(),
            case_sensitive: true,
            whole_word: false,
            use_regex: false,
            wrap_around: true,
            search_mode: SearchMode::Normal,
            results: Vec::new(),
            results_valid: false,
        }
    }

    /// Expand escape sequences for Extended mode.
    fn expand_escapes(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    Some('0') => out.push('\0'),
                    Some('\\') => out.push('\\'),
                    Some(other) => {
                        out.push('\\');
                        out.push(other);
                    }
                    None => out.push('\\'),
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    /// Build a regex from the current query and settings.
    fn build_regex(&self) -> Option<Regex> {
        if self.query.is_empty() {
            return None;
        }

        let pattern = match self.search_mode {
            SearchMode::Normal => regex::escape(&self.query),
            SearchMode::Extended => regex::escape(&Self::expand_escapes(&self.query)),
            SearchMode::Regex => self.query.clone(),
        };

        let pattern = if self.whole_word {
            format!(r"\b{}\b", pattern)
        } else {
            pattern
        };

        let pattern = if self.case_sensitive {
            pattern
        } else {
            format!("(?i){}", pattern)
        };

        Regex::new(&pattern).ok()
    }

    /// Compute the 0-indexed line number for a byte offset in text.
    fn line_number(text: &str, byte_offset: usize) -> usize {
        text[..byte_offset].matches('\n').count()
    }

    /// Get the full line text containing the given byte offset.
    fn line_at(text: &str, byte_offset: usize) -> String {
        let start = text[..byte_offset].rfind('\n').map_or(0, |p| p + 1);
        let end = text[byte_offset..]
            .find('\n')
            .map_or(text.len(), |p| byte_offset + p);
        text[start..end].to_string()
    }

    pub fn find_all(&self, text: &str) -> Vec<SearchMatch> {
        let re = match self.build_regex() {
            Some(r) => r,
            None => return Vec::new(),
        };

        re.find_iter(text)
            .map(|m| SearchMatch {
                start: m.start(),
                end: m.end(),
                line: Self::line_number(text, m.start()),
                line_text: Self::line_at(text, m.start()),
                match_text: m.as_str().to_string(),
            })
            .collect()
    }

    pub fn find_next(&self, text: &str, from_pos: usize) -> Option<SearchMatch> {
        let re = self.build_regex()?;

        // Search from from_pos onward
        if let Some(m) = re.find_at(text, from_pos) {
            return Some(SearchMatch {
                start: m.start(),
                end: m.end(),
                line: Self::line_number(text, m.start()),
                line_text: Self::line_at(text, m.start()),
                match_text: m.as_str().to_string(),
            });
        }

        // Wrap around: search from beginning
        if self.wrap_around {
            if let Some(m) = re.find(text) {
                if m.start() < from_pos {
                    return Some(SearchMatch {
                        start: m.start(),
                        end: m.end(),
                        line: Self::line_number(text, m.start()),
                        line_text: Self::line_at(text, m.start()),
                        match_text: m.as_str().to_string(),
                    });
                }
            }
        }

        None
    }

    pub fn find_prev(&self, text: &str, from_pos: usize) -> Option<SearchMatch> {
        let re = self.build_regex()?;

        // Collect all matches before from_pos
        let mut last_before: Option<SearchMatch> = None;
        for m in re.find_iter(text) {
            if m.start() < from_pos {
                last_before = Some(SearchMatch {
                    start: m.start(),
                    end: m.end(),
                    line: Self::line_number(text, m.start()),
                    line_text: Self::line_at(text, m.start()),
                    match_text: m.as_str().to_string(),
                });
            } else {
                break;
            }
        }

        if last_before.is_some() {
            return last_before;
        }

        // Wrap around: find last match in entire text
        if self.wrap_around {
            let mut last: Option<SearchMatch> = None;
            for m in re.find_iter(text) {
                last = Some(SearchMatch {
                    start: m.start(),
                    end: m.end(),
                    line: Self::line_number(text, m.start()),
                    line_text: Self::line_at(text, m.start()),
                    match_text: m.as_str().to_string(),
                });
            }
            if let Some(ref l) = last {
                if l.start >= from_pos {
                    return last;
                }
            }
        }

        None
    }

    pub fn replace_next(&self, text: &str, from_pos: usize) -> Option<(String, SearchMatch)> {
        let m = self.find_next(text, from_pos)?;

        let replacement = match self.search_mode {
            SearchMode::Extended => Self::expand_escapes(&self.replace_text),
            _ => self.replace_text.clone(),
        };

        let new_text = if self.search_mode == SearchMode::Regex {
            let re = self.build_regex()?;
            // Replace only the specific match
            let mut result = String::with_capacity(text.len());
            result.push_str(&text[..m.start]);
            // Use regex replacement to handle backreferences
            if let Some(captures) = re.captures(&text[m.start..]) {
                let replaced = captures.expand_to_string(&replacement);
                result.push_str(&replaced);
            } else {
                result.push_str(&replacement);
            }
            result.push_str(&text[m.end..]);
            result
        } else {
            let mut result = String::with_capacity(text.len());
            result.push_str(&text[..m.start]);
            result.push_str(&replacement);
            result.push_str(&text[m.end..]);
            result
        };

        Some((new_text, m))
    }

    pub fn replace_all(&self, text: &str) -> (String, usize) {
        let re = match self.build_regex() {
            Some(r) => r,
            None => return (text.to_string(), 0),
        };

        let replacement = match self.search_mode {
            SearchMode::Extended => Self::expand_escapes(&self.replace_text),
            _ => self.replace_text.clone(),
        };

        let count = re.find_iter(text).count();
        let new_text = if self.search_mode == SearchMode::Regex {
            re.replace_all(text, replacement.as_str()).to_string()
        } else {
            re.replace_all(text, regex::NoExpand(replacement.as_str())).to_string()
        };
        (new_text, count)
    }

    pub fn count(&self, text: &str) -> usize {
        let re = match self.build_regex() {
            Some(r) => r,
            None => return 0,
        };
        re.find_iter(text).count()
    }
}

/// Helper trait to expand captures with backreferences.
trait ExpandCaptures {
    fn expand_to_string(&self, replacement: &str) -> String;
}

impl ExpandCaptures for regex::Captures<'_> {
    fn expand_to_string(&self, replacement: &str) -> String {
        let mut dst = String::new();
        self.expand(replacement, &mut dst);
        dst
    }
}
