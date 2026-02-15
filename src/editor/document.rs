use std::path::PathBuf;

use super::bookmarks::BookmarkManager;
use super::buffer::TextBuffer;
use super::cursor::CursorState;
use super::folding::FoldManager;
use crate::io::large_file::LargeFileInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    UTF8,
    UTF8BOM,
    UTF16LE,
    UTF16BE,
    ASCII,
}

impl Encoding {
    /// Parse an encoding name from a CLI string (case-insensitive).
    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('-', "").as_str() {
            "utf8" => Some(Encoding::UTF8),
            "utf8bom" => Some(Encoding::UTF8BOM),
            "utf16le" => Some(Encoding::UTF16LE),
            "utf16be" => Some(Encoding::UTF16BE),
            "ascii" | "latin1" => Some(Encoding::ASCII),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    CRLF,
    LF,
    CR,
}

impl LineEnding {
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::CRLF => "\r\n",
            LineEnding::LF => "\n",
            LineEnding::CR => "\r",
        }
    }
}

/// A document combines a text buffer with cursor state and file metadata.
#[derive(Debug)]
pub struct Document {
    pub path: Option<PathBuf>,
    pub buffer: TextBuffer,
    pub cursor: CursorState,
    pub encoding: Encoding,
    pub line_ending: LineEnding,
    pub language: String,
    pub read_only: bool,
    pub bookmarks: BookmarkManager,
    pub large_file_info: LargeFileInfo,
    pub fold_manager: FoldManager,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    pub fn new() -> Self {
        Self {
            path: None,
            buffer: TextBuffer::new(),
            cursor: CursorState::new(),
            encoding: Encoding::UTF8,
            line_ending: LineEnding::LF,
            language: String::from("Plain Text"),
            read_only: false,
            bookmarks: BookmarkManager::default(),
            large_file_info: LargeFileInfo::default(),
            fold_manager: FoldManager::new(),
        }
    }

    pub fn from_str(text: &str) -> Self {
        Self {
            buffer: TextBuffer::from_str(text),
            ..Self::new()
        }
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    pub fn with_encoding(mut self, enc: Encoding) -> Self {
        self.encoding = enc;
        self
    }

    pub fn with_line_ending(mut self, le: LineEnding) -> Self {
        self.line_ending = le;
        self
    }

    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.language = lang.into();
        self
    }

    pub fn with_large_file_info(mut self, info: LargeFileInfo) -> Self {
        if let Some(limit) = info.undo_limit {
            self.buffer.set_undo_limit(Some(limit));
        }
        self.large_file_info = info;
        self
    }

    pub fn is_modified(&self) -> bool {
        self.buffer.is_modified()
    }

    pub fn title(&self) -> String {
        match &self.path {
            Some(p) => p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".to_string()),
            None => "Untitled".to_string(),
        }
    }
}
