use notepadppp::platform::{
    default_associations, default_context_menu_entries, ContextMenuScope, FileAssociation,
};
use notepadppp::platform::cli::CliArgs;

// ===== CLI Argument Parsing Tests =====

#[test]
fn test_cli_no_args() {
    let args = vec!["notepadppp"];
    let cli = CliArgs::parse(args).unwrap();
    assert!(cli.files.is_empty());
    assert!(!cli.register_shell);
    assert!(!cli.unregister_shell);
}

#[test]
fn test_cli_single_file() {
    let args = vec!["notepadppp", "test.rs"];
    let cli = CliArgs::parse(args).unwrap();
    assert_eq!(cli.files.len(), 1);
    assert_eq!(cli.files[0].to_str().unwrap(), "test.rs");
}

#[test]
fn test_cli_goto_line() {
    let args = vec!["notepadppp", "-n", "42", "file.rs"];
    let cli = CliArgs::parse(args).unwrap();
    assert_eq!(cli.goto_line, Some(42));
}

#[test]
fn test_cli_goto_line_and_column() {
    let args = vec!["notepadppp", "-n", "42", "-c", "10", "file.rs"];
    let cli = CliArgs::parse(args).unwrap();
    assert_eq!(cli.goto_line, Some(42));
    assert_eq!(cli.goto_column, Some(10));
}

#[test]
fn test_cli_encoding_override() {
    let args = vec!["notepadppp", "-e", "utf-16le", "file.txt"];
    let cli = CliArgs::parse(args).unwrap();
    assert_eq!(cli.encoding, Some("utf-16le".to_string()));
}

#[test]
fn test_cli_language_override() {
    let args = vec!["notepadppp", "-l", "rust", "script.txt"];
    let cli = CliArgs::parse(args).unwrap();
    assert_eq!(cli.language, Some("rust".to_string()));
}

#[test]
fn test_cli_read_only() {
    let args = vec!["notepadppp", "-r", "file.txt"];
    let cli = CliArgs::parse(args).unwrap();
    assert!(cli.read_only);
}

#[test]
fn test_cli_register_shell() {
    let args = vec!["notepadppp", "--register-shell"];
    let cli = CliArgs::parse(args).unwrap();
    assert!(cli.register_shell);
}

#[test]
fn test_cli_unregister_shell() {
    let args = vec!["notepadppp", "--unregister-shell"];
    let cli = CliArgs::parse(args).unwrap();
    assert!(cli.unregister_shell);
}

#[test]
fn test_cli_new_instance() {
    let args = vec!["notepadppp", "--new-instance", "file.txt"];
    let cli = CliArgs::parse(args).unwrap();
    assert!(cli.new_instance);
}

#[test]
fn test_cli_file_colon_line_syntax() {
    let args = vec!["notepadppp", "main.rs:42"];
    let cli = CliArgs::parse(args).unwrap();
    assert_eq!(cli.files[0].to_str().unwrap(), "main.rs");
    assert_eq!(cli.goto_line, Some(42));
}

#[test]
fn test_cli_file_colon_line_colon_col_syntax() {
    let args = vec!["notepadppp", "main.rs:42:10"];
    let cli = CliArgs::parse(args).unwrap();
    assert_eq!(cli.files[0].to_str().unwrap(), "main.rs");
    assert_eq!(cli.goto_line, Some(42));
    assert_eq!(cli.goto_column, Some(10));
}

#[test]
fn test_cli_version_flag() {
    let args = vec!["notepadppp", "--version"];
    let cli = CliArgs::parse(args).unwrap();
    assert!(cli.version);
}

#[test]
fn test_cli_help_flag() {
    let args = vec!["notepadppp", "-h"];
    let cli = CliArgs::parse(args).unwrap();
    assert!(cli.help);
}

#[test]
fn test_cli_unknown_option_error() {
    let args = vec!["notepadppp", "--unknown"];
    assert!(CliArgs::parse(args).is_err());
}

