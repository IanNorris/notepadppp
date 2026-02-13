//! Linux shell integration — .desktop files and xdg-mime for file associations.

use crate::platform::{ContextMenuEntry, FileAssociation, PlatformIntegration};
use std::path::PathBuf;

pub struct LinuxIntegration;

impl LinuxIntegration {
    pub fn new() -> Self {
        Self
    }

    /// Generate .desktop file content for XDG integration
    pub fn generate_desktop_file(exe_path: &str, associations: &[FileAssociation]) -> String {
        let mime_types: Vec<String> = associations
            .iter()
            .map(|a| extension_to_mime(&a.extension))
            .collect();

        format!(
            "[Desktop Entry]\n\
             Version=1.0\n\
             Type=Application\n\
             Name=Notepad+++\n\
             Comment=A fast, native text editor for programmers\n\
             Exec={exe_path} %F\n\
             Icon=notepadppp\n\
             Terminal=false\n\
             Categories=TextEditor;Development;Utility;\n\
             MimeType={mime_types};\n\
             StartupNotify=true\n\
             StartupWMClass=notepadppp\n\
             Keywords=text;editor;code;programming;notepad;\n",
            mime_types = mime_types.join(";"),
        )
    }

    /// Generate xdg-mime commands to register file associations
    pub fn generate_mime_registration_script(
        associations: &[FileAssociation],
    ) -> String {
        let mut script = String::from("#!/bin/bash\n");
        script.push_str("# Notepad+++ MIME type registration\n\n");

        // Install desktop file
        script.push_str("# Install .desktop file\n");
        script.push_str("xdg-desktop-menu install --novendor notepadppp.desktop\n\n");

        // Register MIME associations
        script.push_str("# Register MIME type associations\n");
        for assoc in associations {
            let mime = extension_to_mime(&assoc.extension);
            script.push_str(&format!(
                "xdg-mime default notepadppp.desktop {mime}\n"
            ));
        }

        script.push_str("\necho 'Notepad+++ registered successfully!'\n");
        script
    }
}

/// Map file extension to MIME type
fn extension_to_mime(ext: &str) -> String {
    match ext {
        "txt" | "log" | "cfg" | "conf" | "ini" | "env" | "gitignore" | "editorconfig" => {
            "text/plain".to_string()
        }
        "html" | "htm" => "text/html".to_string(),
        "css" => "text/css".to_string(),
        "js" => "application/javascript".to_string(),
        "ts" => "text/typescript".to_string(),
        "jsx" | "tsx" => "text/javascript".to_string(),
        "json" => "application/json".to_string(),
        "xml" => "application/xml".to_string(),
        "yaml" | "yml" => "application/x-yaml".to_string(),
        "toml" => "application/toml".to_string(),
        "rs" => "text/x-rust".to_string(),
        "py" => "text/x-python".to_string(),
        "rb" => "text/x-ruby".to_string(),
        "go" => "text/x-go".to_string(),
        "java" => "text/x-java".to_string(),
        "kt" => "text/x-kotlin".to_string(),
        "c" => "text/x-csrc".to_string(),
        "cpp" => "text/x-c++src".to_string(),
        "h" | "hpp" => "text/x-chdr".to_string(),
        "cs" => "text/x-csharp".to_string(),
        "swift" => "text/x-swift".to_string(),
        "sh" | "bash" | "zsh" => "application/x-shellscript".to_string(),
        "bat" | "cmd" => "application/x-bat".to_string(),
        "ps1" => "application/x-powershell".to_string(),
        "sql" => "application/sql".to_string(),
        "csv" => "text/csv".to_string(),
        "tsv" => "text/tab-separated-values".to_string(),
        "md" | "markdown" => "text/markdown".to_string(),
        "tex" => "text/x-tex".to_string(),
        "rst" => "text/x-rst".to_string(),
        "dockerfile" => "text/x-dockerfile".to_string(),
        "makefile" => "text/x-makefile".to_string(),
        _ => format!("text/x-{ext}"),
    }
}

