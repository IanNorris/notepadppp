# Notepad+++ Feature List

A comprehensive feature list for Notepad+++, a from-scratch Rust text editor targeting Windows (primary), Linux, and macOS.

---

## Phase 1: Core Text Editor (Skeleton)

### 1.1 Basic Text Editing
- [x] Open, create, save, save-as, close files
- [x] Cut, copy, paste, undo/redo (unlimited history)
- [x] Text selection (character, word, line, all)
- [x] Drag and drop text
- [x] Word wrap toggle
- [x] Line numbering in gutter
- [x] Cursor position display (line, column, selection count) in status bar
- [x] Read-only mode toggle

### 1.2 Tabbed Multi-Document Interface
- [x] Multiple files open in tabs
- [x] Tab context menu (close, close all, close others, close to the right)
- [x] Tab reordering via drag and drop
- [x] New untitled tabs
- [x] Modified indicator on tabs (dot/asterisk)
- [x] Tab scrolling when many tabs open
- [x] Double-click tab to close
- [x] Middle-click tab to close

### 1.3 Basic File Operations
- [x] Recent file history
- [x] Reload from disk
- [x] File status auto-detection (external modifications prompt)
- [x] Auto-save / crash recovery
- [x] Open containing folder (in file manager / terminal)

### 1.4 Native Windows UI
- [ ] Win32 native window (no web rendering)
- [x] Menu bar with standard menus (File, Edit, Search, View, Encoding, Language, Settings, Window, Help)
- [x] Toolbar with common actions
- [x] Status bar (encoding, line ending, cursor position, file size, language)
- [x] Context menu on right-click

---

## Phase 2: Multi-View & Window Management

### 2.1 Split View / Multi-Panel Editing
- [x] Split editor into two panels (horizontal/vertical)
- [x] Clone document to other view (same file in two panels)
- [ ] Move tab between views
- [x] Independent scrolling per panel

### 2.2 Window Management
- [ ] Multiple independent windows
- [ ] Always on top toggle
- [x] Full screen mode
- [ ] Minimize to system tray
- [x] Window position/size persistence

### 2.3 Session Management
- [x] Save session (all open files, positions, bookmarks)
- [x] Restore session on startup
- [ ] Named sessions
- [x] Auto-session (restore last session on launch)

---

## Phase 3: Search & Replace

### 3.1 Find & Replace Dialog
- [x] Find (Ctrl+F)
- [x] Replace (Ctrl+H)
- [x] Options: match case, whole word, wrap around
- [x] Search modes: normal, extended (escape sequences), regex
- [x] Count occurrences
- [x] Find next / find previous (F3 / Shift+F3)
- [x] Incremental search (find as you type)
- [x] Highlight all matches in document

### 3.2 Find in Files
- [x] Search across files in directory/subdirectories
- [x] File type filters (e.g., *.rs, *.txt)
- [x] Replace in files
- [x] Results panel with file, line number, matching text
- [x] Click result to jump to file and line

### 3.3 Regular Expressions
- [x] Full regex support (PCRE-compatible)
- [ ] Lookaheads and lookbehinds
- [x] Capture groups and backreferences in replace
- [ ] Multiline matching
- [x] Named groups

### 3.4 Mark & Bookmarks
- [x] Mark all matches (persistent highlighting)
- [x] Bookmark lines matching search
- [x] Navigate between bookmarks (F2 / Shift+F2)
- [x] Copy/cut all bookmarked lines
- [x] Remove all bookmarked/unbookmarked lines
- [x] Toggle bookmark on current line

### 3.5 Search Results History Panel
- [x] Panel showing all matching lines from searches
- [x] Clickable results to jump to location
- [x] Search history (previous searches re-runnable)
- [x] Collapsible results per search

---

## Phase 4: Syntax Highlighting

### 4.1 Built-in Language Support
Syntax highlighting for 80+ languages (via syntect) including:
- [x] C, C++, C#, Objective-C
- [x] Java, Kotlin, Scala
- [x] Python, Ruby, Perl, PHP
- [x] JavaScript, TypeScript, CoffeeScript
- [x] Rust, Go, Swift, Zig
- [x] HTML, XML, XHTML
- [x] CSS, SCSS, LESS
- [x] JSON, YAML, TOML, INI
- [x] SQL, PowerShell, Bash/Shell
- [x] Lua, R, MATLAB
- [x] Markdown, LaTeX
- [x] Assembly (x86, ARM)
- [x] Dockerfile, Makefile
- [x] Diff/Patch files
- [x] And more...

### 4.2 Code Folding
- [x] Fold/unfold code blocks
- [x] Fold all / unfold all
- [x] Fold level (1-9)
- [x] Fold margin with +/- indicators

### 4.3 Bracket Matching
- [x] Matching bracket highlight

### 4.4 User-Defined Languages
- [ ] Custom language definition (keywords, operators, comments, folding)
- [ ] Import/export language definitions
- [ ] Syntax highlighting customization per language

---

## Phase 5: Advanced Editing

