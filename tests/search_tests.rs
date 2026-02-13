use notepadppp::search::{
    FindInFiles, SearchEngine, SearchHistory, SearchMatch, SearchMode,
};
use std::fs;
use tempfile::TempDir;

// ─── Simple literal search ───────────────────────────────────────

#[test]
fn find_all_case_sensitive() {
    let mut engine = SearchEngine::new();
    engine.query = "Hello".to_string();
    let text = "Hello world, hello there, HELLO!";
    let matches = engine.find_all(text);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].match_text, "Hello");
    assert_eq!(matches[0].start, 0);
}

#[test]
fn find_all_case_insensitive() {
    let mut engine = SearchEngine::new();
    engine.query = "hello".to_string();
    engine.case_sensitive = false;
    let text = "Hello world, hello there, HELLO!";
    let matches = engine.find_all(text);
    assert_eq!(matches.len(), 3);
}

#[test]
fn find_all_multiple_lines() {
    let mut engine = SearchEngine::new();
    engine.query = "foo".to_string();
    let text = "foo bar\nbaz foo\nfoo";
    let matches = engine.find_all(text);
    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0].line, 0);
    assert_eq!(matches[1].line, 1);
    assert_eq!(matches[2].line, 2);
}

#[test]
fn find_all_line_text() {
    let mut engine = SearchEngine::new();
    engine.query = "bar".to_string();
    let text = "first line\nsecond bar line\nthird line";
    let matches = engine.find_all(text);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].line_text, "second bar line");
}

// ─── Whole word matching ─────────────────────────────────────────

#[test]
fn whole_word_match() {
    let mut engine = SearchEngine::new();
    engine.query = "the".to_string();
    engine.whole_word = true;
    let text = "the other theme is the best";
    let matches = engine.find_all(text);
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].start, 0);
    assert_eq!(matches[1].match_text, "the");
}

#[test]
fn whole_word_no_partial() {
    let mut engine = SearchEngine::new();
    engine.query = "cat".to_string();
    engine.whole_word = true;
    let text = "concatenate category cat catalog";
    let matches = engine.find_all(text);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].start, text.find("cat ").unwrap() + 0);
}

// ─── Regex search ────────────────────────────────────────────────

#[test]
fn regex_basic_pattern() {
    let mut engine = SearchEngine::new();
    engine.query = r"\d+".to_string();
    engine.search_mode = SearchMode::Regex;
    let text = "abc 123 def 456";
    let matches = engine.find_all(text);
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].match_text, "123");
    assert_eq!(matches[1].match_text, "456");
}

#[test]
fn regex_groups() {
    let mut engine = SearchEngine::new();
    engine.query = r"(\w+)@(\w+)\.(\w+)".to_string();
    engine.search_mode = SearchMode::Regex;
    let text = "email: user@example.com and admin@test.org";
    let matches = engine.find_all(text);
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].match_text, "user@example.com");
}

#[test]
fn regex_word_boundary() {
    let mut engine = SearchEngine::new();
    engine.query = r"\b\w{4}\b".to_string();
    engine.search_mode = SearchMode::Regex;
    let text = "file.txt image.png code";
    let matches = engine.find_all(text);
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].match_text, "file");
    assert_eq!(matches[1].match_text, "code");
}

#[test]
fn regex_case_insensitive() {
    let mut engine = SearchEngine::new();
    engine.query = r"hello".to_string();
    engine.search_mode = SearchMode::Regex;
    engine.case_sensitive = false;
    let text = "Hello HELLO hello";
    let matches = engine.find_all(text);
    assert_eq!(matches.len(), 3);
}

// ─── Extended mode ───────────────────────────────────────────────

#[test]
fn extended_newline() {
    let mut engine = SearchEngine::new();
    engine.query = r"foo\nbar".to_string();
    engine.search_mode = SearchMode::Extended;
    let text = "foo\nbar baz";
    let matches = engine.find_all(text);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].match_text, "foo\nbar");
}

#[test]
fn extended_tab() {
    let mut engine = SearchEngine::new();
    engine.query = r"a\tb".to_string();
    engine.search_mode = SearchMode::Extended;
    let text = "a\tb c";
    let matches = engine.find_all(text);
    assert_eq!(matches.len(), 1);
}