impl PlatformIntegration for LinuxIntegration {
    fn register_file_associations(
        &self,
        exe_path: &str,
        associations: &[FileAssociation],
    ) -> Result<(), String> {
        let desktop_content = Self::generate_desktop_file(exe_path, associations);
        let desktop_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/usr/share"))
            .join("applications");

        std::fs::create_dir_all(&desktop_dir)
            .map_err(|e| format!("Failed to create desktop dir: {e}"))?;

        let desktop_path = desktop_dir.join("notepadppp.desktop");
        std::fs::write(&desktop_path, &desktop_content)
            .map_err(|e| format!("Failed to write .desktop file: {e}"))?;

        // Run xdg-mime for each association
        for assoc in associations {
            let mime = extension_to_mime(&assoc.extension);
            let _ = std::process::Command::new("xdg-mime")
                .args(["default", "notepadppp.desktop", &mime])
                .output();
        }

        Ok(())
    }

    fn unregister_file_associations(
        &self,
        _associations: &[FileAssociation],
    ) -> Result<(), String> {
        let desktop_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/usr/share"))
            .join("applications")
            .join("notepadppp.desktop");
        let _ = std::fs::remove_file(desktop_path);
        Ok(())
    }

    fn register_context_menu(
        &self,
        _exe_path: &str,
        _entries: &[ContextMenuEntry],
    ) -> Result<(), String> {
        // Linux desktop environments don't have a universal context menu mechanism
        // Nautilus scripts, Dolphin service menus, etc. are DE-specific
        Ok(())
    }

    fn unregister_context_menu(
        &self,
        _entries: &[ContextMenuEntry],
    ) -> Result<(), String> {
        Ok(())
    }

    fn set_default_handler(
        &self,
        _exe_path: &str,
        extensions: &[String],
    ) -> Result<(), String> {
        for ext in extensions {
            let mime = extension_to_mime(ext);
            let _ = std::process::Command::new("xdg-mime")
                .args(["default", "notepadppp.desktop", &mime])
                .output();
        }
        Ok(())
    }

    fn is_registered(&self) -> bool {
        let desktop_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/usr/share"))
            .join("applications")
            .join("notepadppp.desktop");
        desktop_path.exists()
    }

    fn config_dir(&self) -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("notepadppp")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::default_associations;

    #[test]
    fn test_generate_desktop_file() {
        let assocs = vec![
            FileAssociation {
                extension: "rs".to_string(),
                description: "Rust".to_string(),
                icon_index: 3,
            },
            FileAssociation {
                extension: "txt".to_string(),
                description: "Text".to_string(),
                icon_index: 1,
            },
        ];
        let desktop = LinuxIntegration::generate_desktop_file("/usr/bin/notepadppp", &assocs);
        assert!(desktop.contains("[Desktop Entry]"));
        assert!(desktop.contains("Name=Notepad+++"));
        assert!(desktop.contains("Exec=/usr/bin/notepadppp %F"));
        assert!(desktop.contains("text/x-rust"));
        assert!(desktop.contains("text/plain"));
        assert!(desktop.contains("Categories=TextEditor"));
    }

    #[test]
    fn test_extension_to_mime_common() {
        assert_eq!(extension_to_mime("rs"), "text/x-rust");
        assert_eq!(extension_to_mime("py"), "text/x-python");
        assert_eq!(extension_to_mime("json"), "application/json");
        assert_eq!(extension_to_mime("html"), "text/html");
        assert_eq!(extension_to_mime("csv"), "text/csv");
        assert_eq!(extension_to_mime("md"), "text/markdown");
        assert_eq!(extension_to_mime("sh"), "application/x-shellscript");
    }

    #[test]
    fn test_extension_to_mime_unknown() {
        assert_eq!(extension_to_mime("xyz"), "text/x-xyz");
    }

    #[test]
    fn test_generate_mime_script() {
        let assocs = default_associations();
        let script = LinuxIntegration::generate_mime_registration_script(&assocs);
        assert!(script.contains("#!/bin/bash"));
        assert!(script.contains("xdg-desktop-menu install"));
        assert!(script.contains("xdg-mime default"));
    }

    #[test]
    fn test_config_dir() {
        let integration = LinuxIntegration::new();
        let dir = integration.config_dir();
        assert!(dir.ends_with("notepadppp"));
    }
}
