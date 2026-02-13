use notepadppp::editor::comments;
use notepadppp::editor::autoclose;

// --- Comment toggle tests ---

#[test]
fn test_line_comment_prefix_rust() {
    assert_eq!(comments::line_comment_prefix("Rust"), Some("//"));
}

#[test]
fn test_line_comment_prefix_python() {
    assert_eq!(comments::line_comment_prefix("Python"), Some("#"));
}

#[test]
fn test_line_comment_prefix_lua() {
    assert_eq!(comments::line_comment_prefix("Lua"), Some("--"));
}

#[test]
fn test_line_comment_prefix_lisp() {
    assert_eq!(comments::line_comment_prefix("Lisp"), Some(";"));
}

#[test]
fn test_line_comment_prefix_html_none() {
    assert_eq!(comments::line_comment_prefix("HTML"), None);
}

#[test]
fn test_line_comment_prefix_css_none() {
    assert_eq!(comments::line_comment_prefix("CSS"), None);
}

#[test]
fn test_line_comment_prefix_unknown_none() {
    assert_eq!(comments::line_comment_prefix("BrainFuck"), None);
}

#[test]
fn test_line_comment_prefix_c_languages() {
    for lang in &["C", "C++", "Java", "JavaScript", "TypeScript", "Go", "Kotlin", "Swift", "C#", "Dart"] {
        assert_eq!(comments::line_comment_prefix(lang), Some("//"), "Failed for {}", lang);
    }
}

#[test]
fn test_line_comment_prefix_hash_languages() {
    for lang in &["Ruby", "Perl", "R", "Shell", "Bash", "PowerShell", "YAML", "TOML", "Makefile", "Dockerfile"] {
        assert_eq!(comments::line_comment_prefix(lang), Some("#"), "Failed for {}", lang);
    }
}

#[test]
fn test_block_comment_delimiters_c_style() {
    assert_eq!(comments::block_comment_delimiters("Rust"), Some(("/*", "*/")));
    assert_eq!(comments::block_comment_delimiters("JavaScript"), Some(("/*", "*/")));
}

#[test]
fn test_block_comment_delimiters_html() {
    assert_eq!(comments::block_comment_delimiters("HTML"), Some(("<!--", "-->")));
}

#[test]
fn test_block_comment_delimiters_python() {
    assert_eq!(comments::block_comment_delimiters("Python"), Some(("\"\"\"", "\"\"\"")));
}

#[test]
fn test_block_comment_delimiters_lua() {
    assert_eq!(comments::block_comment_delimiters("Lua"), Some(("--[[", "]]")));
}

#[test]
fn test_block_comment_delimiters_unknown_none() {
    assert_eq!(comments::block_comment_delimiters("BrainFuck"), None);
}

// --- Auto-close pair tests ---

#[test]
fn test_closing_pair_parentheses() {
    assert_eq!(autoclose::closing_pair('('), Some(')'));
}

#[test]
fn test_closing_pair_brackets() {
    assert_eq!(autoclose::closing_pair('['), Some(']'));
}

#[test]
fn test_closing_pair_braces() {
    assert_eq!(autoclose::closing_pair('{'), Some('}'));
}

#[test]
fn test_closing_pair_double_quote() {
    assert_eq!(autoclose::closing_pair('"'), Some('"'));
}

#[test]
fn test_closing_pair_single_quote() {
    assert_eq!(autoclose::closing_pair('\''), Some('\''));
}

#[test]
fn test_closing_pair_backtick() {
    assert_eq!(autoclose::closing_pair('`'), Some('`'));
}

#[test]
fn test_closing_pair_no_match() {
    assert_eq!(autoclose::closing_pair('a'), None);
    assert_eq!(autoclose::closing_pair('+'), None);
    assert_eq!(autoclose::closing_pair(')'), None);
}

#[test]
fn test_should_autoclose_end_of_line() {
    assert!(autoclose::should_autoclose('(', None));
    assert!(autoclose::should_autoclose('{', None));
    assert!(autoclose::should_autoclose('"', None));
}