#[test]
fn test_cli_missing_line_value() {
    let args = vec!["notepadppp", "-n"];
    assert!(CliArgs::parse(args).is_err());
}

#[test]
fn test_cli_combined_options() {
    let args = vec![
        "notepadppp",
        "-n", "100",
        "-c", "5",
        "-e", "utf-8",
        "-l", "python",
        "-r",
        "--new-instance",
        "--new-tab-group",
        "file.py",
    ];
    let cli = CliArgs::parse(args).unwrap();
    assert_eq!(cli.goto_line, Some(100));
    assert_eq!(cli.goto_column, Some(5));
    assert_eq!(cli.encoding, Some("utf-8".to_string()));
    assert_eq!(cli.language, Some("python".to_string()));
    assert!(cli.read_only);
    assert!(cli.new_instance);
    assert!(cli.new_tab_group);
}

#[test]
fn test_cli_help_text_content() {
    let help = CliArgs::help_text();
    assert!(help.contains("USAGE:"));
    assert!(help.contains("--register-shell"));
    assert!(help.contains("--unregister-shell"));
    assert!(help.contains("--line"));
    assert!(help.contains("--encoding"));
    assert!(help.contains("--language"));
    assert!(help.contains("--read-only"));
    assert!(help.contains("EXAMPLES:"));
}

// ===== File Association Tests =====

#[test]
fn test_default_associations_comprehensive() {
    let assocs = default_associations();
    // Should have 50+ extensions
    assert!(assocs.len() >= 50, "Expected 50+ associations, got {}", assocs.len());

    // Verify key extensions are present
    let exts: Vec<&str> = assocs.iter().map(|a| a.extension.as_str()).collect();
    for required in &["txt", "rs", "py", "js", "json", "html", "css", "go", "java", "md", "csv", "sh", "bat", "ps1"] {
        assert!(exts.contains(required), "Missing extension: {required}");
    }
}

#[test]
fn test_association_descriptions_not_empty() {
    let assocs = default_associations();
    for assoc in &assocs {
        assert!(!assoc.description.is_empty(), "Empty description for .{}", assoc.extension);
    }
}

#[test]
fn test_association_extensions_lowercase() {
    let assocs = default_associations();
    for assoc in &assocs {
        assert_eq!(
            assoc.extension,
            assoc.extension.to_lowercase(),
            "Extension should be lowercase: {}",
            assoc.extension
        );
    }
}

#[test]
fn test_association_no_dots() {
    let assocs = default_associations();
    for assoc in &assocs {
        assert!(
            !assoc.extension.starts_with('.'),
            "Extension should not start with dot: {}",
            assoc.extension
        );
    }
}

// ===== Context Menu Tests =====

#[test]
fn test_default_context_menu_entries() {
    let entries = default_context_menu_entries(r"C:\notepadppp.exe");
    assert_eq!(entries.len(), 2);

    let all_files = entries.iter().find(|e| e.scope == ContextMenuScope::AllFiles);
    assert!(all_files.is_some());
    assert!(all_files.unwrap().label.contains("Notepad+++"));

    let dirs = entries.iter().find(|e| e.scope == ContextMenuScope::Directories);
    assert!(dirs.is_some());
    assert!(dirs.unwrap().label.contains("folder"));
}

// ===== Windows Registry Generation Tests =====

#[cfg(any())] // Only compile on Windows
mod windows_tests {
    // These would be active on Windows builds
}

// Always-active tests for the cross-platform registry file generation
mod reg_gen {
    use notepadppp::platform::windows::WindowsIntegration;
    use notepadppp::platform::{
        default_associations, default_context_menu_entries, ContextMenuEntry, ContextMenuScope,
        FileAssociation,
    };

    #[test]
    fn test_reg_file_header() {
        let reg = WindowsIntegration::generate_reg_file(r"C:\notepadppp.exe", &[], &[]);
        assert!(reg.starts_with("Windows Registry Editor Version 5.00"));
    }

