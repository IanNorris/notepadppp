use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const DEFAULT_MAX_ENTRIES: usize = 15;

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentFiles {
    entries: Vec<PathBuf>,
    max_entries: usize,
}

impl Default for RecentFiles {
    fn default() -> Self {
        Self::new()
    }
}

impl RecentFiles {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: DEFAULT_MAX_ENTRIES,
        }
    }

    pub fn with_max(max: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries: max,
        }
    }

    /// Add a path to the front. Removes duplicates and trims to max.
    pub fn add(&mut self, path: PathBuf) {
        self.entries.retain(|p| p != &path);
        self.entries.insert(0, path);
        self.entries.truncate(self.max_entries);
    }

    /// Remove a path from the list.
    pub fn remove(&mut self, path: &Path) {
        self.entries.retain(|p| p != path);
    }

    pub fn list(&self) -> &[PathBuf] {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Save the recent files list as JSON.
    pub fn save(&self, config_path: &Path) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write(config_path, json)
    }

    /// Load the recent files list from a JSON file.
    pub fn load(config_path: &Path) -> io::Result<Self> {
        let data = fs::read_to_string(config_path)?;
        let rf: RecentFiles = serde_json::from_str(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(rf)
    }
}
