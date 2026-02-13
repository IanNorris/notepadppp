use notepadppp::editor::function_list::{extract_symbols, SymbolKind};

#[test]
fn test_rust_functions() {
    let code = "fn main() {\n}\npub fn helper() {\n}\n";
    let symbols = extract_symbols(code, "Rust");
    assert_eq!(symbols.len(), 2);
    assert_eq!(symbols[0].name, "main");
    assert_eq!(symbols[1].name, "helper");
}

#[test]
fn test_rust_structs_enums() {
    let code = "pub struct Foo {\n}\nenum Bar {\n}\n";
    let symbols = extract_symbols(code, "Rust");
    assert_eq!(symbols.len(), 2);
    assert_eq!(symbols[0].kind, SymbolKind::Struct);
    assert_eq!(symbols[1].kind, SymbolKind::Enum);
}

#[test]
fn test_python_functions() {
    let code = "def hello():\n    pass\nclass MyClass:\n    pass\n";
    let symbols = extract_symbols(code, "Python");
    assert_eq!(symbols.len(), 2);
    assert_eq!(symbols[0].name, "hello");
    assert_eq!(symbols[1].name, "MyClass");
}

#[test]
fn test_javascript_functions() {
    let code = "function greet() {}\nclass App {}\nconst handler = () => {}\n";
    let symbols = extract_symbols(code, "JavaScript");
    assert!(symbols.len() >= 2);
    assert_eq!(symbols[0].name, "greet");
}

#[test]
fn test_empty_text() {
    let symbols = extract_symbols("", "Rust");
    assert!(symbols.is_empty());
}

#[test]
fn test_no_functions() {
    let code = "let x = 5;\nlet y = 10;\n";
    let symbols = extract_symbols(code, "Rust");
    assert!(symbols.is_empty());
}

#[test]
fn test_impl_blocks() {
    let code = "impl Foo {\n    fn bar() {}\n}\n";
    let symbols = extract_symbols(code, "Rust");
    assert!(symbols.len() >= 1);
}
