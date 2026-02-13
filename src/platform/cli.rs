use std::path::PathBuf;

/// Parsed command-line arguments for Notepad+++
#[derive(Debug, Clone, Default)]
pub struct CliArgs {
    /// Files to open
    pub files: Vec<PathBuf>,
    /// Go to specific line number after opening (1-based)
    pub goto_line: Option<usize>,
    /// Go to specific column after opening (1-based)
    pub goto_column: Option<usize>,
    /// Override encoding for opened files
    pub encoding: Option<String>,
    /// Override language/syntax for opened files
    pub language: Option<String>,
    /// Open in read-only mode
    pub read_only: bool,
    /// Start new instance instead of reusing existing
    pub new_instance: bool,
    /// Register shell integration and exit
    pub register_shell: bool,
    /// Unregister shell integration and exit
    pub unregister_shell: bool,
    /// Open files in a new tab group (split view)
    pub new_tab_group: bool,
    /// Print version and exit
    pub version: bool,
    /// Print help and exit
    pub help: bool,
}

impl CliArgs {
    /// Parse command-line arguments from an iterator of strings.
    pub fn parse<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut result = CliArgs::default();
        let mut args_iter = args.into_iter().peekable();

        // Skip the program name (first arg)
        args_iter.next();

        while let Some(arg) = args_iter.next() {
            let arg = arg.as_ref();
            match arg {
                "-n" | "--line" | "-goto" => {
                    let val = args_iter
                        .next()
                        .ok_or_else(|| format!("Missing value for {arg}"))?;
                    let line: usize = val
                        .as_ref()
                        .parse()
                        .map_err(|_| format!("Invalid line number: {}", val.as_ref()))?;
                    if line == 0 {
                        return Err("Line number must be >= 1".to_string());
                    }
                    result.goto_line = Some(line);
                }
                "-c" | "--column" => {
                    let val = args_iter
                        .next()
                        .ok_or_else(|| format!("Missing value for {arg}"))?;
                    let col: usize = val
                        .as_ref()
                        .parse()
                        .map_err(|_| format!("Invalid column number: {}", val.as_ref()))?;
                    if col == 0 {
                        return Err("Column number must be >= 1".to_string());
                    }
                    result.goto_column = Some(col);
                }
                "-e" | "--encoding" => {
                    let val = args_iter
                        .next()
                        .ok_or_else(|| format!("Missing value for {arg}"))?;
                    result.encoding = Some(val.as_ref().to_string());
                }
                "-l" | "--language" => {
                    let val = args_iter
                        .next()
                        .ok_or_else(|| format!("Missing value for {arg}"))?;
                    result.language = Some(val.as_ref().to_string());
                }
                "-r" | "--read-only" => {
                    result.read_only = true;
                }
                "--new-instance" => {
                    result.new_instance = true;
                }
                "--register-shell" => {
                    result.register_shell = true;
                }
                "--unregister-shell" => {
                    result.unregister_shell = true;
                }
                "--new-tab-group" => {
                    result.new_tab_group = true;
                }
                "-v" | "--version" => {
                    result.version = true;
                }
                "-h" | "--help" => {
                    result.help = true;
                }
                _ => {
                    // Handle -nNUM (e.g., -n42) combined form
                    if arg.starts_with("-n") && arg.len() > 2 && arg[2..].chars().all(|c| c.is_ascii_digit()) {
                        let line: usize = arg[2..]
                            .parse()
                            .map_err(|_| format!("Invalid line number in: {arg}"))?;
                        if line == 0 {
                            return Err("Line number must be >= 1".to_string());
                        }
                        result.goto_line = Some(line);
                    } else if arg.starts_with('-') {
                        return Err(format!("Unknown option: {arg}"));
                    } else {
                        // File path — handle file:line:col syntax
                        if let Some((path, line, col)) = parse_file_location(arg) {
                            result.files.push(PathBuf::from(path));
                            if result.goto_line.is_none() {
                                result.goto_line = Some(line);
                            }
                            if let Some(c) = col {
                                if result.goto_column.is_none() {
                                    result.goto_column = Some(c);
                                }
                            }
                        } else {
                            result.files.push(PathBuf::from(arg));
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// Generate help text
    pub fn help_text() -> String {
        format!(
            "Notepad+++ v{version} — A fast, native text editor for programmers

USAGE:
    notepadppp [OPTIONS] [FILES...]

FILES:
    file.txt            Open file
    file.txt:42         Open file at line 42
    file.txt:42:10      Open file at line 42, column 10

OPTIONS:
    -n, --line NUM      Go to line number after opening
    -c, --column NUM    Go to column number after opening
    -e, --encoding ENC  Set file encoding (utf-8, utf-16le, utf-16be, ascii, latin1)
    -l, --language LANG Override syntax language (rust, python, javascript, etc.)
    -r, --read-only     Open files in read-only mode
    --new-instance      Force a new editor window
    --new-tab-group     Open files in a new split panel
    --register-shell    Register shell integration (context menu, file associations)
    --unregister-shell  Remove shell integration
    -v, --version       Print version and exit
    -h, --help          Print this help and exit

EXAMPLES:
    notepadppp file.rs
    notepadppp -n 42 main.rs
    notepadppp main.rs:100:5
    notepadppp --language python script.txt
    notepadppp --register-shell",
            version = env!("CARGO_PKG_VERSION"),
        )
    }
}

/// Parse `file:line` or `file:line:col` syntax.
/// Returns `(file_path, line, optional_col)` if the pattern matches.
fn parse_file_location(arg: &str) -> Option<(&str, usize, Option<usize>)> {
    // Don't match Windows drive letters like C:\path
    if arg.len() >= 2 && arg.as_bytes()[1] == b':' && arg.as_bytes()[0].is_ascii_alphabetic() {
        // Check for C:\path:line pattern
        let rest = &arg[2..];
        if let Some(colon_pos) = rest.rfind(':') {
            let after_colon = &rest[colon_pos + 1..];
            if let Ok(num) = after_colon.parse::<usize>() {
                if num > 0 {
                    let before = &arg[..2 + colon_pos];
                    // Check for file:line:col
                    if let Some(second_colon) = before[2..].rfind(':') {
                        let mid = &before[2 + second_colon + 1..];
                        if let Ok(line) = mid.parse::<usize>() {
                            if line > 0 {
                                let file = &arg[..2 + second_colon];
                                return Some((file, line, Some(num)));
                            }
                        }
                    }
                    return Some((before, num, None));
                }
            }
        }
        return None;
    }

    // Unix-style: find last colon
    let colon_pos = arg.rfind(':')?;
    if colon_pos == 0 {
        return None;
    }

    let after = &arg[colon_pos + 1..];
    if after.is_empty() {
        return None;
    }

    if let Ok(num) = after.parse::<usize>() {
        if num == 0 {
            return None;
        }
        let before = &arg[..colon_pos];
        // Check for file:line:col
        if let Some(second_colon) = before.rfind(':') {
            if second_colon > 0 {
                let mid = &before[second_colon + 1..];
                if let Ok(line) = mid.parse::<usize>() {
                    if line > 0 {
                        let file = &before[..second_colon];
                        return Some((file, line, Some(num)));
                    }
                }
            }
        }
        Some((before, num, None))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_no_args() {
        let args = vec!["notepadppp"];
        let cli = CliArgs::parse(args).unwrap();
        assert!(cli.files.is_empty());
        assert!(cli.goto_line.is_none());
    }

    #[test]
    fn test_parse_single_file() {
        let args = vec!["notepadppp", "file.txt"];
        let cli = CliArgs::parse(args).unwrap();
        assert_eq!(cli.files, vec![PathBuf::from("file.txt")]);
    }

    #[test]
    fn test_parse_multiple_files() {
        let args = vec!["notepadppp", "a.rs", "b.py", "c.js"];
        let cli = CliArgs::parse(args).unwrap();
        assert_eq!(cli.files.len(), 3);
    }

    #[test]
    fn test_parse_goto_line() {
        let args = vec!["notepadppp", "-n", "42", "file.rs"];
        let cli = CliArgs::parse(args).unwrap();
        assert_eq!(cli.goto_line, Some(42));
        assert_eq!(cli.files, vec![PathBuf::from("file.rs")]);
    }

    #[test]
    fn test_parse_goto_line_long() {
        let args = vec!["notepadppp", "--line", "100", "file.rs"];
        let cli = CliArgs::parse(args).unwrap();
        assert_eq!(cli.goto_line, Some(100));
    }

    #[test]
    fn test_parse_goto_column() {
        let args = vec!["notepadppp", "-c", "10", "file.rs"];
        let cli = CliArgs::parse(args).unwrap();
        assert_eq!(cli.goto_column, Some(10));
    }

    #[test]
    fn test_parse_encoding() {
        let args = vec!["notepadppp", "-e", "utf-16le", "file.txt"];
        let cli = CliArgs::parse(args).unwrap();
        assert_eq!(cli.encoding, Some("utf-16le".to_string()));
    }

    #[test]
    fn test_parse_language() {
        let args = vec!["notepadppp", "--language", "rust", "script.txt"];
        let cli = CliArgs::parse(args).unwrap();
        assert_eq!(cli.language, Some("rust".to_string()));
    }

    #[test]
    fn test_parse_read_only() {
        let args = vec!["notepadppp", "-r", "file.txt"];
        let cli = CliArgs::parse(args).unwrap();
        assert!(cli.read_only);
    }

    #[test]
    fn test_parse_new_instance() {
        let args = vec!["notepadppp", "--new-instance", "file.txt"];
        let cli = CliArgs::parse(args).unwrap();
        assert!(cli.new_instance);
    }

    #[test]
    fn test_parse_register_shell() {
        let args = vec!["notepadppp", "--register-shell"];
        let cli = CliArgs::parse(args).unwrap();
        assert!(cli.register_shell);
    }

    #[test]
    fn test_parse_unregister_shell() {
        let args = vec!["notepadppp", "--unregister-shell"];
        let cli = CliArgs::parse(args).unwrap();
        assert!(cli.unregister_shell);
    }

    #[test]
    fn test_parse_version() {
        let args = vec!["notepadppp", "--version"];
        let cli = CliArgs::parse(args).unwrap();
        assert!(cli.version);
    }

    #[test]
    fn test_parse_help() {
        let args = vec!["notepadppp", "-h"];
        let cli = CliArgs::parse(args).unwrap();
        assert!(cli.help);
    }

    #[test]
    fn test_parse_combined_short_line() {
        let args = vec!["notepadppp", "-n42", "file.rs"];
        let cli = CliArgs::parse(args).unwrap();
        assert_eq!(cli.goto_line, Some(42));
    }

    #[test]
    fn test_parse_file_colon_line() {
        let args = vec!["notepadppp", "main.rs:42"];
        let cli = CliArgs::parse(args).unwrap();
        assert_eq!(cli.files, vec![PathBuf::from("main.rs")]);
        assert_eq!(cli.goto_line, Some(42));
    }

    #[test]
    fn test_parse_file_colon_line_colon_col() {
        let args = vec!["notepadppp", "main.rs:42:10"];
        let cli = CliArgs::parse(args).unwrap();
        assert_eq!(cli.files, vec![PathBuf::from("main.rs")]);
        assert_eq!(cli.goto_line, Some(42));
        assert_eq!(cli.goto_column, Some(10));
    }

    #[test]
    fn test_parse_all_options() {
        let args = vec![
            "notepadppp",
            "-n", "10",
            "-c", "5",
            "-e", "utf-8",
            "-l", "python",
            "-r",
            "--new-instance",
            "--new-tab-group",
            "file.py",
        ];
        let cli = CliArgs::parse(args).unwrap();
        assert_eq!(cli.goto_line, Some(10));
        assert_eq!(cli.goto_column, Some(5));
        assert_eq!(cli.encoding, Some("utf-8".to_string()));
        assert_eq!(cli.language, Some("python".to_string()));
        assert!(cli.read_only);
        assert!(cli.new_instance);
        assert!(cli.new_tab_group);
        assert_eq!(cli.files, vec![PathBuf::from("file.py")]);
    }

    #[test]
    fn test_parse_unknown_option_error() {
        let args = vec!["notepadppp", "--foobar"];
        assert!(CliArgs::parse(args).is_err());
    }

    #[test]
    fn test_parse_missing_value_error() {
        let args = vec!["notepadppp", "-n"];
        assert!(CliArgs::parse(args).is_err());
    }

    #[test]
    fn test_parse_invalid_line_error() {
        let args = vec!["notepadppp", "-n", "abc"];
        assert!(CliArgs::parse(args).is_err());
    }

    #[test]
    fn test_parse_zero_line_error() {
        let args = vec!["notepadppp", "-n", "0"];
        assert!(CliArgs::parse(args).is_err());
    }

    #[test]
    fn test_help_text_contains_usage() {
        let text = CliArgs::help_text();
        assert!(text.contains("USAGE:"));
        assert!(text.contains("--line"));
        assert!(text.contains("--encoding"));
        assert!(text.contains("--register-shell"));
    }

    #[test]
    fn test_parse_file_location_unix() {
        let result = parse_file_location("main.rs:42");
        assert_eq!(result, Some(("main.rs", 42, None)));
    }

    #[test]
    fn test_parse_file_location_with_col() {
        let result = parse_file_location("main.rs:42:10");
        assert_eq!(result, Some(("main.rs", 42, Some(10))));
    }

    #[test]
    fn test_parse_file_location_no_colon() {
        let result = parse_file_location("main.rs");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_file_location_not_number() {
        let result = parse_file_location("main.rs:abc");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_goto_alias() {
        let args = vec!["notepadppp", "-goto", "50", "file.rs"];
        let cli = CliArgs::parse(args).unwrap();
        assert_eq!(cli.goto_line, Some(50));
    }
}
