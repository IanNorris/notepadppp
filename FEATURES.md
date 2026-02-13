# Notepad+++ Feature List

A comprehensive feature list based on research of Notepad++ and its most popular plugins. Notepad+++ is a from-scratch Rust reimplementation targeting Windows (primary), Linux, and macOS.

---

## Phase 1: Core Text Editor (Skeleton)

### 1.1 Basic Text Editing
- [ ] Open, create, save, save-as, close files
- [ ] Cut, copy, paste, undo/redo (unlimited history)
- [ ] Text selection (character, word, line, all)
- [ ] Drag and drop text
- [ ] Word wrap toggle
- [ ] Line numbering in gutter
- [ ] Cursor position display (line, column, selection count) in status bar
- [ ] Read-only mode toggle

### 1.2 Tabbed Multi-Document Interface
- [ ] Multiple files open in tabs
- [ ] Tab context menu (close, close all, close others, close to the right)
- [ ] Tab reordering via drag and drop
- [ ] New untitled tabs
- [ ] Modified indicator on tabs (dot/asterisk)
- [ ] Tab scrolling when many tabs open
- [ ] Double-click tab to close

### 1.3 Basic File Operations
- [ ] Recent file history
- [ ] Reload from disk
- [ ] File status auto-detection (external modifications prompt)
- [ ] Auto-save / crash recovery
- [ ] Open containing folder (in file manager / terminal)

### 1.4 Native Windows UI
- [ ] Win32 native window (no web rendering)
- [ ] Menu bar with standard menus (File, Edit, Search, View, Encoding, Language, Settings, Window, Help)
- [ ] Toolbar with common actions
- [ ] Status bar (encoding, line ending, cursor position, file size, language)
- [ ] Context menu on right-click

---

## Phase 2: Multi-View & Window Management

### 2.1 Split View / Multi-Panel Editing
- [ ] Split editor into two panels (horizontal/vertical)
- [ ] Clone document to other view (same file in two panels)
- [ ] Move tab between views
- [ ] Independent scrolling per panel

### 2.2 Window Management
- [ ] Multiple independent windows
- [ ] Always on top toggle
- [ ] Full screen mode
- [ ] Minimize to system tray
- [ ] Window position/size persistence

### 2.3 Session Management
- [ ] Save session (all open files, positions, bookmarks)
- [ ] Restore session on startup
- [ ] Named sessions
- [ ] Auto-session (restore last session on launch)

---

## Phase 3: Search & Replace

### 3.1 Find & Replace Dialog
- [ ] Find (Ctrl+F)
- [ ] Replace (Ctrl+H)
- [ ] Options: match case, whole word, wrap around
- [ ] Search modes: normal, extended (escape sequences), regex
- [ ] Count occurrences
- [ ] Find next / find previous (F3 / Shift+F3)
- [ ] Incremental search (find as you type)
- [ ] Highlight all matches in document

### 3.2 Find in Files
- [ ] Search across files in directory/subdirectories
- [ ] File type filters (e.g., *.rs, *.txt)
- [ ] Replace in files
- [ ] Results panel with file, line number, matching text
- [ ] Click result to jump to file and line

### 3.3 Regular Expressions
- [ ] Full regex support (PCRE-compatible)
- [ ] Lookaheads and lookbehinds
- [ ] Capture groups and backreferences in replace
- [ ] Multiline matching
- [ ] Named groups

### 3.4 Mark & Bookmarks
- [ ] Mark all matches (persistent highlighting)
- [ ] Bookmark lines matching search
- [ ] Navigate between bookmarks (F2 / Shift+F2)
- [ ] Copy/cut all bookmarked lines
- [ ] Remove all bookmarked/unbookmarked lines
- [ ] Toggle bookmark on current line

### 3.5 Search Results History Panel
- [ ] Panel showing all matching lines from searches
- [ ] Clickable results to jump to location
- [ ] Search history (previous searches re-runnable)
- [ ] Collapsible results per search

---

## Phase 4: Syntax Highlighting

