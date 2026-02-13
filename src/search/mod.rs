pub mod engine;
pub mod find_in_files;
pub mod history;

pub use engine::{SearchEngine, SearchMatch, SearchMode};
pub use find_in_files::{FileSearchResult, FindInFiles};
pub use history::{SearchHistory, SearchHistoryEntry};
