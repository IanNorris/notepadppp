use super::engine::{SearchEngine, SearchMatch, SearchMode};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct FileSearchResult {
    pub path: PathBuf,
    pub matches: Vec<SearchMatch>,
}

#[derive(Debug)]
pub struct ReplaceInFilesResult {
    pub path: PathBuf,
    pub replacements: usize,
}

pub struct FindInFiles {
    pub results: Vec<FileSearchResult>,
}

impl FindInFiles {
    /// Check if a filename matches a filter pattern like "*.rs" or "*.txt;*.md".
    fn matches_filter(filename: &str, filter: &str) -> bool {
        if filter.is_empty() || filter == "*" || filter == "*.*" {
            return true;
        }
        for pattern in filter.split(';') {
            let pattern = pattern.trim();
            if pattern.is_empty() {
                continue;
            }
            if glob_match::glob_match(pattern, filename) {
                return true;
            }
        }
        false
    }

    pub fn search(
        dir: &Path,
        query: &str,
        file_filter: &str,
        recursive: bool,
        case_sensitive: bool,
        use_regex: bool,
    ) -> Result<Vec<FileSearchResult>, Box<dyn std::error::Error>> {
        let mut engine = SearchEngine::new();
        engine.query = query.to_string();
        engine.case_sensitive = case_sensitive;
        engine.use_regex = use_regex;
        engine.search_mode = if use_regex {
            SearchMode::Regex
        } else {
            SearchMode::Normal
        };

        let walker = if recursive {
            WalkDir::new(dir)
        } else {
            WalkDir::new(dir).max_depth(1)
        };

        let mut results = Vec::new();

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }

            let filename = entry.file_name().to_string_lossy();
            if !Self::matches_filter(&filename, file_filter) {
                continue;
            }

            // Skip binary files by attempting to read as UTF-8
            let content = match fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let matches = engine.find_all(&content);
            if !matches.is_empty() {
                results.push(FileSearchResult {
                    path: entry.path().to_path_buf(),
                    matches,
                });
            }
        }

        Ok(results)
    }

    pub fn replace_in_files(
        dir: &Path,
        query: &str,
        replacement: &str,
        file_filter: &str,
        recursive: bool,
        case_sensitive: bool,
        use_regex: bool,
    ) -> Result<Vec<ReplaceInFilesResult>, Box<dyn std::error::Error>> {
        let mut engine = SearchEngine::new();
        engine.query = query.to_string();
        engine.replace_text = replacement.to_string();
        engine.case_sensitive = case_sensitive;
        engine.use_regex = use_regex;
        engine.search_mode = if use_regex {
            SearchMode::Regex
        } else {
            SearchMode::Normal
        };

        let walker = if recursive {
            WalkDir::new(dir)
        } else {
            WalkDir::new(dir).max_depth(1)
        };

        let mut results = Vec::new();

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }

            let filename = entry.file_name().to_string_lossy();
            if !Self::matches_filter(&filename, file_filter) {
                continue;
            }

            let content = match fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let (new_content, count) = engine.replace_all(&content);
            if count > 0 {
                fs::write(entry.path(), &new_content)?;
                results.push(ReplaceInFilesResult {
                    path: entry.path().to_path_buf(),
                    replacements: count,
                });
            }
        }

        Ok(results)
    }
}
