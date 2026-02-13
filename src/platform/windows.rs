//! Windows shell integration — registry operations for file associations,
//! context menu entries, and default handler registration.
//!
//! This module generates `.reg` files and PowerShell scripts for shell integration.
//! Direct registry access requires the `winreg` crate on Windows.

use crate::platform::{ContextMenuEntry, ContextMenuScope, FileAssociation, PlatformIntegration};
use std::path::{Path, PathBuf};

pub struct WindowsIntegration;

impl WindowsIntegration {
    pub fn new() -> Self {
        Self
    }

    /// Generate a .reg file content for importing file associations and context menu
    pub fn generate_reg_file(
        exe_path: &str,
        associations: &[FileAssociation],
        context_menu: &[ContextMenuEntry],
    ) -> String {
        let exe_escaped = exe_path.replace('\\', "\\\\");
        let mut reg = String::from("Windows Registry Editor Version 5.00\r\n\r\n");

        // ProgID for Notepad+++
        reg.push_str("; === Notepad+++ ProgID ===\r\n");
        reg.push_str("[HKEY_CURRENT_USER\\Software\\Classes\\NotepadPPP.File]\r\n");
        reg.push_str("@=\"Notepad+++ File\"\r\n\r\n");
        reg.push_str(&format!(
            "[HKEY_CURRENT_USER\\Software\\Classes\\NotepadPPP.File\\DefaultIcon]\r\n\
             @=\"{exe_escaped},0\"\r\n\r\n"
        ));
        reg.push_str(&format!(
            "[HKEY_CURRENT_USER\\Software\\Classes\\NotepadPPP.File\\shell\\open\\command]\r\n\
             @=\"\\\"{exe_escaped}\\\" \\\"%1\\\"\"\r\n\r\n"
        ));

        // File associations
        reg.push_str("; === File Associations ===\r\n");
        for assoc in associations {
            reg.push_str(&format!(
                "[HKEY_CURRENT_USER\\Software\\Classes\\.{ext}]\r\n\
                 @=\"NotepadPPP.File\"\r\n\r\n\
                 [HKEY_CURRENT_USER\\Software\\Classes\\.{ext}\\OpenWithProgids]\r\n\
                 \"NotepadPPP.File\"=\"\"\r\n\r\n",
                ext = assoc.extension
            ));
        }

        // Context menu entries
        reg.push_str("; === Context Menu Entries ===\r\n");
        for entry in context_menu {
            match &entry.scope {
                ContextMenuScope::AllFiles => {
                    reg.push_str(&format!(
                        "[HKEY_CURRENT_USER\\Software\\Classes\\*\\shell\\{id}]\r\n\
                         @=\"{label}\"\r\n\
                         \"Icon\"=\"{exe_escaped},0\"\r\n\r\n\
                         [HKEY_CURRENT_USER\\Software\\Classes\\*\\shell\\{id}\\command]\r\n\
                         @=\"\\\"{exe_escaped}\\\" \\\"%1\\\"\"\r\n\r\n",
                        id = entry.id,
                        label = entry.label,
                    ));
                }
                ContextMenuScope::Directories => {
                    // Background context menu (right-click in folder)
                    reg.push_str(&format!(
                        "[HKEY_CURRENT_USER\\Software\\Classes\\Directory\\Background\\shell\\{id}]\r\n\
                         @=\"{label}\"\r\n\
                         \"Icon\"=\"{exe_escaped},0\"\r\n\r\n\
                         [HKEY_CURRENT_USER\\Software\\Classes\\Directory\\Background\\shell\\{id}\\command]\r\n\
                         @=\"\\\"{exe_escaped}\\\" \\\"%V\\\"\"\r\n\r\n",
                        id = entry.id,
                        label = entry.label,
                    ));
                    // Directory context menu (right-click on folder)
                    reg.push_str(&format!(
                        "[HKEY_CURRENT_USER\\Software\\Classes\\Directory\\shell\\{id}]\r\n\
                         @=\"{label}\"\r\n\
                         \"Icon\"=\"{exe_escaped},0\"\r\n\r\n\
                         [HKEY_CURRENT_USER\\Software\\Classes\\Directory\\shell\\{id}\\command]\r\n\
                         @=\"\\\"{exe_escaped}\\\" \\\"%1\\\"\"\r\n\r\n",
                        id = entry.id,
                        label = entry.label,
                    ));
                }
                ContextMenuScope::Extensions(exts) => {
                    for ext in exts {
                        reg.push_str(&format!(
                            "[HKEY_CURRENT_USER\\Software\\Classes\\.{ext}\\shell\\{id}]\r\n\
                             @=\"{label}\"\r\n\
                             \"Icon\"=\"{exe_escaped},0\"\r\n\r\n\
                             [HKEY_CURRENT_USER\\Software\\Classes\\.{ext}\\shell\\{id}\\command]\r\n\
                             @=\"\\\"{exe_escaped}\\\" \\\"%1\\\"\"\r\n\r\n",
                            id = entry.id,
                            label = entry.label,
                        ));
                    }
                }
            }
        }

        reg
    }

