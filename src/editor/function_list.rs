/// Extract function/method/class names from source code.
/// Returns a list of (line_number, name, kind) tuples.
pub fn extract_symbols(text: &str, language: &str) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        match language.to_lowercase().as_str() {
            "rust" => {
                if let Some(name) = extract_rust_symbol(trimmed) {
                    symbols.push(Symbol { line: i, name, kind: name_to_kind(trimmed) });
                }
            }
            "python" => {
                if let Some(name) = extract_python_symbol(trimmed) {
                    symbols.push(Symbol { line: i, name, kind: name_to_kind(trimmed) });
                }
            }
            "javascript" | "typescript" => {
                if let Some(name) = extract_js_symbol(trimmed) {
                    symbols.push(Symbol { line: i, name, kind: name_to_kind(trimmed) });
                }
            }
            "c" | "c++" | "c#" | "java" | "go" | "kotlin" | "swift" => {
                if let Some(name) = extract_c_family_symbol(trimmed) {
                    symbols.push(Symbol { line: i, name, kind: name_to_kind(trimmed) });
                }
            }
            _ => {
                // Generic: look for common patterns
                if let Some(name) = extract_generic_symbol(trimmed) {
                    symbols.push(Symbol { line: i, name, kind: SymbolKind::Function });
                }
            }
        }
    }

    symbols
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub line: usize,
    pub name: String,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    Class,
    Struct,
    Enum,
    Trait,
    Interface,
    Method,
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolKind::Function => write!(f, "fn"),
            SymbolKind::Class => write!(f, "class"),
            SymbolKind::Struct => write!(f, "struct"),
            SymbolKind::Enum => write!(f, "enum"),
            SymbolKind::Trait => write!(f, "trait"),
            SymbolKind::Interface => write!(f, "iface"),
            SymbolKind::Method => write!(f, "method"),
        }
    }
}

fn name_to_kind(line: &str) -> SymbolKind {
    if line.contains("class ") { SymbolKind::Class }
    else if line.contains("struct ") { SymbolKind::Struct }
    else if line.contains("enum ") { SymbolKind::Enum }
    else if line.contains("trait ") { SymbolKind::Trait }
    else if line.contains("interface ") { SymbolKind::Interface }
    else { SymbolKind::Function }
}

fn extract_rust_symbol(line: &str) -> Option<String> {
    if line.starts_with("fn ") || line.starts_with("pub fn ") || line.starts_with("pub(crate) fn ") ||
       line.starts_with("async fn ") || line.starts_with("pub async fn ") {
        extract_name_after(line, "fn ")
    } else if line.starts_with("struct ") || line.starts_with("pub struct ") {
        extract_name_after(line, "struct ")
    } else if line.starts_with("enum ") || line.starts_with("pub enum ") {
        extract_name_after(line, "enum ")
    } else if line.starts_with("trait ") || line.starts_with("pub trait ") {
        extract_name_after(line, "trait ")
    } else if line.starts_with("impl ") {
        let rest = &line[5..];
        let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '<' || *c == '>').collect();
        if !name.is_empty() { Some(format!("impl {}", name)) } else { None }
    } else if line.contains(" fn ") && !line.starts_with("//") {
        extract_name_after(line, "fn ")
    } else {
        None
    }
}

fn extract_python_symbol(line: &str) -> Option<String> {
    if line.starts_with("def ") || line.starts_with("async def ") {
        extract_name_after(line, "def ")
    } else if line.starts_with("class ") {
        extract_name_after(line, "class ")
    } else {
        None
    }
}

fn extract_js_symbol(line: &str) -> Option<String> {
    if line.starts_with("function ") || line.starts_with("async function ") ||
       line.starts_with("export function ") || line.starts_with("export async function ") {
        extract_name_after(line, "function ")
    } else if line.starts_with("class ") || line.starts_with("export class ") {
        extract_name_after(line, "class ")
    } else if line.contains("const ") && line.contains(" = ") && (line.contains("=>") || line.contains("function")) {
        extract_name_after(line, "const ")
    } else {
        None
    }
}

fn extract_c_family_symbol(line: &str) -> Option<String> {
    if line.starts_with("class ") || line.starts_with("public class ") {
        extract_name_after(line, "class ")
    } else if line.starts_with("struct ") {
        extract_name_after(line, "struct ")
    } else if line.starts_with("enum ") {
        extract_name_after(line, "enum ")
    } else if line.starts_with("interface ") || line.starts_with("public interface ") {
        extract_name_after(line, "interface ")
    } else if line.contains('(') && !line.starts_with("//") && !line.starts_with('#')
        && !line.starts_with("if ") && !line.starts_with("for ") && !line.starts_with("while ")
        && !line.starts_with("switch ") && !line.starts_with("catch ")
        && !line.starts_with("return ") && !line.starts_with("throw ")
        && !line.starts_with(',') && !line.starts_with(':')
        && !line.starts_with('.')
        // Exclude statements (ending with ;) — these are calls/declarations, not definitions
        && !line.ends_with(';')
    {
        let before_paren = line.split('(').next()?;
        // Exclude method calls (. or -> before paren)
        if before_paren.contains('.') || before_paren.contains("->") {
            return None;
        }
        // Exclude assignments
        if before_paren.contains('=') {
            return None;
        }
        let words: Vec<&str> = before_paren.split_whitespace().collect();
        if words.len() >= 2 {
            let name = words.last()?;
            if name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == ':' || c == '~') {
                let clean_name = name.trim_start_matches('~');
                let clean_name = if let Some(pos) = clean_name.rfind("::") {
                    &clean_name[pos + 2..]
                } else {
                    clean_name
                };
                if clean_name.is_empty() {
                    return None;
                }
                let first = words[0];
                if first == "else" || first == "do" || first == "case"
                    || first == "delete" || first == "new" || first == "sizeof"
                    || first == "typeof" || first == "alignof"
                {
                    return None;
                }
                return Some(name.to_string());
            }
        }
        None
    } else {
        None
    }
}

fn extract_generic_symbol(line: &str) -> Option<String> {
    if line.starts_with("function ") || line.starts_with("func ") ||
       line.starts_with("def ") || line.starts_with("fn ") ||
       line.starts_with("sub ") {
        let keyword = line.split_whitespace().next()?;
        extract_name_after(line, &format!("{} ", keyword))
    } else {
        None
    }
}

fn extract_name_after(line: &str, keyword: &str) -> Option<String> {
    let idx = line.find(keyword)?;
    let rest = &line[idx + keyword.len()..];
    let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
    if name.is_empty() { None } else { Some(name) }
}
