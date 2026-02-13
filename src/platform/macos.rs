//! macOS shell integration — Info.plist and Launch Services for file associations.

use crate::platform::{ContextMenuEntry, FileAssociation, PlatformIntegration};
use std::path::PathBuf;

pub struct MacOSIntegration;

impl MacOSIntegration {
    pub fn new() -> Self {
        Self
    }

    /// Generate Info.plist document type entries for file associations
    pub fn generate_plist_document_types(associations: &[FileAssociation]) -> String {
        let mut plist = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        plist.push_str("<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n");
        plist.push_str("<plist version=\"1.0\">\n<dict>\n");
        plist.push_str("    <key>CFBundleName</key>\n    <string>Notepad+++</string>\n");
        plist.push_str("    <key>CFBundleIdentifier</key>\n    <string>com.notepadppp.editor</string>\n");
        plist.push_str("    <key>CFBundleVersion</key>\n    <string>0.1.0</string>\n");
        plist.push_str("    <key>CFBundleDocumentTypes</key>\n    <array>\n");

        for assoc in associations {
            plist.push_str("        <dict>\n");
            plist.push_str(&format!(
                "            <key>CFBundleTypeName</key>\n            <string>{}</string>\n",
                assoc.description
            ));
            plist.push_str("            <key>CFBundleTypeRole</key>\n            <string>Editor</string>\n");
            plist.push_str("            <key>CFBundleTypeExtensions</key>\n            <array>\n");
            plist.push_str(&format!(
                "                <string>{}</string>\n",
                assoc.extension
            ));
            plist.push_str("            </array>\n");
            plist.push_str("        </dict>\n");
        }

        plist.push_str("    </array>\n</dict>\n</plist>\n");
        plist
    }
}

impl PlatformIntegration for MacOSIntegration {
    fn register_file_associations(
        &self,
        _exe_path: &str,
        _associations: &[FileAssociation],
    ) -> Result<(), String> {
        // On macOS, associations are set via Info.plist in the .app bundle
        // The app bundle creation is handled by the build/packaging step
        Ok(())
    }

    fn unregister_file_associations(
        &self,
        _associations: &[FileAssociation],
    ) -> Result<(), String> {
        Ok(())
    }

    fn register_context_menu(
        &self,
        _exe_path: &str,
        _entries: &[ContextMenuEntry],
    ) -> Result<(), String> {
        // macOS Finder context menu requires a Finder Sync Extension or Services
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
        _extensions: &[String],
    ) -> Result<(), String> {
        Ok(())
    }

    fn is_registered(&self) -> bool {
        false
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

    #[test]
    fn test_generate_plist() {
        let assocs = vec![
            FileAssociation {
                extension: "rs".to_string(),
                description: "Rust Source File".to_string(),
                icon_index: 3,
            },
        ];
        let plist = MacOSIntegration::generate_plist_document_types(&assocs);
        assert!(plist.contains("CFBundleDocumentTypes"));
        assert!(plist.contains("Rust Source File"));
        assert!(plist.contains("<string>rs</string>"));
        assert!(plist.contains("com.notepadppp.editor"));
    }

    #[test]
    fn test_config_dir() {
        let integration = MacOSIntegration::new();
        let dir = integration.config_dir();
        assert!(dir.ends_with("notepadppp"));
    }
}