    /// Generate a .reg file to remove all Notepad+++ shell integration
    pub fn generate_uninstall_reg(
        associations: &[FileAssociation],
        context_menu: &[ContextMenuEntry],
    ) -> String {
        let mut reg = String::from("Windows Registry Editor Version 5.00\r\n\r\n");

        reg.push_str("; === Remove Notepad+++ ProgID ===\r\n");
        reg.push_str("[-HKEY_CURRENT_USER\\Software\\Classes\\NotepadPPP.File]\r\n\r\n");

        reg.push_str("; === Remove File Associations ===\r\n");
        for assoc in associations {
            reg.push_str(&format!(
                "[-HKEY_CURRENT_USER\\Software\\Classes\\.{}]\r\n\r\n",
                assoc.extension
            ));
        }

        reg.push_str("; === Remove Context Menu Entries ===\r\n");
        for entry in context_menu {
            match &entry.scope {
                ContextMenuScope::AllFiles => {
                    reg.push_str(&format!(
                        "[-HKEY_CURRENT_USER\\Software\\Classes\\*\\shell\\{}]\r\n\r\n",
                        entry.id
                    ));
                }
                ContextMenuScope::Directories => {
                    reg.push_str(&format!(
                        "[-HKEY_CURRENT_USER\\Software\\Classes\\Directory\\Background\\shell\\{}]\r\n\
                         [-HKEY_CURRENT_USER\\Software\\Classes\\Directory\\shell\\{}]\r\n\r\n",
                        entry.id, entry.id
                    ));
                }
                ContextMenuScope::Extensions(exts) => {
                    for ext in exts {
                        reg.push_str(&format!(
                            "[-HKEY_CURRENT_USER\\Software\\Classes\\.{}\\shell\\{}]\r\n\r\n",
                            ext, entry.id
                        ));
                    }
                }
            }
        }

        reg
    }

