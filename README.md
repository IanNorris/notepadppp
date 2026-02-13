# Notepad+++ 📝

A fast, native text editor for programmers — reimagining Notepad++ in Rust.

## Why?

Notepad++ has been repeatedly compromised and is now banned at many companies. Notepad+++ is a clean-room Rust reimplementation that provides the same powerful features with better security, performance, and cross-platform support.

## Features

See [FEATURES.md](FEATURES.md) for the complete feature list and implementation roadmap.

### Current (Skeleton - v0.1.0)
- Multi-tab document editing
- Undo/redo with unlimited history
- File open/save with encoding detection (UTF-8, UTF-8 BOM, UTF-16 LE/BE, ANSI)
- Line ending detection and conversion (CRLF, LF, CR)
- Line operations (duplicate, delete, move, sort, trim, dedup)
- Word wrap toggle
- Line numbers
- Zoom in/out
- Recent files
- Keyboard shortcuts (Ctrl+N/O/S/W/Z/Y and more)
- Status bar (line:col, encoding, EOL, language, tab count)

### Planned
- Split view / multi-panel editing
- Powerful regex search with results history panel
- Syntax highlighting for 80+ languages
- Multi-cursor / column mode editing
- Markdown live preview
- JSON tools (format, validate, tree view)
- CSV viewer/editor
- File comparison (diff)
- Hex viewer/editor
- Macro recording & playback
- And much more...

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
├── main.rs              # Entry point
├── lib.rs               # Library crate
├── editor/
│   ├── buffer.rs        # Rope-based text buffer with undo/redo
│   ├── cursor.rs        # Cursor and selection state
│   ├── document.rs      # Document model (buffer + cursor + metadata)
│   └── tab_manager.rs   # Multi-tab management
├── io/
│   ├── file_io.rs       # File reading/writing with encoding support
│   └── recent_files.rs  # Recent file history persistence
└── ui/
    ├── app.rs           # Main egui application
    └── editor_widget.rs # Custom editor rendering widget
```

## License

MIT
