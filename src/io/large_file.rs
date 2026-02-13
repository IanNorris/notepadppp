use std::path::Path;

/// Files above this size get large-file optimizations (syntax highlighting disabled, undo limit).
pub const LARGE_FILE_THRESHOLD: usize = 10 * 1024 * 1024; // 10 MB

/// Files above this size also disable minimap and function list.
pub const VERY_LARGE_FILE_THRESHOLD: usize = 100 * 1024 * 1024; // 100 MB

/// Undo history limit for large files to save memory.
pub const LARGE_FILE_UNDO_LIMIT: usize = 100;

/// Metadata about a file's size category and which features should be disabled.
#[derive(Debug, Clone)]
pub struct LargeFileInfo {
    pub file_size: u64,
    pub is_large: bool,
    pub is_very_large: bool,
    pub syntax_disabled: bool,
    pub minimap_disabled: bool,
    pub function_list_disabled: bool,
    pub undo_limit: Option<usize>,
}

impl Default for LargeFileInfo {
    fn default() -> Self {
        Self::from_size(0)
    }
}

impl LargeFileInfo {
    /// Create from a file path by reading its metadata.
    pub fn from_path(path: &Path) -> std::io::Result<Self> {
        let meta = std::fs::metadata(path)?;
        Ok(Self::from_size(meta.len()))
    }

    /// Create from a known file size.
    pub fn from_size(size: u64) -> Self {
        let is_large = size as usize >= LARGE_FILE_THRESHOLD;
        let is_very_large = size as usize >= VERY_LARGE_FILE_THRESHOLD;
        Self {
            file_size: size,
            is_large,
            is_very_large,
            syntax_disabled: is_large,
            minimap_disabled: is_very_large,
            function_list_disabled: is_very_large,
            undo_limit: if is_large {
                Some(LARGE_FILE_UNDO_LIMIT)
            } else {
                None
            },
        }
    }

    /// Human-readable size string (e.g. "12.5 MB").
    pub fn size_display(&self) -> String {
        let mb = self.file_size as f64 / (1024.0 * 1024.0);
        if mb >= 1.0 {
            format!("{:.1} MB", mb)
        } else {
            let kb = self.file_size as f64 / 1024.0;
            format!("{:.1} KB", kb)
        }
    }
}