    /// Generate a PowerShell script for registering shell integration
    /// (alternative to .reg files — works without admin rights via HKCU)
    pub fn generate_powershell_register(
        exe_path: &str,
        associations: &[FileAssociation],
        context_menu: &[ContextMenuEntry],
    ) -> String {
        let mut ps = String::new();
        ps.push_str("# Notepad+++ Shell Integration Registration\r\n");
        ps.push_str("# Run this script in PowerShell to register file associations and context menu\r\n\r\n");

        ps.push_str(&format!(
            "$exePath = \"{}\"\r\n\r\n",
            exe_path.replace('\\', "\\\\")
        ));

        // ProgID
        ps.push_str("# Create ProgID\r\n");
        ps.push_str("New-Item -Path 'HKCU:\\Software\\Classes\\NotepadPPP.File' -Force | Out-Null\r\n");
        ps.push_str("Set-ItemProperty -Path 'HKCU:\\Software\\Classes\\NotepadPPP.File' -Name '(Default)' -Value 'Notepad+++ File'\r\n");
        ps.push_str("New-Item -Path 'HKCU:\\Software\\Classes\\NotepadPPP.File\\DefaultIcon' -Force | Out-Null\r\n");
        ps.push_str("Set-ItemProperty -Path 'HKCU:\\Software\\Classes\\NotepadPPP.File\\DefaultIcon' -Name '(Default)' -Value \"$exePath,0\"\r\n");
        ps.push_str("New-Item -Path 'HKCU:\\Software\\Classes\\NotepadPPP.File\\shell\\open\\command' -Force | Out-Null\r\n");
        ps.push_str("Set-ItemProperty -Path 'HKCU:\\Software\\Classes\\NotepadPPP.File\\shell\\open\\command' -Name '(Default)' -Value \"\"\"$exePath\"\" \"\"%1\"\"\"\r\n\r\n");

        // File associations
        ps.push_str("# Register file associations\r\n");
        ps.push_str("$extensions = @(\r\n");
        for (i, assoc) in associations.iter().enumerate() {
            let comma = if i < associations.len() - 1 { "," } else { "" };
            ps.push_str(&format!("    '{}'{}\r\n", assoc.extension, comma));
        }
        ps.push_str(")\r\n\r\n");
        ps.push_str("foreach ($ext in $extensions) {\r\n");
        ps.push_str("    New-Item -Path \"HKCU:\\Software\\Classes\\.$ext\" -Force | Out-Null\r\n");
        ps.push_str("    New-Item -Path \"HKCU:\\Software\\Classes\\.$ext\\OpenWithProgids\" -Force | Out-Null\r\n");
        ps.push_str("    New-ItemProperty -Path \"HKCU:\\Software\\Classes\\.$ext\\OpenWithProgids\" -Name 'NotepadPPP.File' -Value '' -PropertyType String -Force | Out-Null\r\n");
        ps.push_str("}\r\n\r\n");

        // Context menu
        ps.push_str("# Register context menu entries\r\n");
        for entry in context_menu {
            match &entry.scope {
                ContextMenuScope::AllFiles => {
                    ps.push_str(&format!(
                        "New-Item -Path 'HKCU:\\Software\\Classes\\*\\shell\\{}\\command' -Force | Out-Null\r\n\
                         Set-ItemProperty -Path 'HKCU:\\Software\\Classes\\*\\shell\\{}' -Name '(Default)' -Value '{}'\r\n\
                         Set-ItemProperty -Path 'HKCU:\\Software\\Classes\\*\\shell\\{}' -Name 'Icon' -Value \"$exePath,0\"\r\n\
                         Set-ItemProperty -Path 'HKCU:\\Software\\Classes\\*\\shell\\{}\\command' -Name '(Default)' -Value \"\"\"$exePath\"\" \"\"%1\"\"\"\r\n\r\n",
                        entry.id, entry.id, entry.label, entry.id, entry.id
                    ));
                }
                ContextMenuScope::Directories => {
                    ps.push_str(&format!(
                        "New-Item -Path 'HKCU:\\Software\\Classes\\Directory\\Background\\shell\\{}\\command' -Force | Out-Null\r\n\
                         Set-ItemProperty -Path 'HKCU:\\Software\\Classes\\Directory\\Background\\shell\\{}' -Name '(Default)' -Value '{}'\r\n\
                         Set-ItemProperty -Path 'HKCU:\\Software\\Classes\\Directory\\Background\\shell\\{}' -Name 'Icon' -Value \"$exePath,0\"\r\n\
                         Set-ItemProperty -Path 'HKCU:\\Software\\Classes\\Directory\\Background\\shell\\{}\\command' -Name '(Default)' -Value \"\"\"$exePath\"\" \"\"%V\"\"\"\r\n\r\n\
                         New-Item -Path 'HKCU:\\Software\\Classes\\Directory\\shell\\{}\\command' -Force | Out-Null\r\n\
                         Set-ItemProperty -Path 'HKCU:\\Software\\Classes\\Directory\\shell\\{}' -Name '(Default)' -Value '{}'\r\n\
                         Set-ItemProperty -Path 'HKCU:\\Software\\Classes\\Directory\\shell\\{}' -Name 'Icon' -Value \"$exePath,0\"\r\n\
                         Set-ItemProperty -Path 'HKCU:\\Software\\Classes\\Directory\\shell\\{}\\command' -Name '(Default)' -Value \"\"\"$exePath\"\" \"\"%1\"\"\"\r\n\r\n",
                        entry.id, entry.id, entry.label, entry.id, entry.id,
                        entry.id, entry.id, entry.label, entry.id, entry.id
                    ));
                }
                ContextMenuScope::Extensions(_) => {}
            }
        }

        ps.push_str("# Notify shell of changes\r\n");
        ps.push_str("$signature = @'\r\n");
        ps.push_str("[DllImport(\"shell32.dll\", CharSet = CharSet.Auto, SetLastError = true)]\r\n");
        ps.push_str("public static extern void SHChangeNotify(int wEventId, int uFlags, IntPtr dwItem1, IntPtr dwItem2);\r\n");
        ps.push_str("'@\r\n");
        ps.push_str("$shell = Add-Type -MemberDefinition $signature -Name 'ShellNotify' -Namespace 'Win32' -PassThru\r\n");
        ps.push_str("$shell::SHChangeNotify(0x08000000, 0x0000, [IntPtr]::Zero, [IntPtr]::Zero)\r\n\r\n");
        ps.push_str("Write-Host 'Notepad+++ shell integration registered successfully!' -ForegroundColor Green\r\n");

        ps
    }