#[test]
fn test_should_autoclose_before_whitespace() {
    assert!(autoclose::should_autoclose('(', Some(' ')));
    assert!(autoclose::should_autoclose('{', Some('\n')));
    assert!(autoclose::should_autoclose('[', Some('\t')));
}

#[test]
fn test_should_autoclose_before_closing_bracket() {
    assert!(autoclose::should_autoclose('(', Some(')')));
    assert!(autoclose::should_autoclose('{', Some('}')));
    assert!(autoclose::should_autoclose('[', Some(']')));
}

#[test]
fn test_should_autoclose_before_comma_semicolon() {
    assert!(autoclose::should_autoclose('(', Some(',')));
    assert!(autoclose::should_autoclose('{', Some(';')));
}

#[test]
fn test_should_not_autoclose_before_letter() {
    assert!(!autoclose::should_autoclose('(', Some('a')));
    assert!(!autoclose::should_autoclose('{', Some('x')));
}

#[test]
fn test_should_not_autoclose_non_pair() {
    assert!(!autoclose::should_autoclose('a', None));
    assert!(!autoclose::should_autoclose('+', Some(' ')));
}

// --- Comment toggle simulation tests ---
// Test the comment toggling logic directly

fn toggle_comment_line(line: &str, prefix: &str) -> String {
    let prefix_space = format!("{} ", prefix);
    let trimmed = line.trim_start();
    if trimmed.starts_with(&prefix_space) {
        let indent = line.len() - trimmed.len();
        let rest = &trimmed[prefix_space.len()..];
        format!("{}{}", &line[..indent], rest)
    } else if trimmed.starts_with(prefix) {
        let indent = line.len() - trimmed.len();
        let rest = &trimmed[prefix.len()..];
        format!("{}{}", &line[..indent], rest)
    } else {
        let indent = line.len() - trimmed.len();
        format!("{}{} {}", &line[..indent], prefix, trimmed)
    }
}

#[test]
fn test_toggle_comment_add_rust() {
    assert_eq!(toggle_comment_line("let x = 5;", "//"), "// let x = 5;");
}

#[test]
fn test_toggle_comment_remove_rust() {
    assert_eq!(toggle_comment_line("// let x = 5;", "//"), "let x = 5;");
}

#[test]
fn test_toggle_comment_add_with_indent() {
    assert_eq!(toggle_comment_line("    let x = 5;", "//"), "    // let x = 5;");
}

#[test]
fn test_toggle_comment_remove_with_indent() {
    assert_eq!(toggle_comment_line("    // let x = 5;", "//"), "    let x = 5;");
}

#[test]
fn test_toggle_comment_add_python() {
    assert_eq!(toggle_comment_line("x = 5", "#"), "# x = 5");
}

#[test]
fn test_toggle_comment_remove_python() {
    assert_eq!(toggle_comment_line("# x = 5", "#"), "x = 5");
}

#[test]
fn test_toggle_comment_remove_no_space() {
    assert_eq!(toggle_comment_line("//let x = 5;", "//"), "let x = 5;");
}

#[test]
fn test_toggle_comment_empty_line() {
    assert_eq!(toggle_comment_line("", "//"), "// ");
}

#[test]
fn test_toggle_comment_lua() {
    assert_eq!(toggle_comment_line("local x = 5", "--"), "-- local x = 5");
    assert_eq!(toggle_comment_line("-- local x = 5", "--"), "local x = 5");
}

#[test]
fn test_toggle_comment_lisp() {
    assert_eq!(toggle_comment_line("(defun hello)", ";"), "; (defun hello)");
    assert_eq!(toggle_comment_line("; (defun hello)", ";"), "(defun hello)");
}

#[test]
fn test_toggle_comment_roundtrip() {
    let original = "    fn main() {}";
    let commented = toggle_comment_line(original, "//");
    assert_eq!(commented, "    // fn main() {}");
    let uncommented = toggle_comment_line(&commented, "//");
    assert_eq!(uncommented, original);
}
