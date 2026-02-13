use super::engine::SearchMatch;
use std::time::Instant;

pub struct SearchHistoryEntry {
    pub query: String,
    pub results: Vec<SearchMatch>,
    pub timestamp: Instant,
}

pub struct SearchHistory {
    entries: Vec<SearchHistoryEntry>,
    max_entries: usize,
}

impl SearchHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    pub fn add(&mut self, query: String, results: Vec<SearchMatch>) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(SearchHistoryEntry {
            query,
            results,
            timestamp: Instant::now(),
        });
    }

    pub fn entries(&self) -> &[SearchHistoryEntry] {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