### 5.1 Column Mode Editing
- [x] Rectangular selection (Alt+Shift+Arrow)
- [x] Column insert (type into column selection)
- [x] Column cut/copy/paste

### 5.2 Multi-Cursor / Multi-Selection
- [x] Ctrl+click to add cursors
- [x] Ctrl+D to select next occurrence
- [x] Select all occurrences
- [x] Type at all cursors simultaneously

### 5.3 Line Operations
- [x] Duplicate line (Ctrl+D)
- [x] Delete line (Ctrl+Shift+K)
- [x] Move line up/down (Ctrl+Shift+Up/Down)
- [x] Join lines
- [x] Split lines
- [x] Sort lines ascending/descending
- [x] Sort lines case-insensitive
- [x] Sort lines numerically
- [ ] Sort lines by column
- [x] Remove duplicate lines
- [x] Remove empty lines
- [x] Remove empty lines (containing blank characters)
- [x] Insert blank line above/below
- [x] Reverse line order

### 5.4 Text Transformations
- [x] Convert case: UPPER, lower, Title, Sentence, iNVERSE
- [x] Trim trailing whitespace
- [x] Trim leading whitespace
- [x] Trim both
- [x] Tab to spaces / spaces to tabs conversion
- [x] Comment/uncomment line (language-aware)
- [ ] Block comment/uncomment
- [x] Auto-indent

### 5.5 Auto-Completion
- [ ] Word completion (from current document)
- [ ] Function/parameter hints (for supported languages)
- [x] Bracket/quote auto-close
- [x] Matching bracket highlight

### 5.6 Macro Recording & Playback
- [x] Record macro (keystrokes)
- [x] Play macro
- [x] Play macro multiple times / until end of file


---

## Phase 6: Encoding & Line Endings

### 6.1 Encoding Support
- [x] UTF-8 (with and without BOM)
- [ ] UTF-16 LE/BE (with and without BOM)
- [x] ANSI / Windows codepages
- [ ] ISO-8859 family
- [x] Encoding display in status bar
- [x] Convert between encodings (menu action)
- [ ] Set default encoding for new files

### 6.2 Line Ending (EOL) Management
- [x] Windows (CRLF)
- [x] Unix/Linux (LF)
- [x] Old Mac (CR)
- [x] EOL display in status bar
- [x] Convert between line endings
- [ ] Show/hide EOL characters
- [ ] Set default EOL for new files
- [x] Mixed EOL detection and notification

---

## Phase 7: Navigation & Panels

### 7.1 Document Map (Mini-map)
- [x] Zoomed-out view of entire document
- [x] Click to navigate
- [x] Highlight visible region

### 7.2 Function List Panel
- [x] List all functions/methods/classes in current file
- [x] Click to jump to definition
- [x] Language-aware parsing

### 7.3 Folder as Workspace
- [ ] Tree view of project directory
- [ ] Open files from tree
- [ ] File operations (rename, delete, new file/folder)
- [ ] Filter/search within workspace

### 7.4 File Explorer Panel
- [ ] Browse filesystem
- [ ] Favorites/bookmarks for directories

### 7.5 Navigation Commands
- [x] Go to Line (Ctrl+G)
- [x] Command Palette (Ctrl+P)

---

## Phase 8: Appearance & Theming

### 8.1 Color Themes
- [x] Dark theme built-in
- [x] Light theme built-in
- [ ] Simple color customization (background, foreground, selection, caret)
- [ ] Per-language syntax colors
- [ ] Import/export themes

### 8.2 Display Options
- [x] Show/hide whitespace characters
- [ ] Show/hide line endings
- [ ] Show/hide indent guides
- [x] Zoom in/out (Ctrl+scroll wheel)
- [ ] Font family and size selection
- [ ] Line spacing adjustment

---

## Phase 9: Built-in Tools

### 9.1 Markdown Viewer
- [x] Live preview panel for Markdown files
- [ ] Synchronized scrolling with editor
- [ ] Basic Markdown rendering (headings, bold, italic, links, images, code blocks, tables, lists)
- [ ] Export to HTML

### 9.2 JSON Tools
- [x] Format/pretty-print JSON
- [x] Compact/minify JSON
- [x] Validate JSON (show errors)
- [ ] JSON tree viewer panel
- [ ] JSON path navigation
- [x] Sort JSON keys

### 9.3 CSV Viewer/Editor
- [x] Tabular view for CSV files
- [ ] Column alignment/colorization
- [x] Sort by column
- [x] Filter rows
- [x] Add/remove columns
- [ ] CSV validation
- [ ] Toggle between text and table view

### 9.4 File Comparison (Diff)
- [x] Side-by-side comparison of two files
- [ ] Color-coded diff (added, removed, changed)
- [ ] Navigate between differences
- [ ] Merge changes between files

### 9.5 Hex Viewer/Editor
- [x] View file in hexadecimal
- [ ] Edit hex values
- [x] ASCII sidebar
- [ ] Go to offset
- [x] Toggle between text and hex view (Ctrl+Shift+H)