#[test]
fn extended_carriage_return() {
    let mut engine = SearchEngine::new();
    engine.query = r"line\r\n".to_string();
    engine.search_mode = SearchMode::Extended;
    let text = "line\r\nnext";
    let matches = engine.find_all(text);
    assert_eq!(matches.len(), 1);
}

#[test]
fn extended_backslash() {
    let mut engine = SearchEngine::new();
    engine.query = r"a\\b".to_string();
    engine.search_mode = SearchMode::Extended;
    let text = r"a\b c";
    let matches = engine.find_all(text);
    assert_eq!(matches.len(), 1);
}

// ─── Find next from position ────────────────────────────────────

#[test]
fn find_next_from_middle() {
    let mut engine = SearchEngine::new();
    engine.query = "cat".to_string();
    engine.wrap_around = false;
    let text = "cat dog cat fish cat";
    let m = engine.find_next(text, 4).unwrap();
    assert_eq!(m.start, 8);
}

#[test]
fn find_next_wraps_around() {
    let mut engine = SearchEngine::new();
    engine.query = "cat".to_string();
    engine.wrap_around = true;
    let text = "cat dog fish";
    let m = engine.find_next(text, 4).unwrap();
    assert_eq!(m.start, 0);
}

#[test]
fn find_next_no_wrap_returns_none() {
    let mut engine = SearchEngine::new();
    engine.query = "cat".to_string();
    engine.wrap_around = false;
    let text = "cat dog fish";
    let m = engine.find_next(text, 4);
    assert!(m.is_none());
}

// ─── Find previous ──────────────────────────────────────────────

#[test]
fn find_prev_from_end() {
    let mut engine = SearchEngine::new();
    engine.query = "cat".to_string();
    engine.wrap_around = false;
    let text = "cat dog cat fish cat";
    let m = engine.find_prev(text, 20).unwrap();
    assert_eq!(m.start, 17);
}

#[test]
fn find_prev_from_middle() {
    let mut engine = SearchEngine::new();
    engine.query = "cat".to_string();
    engine.wrap_around = false;
    let text = "cat dog cat fish cat";
    let m = engine.find_prev(text, 10).unwrap();
    assert_eq!(m.start, 8);
}

#[test]
fn find_prev_wraps_around() {
    let mut engine = SearchEngine::new();
    engine.query = "cat".to_string();
    engine.wrap_around = true;
    let text = "dog fish cat";
    let m = engine.find_prev(text, 0).unwrap();
    assert_eq!(m.start, 9);
}

#[test]
fn find_prev_no_wrap_returns_none() {
    let mut engine = SearchEngine::new();
    engine.query = "cat".to_string();
    engine.wrap_around = false;
    let text = "dog fish cat";
    let m = engine.find_prev(text, 0);
    assert!(m.is_none());
}

// ─── Replace next ────────────────────────────────────────────────

#[test]
fn replace_next_basic() {
    let mut engine = SearchEngine::new();
    engine.query = "world".to_string();
    engine.replace_text = "Rust".to_string();
    let text = "hello world!";
    let (new_text, m) = engine.replace_next(text, 0).unwrap();
    assert_eq!(new_text, "hello Rust!");
    assert_eq!(m.match_text, "world");
}

#[test]
fn replace_next_from_position() {
    let mut engine = SearchEngine::new();
    engine.query = "a".to_string();
    engine.replace_text = "X".to_string();
    engine.wrap_around = false;
    let text = "a b a c a";
    let (new_text, m) = engine.replace_next(text, 3).unwrap();
    assert_eq!(new_text, "a b X c a");
    assert_eq!(m.start, 4);
}

#[test]
fn replace_next_regex_backreference() {
    let mut engine = SearchEngine::new();
    engine.query = r"(\w+)@(\w+)".to_string();
    engine.replace_text = "$2/$1".to_string();
    engine.search_mode = SearchMode::Regex;
    let text = "user@host rest";
    let (new_text, _) = engine.replace_next(text, 0).unwrap();
    assert_eq!(new_text, "host/user rest");
}

// ─── Replace all ─────────────────────────────────────────────────

#[test]
fn replace_all_basic() {
    let mut engine = SearchEngine::new();
    engine.query = "cat".to_string();
    engine.replace_text = "dog".to_string();
    let text = "cat and cat and cat";
    let (new_text, count) = engine.replace_all(text);
    assert_eq!(new_text, "dog and dog and dog");
    assert_eq!(count, 3);
}