    /// Generate a PowerShell script for unregistering shell integration
    pub fn generate_powershell_unregister(
        associations: &[FileAssociation],
        context_menu: &[ContextMenuEntry],
    ) -> String {
        let mut ps = String::new();
        ps.push_str("# Notepad+++ Shell Integration Removal\r\n\r\n");

        ps.push_str("# Remove ProgID\r\n");
        ps.push_str("Remove-Item -Path 'HKCU:\\Software\\Classes\\NotepadPPP.File' -Recurse -Force -ErrorAction SilentlyContinue\r\n\r\n");

        ps.push_str("# Remove file associations\r\n");
        for assoc in associations {
            ps.push_str(&format!(
                "Remove-ItemProperty -Path 'HKCU:\\Software\\Classes\\.{}\\OpenWithProgids' -Name 'NotepadPPP.File' -Force -ErrorAction SilentlyContinue\r\n",
                assoc.extension
            ));
        }
        ps.push_str("\r\n");

        ps.push_str("# Remove context menu entries\r\n");
        for entry in context_menu {
            match &entry.scope {
                ContextMenuScope::AllFiles => {
                    ps.push_str(&format!(
                        "Remove-Item -Path 'HKCU:\\Software\\Classes\\*\\shell\\{}' -Recurse -Force -ErrorAction SilentlyContinue\r\n",
                        entry.id
                    ));
                }
                ContextMenuScope::Directories => {
                    ps.push_str(&format!(
                        "Remove-Item -Path 'HKCU:\\Software\\Classes\\Directory\\Background\\shell\\{}' -Recurse -Force -ErrorAction SilentlyContinue\r\n\
                         Remove-Item -Path 'HKCU:\\Software\\Classes\\Directory\\shell\\{}' -Recurse -Force -ErrorAction SilentlyContinue\r\n",
                        entry.id, entry.id
                    ));
                }
                ContextMenuScope::Extensions(_) => {}
            }
        }

        ps.push_str("\r\n# Notify shell of changes\r\n");
        ps.push_str("$signature = @'\r\n");
        ps.push_str("[DllImport(\"shell32.dll\", CharSet = CharSet.Auto, SetLastError = true)]\r\n");
        ps.push_str("public static extern void SHChangeNotify(int wEventId, int uFlags, IntPtr dwItem1, IntPtr dwItem2);\r\n");
        ps.push_str("'@\r\n");
        ps.push_str("$shell = Add-Type -MemberDefinition $signature -Name 'ShellNotify' -Namespace 'Win32' -PassThru\r\n");
        ps.push_str("$shell::SHChangeNotify(0x08000000, 0x0000, [IntPtr]::Zero, [IntPtr]::Zero)\r\n\r\n");
        ps.push_str("Write-Host 'Notepad+++ shell integration removed.' -ForegroundColor Yellow\r\n");

        ps
    }
}