    #[test]
    fn test_reg_file_progid() {
        let reg = WindowsIntegration::generate_reg_file(r"C:\notepadppp.exe", &[], &[]);
        assert!(reg.contains("[HKEY_CURRENT_USER\\Software\\Classes\\NotepadPPP.File]"));
        assert!(reg.contains("shell\\open\\command"));
    }

    #[test]
    fn test_reg_file_extensions() {
        let assocs = vec![
            FileAssociation {
                extension: "rs".to_string(),
                description: "Rust".to_string(),
                icon_index: 3,
            },
            FileAssociation {
                extension: "py".to_string(),
                description: "Python".to_string(),
                icon_index: 3,
            },
        ];
        let reg = WindowsIntegration::generate_reg_file(r"C:\notepadppp.exe", &assocs, &[]);
        assert!(reg.contains(".rs"));
        assert!(reg.contains(".py"));
        assert!(reg.contains("OpenWithProgids"));
    }

    #[test]
    fn test_reg_file_context_menu_all() {
        let entries = vec![ContextMenuEntry {
            id: "NotepadPPP.Open".to_string(),
            label: "Open with Notepad+++".to_string(),
            scope: ContextMenuScope::AllFiles,
        }];
        let reg = WindowsIntegration::generate_reg_file(r"C:\notepadppp.exe", &[], &entries);
        assert!(reg.contains("*\\shell\\NotepadPPP.Open"));
    }

    #[test]
    fn test_reg_file_context_menu_dirs() {
        let entries = vec![ContextMenuEntry {
            id: "NotepadPPP.Folder".to_string(),
            label: "Open folder".to_string(),
            scope: ContextMenuScope::Directories,
        }];
        let reg = WindowsIntegration::generate_reg_file(r"C:\notepadppp.exe", &[], &entries);
        assert!(reg.contains("Directory\\Background\\shell\\NotepadPPP.Folder"));
        assert!(reg.contains("Directory\\shell\\NotepadPPP.Folder"));
    }

    #[test]
    fn test_uninstall_reg_removes_keys() {
        let assocs = default_associations();
        let entries = default_context_menu_entries("");
        let reg = WindowsIntegration::generate_uninstall_reg(&assocs, &entries);
        assert!(reg.contains("[-HKEY_CURRENT_USER\\Software\\Classes\\NotepadPPP.File]"));
        assert!(reg.contains("[-HKEY_CURRENT_USER\\Software\\Classes\\.txt]"));
    }

    #[test]
    fn test_powershell_register_script() {
        let assocs = vec![FileAssociation {
            extension: "rs".to_string(),
            description: "Rust".to_string(),
            icon_index: 3,
        }];
        let entries = default_context_menu_entries(r"C:\notepadppp.exe");
        let ps =
            WindowsIntegration::generate_powershell_register(r"C:\notepadppp.exe", &assocs, &entries);
        assert!(ps.contains("New-Item"));
        assert!(ps.contains("SHChangeNotify"));
        assert!(ps.contains("NotepadPPP.File"));
    }

    #[test]
    fn test_powershell_unregister_script() {
        let assocs = default_associations();
        let entries = default_context_menu_entries("");
        let ps = WindowsIntegration::generate_powershell_unregister(&assocs, &entries);
        assert!(ps.contains("Remove-Item"));
        assert!(ps.contains("Remove-ItemProperty"));
    }

    #[test]
    fn test_full_reg_file_all_extensions() {
        let assocs = default_associations();
        let entries = default_context_menu_entries(r"C:\notepadppp.exe");
        let reg = WindowsIntegration::generate_reg_file(r"C:\notepadppp.exe", &assocs, &entries);
        for a in &assocs {
            assert!(reg.contains(&format!(".{}", a.extension)));
        }
    }
}

// ===== Linux Integration Tests =====

