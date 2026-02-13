pub mod cli;
pub mod single_instance;

// Platform modules are always compiled — they generate scripts/configs
// that work cross-platform. OS-specific operations use #[cfg] internally.
pub mod windows;
pub mod linux;
pub mod macos;

/// File type association for shell integration
#[derive(Debug, Clone, PartialEq)]
pub struct FileAssociation {
    /// File extension without dot (e.g., "txt", "rs", "json")
    pub extension: String,
    /// Human-readable description (e.g., "Rust Source File")
    pub description: String,
    /// Icon index in the executable's resource table
    pub icon_index: u32,
}

/// Context menu entry configuration
#[derive(Debug, Clone, PartialEq)]
pub struct ContextMenuEntry {
    /// Registry/desktop key name
    pub id: String,
    /// Display text in context menu
    pub label: String,
    /// Whether this entry applies to all files or specific extensions
    pub scope: ContextMenuScope,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContextMenuScope {
    /// Show for all file types (via * or Directory\Background)
    AllFiles,
    /// Show for directories (right-click on folder)
    Directories,
    /// Show for specific extensions only
    Extensions(Vec<String>),
}

/// Common file extensions for a text editor targeting programmers
pub fn default_associations() -> Vec<FileAssociation> {
    let assocs = vec![
        // Plain text
        ("txt", "Text File", 1),
        ("log", "Log File", 1),
        ("cfg", "Configuration File", 1),
        ("conf", "Configuration File", 1),
        ("ini", "INI Configuration File", 1),
        // Web
        ("html", "HTML File", 2),
        ("htm", "HTML File", 2),
        ("css", "CSS Stylesheet", 2),
        ("js", "JavaScript File", 2),
        ("ts", "TypeScript File", 2),
        ("jsx", "JSX File", 2),
        ("tsx", "TSX File", 2),
        ("json", "JSON File", 2),
        ("xml", "XML File", 2),
        ("yaml", "YAML File", 2),
        ("yml", "YAML File", 2),
        ("toml", "TOML File", 2),
        // Programming
        ("rs", "Rust Source File", 3),
        ("py", "Python Script", 3),
        ("rb", "Ruby Script", 3),
        ("go", "Go Source File", 3),
        ("java", "Java Source File", 3),
        ("kt", "Kotlin Source File", 3),
        ("c", "C Source File", 3),
        ("cpp", "C++ Source File", 3),
        ("h", "C/C++ Header File", 3),
        ("hpp", "C++ Header File", 3),
        ("cs", "C# Source File", 3),
        ("swift", "Swift Source File", 3),
        ("zig", "Zig Source File", 3),
        ("lua", "Lua Script", 3),
        ("pl", "Perl Script", 3),
        ("php", "PHP Script", 3),
        ("r", "R Script", 3),
        ("scala", "Scala Source File", 3),
        // Shell/scripting
        ("sh", "Shell Script", 4),
        ("bash", "Bash Script", 4),
        ("zsh", "Zsh Script", 4),
        ("bat", "Batch File", 4),
        ("cmd", "Command Script", 4),
        ("ps1", "PowerShell Script", 4),
        // Data
        ("csv", "CSV File", 5),
        ("tsv", "Tab-Separated Values", 5),
        ("sql", "SQL File", 5),
        // Markdown/docs
        ("md", "Markdown File", 6),
        ("markdown", "Markdown File", 6),
        ("rst", "reStructuredText File", 6),
        ("tex", "LaTeX File", 6),
        // Config/DevOps
        ("dockerfile", "Dockerfile", 7),
        ("makefile", "Makefile", 7),
        ("env", "Environment File", 7),
        ("gitignore", "Git Ignore File", 7),
        ("editorconfig", "EditorConfig File", 7),
    ];

    assocs
        .into_iter()
        .map(|(ext, desc, icon)| FileAssociation {
            extension: ext.to_string(),
            description: desc.to_string(),
            icon_index: icon,
        })
        .collect()
}

/// Default context menu entries
pub fn default_context_menu_entries(exe_path: &str) -> Vec<ContextMenuEntry> {
    let _ = exe_path;
    vec![
        ContextMenuEntry {
            id: "NotepadPPP.OpenFile".to_string(),
            label: "Open with Notepad+++".to_string(),
            scope: ContextMenuScope::AllFiles,
        },
        ContextMenuEntry {
            id: "NotepadPPP.OpenFolder".to_string(),
            label: "Open folder in Notepad+++".to_string(),
            scope: ContextMenuScope::Directories,
        },
    ]
}

/// Platform integration trait — implemented per OS
pub trait PlatformIntegration {
    /// Register file associations in the OS
    fn register_file_associations(
        &self,
        exe_path: &str,
        associations: &[FileAssociation],
    ) -> Result<(), String>;

    /// Unregister file associations
    fn unregister_file_associations(
        &self,
        associations: &[FileAssociation],
    ) -> Result<(), String>;

    /// Add context menu entries to the OS shell
    fn register_context_menu(
        &self,
        exe_path: &str,
        entries: &[ContextMenuEntry],
    ) -> Result<(), String>;

    /// Remove context menu entries
    fn unregister_context_menu(
        &self,
        entries: &[ContextMenuEntry],
    ) -> Result<(), String>;

    /// Set this application as the default handler for given extensions
    fn set_default_handler(
        &self,
        exe_path: &str,
        extensions: &[String],
    ) -> Result<(), String>;

    /// Check if shell integration is currently registered
    fn is_registered(&self) -> bool;

    /// Get the path where the app should store its configuration
    fn config_dir(&self) -> std::path::PathBuf;
}
