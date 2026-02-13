use std::io;
use std::path::PathBuf;

use super::document::Document;
use crate::io::file_io;

/// Manages multiple open documents as tabs.
#[derive(Debug)]
pub struct TabManager {
    tabs: Vec<Document>,
    active_tab: usize,
    untitled_counter: usize,
    /// Stores the "Untitled N" label for tabs without a path.
    untitled_names: Vec<Option<String>>,
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TabManager {
    pub fn new() -> Self {
        let mut mgr = Self {
            tabs: Vec::new(),
            active_tab: 0,
            untitled_counter: 0,
            untitled_names: Vec::new(),
        };
        mgr.new_tab();
        mgr
    }

    /// Create a new untitled document and return its tab index.
    pub fn new_tab(&mut self) -> usize {
        self.untitled_counter += 1;
        let name = format!("Untitled {}", self.untitled_counter);
        let doc = Document::new();
        self.tabs.push(doc);
        self.untitled_names.push(Some(name));
        let idx = self.tabs.len() - 1;
        self.active_tab = idx;
        idx
    }

    /// Open a file, creating a new tab. Returns the tab index.
    pub fn open_file(&mut self, path: PathBuf) -> io::Result<usize> {
        let (content, encoding, line_ending) = file_io::read_file(&path)?;
        let doc = Document::from_str(&content)
            .with_path(path)
            .with_encoding(encoding)
            .with_line_ending(line_ending);
        self.tabs.push(doc);
        self.untitled_names.push(None);
        let idx = self.tabs.len() - 1;
        self.active_tab = idx;
        Ok(idx)
    }

    /// Close tab at index. Returns the document if it was modified (for save prompt).
    pub fn close_tab(&mut self, index: usize) -> Option<Document> {
        if index >= self.tabs.len() {
            return None;
        }
        let doc = self.tabs.remove(index);
        self.untitled_names.remove(index);

        if self.tabs.is_empty() {
            self.new_tab();
        } else if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        } else if self.active_tab > index {
            self.active_tab -= 1;
        }

        if doc.is_modified() {
            Some(doc)
        } else {
            None
        }
    }

    /// Close all tabs, replacing with a single new untitled tab.
    pub fn close_all(&mut self) {
        self.tabs.clear();
        self.untitled_names.clear();
        self.active_tab = 0;
        self.new_tab();
    }

    /// Save a tab to its existing file path.
    pub fn save_tab(&mut self, index: usize) -> io::Result<()> {
        let doc = self.tabs.get(index).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Tab index out of range")
        })?;
        let path = doc.path.clone().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Document has no file path")
        })?;
        let content = doc.buffer.text();
        let encoding = doc.encoding;
        let line_ending = doc.line_ending;
        file_io::write_file(&path, &content, encoding, line_ending)?;
        self.tabs[index].buffer.set_modified(false);
        Ok(())
    }

    /// Save a tab to a new path.
    pub fn save_tab_as(&mut self, index: usize, path: PathBuf) -> io::Result<()> {
        let doc = self.tabs.get(index).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Tab index out of range")
        })?;
        let content = doc.buffer.text();
        let encoding = doc.encoding;
        let line_ending = doc.line_ending;
        file_io::write_file(&path, &content, encoding, line_ending)?;
        self.tabs[index].path = Some(path);
        self.tabs[index].buffer.set_modified(false);
        self.untitled_names[index] = None;
        Ok(())
    }

    pub fn active_document(&self) -> &Document {
        &self.tabs[self.active_tab]
    }

    pub fn active_document_mut(&mut self) -> &mut Document {
        &mut self.tabs[self.active_tab]
    }

    pub fn set_active(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab = index;
        }
    }

    pub fn active_index(&self) -> usize {
        self.active_tab
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Get the display title for a tab.
    pub fn get_tab_title(&self, index: usize) -> String {
        if index >= self.tabs.len() {
            return String::new();
        }
        if let Some(ref path) = self.tabs[index].path {
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".to_string())
        } else if let Some(Some(name)) = self.untitled_names.get(index) {
            name.clone()
        } else {
            "Untitled".to_string()
        }
    }

    /// Reorder a tab from one position to another.
    pub fn move_tab(&mut self, from: usize, to: usize) {
        if from >= self.tabs.len() || to >= self.tabs.len() || from == to {
            return;
        }
        let doc = self.tabs.remove(from);
        let name = self.untitled_names.remove(from);
        self.tabs.insert(to, doc);
        self.untitled_names.insert(to, name);

        // Adjust active_tab to follow the moved tab if it was active
        if self.active_tab == from {
            self.active_tab = to;
        } else if from < to {
            if self.active_tab > from && self.active_tab <= to {
                self.active_tab -= 1;
            }
        } else if self.active_tab >= to && self.active_tab < from {
            self.active_tab += 1;
        }
    }

    /// Returns true if any open document is modified.
    pub fn is_any_modified(&self) -> bool {
        self.tabs.iter().any(|d| d.is_modified())
    }

    /// Get a reference to a document by index.
    pub fn get_document(&self, index: usize) -> Option<&Document> {
        self.tabs.get(index)
    }

    /// Get a mutable reference to a document by index.
    pub fn get_document_mut(&mut self, index: usize) -> Option<&mut Document> {
        self.tabs.get_mut(index)
    }

    /// Reload a tab's content from disk.
    pub fn reload_tab(&mut self, index: usize) -> Result<(), String> {
        if index >= self.tabs.len() {
            return Err("Invalid tab index".to_string());
        }
        let path = self.tabs[index].path.clone()
            .ok_or("Tab has no file path")?;
        let (content, encoding, line_ending) = file_io::read_file(&path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        let len = self.tabs[index].buffer.len_bytes();
        if len > 0 {
            self.tabs[index].buffer.delete(0, len);
        }
        if !content.is_empty() {
            self.tabs[index].buffer.insert(0, &content);
        }
        self.tabs[index].buffer.set_modified(false);
        self.tabs[index].encoding = encoding;
        self.tabs[index].line_ending = line_ending;
        Ok(())
    }
}