### 4.1 Built-in Language Support
Syntax highlighting and code folding for 80+ languages including:
- [ ] C, C++, C#, Objective-C
- [ ] Java, Kotlin, Scala
- [ ] Python, Ruby, Perl, PHP
- [ ] JavaScript, TypeScript, CoffeeScript
- [ ] Rust, Go, Swift, Zig
- [ ] HTML, XML, XHTML
- [ ] CSS, SCSS, LESS
- [ ] JSON, YAML, TOML, INI
- [ ] SQL, PowerShell, Bash/Shell
- [ ] Lua, R, MATLAB
- [ ] Markdown, LaTeX
- [ ] Assembly (x86, ARM)
- [ ] Dockerfile, Makefile
- [ ] Diff/Patch files
- [ ] And more...

### 4.2 Code Folding
- [ ] Fold/unfold code blocks
- [ ] Fold all / unfold all
- [ ] Fold level (1-9)
- [ ] Fold margin with +/- indicators

### 4.3 User-Defined Languages
- [ ] Custom language definition (keywords, operators, comments, folding)
- [ ] Import/export language definitions
- [ ] Syntax highlighting customization per language

---

## Phase 5: Advanced Editing

### 5.1 Column Mode Editing
- [ ] Rectangular selection (Alt+drag)
- [ ] Column insert (type into column selection)
- [ ] Column cut/copy/paste

### 5.2 Multi-Cursor / Multi-Selection
- [ ] Ctrl+click to add cursors
- [ ] Ctrl+D to select next occurrence
- [ ] Select all occurrences
- [ ] Type at all cursors simultaneously

### 5.3 Line Operations
- [ ] Duplicate line (Ctrl+D)
- [ ] Delete line (Ctrl+Shift+K)
- [ ] Move line up/down (Ctrl+Shift+Up/Down)
- [ ] Join lines
- [ ] Split lines
- [ ] Sort lines ascending/descending
- [ ] Sort lines case-insensitive
- [ ] Sort lines numerically
- [ ] Sort lines by column
- [ ] Remove duplicate lines
- [ ] Remove empty lines
- [ ] Remove empty lines (containing blank characters)
- [ ] Insert blank line above/below
- [ ] Reverse line order

### 5.4 Text Transformations
- [ ] Convert case: UPPER, lower, Title, Sentence, iNVERSE
- [ ] Trim trailing whitespace
- [ ] Trim leading whitespace
- [ ] Trim both
- [ ] Tab to spaces / spaces to tabs conversion
- [ ] Comment/uncomment line (language-aware)
- [ ] Block comment/uncomment
- [ ] Auto-indent

### 5.5 Auto-Completion
- [ ] Word completion (from current document)
- [ ] Function/parameter hints (for supported languages)
- [ ] Bracket/quote auto-close
- [ ] Matching bracket highlight

### 5.6 Macro Recording & Playback
- [ ] Record macro (keystrokes)
- [ ] Play macro
- [ ] Play macro multiple times / until end of file
- [ ] Save macros with keyboard shortcuts
- [ ] Edit/manage saved macros

---

## Phase 6: Encoding & Line Endings

### 6.1 Encoding Support
- [ ] UTF-8 (with and without BOM)
- [ ] UTF-16 LE/BE (with and without BOM)
- [ ] ANSI / Windows codepages
- [ ] ISO-8859 family
- [ ] Encoding display in status bar
- [ ] Convert between encodings (menu action)
- [ ] Set default encoding for new files

### 6.2 Line Ending (EOL) Management
- [ ] Windows (CRLF)
- [ ] Unix/Linux (LF)
- [ ] Old Mac (CR)
- [ ] EOL display in status bar
- [ ] Convert between line endings
- [ ] Show/hide EOL characters
- [ ] Set default EOL for new files
- [ ] Mixed EOL detection and notification

---

## Phase 7: Navigation & Panels

### 7.1 Document Map (Mini-map)
- [ ] Zoomed-out view of entire document
- [ ] Click to navigate
- [ ] Highlight visible region