### 9.6 Binary Disassembler
- [ ] Basic disassembly view for binary/executable files (via Capstone)
- [ ] x86/x86-64 and ARM/AArch64 instruction decoding
- [ ] Symbol loading and display (function names, labels)
- [ ] DIA (Debug Interface Access) symbol support on Windows
- [ ] Navigate to address / offset
- [ ] Disassembly alongside hex view

### 9.7 Quick Hash & Encoding (Selection Tools)
- [ ] SHA-256 hash of selection
- [ ] SHA-1 hash of selection
- [ ] MD5 hash of selection
- [ ] CRC32 of selection
- [ ] Base64 encode/decode selection
- [ ] URL encode/decode selection
- [ ] Hex encode/decode selection
- [ ] HTML entity encode/decode selection
- [ ] Right-click context menu for quick hash/encode actions
- [ ] Results shown inline or in status bar / popup

---

## Phase 10: Advanced Features

### 10.1 Printing & Export
- [ ] Print with syntax highlighting
- [x] Export as HTML (with highlighting)
- [x] Export as RTF
- [ ] Line numbers in print output

### 10.2 MIME / Encoding Tools
- [x] Base64 encode/decode
- [x] URL encode/decode
- [ ] Quoted-Printable encode/decode

- [x] HTML entity encode/decode

### 10.3 XML Tools
- [ ] Pretty-print XML
- [ ] Validate XML
- [ ] XPath evaluation
- [ ] XML syntax checking
- [ ] XSLT transformation

### 10.4 Spell Checking
- [ ] Spell check for comments and strings
- [ ] Multiple dictionary support
- [ ] Add to dictionary
- [ ] Ignore word

### 10.5 Code Snippets
- [ ] Snippet library per language
- [ ] Insert snippet with tab trigger
- [ ] Snippet placeholders/variables
- [ ] User-defined snippets

---

## Phase 11: System Integration

### 11.1 Windows Shell Integration
- [x] "Open with Notepad+++" context menu entry
- [x] File extension associations
- [x] Open from command line with arguments
- [x] Single instance mode (reuse running window)
- [x] Command line options: goto line, encoding, language, etc.
- [x] NSIS installer with shell integration
- [x] PowerShell registration scripts
- [x] Registry (.reg) file generation
- [x] Linux .desktop file and xdg-mime integration
- [x] macOS Info.plist document types generation

### 11.2 Run / Execute
- [ ] Run current file in associated program
- [ ] Run commands from within editor
- [ ] Capture command output in panel

---

## Phase 12: Performance & Large Files

### 12.1 Large File Handling
- [x] Streaming/chunked file loading (files > 1GB)
- [ ] Memory-mapped file reading
- [ ] Virtual scrolling (don't render what's off-screen)
- [x] Warning dialog for large files
- [x] Auto-disable expensive features for large files

### 12.2 Performance Optimizations
- [x] Rope data structure for text storage
- [ ] Incremental syntax highlighting
- [ ] Lazy line rendering
- [x] Efficient undo/redo (operation-based, not snapshot)
- [ ] Multi-threaded search

---

## Phase 13: Settings & Configuration

### 13.1 Preferences
- [x] Preferences dialog
- [ ] General settings (language, toolbar, tabs)
- [ ] Editor settings (font, colors, behavior)
- [ ] Search settings (defaults)
- [ ] File association settings
- [ ] Backup/auto-save settings
- [x] Keyboard shortcut customization
- [x] Configurable keybindings stored as JSON
- [ ] Plugin settings

### 13.2 Portable Mode
- [ ] All settings stored alongside executable
- [ ] No registry usage
- [ ] USB-drive portable deployment

---

## Cross-Platform Considerations

| Feature | Windows | Linux | macOS |
|---------|---------|-------|-------|
| Native UI | Win32 API | GTK4 or Qt | Cocoa/AppKit |
| Shell integration | Context menu, file associations | .desktop files, xdg-mime | Launch Services, Finder integration |
| File paths | UNC paths, drive letters | POSIX paths | POSIX paths |
| Line endings default | CRLF | LF | LF |
| Keyboard shortcuts | Ctrl-based | Ctrl-based | Cmd-based |

---

## Implementation Priority (Build Order)

1. **Skeleton** — Phase 1: Basic editor, tabs, file I/O, native UI
2. **Windows** — Phase 2: Split view, sessions, window management
3. **Search** — Phase 3: Find/replace, regex, find in files, bookmarks
4. **Highlighting** — Phase 4: Syntax highlighting, code folding
5. **Editing** — Phase 5: Column mode, multi-cursor, line ops, macros
6. **Encoding** — Phase 6: Encoding/EOL conversion
7. **Navigation** — Phase 7: Document map, function list, workspace
8. **Theming** — Phase 8: Themes, display options
9. **Tools** — Phase 9: Markdown, JSON, CSV, diff, hex
10. **Advanced** — Phase 10: Print, MIME tools, XML, spell check, snippets
11. **Integration** — Phase 11: Shell integration, run commands
12. **Performance** — Phase 12: Large file handling (woven throughout)
13. **Settings** — Phase 13: Preferences, portable mode
