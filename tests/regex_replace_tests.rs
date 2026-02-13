use notepadppp::search::{SearchEngine, SearchMode};

// ─── Replace with $1 capture group ──────────────────────────────

#[test]
fn replace_with_single_capture_group() {
    let mut engine = SearchEngine::new();
    engine.query = r"(\d+)px".to_string();
    engine.replace_text = "$1 pixels".to_string();
    engine.search_mode = SearchMode::Regex;
    let text = "width: 100px; height: 200px;";
    let (new_text, _) = engine.replace_next(text, 0).unwrap();
    assert_eq!(new_text, "width: 100 pixels; height: 200px;");
}

// ─── Replace with multiple capture groups $1, $2 ────────────────

#[test]
fn replace_with_multiple_capture_groups() {
    let mut engine = SearchEngine::new();
    engine.query = r"(\w+):(\w+)".to_string();
    engine.replace_text = "$2=$1".to_string();
    engine.search_mode = SearchMode::Regex;
    let text = "key:value rest";
    let (new_text, _) = engine.replace_next(text, 0).unwrap();
    assert_eq!(new_text, "value=key rest");
}

#[test]
fn replace_with_three_capture_groups() {
    let mut engine = SearchEngine::new();
    engine.query = r"(\d{4})-(\d{2})-(\d{2})".to_string();
    engine.replace_text = "$2/$3/$1".to_string();
    engine.search_mode = SearchMode::Regex;
    let text = "date: 2025-01-15";
    let (new_text, _) = engine.replace_next(text, 0).unwrap();
    assert_eq!(new_text, "date: 01/15/2025");
}

// ─── Replace with named capture groups ${name} ──────────────────

#[test]
fn replace_with_named_capture_group() {
    let mut engine = SearchEngine::new();
    engine.query = r"(?P<first>\w+)\s+(?P<last>\w+)".to_string();
    engine.replace_text = "$last, $first".to_string();
    engine.search_mode = SearchMode::Regex;
    let text = "John Smith";
    let (new_text, _) = engine.replace_next(text, 0).unwrap();
    assert_eq!(new_text, "Smith, John");
}

#[test]
fn replace_with_named_capture_group_braces() {
    let mut engine = SearchEngine::new();
    engine.query = r"(?P<user>\w+)@(?P<domain>\w+)".to_string();
    engine.replace_text = "${domain}/${user}".to_string();
    engine.search_mode = SearchMode::Regex;
    let text = "admin@server rest";
    let (new_text, _) = engine.replace_next(text, 0).unwrap();
    assert_eq!(new_text, "server/admin rest");
}

// ─── Replace with $0 (whole match) ──────────────────────────────

#[test]
fn replace_with_whole_match_ref() {
    let mut engine = SearchEngine::new();
    engine.query = r"\b\w+\b".to_string();
    engine.replace_text = "[$0]".to_string();
    engine.search_mode = SearchMode::Regex;
    let text = "hello world";
    let (new_text, _) = engine.replace_next(text, 0).unwrap();
    assert_eq!(new_text, "[hello] world");
}

// ─── Replace all with capture groups ────────────────────────────

#[test]
fn replace_all_with_capture_groups() {
    let mut engine = SearchEngine::new();
    engine.query = r"(\w+)@(\w+)\.(\w+)".to_string();
    engine.replace_text = "$1 at $2 dot $3".to_string();
    engine.search_mode = SearchMode::Regex;
    let text = "user@example.com and admin@test.org";
    let (new_text, count) = engine.replace_all(text);
    assert_eq!(new_text, "user at example dot com and admin at test dot org");
    assert_eq!(count, 2);
}

#[test]
fn replace_all_with_whole_match() {
    let mut engine = SearchEngine::new();
    engine.query = r"\d+".to_string();
    engine.replace_text = "($0)".to_string();
    engine.search_mode = SearchMode::Regex;
    let text = "a1 b22 c333";
    let (new_text, count) = engine.replace_all(text);
    assert_eq!(new_text, "a(1) b(22) c(333)");
    assert_eq!(count, 3);
}

// ─── Non-regex replace still works normally ─────────────────────

#[test]
fn non_regex_replace_ignores_dollar_signs() {
    let mut engine = SearchEngine::new();
    engine.query = "hello".to_string();
    engine.replace_text = "$1 world".to_string();
    engine.search_mode = SearchMode::Normal;
    let text = "say hello!";
    let (new_text, _) = engine.replace_next(text, 0).unwrap();
    assert_eq!(new_text, "say $1 world!");
}

#[test]
fn non_regex_replace_all_ignores_dollar_signs() {
    let mut engine = SearchEngine::new();
    engine.query = "x".to_string();
    engine.replace_text = "$0".to_string();
    engine.search_mode = SearchMode::Normal;
    let text = "x y x";
    let (new_text, count) = engine.replace_all(text);
    assert_eq!(new_text, "$0 y $0");
    assert_eq!(count, 2);
}
