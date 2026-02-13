use notepadppp::editor::indent::{compute_indent, get_indent, reindent_paste};

#[test]
fn test_get_indent() {
    assert_eq!(get_indent("    hello"), "    ");
    assert_eq!(get_indent("hello"), "");
    assert_eq!(get_indent("\thello"), "\t");
    assert_eq!(get_indent(""), "");
}

#[test]
fn test_compute_indent_increase() {
    assert_eq!(compute_indent("fn main() {", "    "), "    ");
    assert_eq!(compute_indent("    if x > 0 {", "    "), "        ");
    assert_eq!(compute_indent("let arr = [", "    "), "    ");
}

#[test]
fn test_compute_indent_same() {
    assert_eq!(compute_indent("    let x = 5;", "    "), "    ");
    assert_eq!(compute_indent("println!(\"hello\");", "    "), "");
}

#[test]
fn test_reindent_paste_basic() {
    let pasted = "if x > 0 {\n    println!(\"yes\");\n}";
    let result = reindent_paste(pasted, "    ");
    assert_eq!(result, "if x > 0 {\n        println!(\"yes\");\n    }");
}

#[test]
fn test_reindent_paste_single_line() {
    let pasted = "let x = 5;";
    let result = reindent_paste(pasted, "    ");
    assert_eq!(result, "let x = 5;");
}

#[test]
fn test_reindent_paste_empty_lines() {
    let pasted = "a\n\nb";
    let result = reindent_paste(pasted, "  ");
    assert_eq!(result, "a\n\n  b");
}