#[test]
fn replace_all_case_insensitive() {
    let mut engine = SearchEngine::new();
    engine.query = "hello".to_string();
    engine.replace_text = "hi".to_string();
    engine.case_sensitive = false;
    let text = "Hello HELLO hello";
    let (new_text, count) = engine.replace_all(text);
    assert_eq!(new_text, "hi hi hi");
    assert_eq!(count, 3);
}

#[test]
fn replace_all_no_matches() {
    let mut engine = SearchEngine::new();
    engine.query = "xyz".to_string();
    engine.replace_text = "abc".to_string();
    let text = "hello world";
    let (new_text, count) = engine.replace_all(text);
    assert_eq!(new_text, "hello world");
    assert_eq!(count, 0);
}

#[test]
fn replace_all_regex() {
    let mut engine = SearchEngine::new();
    engine.query = r"\d+".to_string();
    engine.replace_text = "#".to_string();
    engine.search_mode = SearchMode::Regex;
    let text = "a1b22c333";
    let (new_text, count) = engine.replace_all(text);
    assert_eq!(new_text, "a#b#c#");
    assert_eq!(count, 3);
}

// ─── Count matches ───────────────────────────────────────────────

#[test]
fn count_matches() {
    let mut engine = SearchEngine::new();
    engine.query = "the".to_string();
    let text = "the theme is the best of the three";
    assert_eq!(engine.count(text), 4);
}

#[test]
fn count_whole_word() {
    let mut engine = SearchEngine::new();
    engine.query = "the".to_string();
    engine.whole_word = true;
    let text = "the theme is the best of the three";
    assert_eq!(engine.count(text), 3);
}

// ─── No matches / empty query ────────────────────────────────────

#[test]
fn no_matches() {
    let mut engine = SearchEngine::new();
    engine.query = "xyz".to_string();
    let text = "hello world";
    assert_eq!(engine.find_all(text).len(), 0);
    assert!(engine.find_next(text, 0).is_none());
    assert!(engine.find_prev(text, text.len()).is_none());
    assert_eq!(engine.count(text), 0);
}

#[test]
fn empty_query() {
    let engine = SearchEngine::new();
    let text = "hello world";
    assert_eq!(engine.find_all(text).len(), 0);
    assert!(engine.find_next(text, 0).is_none());
    assert_eq!(engine.count(text), 0);
    let (new_text, count) = engine.replace_all(text);
    assert_eq!(new_text, "hello world");
    assert_eq!(count, 0);
}

// ─── Unicode text search ─────────────────────────────────────────

#[test]
fn unicode_search() {
    let mut engine = SearchEngine::new();
    engine.query = "日本".to_string();
    let text = "Hello 日本語 and 日本!";
    let matches = engine.find_all(text);
    assert_eq!(matches.len(), 2);
}

#[test]
fn unicode_emoji_search() {
    let mut engine = SearchEngine::new();
    engine.query = "🦀".to_string();
    let text = "Rust 🦀 is great 🦀!";
    let matches = engine.find_all(text);
    assert_eq!(matches.len(), 2);
}

#[test]
fn unicode_replace_all() {
    let mut engine = SearchEngine::new();
    engine.query = "café".to_string();
    engine.replace_text = "coffee".to_string();
    let text = "I love café, the best café";
    let (new_text, count) = engine.replace_all(text);
    assert_eq!(new_text, "I love coffee, the best coffee");
    assert_eq!(count, 2);
}

// ─── Find in files ───────────────────────────────────────────────

#[test]
fn find_in_files_basic() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.txt"), "hello world\nhello rust").unwrap();
    fs::write(dir.path().join("other.txt"), "goodbye world").unwrap();
    fs::write(dir.path().join("skip.md"), "hello markdown").unwrap();

    let results = FindInFiles::search(dir.path(), "hello", "*.txt", false, true, false).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].matches.len(), 2);
}

