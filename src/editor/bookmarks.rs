use std::collections::BTreeSet;

#[derive(Default, Clone, Debug)]
pub struct BookmarkManager {
    bookmarks: BTreeSet<usize>,
}

impl BookmarkManager {
    pub fn toggle(&mut self, line: usize) {
        if !self.bookmarks.remove(&line) {
            self.bookmarks.insert(line);
        }
    }

    pub fn add(&mut self, line: usize) {
        self.bookmarks.insert(line);
    }

    pub fn remove(&mut self, line: usize) {
        self.bookmarks.remove(&line);
    }

    pub fn is_bookmarked(&self, line: usize) -> bool {
        self.bookmarks.contains(&line)
    }

    /// Next bookmark after `from_line`, wrapping around if needed.
    pub fn next_bookmark(&self, from_line: usize) -> Option<usize> {
        if self.bookmarks.is_empty() {
            return None;
        }
        // Find first bookmark strictly after from_line
        self.bookmarks
            .range((from_line + 1)..)
            .next()
            .or_else(|| self.bookmarks.iter().next())
            .copied()
    }

    /// Previous bookmark before `from_line`, wrapping around if needed.
    pub fn prev_bookmark(&self, from_line: usize) -> Option<usize> {
        if self.bookmarks.is_empty() {
            return None;
        }
        self.bookmarks
            .range(..from_line)
            .next_back()
            .or_else(|| self.bookmarks.iter().next_back())
            .copied()
    }

    pub fn all_bookmarks(&self) -> Vec<usize> {
        self.bookmarks.iter().copied().collect()
    }

    pub fn clear(&mut self) {
        self.bookmarks.clear();
    }

    pub fn count(&self) -> usize {
        self.bookmarks.len()
    }

    /// Get text of all bookmarked lines.
    pub fn bookmarked_lines<'a>(&self, text: &'a str) -> Vec<String> {
        let lines: Vec<&str> = text.lines().collect();
        self.bookmarks
            .iter()
            .filter_map(|&idx| lines.get(idx).map(|l| l.to_string()))
            .collect()
    }

    /// Remove all bookmarked lines from text.
    pub fn remove_bookmarked_lines(&self, text: &str) -> String {
        text.lines()
            .enumerate()
            .filter(|(i, _)| !self.bookmarks.contains(i))
            .map(|(_, line)| line)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Keep only bookmarked lines.
    pub fn remove_unbookmarked_lines(&self, text: &str) -> String {
        text.lines()
            .enumerate()
            .filter(|(i, _)| self.bookmarks.contains(i))
            .map(|(_, line)| line)
            .collect::<Vec<_>>()
            .join("\n")
    }
}