mod linux_tests {
    use notepadppp::platform::linux::LinuxIntegration;
    use notepadppp::platform::{default_associations, FileAssociation};

    #[test]
    fn test_desktop_file_format() {
        let assocs = vec![FileAssociation {
            extension: "rs".to_string(),
            description: "Rust".to_string(),
            icon_index: 3,
        }];
        let desktop = LinuxIntegration::generate_desktop_file("/usr/bin/notepadppp", &assocs);
        assert!(desktop.contains("[Desktop Entry]"));
        assert!(desktop.contains("Type=Application"));
        assert!(desktop.contains("Name=Notepad+++"));
        assert!(desktop.contains("Exec=/usr/bin/notepadppp %F"));
        assert!(desktop.contains("Terminal=false"));
        assert!(desktop.contains("Categories=TextEditor"));
    }

    #[test]
    fn test_desktop_file_mime_types() {
        let assocs = default_associations();
        let desktop = LinuxIntegration::generate_desktop_file("/usr/bin/notepadppp", &assocs);
        assert!(desktop.contains("MimeType="));
        assert!(desktop.contains("text/plain"));
        assert!(desktop.contains("text/x-rust"));
        assert!(desktop.contains("application/json"));
    }

    #[test]
    fn test_mime_registration_script() {
        let assocs = default_associations();
        let script = LinuxIntegration::generate_mime_registration_script(&assocs);
        assert!(script.starts_with("#!/bin/bash"));
        assert!(script.contains("xdg-desktop-menu install"));
        assert!(script.contains("xdg-mime default"));
    }
}

// ===== macOS Integration Tests =====

mod macos_tests {
    use notepadppp::platform::macos::MacOSIntegration;
    use notepadppp::platform::FileAssociation;

    #[test]
    fn test_plist_format() {
        let assocs = vec![FileAssociation {
            extension: "rs".to_string(),
            description: "Rust Source File".to_string(),
            icon_index: 3,
        }];
        let plist = MacOSIntegration::generate_plist_document_types(&assocs);
        assert!(plist.contains("<?xml version=\"1.0\""));
        assert!(plist.contains("CFBundleDocumentTypes"));
        assert!(plist.contains("<string>rs</string>"));
        assert!(plist.contains("Rust Source File"));
    }

    #[test]
    fn test_plist_multiple_types() {
        let assocs = vec![
            FileAssociation {
                extension: "rs".to_string(),
                description: "Rust".to_string(),
                icon_index: 3,
            },
            FileAssociation {
                extension: "py".to_string(),
                description: "Python".to_string(),
                icon_index: 3,
            },
        ];
        let plist = MacOSIntegration::generate_plist_document_types(&assocs);
        assert!(plist.contains("<string>rs</string>"));
        assert!(plist.contains("<string>py</string>"));
    }
}

// ===== Single Instance Tests =====

mod single_instance_tests {
    use notepadppp::platform::single_instance::InstanceMessage;
    use std::path::PathBuf;

    #[test]
    fn test_instance_message_roundtrip() {
        let msg = InstanceMessage {
            files: vec![PathBuf::from("test.rs")],
            goto_line: Some(42),
            goto_column: Some(10),
            encoding: Some("utf-8".to_string()),
            language: Some("rust".to_string()),
            read_only: true,
            new_tab_group: false,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: InstanceMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.files, vec![PathBuf::from("test.rs")]);
        assert_eq!(decoded.goto_line, Some(42));
        assert_eq!(decoded.goto_column, Some(10));
        assert!(decoded.read_only);
        assert!(!decoded.new_tab_group);
    }

    #[test]
    fn test_instance_message_empty() {
        let msg = InstanceMessage {
            files: vec![],
            goto_line: None,
            goto_column: None,
            encoding: None,
            language: None,
            read_only: false,
            new_tab_group: false,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: InstanceMessage = serde_json::from_str(&json).unwrap();
        assert!(decoded.files.is_empty());
        assert!(decoded.goto_line.is_none());
    }
}
