use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyBinding {
    pub key: String,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl KeyBinding {
    pub fn new(key: &str, ctrl: bool, shift: bool, alt: bool) -> Self {
        Self {
            key: key.into(),
            ctrl,
            shift,
            alt,
        }
    }

    /// Display human-readable shortcut string
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.alt {
            parts.push("Alt");
        }
        parts.push(&self.key);
        parts.join("+")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub bindings: HashMap<String, KeyBinding>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::defaults()
    }
}

impl KeyBindings {
    pub fn defaults() -> Self {
        let mut bindings = HashMap::new();
        // File
        bindings.insert("new_file".into(), KeyBinding::new("N", true, false, false));
        bindings.insert("open_file".into(), KeyBinding::new("O", true, false, false));
        bindings.insert("save".into(), KeyBinding::new("S", true, false, false));
        bindings.insert("save_as".into(), KeyBinding::new("S", true, true, false));
        bindings.insert("close_tab".into(), KeyBinding::new("W", true, false, false));
        // Edit
        bindings.insert("undo".into(), KeyBinding::new("Z", true, false, false));
        bindings.insert("redo".into(), KeyBinding::new("Y", true, false, false));
        // View
        bindings.insert("zoom_in".into(), KeyBinding::new("=", true, false, false));
        bindings.insert("zoom_out".into(), KeyBinding::new("-", true, false, false));
        bindings.insert("zoom_reset".into(), KeyBinding::new("0", true, false, false));
        // Search
        bindings.insert("find".into(), KeyBinding::new("F", true, false, false));
        bindings.insert("replace".into(), KeyBinding::new("H", true, false, false));
        bindings.insert("find_next".into(), KeyBinding::new("F3", false, false, false));
        bindings.insert("find_prev".into(), KeyBinding::new("F3", false, true, false));
        // Navigation
        bindings.insert("goto_line".into(), KeyBinding::new("G", true, false, false));
        bindings.insert("command_palette".into(), KeyBinding::new("P", true, false, false));
        bindings.insert("bracket_jump".into(), KeyBinding::new("]", true, false, false));
        // Bookmarks
        bindings.insert("toggle_bookmark".into(), KeyBinding::new("F2", true, false, false));
        bindings.insert("next_bookmark".into(), KeyBinding::new("F2", false, false, false));
        bindings.insert("prev_bookmark".into(), KeyBinding::new("F2", false, true, false));
        // Tools
        bindings.insert("format_json".into(), KeyBinding::new("J", true, true, false));
        bindings.insert("toggle_macro_recording".into(), KeyBinding::new("R", true, true, false));
        bindings.insert("play_last_macro".into(), KeyBinding::new("P", true, true, false));
        // Multi-cursor
        bindings.insert("select_next".into(), KeyBinding::new("D", true, false, false));
        // Hex toggle
        bindings.insert("toggle_hex_view".into(), KeyBinding::new("H", true, true, false));
        // Escape
        bindings.insert("escape".into(), KeyBinding::new("Escape", false, false, false));
        // Line operations
        bindings.insert("delete_line".into(), KeyBinding::new("K", true, true, false));
        bindings.insert("join_lines".into(), KeyBinding::new("J", true, false, false));
        // Case conversion
        bindings.insert("case_upper".into(), KeyBinding::new("U", true, true, false));
        bindings.insert("case_lower".into(), KeyBinding::new("U", true, false, false));
        // Comment toggle
        bindings.insert("toggle_comment".into(), KeyBinding::new("/", true, false, false));
        // Select all occurrences
        bindings.insert("select_all_occurrences".into(), KeyBinding::new("L", true, true, false));
        // Folding
        bindings.insert("toggle_fold".into(), KeyBinding::new("[", true, true, false));
        Self { bindings }
    }

    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read_to_string(path)?;
        let kb: KeyBindings = serde_json::from_str(&data)?;
        Ok(kb)
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn keybindings_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("notepadppp")
            .join("keybindings.json")
    }

    /// Check if the given key and modifiers match a keybinding action.
    /// `key` should be the key name (e.g. "n", "s", "F3", "Escape", "]").
    /// Comparison is case-insensitive for single character keys.
    pub fn matches(&self, action: &str, key: &str, ctrl: bool, shift: bool, alt: bool) -> bool {
        if let Some(binding) = self.bindings.get(action) {
            binding.ctrl == ctrl
                && binding.shift == shift
                && binding.alt == alt
                && binding.key.eq_ignore_ascii_case(key)
        } else {
            false
        }
    }

    /// Returns human-readable shortcut string like "Ctrl+G"
    pub fn display_shortcut(&self, action: &str) -> String {
        self.bindings
            .get(action)
            .map(|b| b.display())
            .unwrap_or_default()
    }

    /// Returns sorted list of (action_name, display_name, shortcut_string)
    pub fn all_actions_sorted(&self) -> Vec<(String, String, String)> {
        let display_names: HashMap<&str, &str> = [
            ("new_file", "New File"),
            ("open_file", "Open File"),
            ("save", "Save"),
            ("save_as", "Save As"),
            ("close_tab", "Close Tab"),
            ("undo", "Undo"),
            ("redo", "Redo"),
            ("zoom_in", "Zoom In"),
            ("zoom_out", "Zoom Out"),
            ("zoom_reset", "Reset Zoom"),
            ("find", "Find"),
            ("replace", "Replace"),
            ("find_next", "Find Next"),
            ("find_prev", "Find Previous"),
            ("goto_line", "Go to Line"),
            ("command_palette", "Command Palette"),
            ("bracket_jump", "Jump to Matching Bracket"),
            ("toggle_bookmark", "Toggle Bookmark"),
            ("next_bookmark", "Next Bookmark"),
            ("prev_bookmark", "Previous Bookmark"),
            ("format_json", "Format JSON"),
            ("toggle_macro_recording", "Start/Stop Macro Recording"),
            ("play_last_macro", "Play Last Macro"),
            ("select_next", "Select Next Occurrence"),
            ("select_all_occurrences", "Select All Occurrences"),
            ("toggle_comment", "Toggle Comment"),
            ("toggle_hex_view", "Toggle Hex View"),
            ("toggle_fold", "Toggle Fold"),
            ("escape", "Escape / Close"),
        ]
        .into_iter()
        .collect();

        let mut actions: Vec<(String, String, String)> = self
            .bindings
            .iter()
            .map(|(action, binding)| {
                let display = display_names
                    .get(action.as_str())
                    .unwrap_or(&action.as_str())
                    .to_string();
                (action.clone(), display, binding.display())
            })
            .collect();
        actions.sort_by(|a, b| a.1.cmp(&b.1));
        actions
    }
}
