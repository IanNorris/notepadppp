/// Get the line comment prefix for a given language
pub fn line_comment_prefix(language: &str) -> Option<&'static str> {
    match language {
        "Rust" | "Go" | "C" | "C++" | "Java" | "JavaScript" | "TypeScript" | "Kotlin" |
        "Scala" | "Swift" | "Zig" | "Objective-C" | "C#" | "Dart" => Some("//"),
        "Python" | "Ruby" | "Perl" | "R" | "Shell" | "Bash" | "PowerShell" | "YAML" |
        "TOML" | "Makefile" | "Dockerfile" | "CoffeeScript" => Some("#"),
        "Lua" | "SQL" | "Haskell" => Some("--"),
        "Lisp" | "Clojure" | "Scheme" => Some(";"),
        "HTML" | "XML" => None,  // Block comments only
        "CSS" | "SCSS" | "LESS" => None, // Block comments only
        _ => None,
    }
}

/// Get the block comment delimiters for a given language
pub fn block_comment_delimiters(language: &str) -> Option<(&'static str, &'static str)> {
    match language {
        "C" | "C++" | "Java" | "JavaScript" | "TypeScript" | "Rust" | "Go" | "Kotlin" |
        "Scala" | "Swift" | "C#" | "Dart" | "CSS" | "SCSS" | "LESS" | "Objective-C" => Some(("/*", "*/")),
        "HTML" | "XML" => Some(("<!--", "-->")),
        "Python" => Some(("\"\"\"", "\"\"\"")),
        "Lua" => Some(("--[[", "]]")),
        _ => None,
    }
}