impl PlatformIntegration for WindowsIntegration {
    fn register_file_associations(
        &self,
        exe_path: &str,
        associations: &[FileAssociation],
    ) -> Result<(), String> {
        // On Windows, this would write to the registry directly using winreg
        // For cross-platform builds, we generate scripts
        let reg_content = Self::generate_reg_file(
            exe_path,
            associations,
            &crate::platform::default_context_menu_entries(exe_path),
        );
        let reg_path = self.config_dir().join("register.reg");
        std::fs::write(&reg_path, &reg_content)
            .map_err(|e| format!("Failed to write reg file: {e}"))?;

        let ps_content = Self::generate_powershell_register(
            exe_path,
            associations,
            &crate::platform::default_context_menu_entries(exe_path),
        );
        let ps_path = self.config_dir().join("register.ps1");
        std::fs::write(&ps_path, &ps_content)
            .map_err(|e| format!("Failed to write PowerShell script: {e}"))?;

        // On actual Windows, execute the registry import
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("reg")
                .args(["import", &reg_path.to_string_lossy()])
                .output()
                .map_err(|e| format!("Failed to import registry: {e}"))?;
        }

        Ok(())
    }

    fn unregister_file_associations(
        &self,
        associations: &[FileAssociation],
    ) -> Result<(), String> {
        let reg_content = Self::generate_uninstall_reg(
            associations,
            &crate::platform::default_context_menu_entries(""),
        );
        let reg_path = self.config_dir().join("unregister.reg");
        std::fs::write(&reg_path, &reg_content)
            .map_err(|e| format!("Failed to write reg file: {e}"))?;

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("reg")
                .args(["import", &reg_path.to_string_lossy()])
                .output()
                .map_err(|e| format!("Failed to import registry: {e}"))?;
        }

        Ok(())
    }

    fn register_context_menu(
        &self,
        exe_path: &str,
        entries: &[ContextMenuEntry],
    ) -> Result<(), String> {
        // Context menu is handled as part of register_file_associations
        let _ = (exe_path, entries);
        Ok(())
    }

    fn unregister_context_menu(
        &self,
        entries: &[ContextMenuEntry],
    ) -> Result<(), String> {
        let _ = entries;
        Ok(())
    }

    fn set_default_handler(
        &self,
        exe_path: &str,
        extensions: &[String],
    ) -> Result<(), String> {
        // Setting default handler in Windows 10+ requires user interaction
        // We register OpenWithProgids which shows in the "Open With" menu
        let _ = (exe_path, extensions);
        Ok(())
    }

    fn is_registered(&self) -> bool {
        // Check if ProgID exists in registry
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("reg")
                .args(["query", r"HKCU\Software\Classes\NotepadPPP.File"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        #[cfg(not(target_os = "windows"))]
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
    use crate::platform::{default_associations, default_context_menu_entries, ContextMenuScope};

    #[test]
    fn test_generate_reg_file_contains_progid() {
        let reg = WindowsIntegration::generate_reg_file(
            r"C:\Program Files\Notepad+++\notepadppp.exe",
            &[],
            &[],
        );
        assert!(reg.contains("Windows Registry Editor Version 5.00"));
        assert!(reg.contains("NotepadPPP.File"));
        assert!(reg.contains("shell\\open\\command"));
    }

    #[test]
    fn test_generate_reg_file_contains_associations() {
        let assocs = vec![FileAssociation {
            extension: "rs".to_string(),
            description: "Rust Source File".to_string(),
            icon_index: 3,
        }];
        let reg = WindowsIntegration::generate_reg_file(
            r"C:\notepadppp.exe",
            &assocs,
            &[],
        );
        assert!(reg.contains(".rs"));
        assert!(reg.contains("OpenWithProgids"));
    }

    #[test]
    fn test_generate_reg_file_contains_context_menu_all_files() {
        let entries = vec![ContextMenuEntry {
            id: "NotepadPPP.Open".to_string(),
            label: "Open with Notepad+++".to_string(),
            scope: ContextMenuScope::AllFiles,
        }];
        let reg = WindowsIntegration::generate_reg_file(
            r"C:\notepadppp.exe",
            &[],
            &entries,
        );
        assert!(reg.contains(r"*\shell\NotepadPPP.Open"));
        assert!(reg.contains("Open with Notepad+++"));
    }

    #[test]
    fn test_generate_reg_file_contains_directory_context_menu() {
        let entries = vec![ContextMenuEntry {
            id: "NotepadPPP.Folder".to_string(),
            label: "Open folder in Notepad+++".to_string(),
            scope: ContextMenuScope::Directories,
        }];
        let reg = WindowsIntegration::generate_reg_file(
            r"C:\notepadppp.exe",
            &[],
            &entries,
        );
        assert!(reg.contains(r"Directory\Background\shell\NotepadPPP.Folder"));
        assert!(reg.contains(r"Directory\shell\NotepadPPP.Folder"));
    }

    #[test]
    fn test_generate_uninstall_reg() {
        let assocs = vec![FileAssociation {
            extension: "txt".to_string(),
            description: "Text".to_string(),
            icon_index: 1,
        }];
        let entries = vec![ContextMenuEntry {
            id: "NotepadPPP.Open".to_string(),
            label: "Open with Notepad+++".to_string(),
            scope: ContextMenuScope::AllFiles,
        }];
        let reg = WindowsIntegration::generate_uninstall_reg(&assocs, &entries);
        assert!(reg.contains("[-HKEY_CURRENT_USER\\Software\\Classes\\NotepadPPP.File]"));
        assert!(reg.contains("[-HKEY_CURRENT_USER\\Software\\Classes\\.txt]"));
        assert!(reg.contains("[-HKEY_CURRENT_USER\\Software\\Classes\\*\\shell\\NotepadPPP.Open]"));
    }

    #[test]
    fn test_generate_powershell_register() {
        let assocs = vec![FileAssociation {
            extension: "rs".to_string(),
            description: "Rust".to_string(),
            icon_index: 3,
        }];
        let entries = default_context_menu_entries(r"C:\notepadppp.exe");
        let ps = WindowsIntegration::generate_powershell_register(
            r"C:\notepadppp.exe",
            &assocs,
            &entries,
        );
        assert!(ps.contains("New-Item"));
        assert!(ps.contains("NotepadPPP.File"));
        assert!(ps.contains("SHChangeNotify"));
        assert!(ps.contains("'rs'"));
    }

    #[test]
    fn test_generate_powershell_unregister() {
        let assocs = vec![FileAssociation {
            extension: "txt".to_string(),
            description: "Text".to_string(),
            icon_index: 1,
        }];
        let entries = default_context_menu_entries("");
        let ps = WindowsIntegration::generate_powershell_unregister(&assocs, &entries);
        assert!(ps.contains("Remove-Item"));
        assert!(ps.contains("NotepadPPP.File"));
    }

    #[test]
    fn test_default_associations_not_empty() {
        let assocs = default_associations();
        assert!(assocs.len() > 40);
        assert!(assocs.iter().any(|a| a.extension == "rs"));
        assert!(assocs.iter().any(|a| a.extension == "txt"));
        assert!(assocs.iter().any(|a| a.extension == "json"));
        assert!(assocs.iter().any(|a| a.extension == "py"));
    }

    #[test]
    fn test_default_context_menu_entries() {
        let entries = default_context_menu_entries(r"C:\notepadppp.exe");
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|e| e.scope == ContextMenuScope::AllFiles));
        assert!(entries.iter().any(|e| e.scope == ContextMenuScope::Directories));
    }

    #[test]
    fn test_reg_file_escapes_backslashes() {
        let reg = WindowsIntegration::generate_reg_file(
            r"C:\Program Files\Notepad+++\notepadppp.exe",
            &[],
            &[],
        );
        assert!(reg.contains("C:\\\\Program Files\\\\Notepad+++\\\\notepadppp.exe"));
    }

    #[test]
    fn test_full_reg_file_with_defaults() {
        let assocs = default_associations();
        let entries = default_context_menu_entries(r"C:\notepadppp.exe");
        let reg = WindowsIntegration::generate_reg_file(r"C:\notepadppp.exe", &assocs, &entries);
        // Should contain entries for all extensions
        for assoc in &assocs {
            assert!(
                reg.contains(&format!(".{}", assoc.extension)),
                "Missing extension: {}",
                assoc.extension
            );
        }
    }

    #[test]
    fn test_config_dir() {
        let integration = WindowsIntegration::new();
        let dir = integration.config_dir();
        assert!(dir.ends_with("notepadppp"));
    }
}