### 7.2 Function List Panel
- [ ] List all functions/methods/classes in current file
- [ ] Click to jump to definition
- [ ] Language-aware parsing

### 7.3 Folder as Workspace
- [ ] Tree view of project directory
- [ ] Open files from tree
- [ ] File operations (rename, delete, new file/folder)
- [ ] Filter/search within workspace

### 7.4 File Explorer Panel
- [ ] Browse filesystem
- [ ] Favorites/bookmarks for directories

---

## Phase 8: Appearance & Theming

### 8.1 Color Themes
- [ ] Light and dark themes built-in
- [ ] Simple color customization (background, foreground, selection, caret)
- [ ] Per-language syntax colors
- [ ] Import/export themes

### 8.2 Display Options
- [ ] Show/hide whitespace characters
- [ ] Show/hide line endings
- [ ] Show/hide indent guides
- [ ] Zoom in/out (Ctrl+scroll wheel)
- [ ] Font family and size selection
- [ ] Line spacing adjustment

---

## Phase 9: Built-in Tools

### 9.1 Markdown Viewer
- [ ] Live preview panel for Markdown files
- [ ] Synchronized scrolling with editor
- [ ] Basic Markdown rendering (headings, bold, italic, links, images, code blocks, tables, lists)
- [ ] Export to HTML

### 9.2 JSON Tools
- [ ] Format/pretty-print JSON
- [ ] Compact/minify JSON
- [ ] Validate JSON (show errors)
- [ ] JSON tree viewer panel
- [ ] JSON path navigation
- [ ] Sort JSON keys

### 9.3 CSV Viewer/Editor
- [ ] Tabular view for CSV files
- [ ] Column alignment/colorization
- [ ] Sort by column
- [ ] Filter rows
- [ ] Add/remove columns
- [ ] CSV validation
- [ ] Toggle between text and table view

### 9.4 File Comparison (Diff)
- [ ] Side-by-side comparison of two files
- [ ] Color-coded diff (added, removed, changed)
- [ ] Navigate between differences
- [ ] Merge changes between files

### 9.5 Hex Viewer/Editor
- [ ] View file in hexadecimal
- [ ] Edit hex values
- [ ] ASCII sidebar
- [ ] Go to offset

---

## Phase 10: Advanced Features

### 10.1 Printing & Export
- [ ] Print with syntax highlighting
- [ ] Export as HTML (with highlighting)
- [ ] Export as RTF
- [ ] Line numbers in print output

### 10.2 MIME / Encoding Tools
- [ ] Base64 encode/decode
- [ ] URL encode/decode
- [ ] Quoted-Printable encode/decode
- [ ] SAML decode
- [ ] HTML entity encode/decode

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
- [ ] "Open with Notepad+++" context menu entry
- [ ] File extension associations
- [ ] Open from command line with arguments
- [ ] Single instance mode (reuse running window)
- [ ] Command line options: goto line, encoding, language, etc.

### 11.2 Run / Execute
- [ ] Run current file in associated program
- [ ] Run commands from within editor
- [ ] Capture command output in panel

### 11.3 FTP/SFTP (Optional)
- [ ] Connect to remote servers
- [ ] Edit remote files
- [ ] File transfer

---

## Phase 12: Performance & Large Files

### 12.1 Large File Handling
- [ ] Streaming/chunked file loading (files > 1GB)
- [ ] Memory-mapped file reading
- [ ] Virtual scrolling (don't render what's off-screen)
- [ ] Background file loading with progress

### 12.2 Performance Optimizations
- [ ] Rope or piece-table data structure for text storage
- [ ] Incremental syntax highlighting
- [ ] Lazy line rendering
- [ ] Efficient undo/redo (operation-based, not snapshot)
- [ ] Multi-threaded search

---

## Phase 13: Settings & Configuration

### 13.1 Preferences
- [ ] General settings (language, toolbar, tabs)
- [ ] Editor settings (font, colors, behavior)
- [ ] Search settings (defaults)
- [ ] File association settings
- [ ] Backup/auto-save settings
- [ ] Keyboard shortcut customization
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