#[test]
fn find_in_files_multiple_filters() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), "hello").unwrap();
    fs::write(dir.path().join("b.md"), "hello").unwrap();
    fs::write(dir.path().join("c.rs"), "hello").unwrap();

    let results =
        FindInFiles::search(dir.path(), "hello", "*.txt;*.md", false, true, false).unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn find_in_files_recursive() {
    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("sub");
    fs::create_dir(&sub).unwrap();
    fs::write(dir.path().join("top.txt"), "match here").unwrap();
    fs::write(sub.join("nested.txt"), "match here too").unwrap();

    let results = FindInFiles::search(dir.path(), "match", "*.txt", true, true, false).unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn find_in_files_not_recursive() {
    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("sub");
    fs::create_dir(&sub).unwrap();
    fs::write(dir.path().join("top.txt"), "match here").unwrap();
    fs::write(sub.join("nested.txt"), "match here too").unwrap();

    let results = FindInFiles::search(dir.path(), "match", "*.txt", false, true, false).unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn find_in_files_case_insensitive() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.txt"), "Hello HELLO hello").unwrap();

    let results = FindInFiles::search(dir.path(), "hello", "*.txt", false, false, false).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].matches.len(), 3);
}

#[test]
fn find_in_files_regex() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.txt"), "abc 123 def 456").unwrap();

    let results = FindInFiles::search(dir.path(), r"\d+", "*.txt", false, true, true).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].matches.len(), 2);
}

#[test]
fn find_in_files_no_matches() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.txt"), "hello world").unwrap();

    let results =
        FindInFiles::search(dir.path(), "xyz123", "*.txt", false, true, false).unwrap();
    assert_eq!(results.len(), 0);
}

// ─── Search history ──────────────────────────────────────────────

#[test]
fn history_add_and_retrieve() {
    let mut history = SearchHistory::new(10);
    history.add("test".to_string(), vec![]);
    history.add("hello".to_string(), vec![]);
    assert_eq!(history.entries().len(), 2);
    assert_eq!(history.entries()[0].query, "test");
    assert_eq!(history.entries()[1].query, "hello");
}

#[test]
fn history_max_entries() {
    let mut history = SearchHistory::new(3);
    history.add("a".to_string(), vec![]);
    history.add("b".to_string(), vec![]);
    history.add("c".to_string(), vec![]);
    history.add("d".to_string(), vec![]);
    assert_eq!(history.entries().len(), 3);
    assert_eq!(history.entries()[0].query, "b");
    assert_eq!(history.entries()[2].query, "d");
}

#[test]
fn history_clear() {
    let mut history = SearchHistory::new(10);
    history.add("test".to_string(), vec![]);
    history.clear();
    assert_eq!(history.entries().len(), 0);
}

#[test]
fn history_with_results() {
    let mut engine = SearchEngine::new();
    engine.query = "foo".to_string();
    let text = "foo bar foo";
    let results = engine.find_all(text);

    let mut history = SearchHistory::new(10);
    history.add("foo".to_string(), results);
    assert_eq!(history.entries()[0].results.len(), 2);
}

// ─── Extended mode replace ───────────────────────────────────────

#[test]
fn replace_all_extended_mode() {
    let mut engine = SearchEngine::new();
    engine.query = r"\t".to_string();
    engine.replace_text = "    ".to_string();
    engine.search_mode = SearchMode::Extended;
    let text = "a\tb\tc";
    let (new_text, count) = engine.replace_all(text);
    assert_eq!(new_text, "a    b    c");
    assert_eq!(count, 2);
}

// ─── Edge cases ──────────────────────────────────────────────────

#[test]
fn search_at_boundaries() {
    let mut engine = SearchEngine::new();
    engine.query = "x".to_string();
    let text = "xax";
    let matches = engine.find_all(text);
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].start, 0);
    assert_eq!(matches[1].start, 2);
}

#[test]
fn replace_with_empty_string() {
    let mut engine = SearchEngine::new();
    engine.query = "remove".to_string();
    engine.replace_text = String::new();
    let text = "please remove this remove text";
    let (new_text, count) = engine.replace_all(text);
    assert_eq!(new_text, "please  this  text");
    assert_eq!(count, 2);
}

#[test]
fn find_next_at_exact_match_position() {
    let mut engine = SearchEngine::new();
    engine.query = "ab".to_string();
    engine.wrap_around = false;
    let text = "ab cd ab";
    let m = engine.find_next(text, 0).unwrap();
    assert_eq!(m.start, 0);
}

#[test]
fn replace_next_no_match_returns_none() {
    let mut engine = SearchEngine::new();
    engine.query = "xyz".to_string();
    engine.replace_text = "abc".to_string();
    engine.wrap_around = false;
    let text = "hello world";
    assert!(engine.replace_next(text, 0).is_none());
}
