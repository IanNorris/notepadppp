use notepadppp::editor::brackets::{find_matching_bracket, is_bracket};

#[test]
fn test_find_matching_paren_forward() {
    let text = "fn main() {}";
    // '(' is at byte 7
    assert_eq!(find_matching_bracket(text, 7), Some(8));
}

#[test]
fn test_find_matching_paren_backward() {
    let text = "fn main() {}";
    // ')' is at byte 8
    assert_eq!(find_matching_bracket(text, 8), Some(7));
}

#[test]
fn test_find_matching_brace_nested() {
    let text = "{ { } }";
    assert_eq!(find_matching_bracket(text, 0), Some(6));
    assert_eq!(find_matching_bracket(text, 2), Some(4));
}

#[test]
fn test_find_matching_bracket_square() {
    let text = "let a = [1, [2, 3], 4];";
    assert_eq!(find_matching_bracket(text, 8), Some(21));
    assert_eq!(find_matching_bracket(text, 12), Some(17));
}

#[test]
fn test_no_match() {
    let text = "((()";
    assert_eq!(find_matching_bracket(text, 0), None);
}

#[test]
fn test_not_a_bracket() {
    let text = "hello";
    assert_eq!(find_matching_bracket(text, 0), None);
}

#[test]
fn test_is_bracket_fn() {
    let text = "()[]{}abc";
    assert!(is_bracket(text, 0));
    assert!(is_bracket(text, 1));
    assert!(is_bracket(text, 2));
    assert!(is_bracket(text, 3));
    assert!(is_bracket(text, 4));
    assert!(is_bracket(text, 5));
    assert!(!is_bracket(text, 6));
}

#[test]
fn test_multiline_brackets() {
    let text = "fn main() {\n    let x = 5;\n}";
    assert_eq!(find_matching_bracket(text, 10), Some(27)); // { -> }
    assert_eq!(find_matching_bracket(text, 27), Some(10)); // } -> {
}
