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

// =====================================================
// Additional coverage tests
// =====================================================

#[test]
fn test_c_extraction_main() {
    let code = "int main() {\n}\n";
    let symbols = extract_symbols(code, "C");
    assert!(symbols.iter().any(|s| s.name == "main"));
}

#[test]
fn test_c_extraction_void_foo() {
    let code = "void foo() {\n}\n";
    let symbols = extract_symbols(code, "C");
    assert!(symbols.iter().any(|s| s.name == "foo"));
}

#[test]
fn test_cpp_extraction_class() {
    let code = "class MyClass {\n};\n";
    let symbols = extract_symbols(code, "C++");
    assert!(symbols.iter().any(|s| s.name == "MyClass" && s.kind == SymbolKind::Class));
}

#[test]
fn test_java_public_void_method() {
    let code = "public void method() {\n}\n";
    let symbols = extract_symbols(code, "Java");
    assert!(symbols.iter().any(|s| s.name == "method"));
}

#[test]
fn test_java_class() {
    let code = "public class Foo {\n}\n";
    let symbols = extract_symbols(code, "Java");
    assert!(symbols.iter().any(|s| s.name == "Foo" && s.kind == SymbolKind::Class));
}

#[test]
fn test_go_func_main() {
    let code = "func main() {\n}\n";
    let symbols = extract_symbols(code, "Go");
    assert!(symbols.iter().any(|s| s.name == "main"));
}

#[test]
fn test_go_struct() {
    let code = "struct MyStruct {\n}\n";
    let symbols = extract_symbols(code, "Go");
    assert!(symbols.iter().any(|s| s.name == "MyStruct" && s.kind == SymbolKind::Struct));
}

#[test]
fn test_typescript_extraction() {
    let code = "function greet() {}\nclass App {}\nexport function helper() {}\n";
    let symbols = extract_symbols(code, "TypeScript");
    assert!(symbols.iter().any(|s| s.name == "greet"));
    assert!(symbols.iter().any(|s| s.name == "App"));
    assert!(symbols.iter().any(|s| s.name == "helper"));
}

#[test]
fn test_unknown_language_generic_extraction() {
    let code = "function hello() {}\ndef world():\n    pass\nfn rust_fn() {}\n";
    let symbols = extract_symbols(code, "UnknownLang");
    assert!(symbols.iter().any(|s| s.name == "hello"));
    assert!(symbols.iter().any(|s| s.name == "world"));
    assert!(symbols.iter().any(|s| s.name == "rust_fn"));
}

#[test]
fn test_generic_sub_keyword() {
    let code = "sub my_sub() {\n}\n";
    let symbols = extract_symbols(code, "Perl6Whatever");
    assert!(symbols.iter().any(|s| s.name == "my_sub"));
}

#[test]
fn test_symbol_kind_display() {
    assert_eq!(format!("{}", SymbolKind::Function), "fn");
    assert_eq!(format!("{}", SymbolKind::Class), "class");
    assert_eq!(format!("{}", SymbolKind::Struct), "struct");
    assert_eq!(format!("{}", SymbolKind::Enum), "enum");
    assert_eq!(format!("{}", SymbolKind::Trait), "trait");
    assert_eq!(format!("{}", SymbolKind::Interface), "iface");
    assert_eq!(format!("{}", SymbolKind::Method), "method");
}

#[test]
fn test_line_number_verification() {
    let code = "// comment\nfn first() {}\n// another\nfn second() {}\n";
    let symbols = extract_symbols(code, "Rust");
    assert!(symbols.iter().any(|s| s.name == "first" && s.line == 1));
    assert!(symbols.iter().any(|s| s.name == "second" && s.line == 3));
}

#[test]
fn test_name_to_kind_class() {
    let code = "class Foo {\n}\n";
    let symbols = extract_symbols(code, "C++");
    assert!(symbols.iter().any(|s| s.kind == SymbolKind::Class));
}

#[test]
fn test_name_to_kind_struct() {
    let code = "struct Bar {\n}\n";
    let symbols = extract_symbols(code, "C");
    assert!(symbols.iter().any(|s| s.kind == SymbolKind::Struct));
}

#[test]
fn test_name_to_kind_enum() {
    let code = "enum Color {\n}\n";
    let symbols = extract_symbols(code, "Rust");
    assert!(symbols.iter().any(|s| s.kind == SymbolKind::Enum));
}

#[test]
fn test_name_to_kind_trait() {
    let code = "trait Drawable {\n}\n";
    let symbols = extract_symbols(code, "Rust");
    assert!(symbols.iter().any(|s| s.kind == SymbolKind::Trait));
}

#[test]
fn test_name_to_kind_interface() {
    let code = "interface Runnable {\n}\n";
    let symbols = extract_symbols(code, "Java");
    assert!(symbols.iter().any(|s| s.kind == SymbolKind::Interface));
}

#[test]
fn test_c_sharp_extraction() {
    let code = "public void Run() {\n}\n";
    let symbols = extract_symbols(code, "C#");
    assert!(symbols.iter().any(|s| s.name == "Run"));
}

#[test]
fn test_kotlin_extraction() {
    let code = "fun doSomething() {\n}\n";
    let symbols = extract_symbols(code, "Kotlin");
    assert!(symbols.iter().any(|s| s.name == "doSomething"));
}

#[test]
fn test_swift_extraction() {
    let code = "func swiftFunc() {\n}\n";
    let symbols = extract_symbols(code, "Swift");
    assert!(symbols.iter().any(|s| s.name == "swiftFunc"));
}
