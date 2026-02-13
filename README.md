# Notepad+++ 📝

A fast, native text editor for programmers — built from scratch in Rust.

## Why?

Most programmer's text editors are either bloated Electron apps or ancient C/C++ codebases with growing security concerns. Notepad+++ is a modern, memory-safe alternative built in Rust with native UI, delivering powerful editing features with strong security and cross-platform support.

## Features

See [FEATURES.md](FEATURES.md) for the complete feature list and implementation roadmap.

### ✅ What's Working Now

**Core Editing** — Full-featured tabbed editor with unlimited undo/redo, word wrap, line numbers, auto-save, crash recovery, and session restore.

**Search & Replace** — Find/Replace with regex, match case, whole word, incremental search, match highlighting, find-in-files with results panel, and bookmarks.

**Syntax Highlighting** — 80+ languages powered by syntect, with matching bracket highlighting.

**Advanced Editing** — Multi-cursor editing, rectangular/column selection, macro recording & playback, line sorting, dedup, auto-indent, and tab/space conversion.

**Encoding & EOL** — UTF-8 (with/without BOM), ANSI, encoding conversion, CRLF/LF/CR line endings with mixed-EOL detection.

**Navigation** — Document minimap, function list panel, Go to Line (Ctrl+G), and Command Palette (Ctrl+P).

**Built-in Tools** — Markdown live preview, JSON format/validate/minify, CSV tabular view, side-by-side file diff, hex viewer with ASCII sidebar, HTML export, Base64/URL encode/decode.

**Performance** — Rope-based text storage, efficient undo/redo, streaming large file loading with auto-disable of expensive features.

**Customization** — Dark theme, split editor panels, configurable keyboard shortcuts (JSON), zoom, whitespace display, preferences dialog.

### 🔮 Planned
- Code folding
- Folder-as-workspace tree view
- Light theme and full theme customization
- Spell checking, code snippets, auto-completion
- Windows shell integration
- And more — see [FEATURES.md](FEATURES.md)

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run
cargo run
```

## Requirements
- Rust 1.75+ (2024 edition)
- On Linux: `libgtk-3-dev` (for file dialogs)

## Architecture

```
src/
├── main.rs                  # Entry point
├── lib.rs                   # Library crate
├── editor/
│   ├── bookmarks.rs         # Line bookmarks
│   ├── brackets.rs          # Bracket matching
│   ├── buffer.rs            # Rope-based text buffer with undo/redo
│   ├── column_select.rs     # Rectangular/column selection
│   ├── cursor.rs            # Cursor and selection state
│   ├── document.rs          # Document model (buffer + cursor + metadata)
│   ├── function_list.rs     # Function/method list parsing
│   ├── indent.rs            # Auto-indent logic
│   ├── macros.rs            # Macro recording & playback
│   ├── multi_cursor.rs      # Multi-cursor editing
│   ├── syntax.rs            # Syntax highlighting (syntect)
│   └── tab_manager.rs       # Multi-tab management
├── io/
│   ├── file_io.rs           # File reading/writing with encoding support
│   ├── keybindings.rs       # Configurable keyboard shortcuts
│   ├── large_file.rs        # Large file handling & streaming
│   ├── line_ending_detect.rs # EOL detection & conversion
│   ├── recent_files.rs      # Recent file history persistence
│   ├── session.rs           # Session save/restore
│   └── settings.rs          # Preferences & configuration
├── search/
│   ├── engine.rs            # Search engine (regex, match options)
│   ├── find_in_files.rs     # Search across files in directory
│   └── history.rs           # Search history
├── tools/
│   ├── csv_viewer.rs        # CSV tabular view
│   ├── diff_tool.rs         # File comparison (diff)
│   ├── export.rs            # HTML export
│   ├── hex_viewer.rs        # Hex viewer with ASCII sidebar
│   ├── json_tools.rs        # JSON format/validate/minify
│   ├── markdown_viewer.rs   # Markdown live preview
│   └── mime_tools.rs        # Base64/URL encode/decode
└── ui/
    ├── app.rs               # Main egui application
    ├── editor_widget.rs     # Custom editor rendering widget
    ├── search_dialog.rs     # Find/Replace dialog
    └── split_view.rs        # Split editor panels
```

## License

MIT
