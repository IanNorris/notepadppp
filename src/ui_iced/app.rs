use iced::keyboard;
use iced::mouse;
use iced::widget::{button, column, container, mouse_area, opaque, row, scrollable, stack, text, text_editor, text_input, Space};
use iced::advanced::text::Wrapping;
use iced::{Element, Font, Length, Subscription, Task, Theme};

use crate::editor::document::{Document, Encoding, LineEnding};
use crate::editor::folding::FoldManager;
use crate::editor::macros::MacroRecorder;
use crate::editor::tab_manager::TabManager;
use crate::editor::marks::MarkManager;
use crate::search::{SearchEngine, SearchMatch, SearchMode};
use crate::search::find_in_files::FileSearchResult;
use crate::platform::cli::CliArgs;
use crate::platform::single_instance::InstanceListener;

use super::search_results_panel::SearchResultsManager;

use super::highlighter::{SyntectHighlighter, SyntectSettings};
use super::menu_bar;
use super::theme::{AppColors, AppTheme};

/// CLI arguments passed from main() to the app via OnceLock.
pub static CLI_ARGS: std::sync::OnceLock<CliArgs> = std::sync::OnceLock::new();

/// Instance listener for single-instance mode, passed from main().
pub static INSTANCE_LISTENER: std::sync::OnceLock<std::sync::Mutex<Option<InstanceListener>>> =
    std::sync::OnceLock::new();

#[derive(Debug, Clone, PartialEq)]
pub enum SplitMode {
    None,
    Horizontal, // side by side
    Vertical,   // top/bottom
}

/// State for tab drag-reordering.
#[derive(Debug, Clone)]
pub struct TabDragState {
    pub from_index: usize,
    pub is_split: bool,
    pub target_index: Option<usize>,
}

/// Per-tab state that pairs an iced text_editor::Content with our Document index.
pub struct TabContent {
    pub content: text_editor::Content,
    pub disasm_state: Option<crate::tools::disasm::Disassembler>,
    pub disasm_offset: usize,
    pub show_disasm: bool,
}

impl TabContent {
    fn new() -> Self {
        Self {
            content: text_editor::Content::new(),
            disasm_state: None,
            disasm_offset: 0,
            show_disasm: false,
        }
    }

    fn with_text(s: &str) -> Self {
        Self {
            content: text_editor::Content::with_text(s),
            disasm_state: None,
            disasm_offset: 0,
            show_disasm: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    EditorAction(text_editor::Action),
    NewTab,
    OpenFile,
    FileOpened(Result<(String, std::path::PathBuf), String>),
    FilesOpened(Vec<Result<(String, std::path::PathBuf), String>>),
    Save,
    SaveAs,
    FileSaved(Result<std::path::PathBuf, String>),
    CloseTab(usize),
    SelectTab(usize),
    Undo,
    Redo,

    // Menu state
    MenuToggle(String),
    MenuSwitch(String),
    MenuClose,
    SubMenuToggle(String),

    // File
    SaveAll,
    CloseAll,
    SaveSession,
    LoadSession,
    ToggleAutoRestore,
    ExportHtml,
    ExportRtf,
    Exit,

    // Edit
    SelectAll,
    ToggleComment,
    // Line operations
    DuplicateLine,
    DeleteLine,
    MoveLineUp,
    MoveLineDown,
    SortAsc,
    SortDesc,
    RemoveEmpty,
    RemoveDuplicates,
    TrimTrailing,
    JoinLines,
    SplitLine,
    InsertAbove,
    InsertBelow,
    ReverseLines,
    SortCaseInsensitive,
    SortNumeric,
    TrimLeading,
    TrimBoth,
    // Case
    UpperCase,
    LowerCase,
    TitleCase,
    SentenceCase,
    InverseCase,
    // Encoding / line ending
    SetEncoding(Encoding),
    SetLineEnding(LineEnding),

    // Search
    ShowFind,
    ShowReplace,
    ShowFindInFiles,
    SelectAllOccurrences,
    GotoLine,
    ToggleBookmark,
    NextBookmark,
    PrevBookmark,
    ClearBookmarks,
    CopyBookmarkedLines,
    RemoveBookmarkedLines,
    RemoveUnbookmarkedLines,

    // Edit
    ToggleReadOnly,

    // View
    ToggleWordWrap,
    ToggleLineNumbers,
    ToggleWhitespace,
    ToggleStatusBar,
    ToggleMinimap,
    ToggleFunctionList,
    ToggleFullScreen,
    SplitHorizontal,
    SplitVertical,
    RemoveSplit,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ToggleToolbar,
    ToggleFold,
    ToggleFoldAt(usize),
    FoldAll,
    UnfoldAll,
    FoldLevel(usize),

    // Language
    SetLanguage(String),

    // Tools
    JsonFormat,
    JsonCompact,
    JsonValidate,
    JsonSortKeys,
    ToggleMarkdownPreview,
    ToggleCsvViewer,
    MimeBase64Encode,
    MimeBase64Decode,
    MimeUrlEncode,
    MimeUrlDecode,
    MimeHtmlEncode,
    MimeHtmlDecode,
    MimeHexEncode,
    MimeHexDecode,
    CompareFiles,
    ToggleHexViewer,
    ToggleDisasm,
    DisasmGotoAddress(String),
    DisasmGotoAddressInput(String),
    DisasmGotoSymbol(String),
    DisasmGotoSymbolInput(String),
    DisasmSetBaseAddress(String),
    DisasmBaseAddressInput(String),
    DisasmSetArch(crate::tools::disasm::DisasmArch),
    DisasmScroll(i32),
    DisasmToggleAddress,
    DisasmToggleBytes,
    DisasmToggleResolved,
    DisasmToggleRawComment,
    DisasmToggleArrows,
    DisasmContextMenu,
    DisasmCloseContextMenu,
    DisasmToggleSymBrowser,
    DisasmSymFilterInput(String),
    DisasmGotoSymFromBrowser(u64),
    DisasmNavigateToAddress(u64),
    DisasmToggleEditMode,
    DisasmEditHexInput(String),
    DisasmEditHexCommit(usize),
    DisasmCopySelection,
    DisasmCopyText(String),
    HashSha256,
    HashSha1,
    HashMd5,
    HashCrc32,

    // Editor context menu
    EditorContextMenu,
    CloseEditorContextMenu,

    // Split pane
    SplitEditorAction(text_editor::Action),
    SplitNewTab,
    SplitSelectTab(usize),
    SplitCloseTab(usize),
    SetActivePane(usize),

    // Tab management across panes
    NewTabInActivePane,
    FileDropped(std::path::PathBuf),
    MoveTabToSplit(usize),
    MoveTabFromSplit(usize),

    // Tab context menu
    TabContextMenu(usize, bool), // (tab_index, is_split_pane)
    TabContextClose(usize, bool),
    TabContextCloseOthers(usize, bool),
    TabContextCloseAll(bool),
    TabContextCloseToLeft(usize, bool),
    TabContextCloseToRight(usize, bool),
    TabContextCloseUnmodified(bool),
    TabContextOpenInExplorer(usize, bool),
    TabContextOpenTerminalHere(usize, bool),
    TabContextCopyFilename(usize, bool),
    TabContextCopyFullPath(usize, bool),
    TabContextRename(usize, bool),
    TabContextRenameInput(String),
    TabContextRenameConfirm,
    TabContextRenameCancel,
    TabContextMoveToOtherPane(usize, bool),
    TabContextCloneToOtherView(usize, bool),
    CloseTabContextMenu,

    // Middle-click tab close
    MiddleClickTab(usize, bool), // (tab_index, is_split_pane)

    // Tab drag reordering
    TabDragStart(usize, bool),        // (tab_index, is_split_pane)
    TabDragOver(usize, bool),         // (target_index, is_split_pane)
    TabDragEnd,
    TabDragCancel,

    // Macro
    ToggleMacroRecording,
    PlayLastMacro,
    PlayMacroMultiple,

    // Settings
    ShowPreferences,
    ShowKeybindings,
    SavePreferences,
    CancelPreferences,
    CloseKeybindingsDialog,
    PrefFontSizeIncrease,
    PrefFontSizeDecrease,
    PrefTabSizeChanged(String),
    PrefSetTheme(String),
    PrefToggleAutoSave(bool),
    PrefToggleWordWrap(bool),
    PrefToggleLineNumbers(bool),
    PrefToggleWhitespace(bool),
    PrefFontFamilyChanged(String),
    PrefLineSpacingChanged(String),
    PrefColorBgChanged(String),
    PrefColorFgChanged(String),
    PrefColorSelChanged(String),
    PrefColorCaretChanged(String),
    ImportTheme,
    ExportTheme,
    ThemeImported(Result<(String, std::path::PathBuf), String>),
    ThemeExported(Result<std::path::PathBuf, String>),

    // View toggles
    ToggleIndentGuides,
    ToggleLineEndings,
    ShowAbout,
    CloseAbout,

    // Theme
    SetTheme(String),

    // Search panel
    FindQueryChanged(String),
    ReplaceTextChanged(String),
    ToggleCaseSensitive,
    ToggleWholeWord,
    ToggleRegex,
    FindNext,
    FindPrev,
    ReplaceNext,
    ReplaceAll,
    CloseSearch,
    ClickSearchResult(usize),

    // Go to line
    GotoLineInputChanged(String),
    GotoLineConfirm,
    GotoLineClose,

    // Async results
    SessionFileChosen(Result<std::path::PathBuf, String>),
    ExportSaved(Result<(), String>),
    CompareFileLoaded(Result<String, String>),

    // Keyboard
    EscapePressed,

    // Drag floating panels
    DragFindStart,
    DragFindMove(iced::Point),
    DragFindEnd,

    // Panel messages
    GotoSymbol(usize),
    CsvSortColumn(usize),
    CsvToggleHeaders,

    // Find in Files
    FifQueryChanged(String),
    FifDirectoryChanged(String),
    FifFileFilterChanged(String),
    FifToggleRecursive,
    FifToggleCaseSensitive,
    FifToggleRegex,
    FifSearch,
    FifSearchComplete(Result<Vec<FileSearchResult>, String>),
    FifClickResult(std::path::PathBuf, usize),
    CloseFindInFiles,
    DragFifStart,
    DragFifMove(iced::Point),
    DragFifEnd,

    // Mark All
    MarkAll,
    ClearAllMarks,

    // Find All (search current document, show in results panel)
    FindAll,

    // Bookmark from search
    BookmarkMatchingLines,

    // Cut bookmarked lines
    CutBookmarkedLines,

    // Search results panel
    ToggleSearchResultsPanel,
    ClearSearchResults,
    ToggleSearchResultCollapse(usize),
    ClickSearchResultEntry(usize, usize),

    // Single instance
    CheckInstance(iced::time::Instant),
}

pub struct NotepadIced {
    pub tab_manager: TabManager,
    pub tab_contents: Vec<TabContent>,

    // Menu state
    pub active_menu: Option<String>,
    pub expanded_submenus: std::collections::HashSet<String>,

    // View toggles
    pub word_wrap: bool,
    pub show_line_numbers: bool,
    pub show_whitespace: bool,
    pub show_status_bar: bool,
    pub show_toolbar: bool,
    pub show_minimap: bool,
    pub show_function_list: bool,
    pub show_markdown_preview: bool,
    pub show_csv_viewer: bool,
    pub show_hex_viewer: bool,

    // CSV viewer state
    pub csv_sort_column: Option<usize>,
    pub csv_sort_ascending: bool,
    pub csv_has_headers: bool,

    // Session
    pub auto_restore_session: bool,

    // Search
    pub show_find: bool,
    pub show_replace: bool,
    pub search_query: String,
    pub replace_text: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub use_regex: bool,
    pub search_matches: Vec<SearchMatch>,
    pub current_match_index: Option<usize>,

    // Go to line
    pub show_goto_line: bool,
    pub goto_line_input: String,

    // About
    pub show_about: bool,

    // Preferences dialog
    pub show_preferences: bool,
    pub pref_font_size: f32,
    pub pref_tab_size: String,
    pub pref_theme: String,
    pub pref_auto_save: bool,
    pub pref_word_wrap: bool,
    pub pref_show_line_numbers: bool,
    pub pref_show_whitespace: bool,
    pub pref_font_family: String,
    pub pref_line_spacing: String,
    pub pref_color_bg: String,
    pub pref_color_fg: String,
    pub pref_color_sel: String,
    pub pref_color_caret: String,

    // Display toggles
    pub show_indent_guides: bool,
    pub show_line_endings: bool,
    pub line_spacing: f32,
    pub font_family: String,
    pub color_caret: Option<String>,

    // Keybindings dialog
    pub show_keybindings: bool,

    // Floating panel positions (None = default position)
    pub find_panel_pos: Option<(f32, f32)>,
    pub dragging_find_panel: bool,
    pub drag_offset: (f32, f32),
    pub last_mouse_pos: iced::Point,

    // Zoom
    pub font_size: f32,

    // Macros
    pub macro_recorder: MacroRecorder,
    pub last_macro: Option<crate::editor::macros::Macro>,

    // Syntax highlighting
    pub file_extension: String,

    // Code folding
    pub fold_manager: FoldManager,

    // Find in Files
    pub show_find_in_files: bool,
    pub fif_query: String,
    pub fif_directory: String,
    pub fif_file_filter: String,
    pub fif_recursive: bool,
    pub fif_case_sensitive: bool,
    pub fif_use_regex: bool,
    pub fif_results: Vec<FileSearchResult>,
    pub fif_searching: bool,
    pub fif_panel_pos: Option<(f32, f32)>,
    pub dragging_fif_panel: bool,
    pub fif_drag_offset: (f32, f32),

    // Split view
    pub split_mode: SplitMode,
    pub split_tab_manager: TabManager,
    pub split_tab_contents: Vec<TabContent>,
    pub split_font_size: f32,
    pub split_file_extension: String,
    pub split_show_markdown_preview: bool,
    pub active_pane: usize, // 0 = primary, 1 = secondary

    // Theme
    pub theme: AppTheme,

    // Tab context menu: (tab_index, is_split_pane)
    pub tab_context_menu: Option<(usize, bool)>,

    // Tab rename state: (tab_index, is_split_pane, current_input)
    pub tab_rename: Option<(usize, bool, String)>,

    // Full screen mode
    pub is_fullscreen: bool,

    // Double-click tracking for tab close
    pub last_tab_click: Option<(usize, std::time::Instant)>,

    // Tab drag reordering state
    pub tab_drag: Option<TabDragState>,

    // Mark All (persistent highlighting)
    pub mark_manager: MarkManager,

    // Search results panel
    pub show_search_results_panel: bool,
    pub search_results_panel: SearchResultsManager,

    // Editor context menu (right-click on text area)
    pub editor_context_menu: bool,

    // Status bar message (transient feedback)
    pub status_message: Option<(String, std::time::Instant)>,

    // Disassembler
    pub show_disasm: bool,
    pub disasm_state: Option<crate::tools::disasm::Disassembler>,
    pub disasm_offset: usize,
    pub disasm_arch: crate::tools::disasm::DisasmArch,
    pub disasm_goto_addr: String,
    pub disasm_goto_sym: String,
    pub disasm_base_addr_input: String,
    pub disasm_show_address: bool,
    pub disasm_show_bytes: bool,
    pub disasm_show_resolved: bool,
    pub disasm_show_raw_comment: bool,
    pub disasm_show_arrows: bool,
    pub disasm_context_menu: bool,
    pub disasm_sym_browser: bool,
    pub disasm_sym_filter: String,
    pub disasm_edit_mode: bool,
    pub disasm_edit_offset: Option<usize>,
    pub disasm_edit_hex: String,
}

impl Default for NotepadIced {
    fn default() -> Self {
        let settings = crate::io::settings::AppSettings::load(
            &crate::io::settings::AppSettings::settings_path(),
        )
        .unwrap_or_default();
        let mut theme = AppColors::theme_by_name(&settings.theme);
        theme.apply_color_overrides(&settings);
        let mut state = Self {
            tab_manager: TabManager::new(),
            tab_contents: vec![TabContent::new()],
            active_menu: None,
            expanded_submenus: std::collections::HashSet::new(),
            word_wrap: settings.word_wrap,
            show_line_numbers: settings.show_line_numbers,
            show_whitespace: settings.show_whitespace,
            show_toolbar: true,
            show_status_bar: true,
            show_minimap: false,
            show_function_list: false,
            show_markdown_preview: false,
            show_csv_viewer: false,
            show_hex_viewer: false,
            csv_sort_column: None,
            csv_sort_ascending: true,
            csv_has_headers: true,
            auto_restore_session: false,
            show_find: false,
            show_replace: false,
            search_query: String::new(),
            replace_text: String::new(),
            case_sensitive: false,
            whole_word: false,
            use_regex: false,
            search_matches: Vec::new(),
            current_match_index: None,
            show_goto_line: false,
            goto_line_input: String::new(),
            show_about: false,
            show_preferences: false,
            pref_font_size: settings.font_size,
            pref_tab_size: settings.tab_size.to_string(),
            pref_theme: settings.theme.clone(),
            pref_auto_save: settings.auto_save,
            pref_word_wrap: settings.word_wrap,
            pref_show_line_numbers: settings.show_line_numbers,
            pref_show_whitespace: settings.show_whitespace,
            pref_font_family: settings.font_family.clone(),
            pref_line_spacing: format!("{:.1}", settings.line_spacing),
            pref_color_bg: settings.color_background.clone().unwrap_or_default(),
            pref_color_fg: settings.color_foreground.clone().unwrap_or_default(),
            pref_color_sel: settings.color_selection.clone().unwrap_or_default(),
            pref_color_caret: settings.color_caret.clone().unwrap_or_default(),
            show_indent_guides: settings.show_indent_guides,
            show_line_endings: settings.show_line_endings,
            line_spacing: settings.line_spacing,
            font_family: settings.font_family.clone(),
            color_caret: settings.color_caret.clone(),
            show_keybindings: false,
            find_panel_pos: None, // None = right-aligned default
            dragging_find_panel: false,
            drag_offset: (0.0, 0.0),
            last_mouse_pos: iced::Point::ORIGIN,
            font_size: settings.font_size,
            macro_recorder: MacroRecorder::new(),
            last_macro: None,
            file_extension: String::new(),
            fold_manager: FoldManager::new(),
            show_find_in_files: false,
            fif_query: String::new(),
            fif_directory: String::new(),
            fif_file_filter: String::from("*.*"),
            fif_recursive: true,
            fif_case_sensitive: false,
            fif_use_regex: false,
            fif_results: Vec::new(),
            fif_searching: false,
            fif_panel_pos: None,
            dragging_fif_panel: false,
            fif_drag_offset: (0.0, 0.0),
            split_mode: SplitMode::None,
            split_tab_manager: TabManager::new(),
            split_tab_contents: vec![TabContent::new()],
            split_font_size: 14.0,
            split_file_extension: String::new(),
            split_show_markdown_preview: false,
            active_pane: 0,
            theme,
            tab_context_menu: None,
            tab_rename: None,
            is_fullscreen: false,
            last_tab_click: None,
            tab_drag: None,
            mark_manager: MarkManager::default(),
            show_search_results_panel: false,
            search_results_panel: SearchResultsManager::default(),
            editor_context_menu: false,
            status_message: None,
            show_disasm: false,
            disasm_state: None,
            disasm_offset: 0,
            disasm_arch: crate::tools::disasm::DisasmArch::X86_64,
            disasm_goto_addr: String::new(),
            disasm_goto_sym: String::new(),
            disasm_base_addr_input: String::new(),
            disasm_show_address: true,
            disasm_show_bytes: true,
            disasm_show_resolved: true,
            disasm_show_raw_comment: true,
            disasm_show_arrows: true,
            disasm_context_menu: false,
            disasm_sym_browser: false,
            disasm_sym_filter: String::new(),
            disasm_edit_mode: false,
            disasm_edit_offset: None,
            disasm_edit_hex: String::new(),
        };

        // Apply CLI arguments if provided
        if let Some(cli) = CLI_ARGS.get() {
            open_cli_files(&mut state, cli);
        }

        state
    }
}

/// Open files from CLI arguments, applying encoding/language/read_only/goto overrides.
fn open_cli_files(state: &mut NotepadIced, cli: &CliArgs) {
    let mut opened_any = false;
    for path in &cli.files {
        let path = if path.is_absolute() {
            path.clone()
        } else {
            std::env::current_dir().unwrap_or_default().join(path)
        };
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            state.file_extension = ext.to_lowercase();
        }
        match state.tab_manager.open_file(path) {
            Ok(idx) => {
                let doc = state.tab_manager.get_document_mut(idx).unwrap();
                if let Some(ref enc_name) = cli.encoding {
                    if let Some(enc) = Encoding::from_name(enc_name) {
                        doc.encoding = enc;
                    }
                }
                if let Some(ref lang) = cli.language {
                    doc.language = lang.clone();
                } else if !state.file_extension.is_empty() {
                    doc.language = super::highlighter::language_for_extension(&state.file_extension);
                }
                if cli.read_only {
                    doc.read_only = true;
                }
                let is_binary = doc.is_binary;
                let buf_text = doc.buffer.text();
                state.fold_manager.detect_regions(&buf_text);
                // For binary files, use placeholder text instead of loading
                // the full binary content into iced's text_editor
                if is_binary {
                    state.tab_contents.push(TabContent::with_text("[Binary file — use Disassembler or Hex viewer]"));
                } else {
                    state.tab_contents.push(TabContent::with_text(&buf_text));
                }
                // Auto-show disassembler for binary files
                if is_binary {
                    if let Some(ref p) = state.tab_manager.active_document().path {
                        if let Ok(disasm) = crate::tools::disasm::Disassembler::from_file(p) {
                            state.disasm_state = Some(disasm);
                            state.disasm_offset = 0;
                            state.show_disasm = true;
                            // Also save to tab content
                            let idx = state.tab_manager.active_index();
                            if let Some(tc) = state.tab_contents.get_mut(idx) {
                                tc.show_disasm = true;
                            }
                        }
                    }
                }
                opened_any = true;
            }
            Err(e) => {
                log::error!("Failed to open CLI file: {}", e);
            }
        }
    }
    // Apply goto_line/goto_column to the last opened file
    if opened_any {
        if let Some(line) = cli.goto_line {
            state.tab_manager.active_document_mut().cursor.position.line = line.saturating_sub(1);
        }
        if let Some(col) = cli.goto_column {
            state.tab_manager.active_document_mut().cursor.position.col = col.saturating_sub(1);
        }
    }
}

/// Open files from a single-instance message.
fn open_instance_files(state: &mut NotepadIced, msg: &crate::platform::single_instance::InstanceMessage) {
    let mut opened_any = false;
    for path in &msg.files {
        let path = if path.is_absolute() {
            path.clone()
        } else {
            std::env::current_dir().unwrap_or_default().join(path)
        };
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            state.file_extension = ext.to_lowercase();
        }
        match state.tab_manager.open_file(path) {
            Ok(idx) => {
                let doc = state.tab_manager.get_document_mut(idx).unwrap();
                if let Some(ref enc_name) = msg.encoding {
                    if let Some(enc) = Encoding::from_name(enc_name) {
                        doc.encoding = enc;
                    }
                }
                if let Some(ref lang) = msg.language {
                    doc.language = lang.clone();
                } else if !state.file_extension.is_empty() {
                    doc.language = super::highlighter::language_for_extension(&state.file_extension);
                }
                if msg.read_only {
                    doc.read_only = true;
                }
                let buf_text = doc.buffer.text();
                let is_binary = doc.is_binary;
                state.fold_manager.detect_regions(&buf_text);
                if is_binary {
                    state.tab_contents.push(TabContent::with_text("[Binary file — use Disassembler or Hex viewer]"));
                } else {
                    state.tab_contents.push(TabContent::with_text(&buf_text));
                }
                // Auto-activate disassembler for binary files
                if is_binary {
                    if let Some(ref p) = state.tab_manager.active_document().path {
                        if let Ok(disasm) = crate::tools::disasm::Disassembler::from_file(p) {
                            state.disasm_state = Some(disasm);
                            state.disasm_offset = 0;
                            state.show_disasm = true;
                            let idx = state.tab_manager.active_index();
                            if let Some(tc) = state.tab_contents.get_mut(idx) {
                                tc.show_disasm = true;
                            }
                        }
                    }
                }
                opened_any = true;
            }
            Err(e) => {
                log::error!("Failed to open instance file: {}", e);
            }
        }
    }
    if opened_any {
        if let Some(line) = msg.goto_line {
            state.tab_manager.active_document_mut().cursor.position.line = line.saturating_sub(1);
        }
        if let Some(col) = msg.goto_column {
            state.tab_manager.active_document_mut().cursor.position.col = col.saturating_sub(1);
        }
    }
}

pub fn title(state: &NotepadIced) -> String {
    let doc = state.tab_manager.active_document();
    let tab_title = state
        .tab_manager
        .get_tab_title(state.tab_manager.active_index());
    crate::editor::text_transforms::format_title(&tab_title, doc.is_modified())
}

pub fn update(state: &mut NotepadIced, message: Message) -> Task<Message> {
    // Close menu for most actions (except MenuToggle/MenuClose/EditorAction and dialog-internal messages)
    let should_close_menu = !matches!(
        message,
        Message::MenuToggle(_) | Message::MenuSwitch(_) | Message::MenuClose | Message::SubMenuToggle(_) | Message::EditorAction(_)
            | Message::FileOpened(_) | Message::FilesOpened(_) | Message::FileSaved(_)
            | Message::FindQueryChanged(_) | Message::ReplaceTextChanged(_)
            | Message::ToggleCaseSensitive | Message::ToggleWholeWord | Message::ToggleRegex
            | Message::FindNext | Message::FindPrev | Message::ReplaceNext | Message::ReplaceAll
            | Message::CloseSearch | Message::ClickSearchResult(_)
            | Message::FindAll
            | Message::GotoLineInputChanged(_) | Message::GotoLineConfirm | Message::GotoLineClose
            | Message::CloseAbout
            | Message::SavePreferences | Message::CancelPreferences
            | Message::PrefFontSizeIncrease | Message::PrefFontSizeDecrease
            | Message::PrefTabSizeChanged(_) | Message::PrefSetTheme(_)
            | Message::PrefToggleAutoSave(_) | Message::PrefToggleWordWrap(_)
            | Message::PrefToggleLineNumbers(_) | Message::PrefToggleWhitespace(_)
            | Message::PrefFontFamilyChanged(_) | Message::PrefLineSpacingChanged(_)
            | Message::PrefColorBgChanged(_) | Message::PrefColorFgChanged(_)
            | Message::PrefColorSelChanged(_) | Message::PrefColorCaretChanged(_)
            | Message::ImportTheme | Message::ExportTheme
            | Message::ThemeImported(_) | Message::ThemeExported(_)
            | Message::CloseKeybindingsDialog
            | Message::SessionFileChosen(_) | Message::ExportSaved(_) | Message::CompareFileLoaded(_)
            | Message::EscapePressed
            | Message::DragFindStart | Message::DragFindMove(_) | Message::DragFindEnd
            | Message::FifQueryChanged(_) | Message::FifDirectoryChanged(_) | Message::FifFileFilterChanged(_)
            | Message::FifToggleRecursive | Message::FifToggleCaseSensitive | Message::FifToggleRegex
            | Message::FifSearch | Message::FifSearchComplete(_) | Message::FifClickResult(_, _)
            | Message::CloseFindInFiles
            | Message::DragFifStart | Message::DragFifMove(_) | Message::DragFifEnd
            | Message::SplitEditorAction(_) | Message::SplitNewTab | Message::SplitSelectTab(_)
            | Message::SplitCloseTab(_) | Message::SetActivePane(_)
            | Message::NewTabInActivePane | Message::FileDropped(_)
            | Message::MoveTabToSplit(_) | Message::MoveTabFromSplit(_)
            | Message::TabContextMenu(_, _) | Message::TabContextClose(_, _)
            | Message::TabContextCloseOthers(_, _) | Message::TabContextCloseAll(_)
            | Message::TabContextCloseToLeft(_, _) | Message::TabContextCloseToRight(_, _)
            | Message::TabContextCloseUnmodified(_)
            | Message::TabContextOpenInExplorer(_, _) | Message::TabContextOpenTerminalHere(_, _)
            | Message::TabContextCopyFilename(_, _) | Message::TabContextCopyFullPath(_, _)
            | Message::TabContextRename(_, _) | Message::TabContextRenameInput(_)
            | Message::TabContextRenameConfirm | Message::TabContextRenameCancel
            | Message::TabContextMoveToOtherPane(_, _) | Message::CloseTabContextMenu
            | Message::TabContextCloneToOtherView(_, _)
            | Message::MiddleClickTab(_, _)
            | Message::TabDragStart(_, _) | Message::TabDragOver(_, _)
            | Message::TabDragEnd | Message::TabDragCancel
            | Message::EditorContextMenu | Message::CloseEditorContextMenu
    );
    if should_close_menu {
        state.active_menu = None;
    }

    match message {
        Message::EditorAction(action) => {
            state.active_pane = 0;
            // Block editing actions when document is read-only
            if action.is_edit() && state.tab_manager.active_document().read_only {
                return Task::none();
            }
            if let Some(tc) = state
                .tab_contents
                .get_mut(state.tab_manager.active_index())
            {
                tc.content.perform(action);
                let new_text = tc.content.text();
                let doc = state.tab_manager.active_document_mut();
                let old_text = doc.buffer.text();
                if new_text != old_text {
                    let len = doc.buffer.len_bytes();
                    if len > 0 {
                        doc.buffer.delete(0, len);
                    }
                    doc.buffer.insert(0, &new_text);
                    state.fold_manager.detect_regions(&new_text);
                }
            }
            Task::none()
        }
        Message::NewTab => {
            if state.active_pane == 1 && state.split_mode != SplitMode::None {
                state.split_tab_manager.new_tab();
                state.split_tab_contents.push(TabContent::new());
            } else {
                state.tab_manager.new_tab();
                state.tab_contents.push(TabContent::new());
            }
            Task::none()
        }
        Message::OpenFile => Task::perform(
            async {
                let handles = rfd::AsyncFileDialog::new()
                    .set_title("Open File")
                    .pick_files()
                    .await;
                match handles {
                    Some(files) => {
                        let mut results = Vec::new();
                        for h in files {
                            let path = h.path().to_path_buf();
                            match std::fs::read(&path) {
                                Ok(bytes) => {
                                    let content = String::from_utf8_lossy(&bytes).to_string();
                                    results.push(Ok((content, path)));
                                }
                                Err(e) => results.push(Err(e.to_string())),
                            }
                        }
                        Ok(results)
                    }
                    None => Err("Cancelled".to_string()),
                }
            },
            |result| match result {
                Ok(files) => Message::FilesOpened(files),
                Err(_) => Message::FilesOpened(Vec::new()),
            },
        ),
        Message::FilesOpened(results) => {
            for result in results {
                if let Ok((content, path)) = result {
                    let ext_lower = path.extension().and_then(|e| e.to_str())
                        .map(|e| e.to_lowercase()).unwrap_or_default();
                    if !ext_lower.is_empty() {
                        state.file_extension = ext_lower.clone();
                    }
                    match state.tab_manager.open_file(path) {
                        Ok(idx) => {
                            if !ext_lower.is_empty() {
                                let lang = super::highlighter::language_for_extension(&ext_lower);
                                state.tab_manager.get_document_mut(idx).unwrap().language = lang;
                            }
                            let doc = state.tab_manager.get_document(idx).unwrap();
                            let is_binary = doc.is_binary;
                            let buf_text = doc.buffer.text();
                            state.fold_manager.detect_regions(&buf_text);
                            if is_binary {
                                state.tab_contents.push(TabContent::with_text("[Binary file — use Disassembler or Hex viewer]"));
                            } else {
                                state.tab_contents.push(TabContent::with_text(&buf_text));
                            }
                            if is_binary {
                                if let Some(ref p) = state.tab_manager.active_document().path {
                                    if let Ok(disasm) = crate::tools::disasm::Disassembler::from_file(p) {
                                        state.disasm_state = Some(disasm);
                                        state.disasm_offset = 0;
                                        state.show_disasm = true;
                                        if let Some(tc) = state.tab_contents.get_mut(idx) {
                                            tc.show_disasm = true;
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to open file: {}", e);
                        }
                    }
                }
            }
            Task::none()
        }
        Message::FileOpened(result) => {
            // Delegate single file open to FilesOpened
            update(state, Message::FilesOpened(vec![result]))
        }
        Message::Save => {
            let idx = state.tab_manager.active_index();
            sync_content_to_doc(state, idx);
            let has_path = state.tab_manager.active_document().path.is_some();
            if has_path {
                if let Err(e) = state.tab_manager.save_tab(idx) {
                    log::error!("Save failed: {}", e);
                }
                Task::none()
            } else {
                save_as_dialog()
            }
        }
        Message::SaveAs => {
            let idx = state.tab_manager.active_index();
            sync_content_to_doc(state, idx);
            save_as_dialog()
        }
        Message::FileSaved(result) => {
            if let Ok(path) = result {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    state.file_extension = ext_lower.clone();
                    let lang = super::highlighter::language_for_extension(&ext_lower);
                    state.tab_manager.active_document_mut().language = lang;
                }
                let idx = state.tab_manager.active_index();
                if let Err(e) = state.tab_manager.save_tab_as(idx, path) {
                    log::error!("Save As failed: {}", e);
                }
            }
            Task::none()
        }
        Message::CloseTab(idx) => {
            let actual_idx = if idx == usize::MAX {
                state.tab_manager.active_index()
            } else {
                idx
            };
            if actual_idx < state.tab_contents.len() {
                state.tab_contents.remove(actual_idx);
                state.tab_manager.close_tab(actual_idx);
                while state.tab_contents.len() < state.tab_manager.tab_count() {
                    state.tab_contents.push(TabContent::new());
                }
                // Load disasm state from new active tab
                let new_idx = state.tab_manager.active_index();
                if let Some(tc) = state.tab_contents.get_mut(new_idx) {
                    state.show_disasm = tc.show_disasm;
                    state.disasm_state = tc.disasm_state.take();
                    state.disasm_offset = tc.disasm_offset;
                } else {
                    state.show_disasm = false;
                    state.disasm_state = None;
                    state.disasm_offset = 0;
                }
            }
            Task::none()
        }
        Message::SelectTab(idx) => {
            // Double-click detection: close tab on double-click
            let now = std::time::Instant::now();
            if let Some((last_idx, last_time)) = state.last_tab_click {
                if last_idx == idx && now.duration_since(last_time).as_millis() < 400 {
                    state.last_tab_click = None;
                    state.tab_drag = None;
                    return update(state, Message::CloseTab(idx));
                }
            }
            state.last_tab_click = Some((idx, now));

            // Start tab drag
            state.tab_drag = Some(TabDragState {
                from_index: idx,
                is_split: false,
                target_index: None,
            });

            if idx < state.tab_manager.tab_count() {
                // Save disasm state to the old tab before switching
                let old_idx = state.tab_manager.active_index();
                if let Some(old_tc) = state.tab_contents.get_mut(old_idx) {
                    old_tc.show_disasm = state.show_disasm;
                    old_tc.disasm_state = state.disasm_state.take();
                    old_tc.disasm_offset = state.disasm_offset;
                }

                state.tab_manager.set_active(idx);

                // Load disasm state from the new tab
                if let Some(new_tc) = state.tab_contents.get_mut(idx) {
                    state.show_disasm = new_tc.show_disasm;
                    state.disasm_state = new_tc.disasm_state.take();
                    state.disasm_offset = new_tc.disasm_offset;
                } else {
                    state.show_disasm = false;
                    state.disasm_state = None;
                    state.disasm_offset = 0;
                }
                // Update file extension from the newly active tab's path
                if let Some(path) = &state.tab_manager.active_document().path {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        state.file_extension = ext.to_lowercase();
                    } else {
                        state.file_extension.clear();
                    }
                } else {
                    state.file_extension.clear();
                }
                // Re-detect fold regions for the newly active tab
                if let Some(tc) = state.tab_contents.get(idx) {
                    let text = tc.content.text();
                    state.fold_manager.detect_regions(&text);
                }
            }
            Task::none()
        }
        Message::Undo => {
            let doc = state.tab_manager.active_document_mut();
            doc.buffer.undo();
            let buf_text = doc.buffer.text();
            let idx = state.tab_manager.active_index();
            let mut new_tc = TabContent::with_text(&buf_text);
            if let Some(old_tc) = state.tab_contents.get_mut(idx) {
                new_tc.disasm_state = old_tc.disasm_state.take();
                new_tc.disasm_offset = old_tc.disasm_offset;
                new_tc.show_disasm = old_tc.show_disasm;
            }
            state.tab_contents[idx] = new_tc;
            Task::none()
        }
        Message::Redo => {
            let doc = state.tab_manager.active_document_mut();
            doc.buffer.redo();
            let buf_text = doc.buffer.text();
            let idx = state.tab_manager.active_index();
            let mut new_tc = TabContent::with_text(&buf_text);
            if let Some(old_tc) = state.tab_contents.get_mut(idx) {
                new_tc.disasm_state = old_tc.disasm_state.take();
                new_tc.disasm_offset = old_tc.disasm_offset;
                new_tc.show_disasm = old_tc.show_disasm;
            }
            state.tab_contents[idx] = new_tc;
            Task::none()
        }

        Message::ToggleReadOnly => {
            let doc = state.tab_manager.active_document_mut();
            doc.read_only = !doc.read_only;
            Task::none()
        }

        // ── Menu state ──
        Message::MenuToggle(name) => {
            if state.active_menu.as_ref() == Some(&name) {
                state.active_menu = None;
                state.expanded_submenus.clear();
            } else {
                state.active_menu = Some(name);
                state.expanded_submenus.clear();
            }
            Task::none()
        }
        Message::MenuSwitch(name) => {
            // Switch to a different menu when one is already open (Windows-style hover behavior)
            if state.active_menu.is_some() {
                state.active_menu = Some(name);
                state.expanded_submenus.clear();
            }
            Task::none()
        }
        Message::MenuClose => {
            state.active_menu = None;
            state.expanded_submenus.clear();
            Task::none()
        }
        Message::SubMenuToggle(name) => {
            if state.expanded_submenus.contains(&name) {
                state.expanded_submenus.remove(&name);
            } else {
                state.expanded_submenus.insert(name);
            }
            Task::none()
        }

        // ── File ──
        Message::SaveAll => {
            for i in 0..state.tab_manager.tab_count() {
                sync_content_to_doc(state, i);
                if state.tab_manager.get_document(i).map_or(false, |d| d.path.is_some()) {
                    if let Err(e) = state.tab_manager.save_tab(i) {
                        log::error!("Save All failed for tab {}: {}", i, e);
                    }
                }
            }
            Task::none()
        }
        Message::CloseAll => {
            state.tab_manager.close_all();
            state.tab_contents.clear();
            state.tab_contents.push(TabContent::new());
            Task::none()
        }
        Message::SaveSession => {
            use crate::io::session;
            let sess = session::capture_session(&state.tab_manager, "session");
            if let Some(config_dir) = dirs::config_dir() {
                let dir = config_dir.join("notepadppp").join("sessions");
                let _ = std::fs::create_dir_all(&dir);
                let path = dir.join("session.json");
                let _ = session::save_session(&sess, &path);
            }
            Task::none()
        }
        Message::LoadSession => {
            Task::perform(
                async {
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("Session", &["json"])
                        .set_title("Load Session")
                        .pick_file()
                        .await;
                    match handle {
                        Some(h) => Ok(h.path().to_path_buf()),
                        None => Err("Cancelled".to_string()),
                    }
                },
                Message::SessionFileChosen,
            )
        }
        Message::ToggleAutoRestore => {
            state.auto_restore_session = !state.auto_restore_session;
            Task::none()
        }
        Message::ExportHtml => {
            let text = get_buffer_text(state);
            let doc = state.tab_manager.active_document();
            let ext = doc
                .path
                .as_ref()
                .and_then(|p| p.extension())
                .and_then(|e| e.to_str())
                .unwrap_or("txt")
                .to_string();
            let highlighter = crate::editor::syntax::SyntaxHighlighter::new();
            let html = crate::tools::export::export_html(&text, &ext, &highlighter);
            Task::perform(
                async move {
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("HTML", &["html"])
                        .set_title("Export as HTML")
                        .save_file()
                        .await;
                    match handle {
                        Some(h) => {
                            let path = h.path().to_path_buf();
                            std::fs::write(&path, &html).map_err(|e| e.to_string())
                        }
                        None => Err("Cancelled".to_string()),
                    }
                },
                Message::ExportSaved,
            )
        }
        Message::ExportRtf => {
            let text = get_buffer_text(state);
            let rtf = crate::tools::export::export_rtf(&text);
            Task::perform(
                async move {
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("RTF", &["rtf"])
                        .set_title("Export as RTF")
                        .save_file()
                        .await;
                    match handle {
                        Some(h) => {
                            let path = h.path().to_path_buf();
                            std::fs::write(&path, &rtf).map_err(|e| e.to_string())
                        }
                        None => Err("Cancelled".to_string()),
                    }
                },
                Message::ExportSaved,
            )
        }
        Message::Exit => {
            std::process::exit(0);
        }

        // ── Edit ──
        Message::SelectAll => {
            if let Some(tc) = state.tab_contents.get_mut(state.tab_manager.active_index()) {
                tc.content.perform(text_editor::Action::SelectAll);
            }
            Task::none()
        }
        Message::ToggleComment => {
            apply_line_op(state, |lines, cursor_line| {
                let language = "Rust"; // default
                let prefix = crate::editor::comments::line_comment_prefix(language).unwrap_or("//");
                if cursor_line < lines.len() {
                    lines[cursor_line] = crate::editor::text_transforms::toggle_line_comment(&lines[cursor_line], prefix);
                }
            });
            Task::none()
        }

        // Line operations
        Message::DuplicateLine => {
            apply_line_op(state, |lines, cursor_line| {
                crate::editor::line_ops::duplicate_line(lines, cursor_line);
            });
            Task::none()
        }
        Message::DeleteLine => {
            apply_line_op(state, |lines, cursor_line| {
                crate::editor::line_ops::delete_line(lines, cursor_line);
            });
            Task::none()
        }
        Message::MoveLineUp => {
            apply_line_op(state, |lines, cursor_line| {
                crate::editor::line_ops::move_line_up(lines, cursor_line);
            });
            Task::none()
        }
        Message::MoveLineDown => {
            apply_line_op(state, |lines, cursor_line| {
                crate::editor::line_ops::move_line_down(lines, cursor_line);
            });
            Task::none()
        }
        Message::SortAsc => {
            apply_line_op(state, |lines, _| crate::editor::line_ops::sort_asc(lines));
            Task::none()
        }
        Message::SortDesc => {
            apply_line_op(state, |lines, _| crate::editor::line_ops::sort_desc(lines));
            Task::none()
        }
        Message::RemoveEmpty => {
            apply_line_op(state, |lines, _| crate::editor::line_ops::remove_empty(lines));
            Task::none()
        }
        Message::RemoveDuplicates => {
            apply_line_op(state, |lines, _| crate::editor::line_ops::remove_duplicates(lines));
            Task::none()
        }
        Message::TrimTrailing => {
            apply_line_op(state, |lines, _| crate::editor::line_ops::trim_trailing(lines));
            Task::none()
        }
        Message::JoinLines => {
            apply_line_op(state, |lines, cursor_line| {
                crate::editor::line_ops::join_lines(lines, cursor_line);
            });
            Task::none()
        }
        Message::SplitLine => {
            apply_line_op(state, |lines, cursor_line| {
                crate::editor::line_ops::split_line(lines, cursor_line);
            });
            Task::none()
        }
        Message::InsertAbove => {
            apply_line_op(state, |lines, cursor_line| {
                crate::editor::line_ops::insert_above(lines, cursor_line);
            });
            Task::none()
        }
        Message::InsertBelow => {
            apply_line_op(state, |lines, cursor_line| {
                crate::editor::line_ops::insert_below(lines, cursor_line);
            });
            Task::none()
        }
        Message::ReverseLines => {
            apply_line_op(state, |lines, _| crate::editor::line_ops::reverse_lines(lines));
            Task::none()
        }
        Message::SortCaseInsensitive => {
            apply_line_op(state, |lines, _| crate::editor::line_ops::sort_case_insensitive(lines));
            Task::none()
        }
        Message::SortNumeric => {
            apply_line_op(state, |lines, _| crate::editor::line_ops::sort_numeric(lines));
            Task::none()
        }
        Message::TrimLeading => {
            apply_line_op(state, |lines, _| crate::editor::line_ops::trim_leading(lines));
            Task::none()
        }
        Message::TrimBoth => {
            apply_line_op(state, |lines, _| crate::editor::line_ops::trim_both(lines));
            Task::none()
        }

        // Case
        Message::UpperCase => {
            apply_text_transform(state, |s| s.to_uppercase());
            Task::none()
        }
        Message::LowerCase => {
            apply_text_transform(state, |s| s.to_lowercase());
            Task::none()
        }
        Message::TitleCase => {
            apply_text_transform(state, crate::editor::text_transforms::title_case);
            Task::none()
        }
        Message::SentenceCase => {
            apply_text_transform(state, crate::editor::text_transforms::sentence_case);
            Task::none()
        }
        Message::InverseCase => {
            apply_text_transform(state, crate::editor::text_transforms::inverse_case);
            Task::none()
        }

        // Encoding / line ending
        Message::SetEncoding(enc) => {
            state.tab_manager.active_document_mut().encoding = enc;
            Task::none()
        }
        Message::SetLineEnding(le) => {
            state.tab_manager.active_document_mut().line_ending = le;
            Task::none()
        }

        // ── Search ──
        Message::ShowFind => {
            state.show_find = true;
            state.show_replace = false;
            state.active_menu = None;
            Task::none()
        }
        Message::ShowReplace => {
            state.show_find = true;
            state.show_replace = true;
            state.active_menu = None;
            Task::none()
        }
        Message::ShowFindInFiles => {
            state.show_find_in_files = !state.show_find_in_files;
            // Pre-fill directory from current file's parent if available
            if state.show_find_in_files && state.fif_directory.is_empty() {
                if let Some(path) = &state.tab_manager.active_document().path {
                    if let Some(parent) = path.parent() {
                        state.fif_directory = parent.display().to_string();
                    }
                }
            }
            state.active_menu = None;
            Task::none()
        }
        Message::SelectAllOccurrences => {
            if !state.search_query.is_empty() {
                run_search(state);
            }
            Task::none()
        }
        Message::GotoLine => {
            state.show_goto_line = true;
            state.goto_line_input.clear();
            state.active_menu = None;
            Task::none()
        }
        Message::ToggleBookmark => {
            let line = state.tab_manager.active_document().cursor.position.line;
            state.tab_manager.active_document_mut().bookmarks.toggle(line);
            Task::none()
        }
        Message::NextBookmark => {
            let line = state.tab_manager.active_document().cursor.position.line;
            if let Some(next) = state.tab_manager.active_document().bookmarks.next_bookmark(line) {
                state.tab_manager.active_document_mut().cursor.position.line = next;
                state.tab_manager.active_document_mut().cursor.position.col = 0;
            }
            Task::none()
        }
        Message::PrevBookmark => {
            let line = state.tab_manager.active_document().cursor.position.line;
            if let Some(prev) = state.tab_manager.active_document().bookmarks.prev_bookmark(line) {
                state.tab_manager.active_document_mut().cursor.position.line = prev;
                state.tab_manager.active_document_mut().cursor.position.col = 0;
            }
            Task::none()
        }
        Message::ClearBookmarks => {
            state.tab_manager.active_document_mut().bookmarks.clear();
            Task::none()
        }
        Message::CopyBookmarkedLines => {
            let text = get_buffer_text(state);
            let lines = state.tab_manager.active_document().bookmarks.bookmarked_lines(&text);
            if !lines.is_empty() {
                let combined = lines.join("\n");
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(combined);
                }
            }
            Task::none()
        }
        Message::RemoveBookmarkedLines => {
            let text = get_buffer_text(state);
            let result = state.tab_manager.active_document().bookmarks.remove_bookmarked_lines(&text);
            set_buffer_text(state, &result);
            state.tab_manager.active_document_mut().bookmarks.clear();
            Task::none()
        }
        Message::RemoveUnbookmarkedLines => {
            let text = get_buffer_text(state);
            let result = state.tab_manager.active_document().bookmarks.remove_unbookmarked_lines(&text);
            set_buffer_text(state, &result);
            state.tab_manager.active_document_mut().bookmarks.clear();
            Task::none()
        }
        Message::CutBookmarkedLines => {
            let text = get_buffer_text(state);
            let (remaining, cut) = state.tab_manager.active_document().bookmarks.cut_bookmarked_lines(&text);
            if !cut.is_empty() {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(cut);
                }
                set_buffer_text(state, &remaining);
                state.tab_manager.active_document_mut().bookmarks.clear();
            }
            Task::none()
        }

        // ── Mark All ──
        Message::MarkAll => {
            if !state.search_matches.is_empty() {
                let ranges: Vec<(usize, usize)> = state.search_matches.iter()
                    .map(|m| (m.start, m.end))
                    .collect();
                state.mark_manager.mark_all(ranges);
                // Also add to search results panel
                let results: Vec<super::search_results_panel::SearchResultMatch> = state.search_matches.iter()
                    .map(|m| super::search_results_panel::SearchResultMatch {
                        line: m.line,
                        line_text: m.line_text.clone(),
                        file_path: None,
                    })
                    .collect();
                state.search_results_panel.add_search(state.search_query.clone(), results);
            }
            Task::none()
        }
        Message::ClearAllMarks => {
            state.mark_manager.clear();
            Task::none()
        }

        // ── Find All ──
        Message::FindAll => {
            if !state.search_matches.is_empty() {
                let results: Vec<super::search_results_panel::SearchResultMatch> = state.search_matches.iter()
                    .map(|m| super::search_results_panel::SearchResultMatch {
                        line: m.line,
                        line_text: m.line_text.clone(),
                        file_path: None,
                    })
                    .collect();
                state.search_results_panel.add_search(state.search_query.clone(), results);
                state.show_search_results_panel = true;
            }
            Task::none()
        }

        // ── Bookmark matching lines ──
        Message::BookmarkMatchingLines => {
            for m in &state.search_matches {
                state.tab_manager.active_document_mut().bookmarks.add(m.line);
            }
            Task::none()
        }

        // ── Search Results Panel ──
        Message::ToggleSearchResultsPanel => {
            state.show_search_results_panel = !state.show_search_results_panel;
            state.active_menu = None;
            Task::none()
        }
        Message::ClearSearchResults => {
            state.search_results_panel.clear();
            Task::none()
        }
        Message::ToggleSearchResultCollapse(idx) => {
            state.search_results_panel.toggle_collapse(idx);
            Task::none()
        }
        Message::ClickSearchResultEntry(entry_idx, match_idx) => {
            if let Some(entry) = state.search_results_panel.entries.get(entry_idx) {
                if let Some(m) = entry.matches.get(match_idx) {
                    if let Some(ref file_path) = m.file_path {
                        // FiF result: open/switch to the file via FifClickResult logic
                        let path = file_path.clone();
                        let line = m.line;
                        return update(state, Message::FifClickResult(path, line));
                    }
                    let target_line = m.line;
                    let idx = state.tab_manager.active_index();
                    if let Some(tc) = state.tab_contents.get_mut(idx) {
                        tc.content.perform(text_editor::Action::Move(text_editor::Motion::DocumentStart));
                        for _ in 0..target_line {
                            tc.content.perform(text_editor::Action::Move(text_editor::Motion::Down));
                        }
                        tc.content.perform(text_editor::Action::Move(text_editor::Motion::Home));
                    }
                }
            }
            Task::none()
        }

        // ── Single Instance ──
        Message::CheckInstance(_) => {
            if let Some(mutex) = INSTANCE_LISTENER.get() {
                if let Ok(guard) = mutex.lock() {
                    if let Some(ref listener) = *guard {
                        if let Some(msg) = listener.try_recv() {
                            drop(guard);
                            open_instance_files(state, &msg);
                        }
                    }
                }
            }
            Task::none()
        }

        // ── View ──
        Message::ToggleWordWrap => {
            state.word_wrap = !state.word_wrap;
            Task::none()
        }
        Message::ToggleLineNumbers => {
            state.show_line_numbers = !state.show_line_numbers;
            Task::none()
        }
        Message::ToggleWhitespace => {
            state.show_whitespace = !state.show_whitespace;
            Task::none()
        }
        Message::ToggleStatusBar => {
            state.show_status_bar = !state.show_status_bar;
            Task::none()
        }
        Message::ToggleToolbar => {
            state.show_toolbar = !state.show_toolbar;
            Task::none()
        }
        Message::ToggleMinimap => {
            state.show_minimap = !state.show_minimap;
            Task::none()
        }
        Message::ToggleFunctionList => {
            state.show_function_list = !state.show_function_list;
            Task::none()
        }
        Message::ToggleFullScreen => {
            state.is_fullscreen = !state.is_fullscreen;
            let is_fs = state.is_fullscreen;
            iced::window::get_oldest().and_then(move |id| {
                let mode = if is_fs {
                    iced::window::Mode::Fullscreen
                } else {
                    iced::window::Mode::Windowed
                };
                iced::window::change_mode(id, mode)
            })
        }
        Message::SplitHorizontal => {
            if state.split_mode == SplitMode::Vertical {
                // Switch from vertical to horizontal, keep existing split state
                state.split_mode = SplitMode::Horizontal;
            } else if state.split_mode == SplitMode::None {
                state.split_mode = SplitMode::Horizontal;
                state.split_tab_manager = TabManager::new();
                state.split_tab_contents = vec![TabContent::new()];
                state.split_font_size = state.font_size;
                state.split_file_extension.clear();
                state.active_pane = 1;
            }
            Task::none()
        }
        Message::SplitVertical => {
            if state.split_mode == SplitMode::Horizontal {
                // Switch from horizontal to vertical, keep existing split state
                state.split_mode = SplitMode::Vertical;
            } else if state.split_mode == SplitMode::None {
                state.split_mode = SplitMode::Vertical;
                state.split_tab_manager = TabManager::new();
                state.split_tab_contents = vec![TabContent::new()];
                state.split_font_size = state.font_size;
                state.split_file_extension.clear();
                state.active_pane = 1;
            }
            Task::none()
        }
        Message::RemoveSplit => {
            state.split_mode = SplitMode::None;
            state.split_tab_manager = TabManager::new();
            state.split_tab_contents = vec![TabContent::new()];
            state.split_font_size = 14.0;
            state.split_file_extension.clear();
            state.split_show_markdown_preview = false;
            state.active_pane = 0;
            Task::none()
        }
        Message::ZoomIn => {
            if state.active_pane == 1 && state.split_mode != SplitMode::None {
                state.split_font_size = crate::editor::text_transforms::clamp_zoom(state.split_font_size, 2.0);
            } else {
                state.font_size = crate::editor::text_transforms::clamp_zoom(state.font_size, 2.0);
            }
            Task::none()
        }
        Message::ZoomOut => {
            if state.active_pane == 1 && state.split_mode != SplitMode::None {
                state.split_font_size = crate::editor::text_transforms::clamp_zoom(state.split_font_size, -2.0);
            } else {
                state.font_size = crate::editor::text_transforms::clamp_zoom(state.font_size, -2.0);
            }
            Task::none()
        }
        Message::ZoomReset => {
            if state.active_pane == 1 && state.split_mode != SplitMode::None {
                state.split_font_size = 14.0;
            } else {
                state.font_size = 14.0;
            }
            Task::none()
        }
        Message::ToggleFold => {
            state.status_message = Some(("Code folding is not yet supported with the current editor backend".to_string(), std::time::Instant::now()));
            Task::none()
        }
        Message::ToggleFoldAt(_line) => {
            state.status_message = Some(("Code folding is not yet supported with the current editor backend".to_string(), std::time::Instant::now()));
            Task::none()
        }
        Message::FoldAll => {
            state.status_message = Some(("Code folding is not yet supported with the current editor backend".to_string(), std::time::Instant::now()));
            Task::none()
        }
        Message::UnfoldAll => {
            state.status_message = Some(("Code folding is not yet supported with the current editor backend".to_string(), std::time::Instant::now()));
            Task::none()
        }
        Message::FoldLevel(_level) => {
            state.status_message = Some(("Code folding is not yet supported with the current editor backend".to_string(), std::time::Instant::now()));
            Task::none()
        }

        // ── Language ──
        Message::SetLanguage(lang) => {
            state.tab_manager.active_document_mut().language = lang.clone();
            // Try to find an extension for this language in syntect
            if let Some(ext) = super::highlighter::extension_for_language(&lang) {
                state.file_extension = ext;
            }
            Task::none()
        }

        // ── Tools ──
        Message::JsonFormat => {
            let text = get_buffer_text(state);
            match crate::tools::json_tools::format_json(&text) {
                Ok(formatted) => set_buffer_text(state, &formatted),
                Err(e) => log::error!("JSON format error: {}", e),
            }
            Task::none()
        }
        Message::JsonCompact => {
            let text = get_buffer_text(state);
            match crate::tools::json_tools::compact_json(&text) {
                Ok(compacted) => set_buffer_text(state, &compacted),
                Err(e) => log::error!("JSON compact error: {}", e),
            }
            Task::none()
        }
        Message::JsonValidate => {
            let text = get_buffer_text(state);
            match crate::tools::json_tools::validate_json(&text) {
                Ok(()) => log::info!("JSON is valid"),
                Err(e) => log::error!("JSON validation error: {}", e),
            }
            Task::none()
        }
        Message::JsonSortKeys => {
            let text = get_buffer_text(state);
            match crate::tools::json_tools::sort_json_keys(&text) {
                Ok(sorted) => set_buffer_text(state, &sorted),
                Err(e) => log::error!("JSON sort keys error: {}", e),
            }
            Task::none()
        }
        Message::ToggleMarkdownPreview => {
            if state.active_pane == 0 {
                state.show_markdown_preview = !state.show_markdown_preview;
            } else {
                state.split_show_markdown_preview = !state.split_show_markdown_preview;
            }
            Task::none()
        }
        Message::ToggleCsvViewer => {
            state.show_csv_viewer = !state.show_csv_viewer;
            Task::none()
        }
        Message::MimeBase64Encode => {
            state.editor_context_menu = false;
            apply_text_transform(state, crate::tools::mime_tools::base64_encode);
            state.status_message = Some(("Base64 Encoded".to_string(), std::time::Instant::now()));
            Task::none()
        }
        Message::MimeBase64Decode => {
            state.editor_context_menu = false;
            let text = get_buffer_text(state);
            match crate::tools::mime_tools::base64_decode(&text) {
                Ok(decoded) => {
                    set_buffer_text(state, &decoded);
                    state.status_message = Some(("Base64 Decoded".to_string(), std::time::Instant::now()));
                }
                Err(e) => {
                    state.status_message = Some((format!("Base64 decode error: {e}"), std::time::Instant::now()));
                    log::error!("Base64 decode error: {}", e);
                }
            }
            Task::none()
        }
        Message::MimeUrlEncode => {
            state.editor_context_menu = false;
            apply_text_transform(state, crate::tools::mime_tools::url_encode);
            state.status_message = Some(("URL Encoded".to_string(), std::time::Instant::now()));
            Task::none()
        }
        Message::MimeUrlDecode => {
            state.editor_context_menu = false;
            let text = get_buffer_text(state);
            match crate::tools::mime_tools::url_decode(&text) {
                Ok(decoded) => {
                    set_buffer_text(state, &decoded);
                    state.status_message = Some(("URL Decoded".to_string(), std::time::Instant::now()));
                }
                Err(e) => {
                    state.status_message = Some((format!("URL decode error: {e}"), std::time::Instant::now()));
                    log::error!("URL decode error: {}", e);
                }
            }
            Task::none()
        }
        Message::MimeHtmlEncode => {
            state.editor_context_menu = false;
            apply_text_transform(state, crate::tools::mime_tools::html_entity_encode);
            state.status_message = Some(("HTML Entity Encoded".to_string(), std::time::Instant::now()));
            Task::none()
        }
        Message::MimeHtmlDecode => {
            state.editor_context_menu = false;
            apply_text_transform(state, crate::tools::mime_tools::html_entity_decode);
            state.status_message = Some(("HTML Entity Decoded".to_string(), std::time::Instant::now()));
            Task::none()
        }
        Message::MimeHexEncode => {
            state.editor_context_menu = false;
            apply_text_transform(state, crate::tools::mime_tools::hex_encode);
            state.status_message = Some(("Hex Encoded".to_string(), std::time::Instant::now()));
            Task::none()
        }
        Message::MimeHexDecode => {
            state.editor_context_menu = false;
            let text = get_buffer_text(state);
            match crate::tools::mime_tools::hex_decode(&text) {
                Ok(decoded) => {
                    set_buffer_text(state, &decoded);
                    state.status_message = Some(("Hex Decoded".to_string(), std::time::Instant::now()));
                }
                Err(e) => {
                    state.status_message = Some((format!("Hex decode error: {e}"), std::time::Instant::now()));
                    log::error!("Hex decode error: {}", e);
                }
            }
            Task::none()
        }
        Message::HashSha256 => {
            state.editor_context_menu = false;
            apply_hash_transform(state, "SHA-256", crate::tools::hash_tools::sha256);
            Task::none()
        }
        Message::HashSha1 => {
            state.editor_context_menu = false;
            apply_hash_transform(state, "SHA-1", crate::tools::hash_tools::sha1);
            Task::none()
        }
        Message::HashMd5 => {
            state.editor_context_menu = false;
            apply_hash_transform(state, "MD5", crate::tools::hash_tools::md5);
            Task::none()
        }
        Message::HashCrc32 => {
            state.editor_context_menu = false;
            apply_hash_transform(state, "CRC32", crate::tools::hash_tools::crc32);
            Task::none()
        }
        Message::EditorContextMenu => {
            state.editor_context_menu = true;
            Task::none()
        }
        Message::CloseEditorContextMenu => {
            state.editor_context_menu = false;
            Task::none()
        }
        Message::CompareFiles => {
            Task::perform(
                async {
                    let handle = rfd::AsyncFileDialog::new()
                        .set_title("Compare With...")
                        .pick_file()
                        .await;
                    match handle {
                        Some(h) => {
                            let path = h.path().to_path_buf();
                            match std::fs::read_to_string(&path) {
                                Ok(content) => Ok(content),
                                Err(e) => Err(e.to_string()),
                            }
                        }
                        None => Err("Cancelled".to_string()),
                    }
                },
                Message::CompareFileLoaded,
            )
        }
        Message::ToggleHexViewer => {
            state.show_hex_viewer = !state.show_hex_viewer;
            Task::none()
        }
        Message::ToggleDisasm => {
            state.show_disasm = !state.show_disasm;
            if state.show_disasm && state.disasm_state.is_none() {
                // Initialize from current file's bytes
                let content_text = state
                    .tab_contents
                    .get(state.tab_manager.active_index())
                    .map(|tc| tc.content.text())
                    .unwrap_or_default();
                let bytes = content_text.into_bytes();
                if !bytes.is_empty() {
                    let disasm = crate::tools::disasm::Disassembler::from_bytes(
                        bytes,
                        state.disasm_arch,
                        0,
                    );
                    state.disasm_base_addr_input = "0".to_string();
                    state.disasm_offset = 0;
                    state.disasm_state = Some(disasm);
                }
            }
            // Sync to tab
            let idx = state.tab_manager.active_index();
            if let Some(tc) = state.tab_contents.get_mut(idx) {
                tc.show_disasm = state.show_disasm;
            }
            Task::none()
        }
        Message::DisasmGotoAddressInput(s) => {
            state.disasm_goto_addr = s;
            Task::none()
        }
        Message::DisasmGotoAddress(addr_str) => {
            let addr_str = addr_str.trim().trim_start_matches("0x").trim_start_matches("0X");
            if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
                if let Some(ref disasm) = state.disasm_state {
                    if let Some(off) = disasm.address_to_offset(addr) {
                        state.disasm_offset = off;
                    }
                }
            }
            Task::none()
        }
        Message::DisasmGotoSymbolInput(s) => {
            state.disasm_goto_sym = s;
            Task::none()
        }
        Message::DisasmGotoSymbol(name) => {
            if let Some(ref disasm) = state.disasm_state {
                if let Some(sym) = disasm.find_symbol(&name) {
                    let addr = sym.address;
                    if let Some(off) = disasm.address_to_offset(addr) {
                        state.disasm_offset = off;
                    }
                }
            }
            Task::none()
        }
        Message::DisasmBaseAddressInput(s) => {
            state.disasm_base_addr_input = s;
            Task::none()
        }
        Message::DisasmSetBaseAddress(addr_str) => {
            let addr_str = addr_str.trim().trim_start_matches("0x").trim_start_matches("0X");
            if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
                if let Some(ref mut disasm) = state.disasm_state {
                    disasm.set_base_address(addr);
                }
            }
            Task::none()
        }
        Message::DisasmSetArch(arch) => {
            state.disasm_arch = arch;
            if let Some(ref mut disasm) = state.disasm_state {
                disasm.set_arch(arch);
            }
            Task::none()
        }
        Message::DisasmScroll(delta) => {
            if !state.show_disasm || state.disasm_sym_browser {
                return Task::none();
            }
            if let Some(ref disasm) = state.disasm_state {
                if delta == i32::MIN {
                    state.disasm_offset = 0;
                } else if delta == i32::MAX {
                    state.disasm_offset = disasm.bytes().len().saturating_sub(50);
                } else if delta > 0 {
                    // Scroll down: advance by instruction sizes
                    let mut off = state.disasm_offset;
                    for _ in 0..delta.unsigned_abs() {
                        let step = disasm.instruction_size_at(off).unwrap_or(1);
                        off = off.saturating_add(step);
                    }
                    state.disasm_offset = off.min(disasm.bytes().len().saturating_sub(1));
                } else {
                    // Scroll up: step back and align to instruction boundary
                    let abs_delta = delta.unsigned_abs() as usize;
                    // Rough estimate: average x86 instruction is ~3-4 bytes
                    let byte_back = abs_delta * 4;
                    let raw_off = state.disasm_offset.saturating_sub(byte_back);
                    // Align from a nearby anchor
                    let aligned = disasm.align_to_instruction(raw_off);
                    // Disassemble forward from aligned to find the right instruction
                    let lines = disasm.disassemble_range(aligned, abs_delta + 20);
                    // Find the instruction that's `abs_delta` before current offset
                    let mut candidates: Vec<usize> = lines
                        .iter()
                        .filter_map(|l| disasm.address_to_offset(l.address))
                        .filter(|&o| o < state.disasm_offset)
                        .collect();
                    candidates.sort();
                    if candidates.len() >= abs_delta {
                        state.disasm_offset = candidates[candidates.len() - abs_delta];
                    } else if let Some(&first) = candidates.first() {
                        state.disasm_offset = first;
                    } else {
                        state.disasm_offset = aligned;
                    }
                }
            }
            Task::none()
        }
        Message::DisasmToggleAddress => {
            state.disasm_show_address = !state.disasm_show_address;
            state.disasm_context_menu = false;
            Task::none()
        }
        Message::DisasmToggleBytes => {
            state.disasm_show_bytes = !state.disasm_show_bytes;
            state.disasm_context_menu = false;
            Task::none()
        }
        Message::DisasmToggleResolved => {
            state.disasm_show_resolved = !state.disasm_show_resolved;
            state.disasm_context_menu = false;
            Task::none()
        }
        Message::DisasmToggleRawComment => {
            state.disasm_show_raw_comment = !state.disasm_show_raw_comment;
            state.disasm_context_menu = false;
            Task::none()
        }
        Message::DisasmToggleArrows => {
            state.disasm_show_arrows = !state.disasm_show_arrows;
            state.disasm_context_menu = false;
            Task::none()
        }
        Message::DisasmContextMenu => {
            state.disasm_context_menu = !state.disasm_context_menu;
            Task::none()
        }
        Message::DisasmCloseContextMenu => {
            state.disasm_context_menu = false;
            Task::none()
        }
        Message::DisasmToggleSymBrowser => {
            state.disasm_sym_browser = !state.disasm_sym_browser;
            Task::none()
        }
        Message::DisasmSymFilterInput(s) => {
            state.disasm_sym_filter = s;
            Task::none()
        }
        Message::DisasmGotoSymFromBrowser(addr) => {
            if let Some(ref disasm) = state.disasm_state {
                if let Some(off) = disasm.address_to_offset(addr) {
                    state.disasm_offset = off;
                }
            }
            // Keep browser open so user can verify the function
            Task::none()
        }
        Message::DisasmNavigateToAddress(addr) => {
            if let Some(ref disasm) = state.disasm_state {
                if let Some(off) = disasm.address_to_offset(addr) {
                    state.disasm_offset = off;
                }
            }
            Task::none()
        }
        Message::DisasmToggleEditMode => {
            state.disasm_edit_mode = !state.disasm_edit_mode;
            state.disasm_edit_offset = None;
            state.disasm_edit_hex.clear();
            state.disasm_context_menu = false;
            Task::none()
        }
        Message::DisasmEditHexInput(s) => {
            state.disasm_edit_hex = s;
            Task::none()
        }
        Message::DisasmEditHexCommit(offset) => {
            let hex = state.disasm_edit_hex.replace(' ', "");
            let mut bytes_to_write = Vec::new();
            let mut i = 0;
            while i + 1 < hex.len() {
                if let Ok(b) = u8::from_str_radix(&hex[i..i + 2], 16) {
                    bytes_to_write.push(b);
                }
                i += 2;
            }
            if !bytes_to_write.is_empty() {
                if let Some(ref mut disasm) = state.disasm_state {
                    disasm.write_bytes(offset, &bytes_to_write);
                }
            }
            state.disasm_edit_offset = None;
            state.disasm_edit_hex.clear();
            Task::none()
        }
        Message::DisasmCopySelection => {
            if let Some(ref disasm) = state.disasm_state {
                let lines = disasm.disassemble_range(state.disasm_offset, 200);
                let mut output = String::new();
                for line in &lines {
                    if let Some(ref sym) = line.symbol {
                        output.push_str(&format!("<{}>:\n", sym));
                    }
                    if state.disasm_show_address {
                        output.push_str(&format!("{:016X}  ", line.address));
                    }
                    if state.disasm_show_bytes {
                        let hex: String = line.bytes.iter().map(|b| format!("{:02X} ", b)).collect();
                        output.push_str(&format!("{:<24}", hex));
                    }
                    output.push_str(&format!("{:<10} {}", line.mnemonic, line.operands));
                    if state.disasm_show_raw_comment {
                        if let Some(ref c) = line.comment {
                            output.push_str(&format!("  ; {}", c));
                        }
                    }
                    output.push('\n');
                }
                if let Ok(mut clip) = arboard::Clipboard::new() {
                    let _ = clip.set_text(output);
                }
            }
            state.disasm_context_menu = false;
            Task::none()
        }

        Message::DisasmCopyText(s) => {
            if let Ok(mut clip) = arboard::Clipboard::new() {
                let _ = clip.set_text(s);
            }
            Task::none()
        }

        // ── Panel messages ──
        Message::GotoSymbol(line) => {
            // Navigate to the given line in the editor
            let idx = state.tab_manager.active_index();
            if let Some(tc) = state.tab_contents.get_mut(idx) {
                tc.content.perform(text_editor::Action::Move(text_editor::Motion::DocumentStart));
                for _ in 0..line {
                    tc.content.perform(text_editor::Action::Move(text_editor::Motion::Down));
                }
                tc.content.perform(text_editor::Action::Move(text_editor::Motion::Home));
            }
            Task::none()
        }
        Message::CsvSortColumn(col) => {
            if state.csv_sort_column == Some(col) {
                state.csv_sort_ascending = !state.csv_sort_ascending;
            } else {
                state.csv_sort_column = Some(col);
                state.csv_sort_ascending = true;
            }
            Task::none()
        }
        Message::CsvToggleHeaders => {
            state.csv_has_headers = !state.csv_has_headers;
            Task::none()
        }

        // ── Macro ──
        Message::ToggleMacroRecording => {
            if state.macro_recorder.is_recording() {
                state.last_macro = Some(state.macro_recorder.stop_recording("Macro"));
            } else {
                state.macro_recorder.start_recording();
            }
            Task::none()
        }
        Message::PlayLastMacro => {
            if let Some(ref m) = state.last_macro.clone() {
                let doc = state.tab_manager.active_document_mut();
                MacroRecorder::play_macro(m, doc);
                rebuild_content(state);
            }
            Task::none()
        }
        Message::PlayMacroMultiple => {
            if let Some(ref m) = state.last_macro.clone() {
                let doc = state.tab_manager.active_document_mut();
                MacroRecorder::play_macro_n_times(m, doc, 10);
                rebuild_content(state);
            }
            Task::none()
        }

        // ── Settings ──
        Message::ShowPreferences => {
            // Load current settings into pref fields
            let settings = crate::io::settings::AppSettings::load(
                &crate::io::settings::AppSettings::settings_path(),
            )
            .unwrap_or_default();
            state.pref_font_size = settings.font_size;
            state.pref_tab_size = settings.tab_size.to_string();
            state.pref_theme = state.theme.name.clone();
            state.pref_auto_save = settings.auto_save;
            state.pref_word_wrap = settings.word_wrap;
            state.pref_show_line_numbers = settings.show_line_numbers;
            state.pref_show_whitespace = settings.show_whitespace;
            state.pref_font_family = settings.font_family.clone();
            state.pref_line_spacing = format!("{:.1}", settings.line_spacing);
            state.pref_color_bg = settings.color_background.clone().unwrap_or_default();
            state.pref_color_fg = settings.color_foreground.clone().unwrap_or_default();
            state.pref_color_sel = settings.color_selection.clone().unwrap_or_default();
            state.pref_color_caret = settings.color_caret.clone().unwrap_or_default();
            state.show_preferences = true;
            Task::none()
        }
        Message::ShowKeybindings => {
            state.show_keybindings = true;
            Task::none()
        }
        Message::SavePreferences => {
            // Build settings from pref fields and save
            let mut settings = crate::io::settings::AppSettings::load(
                &crate::io::settings::AppSettings::settings_path(),
            )
            .unwrap_or_default();
            settings.font_size = state.pref_font_size;
            settings.tab_size = state.pref_tab_size.parse().unwrap_or(4);
            settings.theme = state.pref_theme.clone();
            settings.auto_save = state.pref_auto_save;
            settings.word_wrap = state.pref_word_wrap;
            settings.show_line_numbers = state.pref_show_line_numbers;
            settings.show_whitespace = state.pref_show_whitespace;
            settings.font_family = state.pref_font_family.clone();
            settings.line_spacing = state.pref_line_spacing.parse().unwrap_or(1.3);
            settings.show_indent_guides = state.show_indent_guides;
            settings.show_line_endings = state.show_line_endings;
            settings.color_background = if state.pref_color_bg.is_empty() { None } else { Some(state.pref_color_bg.clone()) };
            settings.color_foreground = if state.pref_color_fg.is_empty() { None } else { Some(state.pref_color_fg.clone()) };
            settings.color_selection = if state.pref_color_sel.is_empty() { None } else { Some(state.pref_color_sel.clone()) };
            settings.color_caret = if state.pref_color_caret.is_empty() { None } else { Some(state.pref_color_caret.clone()) };
            if let Err(e) = settings.save(&crate::io::settings::AppSettings::settings_path()) {
                log::error!("Failed to save settings: {}", e);
            }
            // Apply to app state
            state.font_size = settings.font_size;
            state.word_wrap = settings.word_wrap;
            state.show_line_numbers = settings.show_line_numbers;
            state.show_whitespace = settings.show_whitespace;
            state.font_family = settings.font_family.clone();
            state.line_spacing = settings.line_spacing;
            state.color_caret = settings.color_caret.clone();
            let mut theme = AppColors::theme_by_name(&state.pref_theme);
            theme.apply_color_overrides(&settings);
            state.theme = theme;
            state.show_preferences = false;
            Task::none()
        }
        Message::CancelPreferences => {
            state.show_preferences = false;
            Task::none()
        }
        Message::CloseKeybindingsDialog => {
            state.show_keybindings = false;
            Task::none()
        }
        Message::PrefFontSizeIncrease => {
            state.pref_font_size = (state.pref_font_size + 1.0).min(48.0);
            Task::none()
        }
        Message::PrefFontSizeDecrease => {
            state.pref_font_size = (state.pref_font_size - 1.0).max(8.0);
            Task::none()
        }
        Message::PrefTabSizeChanged(val) => {
            state.pref_tab_size = val;
            Task::none()
        }
        Message::PrefSetTheme(name) => {
            state.pref_theme = name.clone();
            // Immediately apply the theme
            state.theme = AppColors::theme_by_name(&name);
            Task::none()
        }
        Message::PrefToggleAutoSave(v) => {
            state.pref_auto_save = v;
            Task::none()
        }
        Message::PrefToggleWordWrap(v) => {
            state.pref_word_wrap = v;
            Task::none()
        }
        Message::PrefToggleLineNumbers(v) => {
            state.pref_show_line_numbers = v;
            Task::none()
        }
        Message::PrefToggleWhitespace(v) => {
            state.pref_show_whitespace = v;
            Task::none()
        }
        Message::PrefFontFamilyChanged(val) => {
            state.pref_font_family = val;
            Task::none()
        }
        Message::PrefLineSpacingChanged(val) => {
            state.pref_line_spacing = val;
            Task::none()
        }
        Message::PrefColorBgChanged(val) => {
            state.pref_color_bg = val;
            Task::none()
        }
        Message::PrefColorFgChanged(val) => {
            state.pref_color_fg = val;
            Task::none()
        }
        Message::PrefColorSelChanged(val) => {
            state.pref_color_sel = val;
            Task::none()
        }
        Message::PrefColorCaretChanged(val) => {
            state.pref_color_caret = val;
            Task::none()
        }
        Message::ToggleIndentGuides => {
            state.show_indent_guides = !state.show_indent_guides;
            Task::none()
        }
        Message::ToggleLineEndings => {
            state.show_line_endings = !state.show_line_endings;
            Task::none()
        }
        Message::ImportTheme => {
            let task = Task::perform(
                async {
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("JSON", &["json"])
                        .set_title("Import Theme")
                        .pick_file()
                        .await;
                    match handle {
                        Some(file) => {
                            let path = file.path().to_path_buf();
                            match std::fs::read_to_string(&path) {
                                Ok(contents) => Ok((contents, path)),
                                Err(e) => Err(format!("Failed to read file: {}", e)),
                            }
                        }
                        None => Err("No file selected".into()),
                    }
                },
                Message::ThemeImported,
            );
            return task;
        }
        Message::ThemeImported(result) => {
            if let Ok((contents, _path)) = result {
                match serde_json::from_str::<super::theme::ThemeColors>(&contents) {
                    Ok(tc) => {
                        if let Some(theme) = super::theme::AppTheme::from_theme_colors(&tc) {
                            state.theme = theme;
                            state.pref_theme = state.theme.name.clone();
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to parse theme: {}", e);
                    }
                }
            }
            Task::none()
        }
        Message::ExportTheme => {
            let theme_colors = state.theme.to_theme_colors();
            let task = Task::perform(
                async move {
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("JSON", &["json"])
                        .set_title("Export Theme")
                        .set_file_name("theme.json")
                        .save_file()
                        .await;
                    match handle {
                        Some(file) => {
                            let path = file.path().to_path_buf();
                            match serde_json::to_string_pretty(&theme_colors) {
                                Ok(json) => match std::fs::write(&path, json) {
                                    Ok(()) => Ok(path),
                                    Err(e) => Err(format!("Failed to write: {}", e)),
                                },
                                Err(e) => Err(format!("Failed to serialize: {}", e)),
                            }
                        }
                        None => Err("No file selected".into()),
                    }
                },
                Message::ThemeExported,
            );
            return task;
        }
        Message::ThemeExported(_result) => {
            Task::none()
        }

        // ── Help ──
        Message::ShowAbout => {
            state.show_about = true;
            Task::none()
        }
        Message::CloseAbout => {
            state.show_about = false;
            Task::none()
        }

        // ── Theme ──
        Message::SetTheme(name) => {
            state.theme = AppColors::theme_by_name(&name);
            state.pref_theme = name.clone();
            // Persist theme to settings
            let mut settings = crate::io::settings::AppSettings::load(
                &crate::io::settings::AppSettings::settings_path(),
            )
            .unwrap_or_default();
            settings.theme = name;
            let _ = settings.save(&crate::io::settings::AppSettings::settings_path());
            Task::none()
        }

        // ── Search panel ──
        Message::FindQueryChanged(query) => {
            state.search_query = query;
            run_search(state);
            Task::none()
        }
        Message::ReplaceTextChanged(text) => {
            state.replace_text = text;
            Task::none()
        }
        Message::ToggleCaseSensitive => {
            state.case_sensitive = !state.case_sensitive;
            run_search(state);
            Task::none()
        }
        Message::ToggleWholeWord => {
            state.whole_word = !state.whole_word;
            run_search(state);
            Task::none()
        }
        Message::ToggleRegex => {
            state.use_regex = !state.use_regex;
            run_search(state);
            Task::none()
        }
        Message::FindNext => {
            state.current_match_index = crate::editor::text_transforms::next_match_index(
                state.current_match_index,
                state.search_matches.len(),
            );
            Task::none()
        }
        Message::FindPrev => {
            state.current_match_index = crate::editor::text_transforms::prev_match_index(
                state.current_match_index,
                state.search_matches.len(),
            );
            Task::none()
        }
        Message::ReplaceNext => {
            if let Some(idx) = state.current_match_index {
                if idx < state.search_matches.len() {
                    let m = &state.search_matches[idx];
                    let text = get_buffer_text(state);
                    let mut new_text = String::with_capacity(text.len());
                    new_text.push_str(&text[..m.start]);
                    new_text.push_str(&state.replace_text);
                    new_text.push_str(&text[m.end..]);
                    set_buffer_text(state, &new_text);
                    run_search(state);
                }
            }
            Task::none()
        }
        Message::ReplaceAll => {
            if !state.search_query.is_empty() {
                let text = get_buffer_text(state);
                let mut engine = SearchEngine::new();
                engine.query = state.search_query.clone();
                engine.replace_text = state.replace_text.clone();
                engine.case_sensitive = state.case_sensitive;
                engine.whole_word = state.whole_word;
                engine.use_regex = state.use_regex;
                engine.search_mode = if state.use_regex {
                    SearchMode::Regex
                } else {
                    SearchMode::Normal
                };
                let (new_text, count) = engine.replace_all(&text);
                if count > 0 {
                    set_buffer_text(state, &new_text);
                    run_search(state);
                }
            }
            Task::none()
        }
        Message::CloseSearch => {
            state.show_find = false;
            state.show_replace = false;
            state.search_matches.clear();
            state.current_match_index = None;
            Task::none()
        }
        Message::ClickSearchResult(idx) => {
            if idx < state.search_matches.len() {
                state.current_match_index = Some(idx);
            }
            Task::none()
        }

        // ── Go to line ──
        Message::GotoLineInputChanged(input) => {
            state.goto_line_input = input;
            Task::none()
        }
        Message::GotoLineConfirm => {
            if let Some(target) = crate::editor::text_transforms::parse_goto_line(&state.goto_line_input) {
                state
                    .tab_manager
                    .active_document_mut()
                    .cursor
                    .position
                    .line = target;
                state
                    .tab_manager
                    .active_document_mut()
                    .cursor
                    .position
                    .col = 0;
            }
            state.show_goto_line = false;
            Task::none()
        }
        Message::GotoLineClose => {
            state.show_goto_line = false;
            Task::none()
        }

        // ── Async results ──
        Message::SessionFileChosen(result) => {
            if let Ok(path) = result {
                if let Ok(session) = crate::io::session::load_session(&path) {
                    state.tab_manager.close_all();
                    state.tab_contents.clear();
                    for sf in &session.files {
                        match state.tab_manager.open_file(sf.path.clone()) {
                            Ok(idx) => {
                                let doc = state.tab_manager.get_document(idx).unwrap();
                                let is_binary = doc.is_binary;
                                let buf_text = doc.buffer.text();
                                if is_binary {
                                    state.tab_contents.push(TabContent::with_text("[Binary file — use Disassembler or Hex viewer]"));
                                } else {
                                    state.tab_contents.push(TabContent::with_text(&buf_text));
                                }
                            }
                            Err(_) => {
                                state.tab_manager.new_tab();
                                state.tab_contents.push(TabContent::new());
                            }
                        }
                    }
                    if state.tab_contents.is_empty() {
                        state.tab_contents.push(TabContent::new());
                    }
                }
            }
            Task::none()
        }
        Message::ExportSaved(result) => {
            if let Err(e) = result {
                log::error!("Export failed: {}", e);
            }
            Task::none()
        }
        Message::CompareFileLoaded(result) => {
            match result {
                Ok(other_text) => {
                    let current_text = get_buffer_text(state);
                    let diff_result = crate::tools::diff_tool::diff_texts(&current_text, &other_text);
                    let stats = &diff_result.stats;
                    let header = format!(
                        "=== Diff: {} added, {} removed, {} changed, {} unchanged ===\n\n",
                        stats.added, stats.removed, stats.changed, stats.same
                    );
                    let diff_body = crate::editor::text_transforms::format_diff_output(&diff_result);
                    let diff_text = format!("{}{}", header, diff_body);
                    let idx = state.tab_manager.new_tab();
                    state.tab_manager.set_tab_title(idx, "Diff Result");
                    let doc = state.tab_manager.active_document_mut();
                    let len = doc.buffer.len_bytes();
                    if len > 0 { doc.buffer.delete(0, len); }
                    doc.buffer.insert(0, &diff_text);
                    doc.language = "Diff".to_string();
                    state.tab_contents.push(TabContent::with_text(&diff_text));
                    state.status_message = Some((
                        format!("Diff complete: +{} -{} ~{}", stats.added, stats.removed, stats.changed),
                        std::time::Instant::now(),
                    ));
                }
                Err(e) => {
                    if e != "Cancelled" {
                        state.status_message = Some((format!("Compare failed: {e}"), std::time::Instant::now()));
                    }
                }
            }
            Task::none()
        }

        // ── Find in Files ──
        Message::FifQueryChanged(q) => {
            state.fif_query = q;
            Task::none()
        }
        Message::FifDirectoryChanged(d) => {
            state.fif_directory = d;
            Task::none()
        }
        Message::FifFileFilterChanged(f) => {
            state.fif_file_filter = f;
            Task::none()
        }
        Message::FifToggleRecursive => {
            state.fif_recursive = !state.fif_recursive;
            Task::none()
        }
        Message::FifToggleCaseSensitive => {
            state.fif_case_sensitive = !state.fif_case_sensitive;
            Task::none()
        }
        Message::FifToggleRegex => {
            state.fif_use_regex = !state.fif_use_regex;
            Task::none()
        }
        Message::FifSearch => {
            if state.fif_query.is_empty() || state.fif_directory.is_empty() {
                return Task::none();
            }
            state.fif_searching = true;
            state.fif_results.clear();
            let dir = std::path::PathBuf::from(&state.fif_directory);
            let query = state.fif_query.clone();
            let filter = state.fif_file_filter.clone();
            let recursive = state.fif_recursive;
            let case_sensitive = state.fif_case_sensitive;
            let use_regex = state.fif_use_regex;
            Task::perform(
                async move {
                    crate::search::FindInFiles::search(
                        &dir, &query, &filter, recursive, case_sensitive, use_regex,
                    )
                    .map_err(|e| e.to_string())
                },
                Message::FifSearchComplete,
            )
        }
        Message::FifSearchComplete(result) => {
            state.fif_searching = false;
            match result {
                Ok(results) => {
                    // Add FiF results to the Search Results panel
                    let mut all_matches = Vec::new();
                    for file_result in &results {
                        for m in &file_result.matches {
                            all_matches.push(super::search_results_panel::SearchResultMatch {
                                line: m.line,
                                line_text: format!("{}: {}", file_result.path.display(), m.line_text),
                                file_path: Some(file_result.path.clone()),
                            });
                        }
                    }
                    if !all_matches.is_empty() {
                        state.search_results_panel.add_search(
                            format!("Find in Files: {}", state.fif_query),
                            all_matches,
                        );
                        state.show_search_results_panel = true;
                    }
                    state.fif_results = results;
                }
                Err(e) => log::error!("Find in Files error: {}", e),
            }
            Task::none()
        }
        Message::FifClickResult(path, line) => {
            // Check if the file is already open in a tab
            let mut existing_idx = None;
            for i in 0..state.tab_manager.tab_count() {
                if let Some(doc) = state.tab_manager.get_document(i) {
                    if doc.path.as_ref() == Some(&path) {
                        existing_idx = Some(i);
                        break;
                    }
                }
            }

            if let Some(idx) = existing_idx {
                // Switch to the existing tab
                state.tab_manager.set_active(idx);
            } else {
                // Open the file in a new tab
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            state.file_extension = ext.to_lowercase();
                        }
                        match state.tab_manager.open_file(path) {
                            Ok(idx) => {
                                if !state.file_extension.is_empty() {
                                    let lang = super::highlighter::language_for_extension(&state.file_extension);
                                    state.tab_manager.get_document_mut(idx).unwrap().language = lang;
                                }
                                let doc = state.tab_manager.get_document(idx).unwrap();
                                let buf_text = doc.buffer.text();
                                state.fold_manager.detect_regions(&buf_text);
                                state.tab_contents.push(TabContent::with_text(&buf_text));
                            }
                            Err(_) => {
                                state.tab_manager.new_tab();
                                state.fold_manager.detect_regions(&content);
                                state.tab_contents.push(TabContent::with_text(&content));
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to open file: {}", e);
                        return Task::none();
                    }
                }
            }
            // Go to the matched line
            state.tab_manager.active_document_mut().cursor.position.line = line;
            state.tab_manager.active_document_mut().cursor.position.col = 0;
            Task::none()
        }
        Message::CloseFindInFiles => {
            state.show_find_in_files = false;
            Task::none()
        }

        // ── Keyboard ──
        Message::EscapePressed => {
            if state.disasm_context_menu {
                state.disasm_context_menu = false;
            } else if state.disasm_sym_browser {
                state.disasm_sym_browser = false;
                state.disasm_sym_filter.clear();
            } else if state.editor_context_menu {
                state.editor_context_menu = false;
            } else if state.tab_rename.is_some() {
                state.tab_rename = None;
            } else if state.tab_context_menu.is_some() {
                state.tab_context_menu = None;
            } else if state.show_find {
                state.show_find = false;
                state.show_replace = false;
                state.search_matches.clear();
                state.current_match_index = None;
            } else if state.show_find_in_files {
                state.show_find_in_files = false;
            } else if state.show_goto_line {
                state.show_goto_line = false;
            } else if state.show_about {
                state.show_about = false;
            } else if state.show_preferences {
                state.show_preferences = false;
            } else if state.show_keybindings {
                state.show_keybindings = false;
            } else {
                state.active_menu = None;
            }
            Task::none()
        }

        // ── Drag floating panels ──
        Message::DragFindStart => {
            state.dragging_find_panel = true;
            // last_mouse_pos is local to the title bar mouse_area, so it's
            // already the offset from the panel's top-left corner.
            state.drag_offset = (
                state.last_mouse_pos.x,
                state.last_mouse_pos.y,
            );
            Task::none()
        }
        Message::DragFindMove(point) => {
            state.last_mouse_pos = point;
            if state.dragging_find_panel {
                let x = (point.x - state.drag_offset.0).max(0.0);
                let y = (point.y - state.drag_offset.1).max(0.0);
                state.find_panel_pos = Some((x, y));
            }
            Task::none()
        }
        Message::DragFindEnd => {
            state.dragging_find_panel = false;
            Task::none()
        }
        Message::DragFifStart => {
            state.dragging_fif_panel = true;
            state.fif_drag_offset = (
                state.last_mouse_pos.x,
                state.last_mouse_pos.y,
            );
            Task::none()
        }
        Message::DragFifMove(point) => {
            state.last_mouse_pos = point;
            if state.dragging_fif_panel {
                let x = (point.x - state.fif_drag_offset.0).max(0.0);
                let y = (point.y - state.fif_drag_offset.1).max(0.0);
                state.fif_panel_pos = Some((x, y));
            }
            Task::none()
        }
        Message::DragFifEnd => {
            state.dragging_fif_panel = false;
            Task::none()
        }

        // ── Split pane ──
        Message::SplitEditorAction(action) => {
            state.active_pane = 1;
            if let Some(tc) = state
                .split_tab_contents
                .get_mut(state.split_tab_manager.active_index())
            {
                tc.content.perform(action);
                let new_text = tc.content.text();
                let doc = state.split_tab_manager.active_document_mut();
                let old_text = doc.buffer.text();
                if new_text != old_text {
                    let len = doc.buffer.len_bytes();
                    if len > 0 {
                        doc.buffer.delete(0, len);
                    }
                    doc.buffer.insert(0, &new_text);
                }
            }
            Task::none()
        }
        Message::SplitNewTab => {
            state.active_pane = 1;
            state.split_tab_manager.new_tab();
            state.split_tab_contents.push(TabContent::new());
            Task::none()
        }
        Message::SplitSelectTab(idx) => {
            state.active_pane = 1;
            // Start tab drag for split pane
            state.tab_drag = Some(TabDragState {
                from_index: idx,
                is_split: true,
                target_index: None,
            });
            if idx < state.split_tab_manager.tab_count() {
                state.split_tab_manager.set_active(idx);
                if let Some(path) = &state.split_tab_manager.active_document().path {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        state.split_file_extension = ext.to_lowercase();
                    } else {
                        state.split_file_extension.clear();
                    }
                } else {
                    state.split_file_extension.clear();
                }
            }
            Task::none()
        }
        Message::SplitCloseTab(idx) => {
            state.active_pane = 1;
            let actual_idx = if idx == usize::MAX {
                state.split_tab_manager.active_index()
            } else {
                idx
            };
            if actual_idx < state.split_tab_contents.len() {
                state.split_tab_contents.remove(actual_idx);
                state.split_tab_manager.close_tab(actual_idx);
                while state.split_tab_contents.len() < state.split_tab_manager.tab_count() {
                    state.split_tab_contents.push(TabContent::new());
                }
            }
            // If only the auto-created empty tab remains after closing, close the split
            if state.split_tab_manager.tab_count() == 1 {
                let doc = state.split_tab_manager.active_document();
                let is_empty = doc.buffer.text().trim().is_empty() && doc.path.is_none();
                if is_empty {
                    state.split_mode = SplitMode::None;
                    state.active_pane = 0;
                }
            }
            Task::none()
        }
        Message::SetActivePane(pane) => {
            state.active_pane = pane;
            Task::none()
        }
        Message::NewTabInActivePane => {
            if state.active_pane == 1 && state.split_mode != SplitMode::None {
                state.split_tab_manager.new_tab();
                state.split_tab_contents.push(TabContent::new());
            } else {
                state.tab_manager.new_tab();
                state.tab_contents.push(TabContent::new());
            }
            Task::none()
        }
        Message::FileDropped(path) => {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                let lang = if ext.is_empty() { "Plain Text".to_string() } else { super::highlighter::language_for_extension(&ext) };
                if state.active_pane == 1 && state.split_mode != SplitMode::None {
                    state.split_file_extension = ext;
                    match state.split_tab_manager.open_file(path) {
                        Ok(idx) => {
                            state.split_tab_manager.get_document_mut(idx).unwrap().language = lang;
                            let doc = state.split_tab_manager.get_document(idx).unwrap();
                            let buf_text = doc.buffer.text();
                            state.split_tab_contents.push(TabContent::with_text(&buf_text));
                        }
                        Err(_) => {
                            state.split_tab_manager.new_tab();
                            state.split_tab_contents.push(TabContent::with_text(&content));
                        }
                    }
                } else {
                    state.file_extension = ext;
                    match state.tab_manager.open_file(path) {
                        Ok(idx) => {
                            state.tab_manager.get_document_mut(idx).unwrap().language = lang;
                            let doc = state.tab_manager.get_document(idx).unwrap();
                            let buf_text = doc.buffer.text();
                            state.fold_manager.detect_regions(&buf_text);
                            state.tab_contents.push(TabContent::with_text(&buf_text));
                        }
                        Err(_) => {
                            state.tab_manager.new_tab();
                            state.fold_manager.detect_regions(&content);
                            state.tab_contents.push(TabContent::with_text(&content));
                        }
                    }
                }
            }
            Task::none()
        }
        Message::MoveTabToSplit(idx) => {
            // Move tab at idx from primary pane to split pane
            if idx >= state.tab_contents.len() || idx >= state.tab_manager.tab_count() {
                return Task::none();
            }
            // Ensure split exists
            if state.split_mode == SplitMode::None {
                state.split_mode = SplitMode::Horizontal;
                state.split_tab_manager = TabManager::new();
                state.split_tab_contents = vec![TabContent::new()];
                state.split_font_size = state.font_size;
                state.split_file_extension.clear();
            }
            // Sync content before moving
            sync_content_to_doc(state, idx);
            let tab_content = state.tab_contents.remove(idx);
            if let Some(doc) = state.tab_manager.remove_document(idx) {
                if let Some(ext) = doc.path.as_ref().and_then(|p| p.extension()).and_then(|e| e.to_str()) {
                    state.split_file_extension = ext.to_lowercase();
                }
                // Remove the default empty tab in split if it's the only one and is empty
                if state.split_tab_manager.tab_count() == 1 {
                    let split_doc = state.split_tab_manager.active_document();
                    if split_doc.buffer.text().trim().is_empty() && split_doc.path.is_none() {
                        state.split_tab_contents.remove(0);
                        state.split_tab_manager.remove_document(0);
                    }
                }
                state.split_tab_manager.add_document(doc);
                state.split_tab_contents.push(tab_content);
            }
            // If primary has no tabs left, create one
            if state.tab_manager.tab_count() == 0 {
                state.tab_manager.new_tab();
                state.tab_contents.push(TabContent::new());
            }
            state.active_pane = 1;
            Task::none()
        }
        Message::MoveTabFromSplit(idx) => {
            // Move tab at idx from split pane to primary pane
            if state.split_mode == SplitMode::None {
                return Task::none();
            }
            if idx >= state.split_tab_contents.len() || idx >= state.split_tab_manager.tab_count() {
                return Task::none();
            }
            // Sync split content before moving
            sync_split_content_to_doc(state, idx);
            let tab_content = state.split_tab_contents.remove(idx);
            if let Some(doc) = state.split_tab_manager.remove_document(idx) {
                if let Some(ext) = doc.path.as_ref().and_then(|p| p.extension()).and_then(|e| e.to_str()) {
                    state.file_extension = ext.to_lowercase();
                }
                state.tab_manager.add_document(doc);
                state.tab_contents.push(tab_content);
            }
            // If split has no tabs left, close the split
            if state.split_tab_manager.tab_count() == 0 {
                state.split_mode = SplitMode::None;
                state.split_tab_manager = TabManager::new();
                state.split_tab_contents = vec![TabContent::new()];
                state.active_pane = 0;
            } else {
                // Check if only an empty tab remains
                if state.split_tab_manager.tab_count() == 1 {
                    let split_doc = state.split_tab_manager.active_document();
                    if split_doc.buffer.text().trim().is_empty() && split_doc.path.is_none() {
                        state.split_mode = SplitMode::None;
                        state.active_pane = 0;
                    }
                }
            }
            Task::none()
        }

        // ── Tab context menu ──
        Message::TabContextMenu(idx, is_split) => {
            state.tab_context_menu = Some((idx, is_split));
            Task::none()
        }
        Message::CloseTabContextMenu => {
            state.tab_context_menu = None;
            Task::none()
        }
        Message::TabContextClose(idx, is_split) => {
            state.tab_context_menu = None;
            if is_split {
                return update(state, Message::SplitCloseTab(idx));
            } else {
                return update(state, Message::CloseTab(idx));
            }
        }
        Message::TabContextCloseOthers(idx, is_split) => {
            state.tab_context_menu = None;
            if is_split {
                // Close all split tabs except idx
                let mut i = state.split_tab_manager.tab_count();
                while i > 0 {
                    i -= 1;
                    if i != idx && i < state.split_tab_contents.len() {
                        state.split_tab_contents.remove(i);
                        state.split_tab_manager.close_tab(i);
                    }
                }
                while state.split_tab_contents.len() < state.split_tab_manager.tab_count() {
                    state.split_tab_contents.push(TabContent::new());
                }
            } else {
                let mut i = state.tab_manager.tab_count();
                while i > 0 {
                    i -= 1;
                    if i != idx && i < state.tab_contents.len() {
                        state.tab_contents.remove(i);
                        state.tab_manager.close_tab(i);
                    }
                }
                while state.tab_contents.len() < state.tab_manager.tab_count() {
                    state.tab_contents.push(TabContent::new());
                }
            }
            Task::none()
        }
        Message::TabContextCloseAll(is_split) => {
            state.tab_context_menu = None;
            if is_split {
                state.split_tab_manager.close_all();
                state.split_tab_contents.clear();
                state.split_tab_contents.push(TabContent::new());
                // Close the split entirely
                state.split_mode = SplitMode::None;
                state.active_pane = 0;
            } else {
                state.tab_manager.close_all();
                state.tab_contents.clear();
                state.tab_contents.push(TabContent::new());
            }
            Task::none()
        }
        Message::TabContextCloseToLeft(idx, is_split) => {
            state.tab_context_menu = None;
            if is_split {
                let count_before = idx.min(state.split_tab_contents.len());
                state.split_tab_contents.drain(..count_before);
                state.split_tab_manager.close_tabs_to_left(idx);
            } else {
                let count_before = idx.min(state.tab_contents.len());
                state.tab_contents.drain(..count_before);
                state.tab_manager.close_tabs_to_left(idx);
            }
            Task::none()
        }
        Message::TabContextCloseToRight(idx, is_split) => {
            state.tab_context_menu = None;
            if is_split {
                let mut i = state.split_tab_manager.tab_count();
                while i > idx + 1 {
                    i -= 1;
                    if i < state.split_tab_contents.len() {
                        state.split_tab_contents.remove(i);
                    }
                    state.split_tab_manager.close_tab(i);
                }
            } else {
                let mut i = state.tab_manager.tab_count();
                while i > idx + 1 {
                    i -= 1;
                    if i < state.tab_contents.len() {
                        state.tab_contents.remove(i);
                    }
                    state.tab_manager.close_tab(i);
                }
            }
            Task::none()
        }
        Message::TabContextCloseUnmodified(is_split) => {
            state.tab_context_menu = None;
            if is_split {
                let mut i = state.split_tab_manager.tab_count();
                while i > 0 {
                    i -= 1;
                    if let Some(doc) = state.split_tab_manager.get_document(i) {
                        if !doc.is_modified() {
                            if i < state.split_tab_contents.len() {
                                state.split_tab_contents.remove(i);
                            }
                            state.split_tab_manager.close_tab(i);
                        }
                    }
                }
                if state.split_tab_manager.tab_count() == 0 {
                    state.split_mode = SplitMode::None;
                    state.active_pane = 0;
                }
            } else {
                let mut i = state.tab_manager.tab_count();
                while i > 0 {
                    i -= 1;
                    if let Some(doc) = state.tab_manager.get_document(i) {
                        if !doc.is_modified() {
                            if i < state.tab_contents.len() {
                                state.tab_contents.remove(i);
                            }
                            state.tab_manager.close_tab(i);
                        }
                    }
                }
            }
            Task::none()
        }
        Message::TabContextOpenInExplorer(idx, is_split) => {
            state.tab_context_menu = None;
            let mgr = if is_split { &state.split_tab_manager } else { &state.tab_manager };
            if let Some(doc) = mgr.get_document(idx) {
                if let Some(ref path) = doc.path {
                    if let Some(dir) = path.parent() {
                        let dir = dir.to_path_buf();
                        #[cfg(target_os = "linux")]
                        { let _ = std::process::Command::new("xdg-open").arg(&dir).spawn(); }
                        #[cfg(target_os = "windows")]
                        { let _ = std::process::Command::new("explorer.exe").arg(&dir).spawn(); }
                        #[cfg(target_os = "macos")]
                        { let _ = std::process::Command::new("open").arg(&dir).spawn(); }
                    }
                }
            }
            Task::none()
        }
        Message::TabContextOpenTerminalHere(idx, is_split) => {
            state.tab_context_menu = None;
            let mgr = if is_split { &state.split_tab_manager } else { &state.tab_manager };
            if let Some(doc) = mgr.get_document(idx) {
                if let Some(ref path) = doc.path {
                    if let Some(dir) = path.parent() {
                        let dir = dir.to_path_buf();
                        #[cfg(target_os = "linux")]
                        {
                            use std::os::unix::process::CommandExt;
                            let _ = std::process::Command::new("setsid")
                                .arg("x-terminal-emulator")
                                .current_dir(&dir)
                                .stdin(std::process::Stdio::null())
                                .stdout(std::process::Stdio::null())
                                .stderr(std::process::Stdio::null())
                                .process_group(0)
                                .spawn();
                        }
                        #[cfg(target_os = "windows")]
                        { let _ = std::process::Command::new("cmd.exe").current_dir(&dir).spawn(); }
                        #[cfg(target_os = "macos")]
                        { let _ = std::process::Command::new("open").arg("-a").arg("Terminal").arg(&dir).spawn(); }
                    }
                }
            }
            Task::none()
        }
        Message::TabContextCopyFilename(idx, is_split) => {
            state.tab_context_menu = None;
            let mgr = if is_split { &state.split_tab_manager } else { &state.tab_manager };
            if let Some(doc) = mgr.get_document(idx) {
                if let Some(ref path) = doc.path {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if let Ok(mut clip) = arboard::Clipboard::new() {
                            let _ = clip.set_text(name.to_string());
                        }
                    }
                }
            }
            Task::none()
        }
        Message::TabContextCopyFullPath(idx, is_split) => {
            state.tab_context_menu = None;
            let mgr = if is_split { &state.split_tab_manager } else { &state.tab_manager };
            if let Some(doc) = mgr.get_document(idx) {
                if let Some(ref path) = doc.path {
                    if let Ok(mut clip) = arboard::Clipboard::new() {
                        let _ = clip.set_text(path.to_string_lossy().to_string());
                    }
                }
            }
            Task::none()
        }
        Message::TabContextRename(idx, is_split) => {
            state.tab_context_menu = None;
            let mgr = if is_split { &state.split_tab_manager } else { &state.tab_manager };
            let current_name = mgr.get_tab_title(idx);
            state.tab_rename = Some((idx, is_split, current_name));
            Task::none()
        }
        Message::TabContextRenameInput(val) => {
            if let Some((idx, is_split, _)) = state.tab_rename.take() {
                state.tab_rename = Some((idx, is_split, val));
            }
            Task::none()
        }
        Message::TabContextRenameConfirm => {
            if let Some((idx, is_split, new_name)) = state.tab_rename.take() {
                let mgr = if is_split { &mut state.split_tab_manager } else { &mut state.tab_manager };
                if let Some(doc) = mgr.get_document(idx) {
                    if let Some(ref old_path) = doc.path.clone() {
                        let new_path = old_path.with_file_name(&new_name);
                        if !new_name.is_empty() && new_path != *old_path {
                            if std::fs::rename(old_path, &new_path).is_ok() {
                                if let Some(doc_mut) = mgr.get_document_mut(idx) {
                                    doc_mut.path = Some(new_path);
                                }
                            }
                        }
                    }
                }
            }
            Task::none()
        }
        Message::TabContextRenameCancel => {
            state.tab_rename = None;
            Task::none()
        }
        Message::TabContextMoveToOtherPane(idx, is_split) => {
            state.tab_context_menu = None;
            if is_split {
                return update(state, Message::MoveTabFromSplit(idx));
            } else {
                return update(state, Message::MoveTabToSplit(idx));
            }
        }
        Message::TabContextCloneToOtherView(idx, is_split) => {
            state.tab_context_menu = None;
            if state.split_mode == SplitMode::None {
                // Activate split view first
                state.split_mode = SplitMode::Horizontal;
                state.split_tab_manager = TabManager::new();
                state.split_tab_contents = vec![TabContent::new()];
                state.split_font_size = state.font_size;
                state.split_file_extension.clear();
            }
            if is_split {
                // Clone from split to primary
                if let Some(doc) = state.split_tab_manager.get_document(idx) {
                    let text = doc.buffer.text();
                    let cloned = Document::from_str(&text)
                        .with_encoding(doc.encoding)
                        .with_line_ending(doc.line_ending);
                    let cloned = if let Some(ref p) = doc.path {
                        cloned.with_path(p.clone())
                    } else {
                        cloned
                    };
                    if let Some(ext) = cloned.path.as_ref().and_then(|p| p.extension()).and_then(|e| e.to_str()) {
                        state.file_extension = ext.to_lowercase();
                    }
                    state.tab_manager.add_document(cloned);
                    state.tab_contents.push(TabContent::with_text(&text));
                }
            } else {
                // Clone from primary to split
                if let Some(doc) = state.tab_manager.get_document(idx) {
                    let text = doc.buffer.text();
                    let cloned = Document::from_str(&text)
                        .with_encoding(doc.encoding)
                        .with_line_ending(doc.line_ending);
                    let cloned = if let Some(ref p) = doc.path {
                        cloned.with_path(p.clone())
                    } else {
                        cloned
                    };
                    if let Some(ext) = cloned.path.as_ref().and_then(|p| p.extension()).and_then(|e| e.to_str()) {
                        state.split_file_extension = ext.to_lowercase();
                    }
                    // Remove default empty tab if it's the only one
                    if state.split_tab_manager.tab_count() == 1 {
                        let split_doc = state.split_tab_manager.active_document();
                        if split_doc.buffer.text().trim().is_empty() && split_doc.path.is_none() {
                            state.split_tab_contents.remove(0);
                            state.split_tab_manager.remove_document(0);
                        }
                    }
                    state.split_tab_manager.add_document(cloned);
                    state.split_tab_contents.push(TabContent::with_text(&text));
                }
            }
            Task::none()
        }

        // ── Middle-click tab close ──
        Message::MiddleClickTab(idx, is_split) => {
            if is_split {
                return update(state, Message::SplitCloseTab(idx));
            } else {
                return update(state, Message::CloseTab(idx));
            }
        }

        // ── Tab drag reordering ──
        Message::TabDragStart(idx, is_split) => {
            state.tab_drag = Some(TabDragState {
                from_index: idx,
                is_split,
                target_index: None,
            });
            Task::none()
        }
        Message::TabDragOver(target_idx, is_split) => {
            if let Some(ref mut drag) = state.tab_drag {
                if drag.is_split == is_split {
                    drag.target_index = Some(target_idx);
                }
            }
            Task::none()
        }
        Message::TabDragEnd => {
            if let Some(drag) = state.tab_drag.take() {
                if let Some(to) = drag.target_index {
                    if drag.from_index != to {
                        if drag.is_split {
                            state.split_tab_manager.move_tab(drag.from_index, to);
                            // Reorder tab_contents to match
                            let content = state.split_tab_contents.remove(drag.from_index);
                            state.split_tab_contents.insert(to, content);
                        } else {
                            state.tab_manager.move_tab(drag.from_index, to);
                            let content = state.tab_contents.remove(drag.from_index);
                            state.tab_contents.insert(to, content);
                        }
                    }
                }
            }
            Task::none()
        }
        Message::TabDragCancel => {
            state.tab_drag = None;
            Task::none()
        }
    }
}

pub fn view(state: &NotepadIced) -> Element<'_, Message> {
    let menu_bar = menu_bar::view_menu_bar(&state.active_menu, &state.theme);

    // Build tab bar row: if split is active, show both tab bars side by side
    let tab_bar_area: Element<'_, Message> = if state.split_mode != SplitMode::None {
        let primary_tab_bar = view_tab_bar(state);
        let split_tab_bar = view_split_tab_bar(state);
        match state.split_mode {
            SplitMode::Horizontal => {
                row![
                    container(primary_tab_bar).width(Length::FillPortion(1)),
                    container(split_tab_bar).width(Length::FillPortion(1)),
                ]
                .into()
            }
            SplitMode::Vertical => {
                column![primary_tab_bar, split_tab_bar].into()
            }
            SplitMode::None => unreachable!(),
        }
    } else {
        view_tab_bar(state)
    };

    // Build the main content area based on active viewer panels
    let main_area: Element<'_, Message> = if state.show_csv_viewer {
        super::csv_panel::view_csv_viewer(state, &state.theme)
    } else if state.show_disasm {
        super::disasm_panel::view_disasm_panel(state, &state.theme)
    } else if state.show_hex_viewer {
        super::hex_panel::view_hex_viewer(state, &state.theme)
    } else {
        // Normal editor, possibly with side panels
        let editor = view_editor(state);

        // Wrap primary editor with its own markdown preview if enabled
        let primary_with_md: Element<'_, Message> = if state.show_markdown_preview {
            let primary_content = state
                .tab_contents
                .get(state.tab_manager.active_index())
                .map(|tc| tc.content.text())
                .unwrap_or_default();
            row![
                container(editor).width(Length::FillPortion(1)),
                super::markdown_panel::view_markdown_preview_for_text(&primary_content, &state.theme),
            ]
            .height(Length::Fill)
            .into()
        } else {
            editor
        };

        // Wrap primary editor in mouse_area for active pane tracking + right-click context menu
        let primary_pane: Element<'_, Message> = mouse_area(primary_with_md)
            .on_press(Message::SetActivePane(0))
            .on_right_press(Message::EditorContextMenu)
            .into();

        // Wrap editor with split view if active
        let editor_area = if state.split_mode != SplitMode::None {
            let split_editor = view_split_pane(state);

            // Wrap split pane with its own markdown preview if enabled
            let split_with_md: Element<'_, Message> = if state.split_show_markdown_preview {
                let split_content = state
                    .split_tab_contents
                    .get(state.split_tab_manager.active_index())
                    .map(|tc| tc.content.text())
                    .unwrap_or_default();
                row![
                    container(split_editor).width(Length::FillPortion(1)),
                    super::markdown_panel::view_markdown_preview_for_text(&split_content, &state.theme),
                ]
                .height(Length::Fill)
                .into()
            } else {
                split_editor
            };

            let border_color = state.theme.border;
            let divider = container(Space::new(
                if state.split_mode == SplitMode::Horizontal { Length::Fixed(2.0) } else { Length::Fill },
                if state.split_mode == SplitMode::Vertical { Length::Fixed(2.0) } else { Length::Fill },
            ))
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(border_color)),
                ..Default::default()
            });

            match state.split_mode {
                SplitMode::Horizontal => {
                    row![
                        container(primary_pane).width(Length::FillPortion(1)).height(Length::Fill),
                        divider,
                        container(split_with_md).width(Length::FillPortion(1)).height(Length::Fill),
                    ]
                    .height(Length::Fill)
                    .into()
                }
                SplitMode::Vertical => {
                    column![
                        container(primary_pane).width(Length::Fill).height(Length::FillPortion(1)),
                        divider,
                        container(split_with_md).width(Length::Fill).height(Length::FillPortion(1)),
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                }
                SplitMode::None => unreachable!(),
            }
        } else {
            primary_pane
        };

        if state.show_function_list || state.show_minimap {
            let mut main_row = row![container(editor_area).width(Length::Fill)].height(Length::Fill);
            if state.show_minimap {
                main_row = main_row.push(super::minimap_panel::view_minimap(state, &state.theme));
            }
            if state.show_function_list {
                main_row = main_row.push(super::function_list_panel::view_function_list(state, &state.theme));
            }
            main_row.into()
        } else {
            editor_area
        }
    };

    let mut base_content = column![menu_bar];

    if state.show_toolbar {
        base_content = base_content.push(super::toolbar::view_toolbar(state));
    }

    base_content = base_content.push(tab_bar_area);
    base_content = base_content.push(main_area);

    // Search results panel (docked below editor)
    if state.show_search_results_panel {
        base_content = base_content.push(
            super::search_results_panel::view_search_results_panel(state, &state.theme)
        );
    }

    if state.show_status_bar {
        base_content = base_content.push(view_status_bar(state));
    }

    let base: Element<'_, Message> = container(base_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    // Check if any overlay is needed
    let has_dropdown = state.active_menu.is_some();
    let has_floating = state.show_find || state.show_goto_line || state.show_about || state.show_find_in_files || state.show_preferences || state.show_keybindings;
    let has_context_menu = state.tab_context_menu.is_some();
    let has_editor_context_menu = state.editor_context_menu;

    if !has_dropdown && !has_floating && !has_context_menu && !has_editor_context_menu {
        return base;
    }

    let mut layers: Vec<Element<'_, Message>> = vec![base];

    // Dropdown menu: needs a click-catcher (modal — clicking outside closes it)
    if has_dropdown {
        let click_catcher: Element<'_, Message> = mouse_area(
            container(Space::new(Length::Fill, Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Message::MenuClose)
        .into();
        layers.push(click_catcher);

        if let Some(ref menu_name) = state.active_menu {
            let dropdown = scrollable(menu_bar::view_dropdown(state, menu_name))
                .height(Length::Shrink);

            let left_offset = menu_bar::menu_x_offset(menu_name);

            let tab_bar_bg = state.theme.tab_bar_bg;
            let dropdown_border = state.theme.border;
            let dropdown_overlay: Element<'_, Message> = container(
                opaque(
                    container(dropdown)
                        .max_height(500)
                        .style(move |_theme: &Theme| container::Style {
                            background: Some(iced::Background::Color(tab_bar_bg)),
                            border: iced::Border {
                                color: dropdown_border,
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        }),
                ),
            )
            .padding(iced::Padding { top: 26.0, right: 0.0, bottom: 0.0, left: left_offset })
            .width(Length::Shrink)
            .height(Length::Shrink)
            .into();

            layers.push(dropdown_overlay);
        }
    }

    // Find/Replace — floating non-modal draggable window
    if state.show_find {
        // If dragging, add a full-window mouse tracking layer
        if state.dragging_find_panel {
            let drag_tracker: Element<'_, Message> = mouse_area(
                container(Space::new(Length::Fill, Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .on_move(Message::DragFindMove)
            .on_release(Message::DragFindEnd)
            .into();
            layers.push(drag_tracker);
        }

        let search_widget = opaque(super::search_panel::view_search_panel(state, &state.theme));

        let search_overlay: Element<'_, Message> = match state.find_panel_pos {
            Some((x, y)) => {
                // Absolute position via padding
                container(search_widget)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(iced::Padding { top: y, right: 0.0, bottom: 0.0, left: x })
                    .into()
            }
            None => {
                // Default: right-aligned, below tabs/menu
                container(search_widget)
                    .padding(iced::Padding { top: 60.0, right: 16.0, bottom: 0.0, left: 0.0 })
                    .align_right(Length::Fill)
                    .height(Length::Shrink)
                    .into()
            }
        };

        layers.push(search_overlay);
    }

    // Find in Files — floating non-modal draggable window
    if state.show_find_in_files {
        if state.dragging_fif_panel {
            let drag_tracker: Element<'_, Message> = mouse_area(
                container(Space::new(Length::Fill, Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .on_move(Message::DragFifMove)
            .on_release(Message::DragFifEnd)
            .into();
            layers.push(drag_tracker);
        }

        let fif_widget = opaque(super::find_in_files_panel::view_find_in_files_panel(state, &state.theme));

        let fif_overlay: Element<'_, Message> = match state.fif_panel_pos {
            Some((x, y)) => {
                container(fif_widget)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(iced::Padding { top: y, right: 0.0, bottom: 0.0, left: x })
                    .into()
            }
            None => {
                container(fif_widget)
                    .padding(iced::Padding { top: 60.0, right: 16.0, bottom: 0.0, left: 0.0 })
                    .align_right(Length::Fill)
                    .height(Length::Shrink)
                    .into()
            }
        };

        layers.push(fif_overlay);
    }

    // Go to Line — floating non-modal, centered
    if state.show_goto_line {
        let goto_overlay: Element<'_, Message> = container(
            opaque(super::goto_dialog::view_goto_dialog(state, &state.theme)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();

        layers.push(goto_overlay);
    }

    // About — floating modal with backdrop, centered
    if state.show_about {
        // Semi-transparent backdrop
        let backdrop: Element<'_, Message> = mouse_area(
            container(Space::new(Length::Fill, Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4))),
                    ..Default::default()
                }),
        )
        .on_press(Message::CloseAbout)
        .into();
        layers.push(backdrop);

        let about_overlay: Element<'_, Message> = container(
            opaque(super::about_dialog::view_about_dialog(&state.theme)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();

        layers.push(about_overlay);
    }

    // Preferences — floating centered
    if state.show_preferences {
        let pref_overlay: Element<'_, Message> = container(
            opaque(super::preferences_dialog::view_preferences_dialog(state, &state.theme)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();

        layers.push(pref_overlay);
    }

    // Keybindings — floating centered
    if state.show_keybindings {
        let kb_overlay: Element<'_, Message> = container(
            opaque(super::keybindings_dialog::view_keybindings_dialog(&state.theme)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();

        layers.push(kb_overlay);
    }

    // Tab context menu
    if let Some((tab_idx, is_split)) = state.tab_context_menu {
        // Click catcher to close menu
        let click_catcher: Element<'_, Message> = mouse_area(
            container(Space::new(Length::Fill, Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Message::CloseTabContextMenu)
        .into();
        layers.push(click_catcher);

        let has_split_active = state.split_mode != SplitMode::None;
        let t_menu_bg = state.theme.menu_bg;
        let t_menu_hover = state.theme.menu_hover;
        let t_text = state.theme.text;
        let t_border = state.theme.border;

        let mut menu_items: Vec<Element<'_, Message>> = Vec::new();

        if has_split_active {
            let label = if is_split { "Move to Primary Pane" } else { "Move to Other Pane" };
            menu_items.push(
                button(text(label).size(13))
                    .on_press(Message::TabContextMoveToOtherPane(tab_idx, is_split))
                    .width(Length::Fill)
                    .padding([3, 8])
                    .style(move |_theme: &Theme, status| {
                        let bg = match status {
                            button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                            _ => None,
                        };
                        button::Style {
                            background: bg,
                            text_color: t_text,
                            border: iced::Border { radius: 2.0.into(), ..Default::default() },
                            ..Default::default()
                        }
                    })
                    .into(),
            );
        }

        // Clone to Other View
        menu_items.push(
            button(text("Clone to Other View").size(13))
                .on_press(Message::TabContextCloneToOtherView(tab_idx, is_split))
                .width(Length::Fill)
                .padding([3, 8])
                .style(move |_theme: &Theme, status| {
                    let bg = match status {
                        button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                        _ => None,
                    };
                    button::Style {
                        background: bg,
                        text_color: t_text,
                        border: iced::Border { radius: 2.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                })
                .into(),
        );

        menu_items.push(
            button(text("Close").size(13))
                .on_press(Message::TabContextClose(tab_idx, is_split))
                .width(Length::Fill)
                .padding([3, 8])
                .style(move |_theme: &Theme, status| {
                    let bg = match status {
                        button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                        _ => None,
                    };
                    button::Style {
                        background: bg,
                        text_color: t_text,
                        border: iced::Border { radius: 2.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                })
                .into(),
        );

        menu_items.push(
            button(text("Close Other Tabs").size(13))
                .on_press(Message::TabContextCloseOthers(tab_idx, is_split))
                .width(Length::Fill)
                .padding([3, 8])
                .style(move |_theme: &Theme, status| {
                    let bg = match status {
                        button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                        _ => None,
                    };
                    button::Style {
                        background: bg,
                        text_color: t_text,
                        border: iced::Border { radius: 2.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                })
                .into(),
        );

        menu_items.push(
            button(text("Close All").size(13))
                .on_press(Message::TabContextCloseAll(is_split))
                .width(Length::Fill)
                .padding([3, 8])
                .style(move |_theme: &Theme, status| {
                    let bg = match status {
                        button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                        _ => None,
                    };
                    button::Style {
                        background: bg,
                        text_color: t_text,
                        border: iced::Border { radius: 2.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                })
                .into(),
        );

        menu_items.push(
            button(text("Close All to the Left").size(13))
                .on_press(Message::TabContextCloseToLeft(tab_idx, is_split))
                .width(Length::Fill)
                .padding([3, 8])
                .style(move |_theme: &Theme, status| {
                    let bg = match status {
                        button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                        _ => None,
                    };
                    button::Style {
                        background: bg,
                        text_color: t_text,
                        border: iced::Border { radius: 2.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                })
                .into(),
        );

        menu_items.push(
            button(text("Close All to the Right").size(13))
                .on_press(Message::TabContextCloseToRight(tab_idx, is_split))
                .width(Length::Fill)
                .padding([3, 8])
                .style(move |_theme: &Theme, status| {
                    let bg = match status {
                        button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                        _ => None,
                    };
                    button::Style {
                        background: bg,
                        text_color: t_text,
                        border: iced::Border { radius: 2.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                })
                .into(),
        );

        menu_items.push(
            button(text("Close All Without Changes").size(13))
                .on_press(Message::TabContextCloseUnmodified(is_split))
                .width(Length::Fill)
                .padding([3, 8])
                .style(move |_theme: &Theme, status| {
                    let bg = match status {
                        button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                        _ => None,
                    };
                    button::Style {
                        background: bg,
                        text_color: t_text,
                        border: iced::Border { radius: 2.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                })
                .into(),
        );

        // Separator
        menu_items.push(
            container(Space::new(Length::Fill, 1))
                .width(Length::Fill)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(t_border)),
                    ..Default::default()
                })
                .padding([4, 8])
                .into(),
        );

        // File operations (only for tabs with a file path)
        let mgr = if is_split { &state.split_tab_manager } else { &state.tab_manager };
        let has_path = mgr.get_document(tab_idx).and_then(|d| d.path.as_ref()).is_some();

        if has_path {
            menu_items.push(
                button(text("Open in File Manager").size(13))
                    .on_press(Message::TabContextOpenInExplorer(tab_idx, is_split))
                    .width(Length::Fill)
                    .padding([3, 8])
                    .style(move |_theme: &Theme, status| {
                        let bg = match status {
                            button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                            _ => None,
                        };
                        button::Style {
                            background: bg,
                            text_color: t_text,
                            border: iced::Border { radius: 2.0.into(), ..Default::default() },
                            ..Default::default()
                        }
                    })
                    .into(),
            );

            menu_items.push(
                button(text("Open Terminal Here").size(13))
                    .on_press(Message::TabContextOpenTerminalHere(tab_idx, is_split))
                    .width(Length::Fill)
                    .padding([3, 8])
                    .style(move |_theme: &Theme, status| {
                        let bg = match status {
                            button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                            _ => None,
                        };
                        button::Style {
                            background: bg,
                            text_color: t_text,
                            border: iced::Border { radius: 2.0.into(), ..Default::default() },
                            ..Default::default()
                        }
                    })
                    .into(),
            );

            // Separator
            menu_items.push(
                container(Space::new(Length::Fill, 1))
                    .width(Length::Fill)
                    .style(move |_theme: &Theme| container::Style {
                        background: Some(iced::Background::Color(t_border)),
                        ..Default::default()
                    })
                    .padding([4, 8])
                    .into(),
            );

            menu_items.push(
                button(text("Copy Filename").size(13))
                    .on_press(Message::TabContextCopyFilename(tab_idx, is_split))
                    .width(Length::Fill)
                    .padding([3, 8])
                    .style(move |_theme: &Theme, status| {
                        let bg = match status {
                            button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                            _ => None,
                        };
                        button::Style {
                            background: bg,
                            text_color: t_text,
                            border: iced::Border { radius: 2.0.into(), ..Default::default() },
                            ..Default::default()
                        }
                    })
                    .into(),
            );

            menu_items.push(
                button(text("Copy Full Path").size(13))
                    .on_press(Message::TabContextCopyFullPath(tab_idx, is_split))
                    .width(Length::Fill)
                    .padding([3, 8])
                    .style(move |_theme: &Theme, status| {
                        let bg = match status {
                            button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                            _ => None,
                        };
                        button::Style {
                            background: bg,
                            text_color: t_text,
                            border: iced::Border { radius: 2.0.into(), ..Default::default() },
                            ..Default::default()
                        }
                    })
                    .into(),
            );

            // Separator
            menu_items.push(
                container(Space::new(Length::Fill, 1))
                    .width(Length::Fill)
                    .style(move |_theme: &Theme| container::Style {
                        background: Some(iced::Background::Color(t_border)),
                        ..Default::default()
                    })
                    .padding([4, 8])
                    .into(),
            );

            menu_items.push(
                button(text("Rename").size(13))
                    .on_press(Message::TabContextRename(tab_idx, is_split))
                    .width(Length::Fill)
                    .padding([3, 8])
                    .style(move |_theme: &Theme, status| {
                        let bg = match status {
                            button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                            _ => None,
                        };
                        button::Style {
                            background: bg,
                            text_color: t_text,
                            border: iced::Border { radius: 2.0.into(), ..Default::default() },
                            ..Default::default()
                        }
                    })
                    .into(),
            );
        }

        let context_menu = container(
            column(menu_items).spacing(0).padding(4).width(Length::Fixed(200.0)),
        )
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_menu_bg)),
            border: iced::Border {
                color: t_border,
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: iced::Vector::new(2.0, 2.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        });

        let context_overlay: Element<'_, Message> = container(opaque(context_menu))
            .padding(iced::Padding { top: 28.0, right: 0.0, bottom: 0.0, left: 100.0 })
            .width(Length::Shrink)
            .height(Length::Shrink)
            .into();

        layers.push(context_overlay);
    }

    // Rename dialog overlay
    if let Some((_, _, ref rename_input)) = state.tab_rename {
        let t_menu_bg = state.theme.menu_bg;
        let t_text = state.theme.text;
        let t_border = state.theme.border;
        let t_menu_hover = state.theme.menu_hover;
        let rename_val = rename_input.clone();

        let backdrop: Element<'_, Message> = mouse_area(
            container(Space::new(Length::Fill, Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4))),
                    ..Default::default()
                }),
        )
        .on_press(Message::TabContextRenameCancel)
        .into();
        layers.push(backdrop);

        let rename_dialog = container(
            column![
                text("Rename File").size(14),
                text_input("New filename...", &rename_val)
                    .on_input(Message::TabContextRenameInput)
                    .on_submit(Message::TabContextRenameConfirm)
                    .size(13)
                    .padding(4),
                row![
                    button(text("Rename").size(12))
                        .on_press(Message::TabContextRenameConfirm)
                        .padding([3, 10])
                        .style(move |_theme: &Theme, status| {
                            let bg = match status {
                                button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                                _ => Some(iced::Background::Color(t_menu_bg)),
                            };
                            button::Style {
                                background: bg,
                                text_color: t_text,
                                border: iced::Border { color: t_border, width: 1.0, radius: 3.0.into() },
                                ..Default::default()
                            }
                        }),
                    button(text("Cancel").size(12))
                        .on_press(Message::TabContextRenameCancel)
                        .padding([3, 10])
                        .style(move |_theme: &Theme, status| {
                            let bg = match status {
                                button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
                                _ => Some(iced::Background::Color(t_menu_bg)),
                            };
                            button::Style {
                                background: bg,
                                text_color: t_text,
                                border: iced::Border { color: t_border, width: 1.0, radius: 3.0.into() },
                                ..Default::default()
                            }
                        }),
                ].spacing(8),
            ]
            .spacing(8)
            .padding(16)
            .width(Length::Fixed(300.0)),
        )
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_menu_bg)),
            border: iced::Border { color: t_border, width: 1.0, radius: 6.0.into() },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: iced::Vector::new(2.0, 2.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        });

        let rename_overlay: Element<'_, Message> = container(opaque(rename_dialog))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();

        layers.push(rename_overlay);
    }

    // Editor context menu (right-click on text area)
    if state.editor_context_menu {
        let click_catcher: Element<'_, Message> = mouse_area(
            container(Space::new(Length::Fill, Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Message::CloseEditorContextMenu)
        .into();
        layers.push(click_catcher);

        let t_menu_bg = state.theme.menu_bg;
        let t_menu_hover = state.theme.menu_hover;
        let t_text = state.theme.text;
        let t_text_dim = state.theme.text_dim;
        let t_border = state.theme.border;

        let has_selection = state
            .tab_contents
            .get(state.tab_manager.active_index())
            .and_then(|tc| tc.content.selection())
            .is_some();

        let ctx_btn = |label: &'static str, msg: Message, enabled: bool| -> Element<'_, Message> {
            let b = button(text(label).size(13).color(if enabled { t_text } else { t_text_dim }))
                .width(Length::Fill)
                .padding([3, 8])
                .style(move |_theme: &Theme, status| {
                    let bg = match status {
                        button::Status::Hovered | button::Status::Pressed if enabled => {
                            Some(iced::Background::Color(t_menu_hover))
                        }
                        _ => None,
                    };
                    button::Style {
                        background: bg,
                        text_color: if enabled { t_text } else { t_text_dim },
                        border: iced::Border { radius: 2.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                });
            if enabled { b.on_press(msg).into() } else { b.into() }
        };

        let ctx_sep = || -> Element<'_, Message> {
            container(Space::new(Length::Fill, Length::Fixed(1.0)))
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(t_border)),
                    ..Default::default()
                })
                .padding([2, 4])
                .into()
        };

        let mut items: Vec<Element<'_, Message>> = Vec::new();

        // Hash submenu items
        items.push(ctx_btn("SHA-256", Message::HashSha256, has_selection));
        items.push(ctx_btn("SHA-1", Message::HashSha1, has_selection));
        items.push(ctx_btn("MD5", Message::HashMd5, has_selection));
        items.push(ctx_btn("CRC32", Message::HashCrc32, has_selection));
        items.push(ctx_sep());

        // Encode submenu items
        items.push(ctx_btn("Base64 Encode", Message::MimeBase64Encode, has_selection));
        items.push(ctx_btn("Base64 Decode", Message::MimeBase64Decode, has_selection));
        items.push(ctx_sep());
        items.push(ctx_btn("URL Encode", Message::MimeUrlEncode, has_selection));
        items.push(ctx_btn("URL Decode", Message::MimeUrlDecode, has_selection));
        items.push(ctx_sep());
        items.push(ctx_btn("Hex Encode", Message::MimeHexEncode, has_selection));
        items.push(ctx_btn("Hex Decode", Message::MimeHexDecode, has_selection));
        items.push(ctx_sep());
        items.push(ctx_btn("HTML Entity Encode", Message::MimeHtmlEncode, has_selection));
        items.push(ctx_btn("HTML Entity Decode", Message::MimeHtmlDecode, has_selection));

        let context_menu = container(
            column(items).spacing(0).padding(4).width(Length::Fixed(200.0)),
        )
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_menu_bg)),
            border: iced::Border {
                color: t_border,
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: iced::Vector::new(2.0, 2.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        });

        let editor_ctx_overlay: Element<'_, Message> = container(opaque(context_menu))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();

        layers.push(editor_ctx_overlay);
    }

    stack(layers).into()
}

pub fn subscription(_state: &NotepadIced) -> Subscription<Message> {
    let keys = keyboard::on_key_press(|key, modifiers| {
        // Escape closes dialogs/menus — handled in update() which checks priority
        if matches!(key.as_ref(), keyboard::Key::Named(keyboard::key::Named::Escape)) {
            return Some(Message::EscapePressed);
        }

        // Function keys
        match key.as_ref() {
            keyboard::Key::Named(keyboard::key::Named::F3) if modifiers.shift() => {
                return Some(Message::FindPrev);
            }
            keyboard::Key::Named(keyboard::key::Named::F3) => {
                return Some(Message::FindNext);
            }
            keyboard::Key::Named(keyboard::key::Named::F2) if modifiers.shift() => {
                return Some(Message::PrevBookmark);
            }
            keyboard::Key::Named(keyboard::key::Named::F2) if modifiers.command() => {
                return Some(Message::ToggleBookmark);
            }
            keyboard::Key::Named(keyboard::key::Named::F2) => {
                return Some(Message::NextBookmark);
            }
            keyboard::Key::Named(keyboard::key::Named::F11) => {
                return Some(Message::ToggleFullScreen);
            }
            _ => {}
        }

        // Disasm scrolling keys (no modifier required)
        match key.as_ref() {
            keyboard::Key::Named(keyboard::key::Named::PageDown) if !modifiers.command() => {
                return Some(Message::DisasmScroll(20));
            }
            keyboard::Key::Named(keyboard::key::Named::PageUp) if !modifiers.command() => {
                return Some(Message::DisasmScroll(-20));
            }
            keyboard::Key::Named(keyboard::key::Named::ArrowDown) if !modifiers.command() => {
                return Some(Message::DisasmScroll(1));
            }
            keyboard::Key::Named(keyboard::key::Named::ArrowUp) if !modifiers.command() => {
                return Some(Message::DisasmScroll(-1));
            }
            keyboard::Key::Named(keyboard::key::Named::Home) if modifiers.command() => {
                return Some(Message::DisasmScroll(i32::MIN));
            }
            keyboard::Key::Named(keyboard::key::Named::End) if modifiers.command() => {
                return Some(Message::DisasmScroll(i32::MAX));
            }
            _ => {}
        }

        if !modifiers.command() {
            return None;
        }

        match key.as_ref() {
            keyboard::Key::Character("n") if !modifiers.shift() => Some(Message::NewTab),
            keyboard::Key::Character("t") if !modifiers.shift() => Some(Message::NewTabInActivePane),
            keyboard::Key::Character("o") => Some(Message::OpenFile),
            keyboard::Key::Character("s") if modifiers.shift() => Some(Message::SaveAs),
            keyboard::Key::Character("s") => Some(Message::Save),
            keyboard::Key::Character("w") => Some(Message::CloseTab(usize::MAX)),
            keyboard::Key::Character("z") if !modifiers.shift() => Some(Message::Undo),
            keyboard::Key::Character("y") => Some(Message::Redo),
            keyboard::Key::Character("a") => Some(Message::SelectAll),
            keyboard::Key::Character("f") if !modifiers.shift() => Some(Message::ShowFind),
            keyboard::Key::Character("h") => Some(Message::ShowReplace),
            keyboard::Key::Character("g") => Some(Message::GotoLine),
            keyboard::Key::Character("/") => Some(Message::ToggleComment),
            keyboard::Key::Character("j") if modifiers.shift() => Some(Message::JsonFormat),
            keyboard::Key::Character("=") if !modifiers.shift() => Some(Message::ZoomIn),
            keyboard::Key::Character("+") => Some(Message::ZoomIn),
            keyboard::Key::Character("-") => Some(Message::ZoomOut),
            keyboard::Key::Character("0") => Some(Message::ZoomReset),
            keyboard::Key::Character("r") if modifiers.shift() => {
                Some(Message::ToggleMacroRecording)
            }
            keyboard::Key::Character("p") if modifiers.shift() => Some(Message::PlayLastMacro),
            keyboard::Key::Character("e") if modifiers.shift() => Some(Message::ToggleReadOnly),
            _ => None,
        }
    });

    let file_drops = iced::event::listen_with(|event, _status, _window| {
        match event {
            iced::Event::Window(iced::window::Event::FileDropped(path)) => {
                Some(Message::FileDropped(path))
            }
            // Mouse button release completes tab drag
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                Some(Message::TabDragEnd)
            }
            // Mouse wheel scrolls the disassembler
            iced::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let lines = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => -y as i32 * 3,
                    mouse::ScrollDelta::Pixels { y, .. } => -(y / 20.0) as i32,
                };
                if lines != 0 {
                    Some(Message::DisasmScroll(lines))
                } else {
                    None
                }
            }
            _ => None,
        }
    });

    let mut subs = vec![keys, file_drops];

    // Poll for incoming single-instance messages
    if INSTANCE_LISTENER.get().is_some() {
        subs.push(
            iced::time::every(std::time::Duration::from_millis(500))
                .map(Message::CheckInstance),
        );
    }

    Subscription::batch(subs)
}

pub fn theme(state: &NotepadIced) -> Theme {
    if state.theme.is_light {
        Theme::Light
    } else {
        Theme::Dark
    }
}

// ── helpers ──

fn sync_content_to_doc(state: &mut NotepadIced, idx: usize) {
    if let Some(tc) = state.tab_contents.get(idx) {
        let new_text = tc.content.text();
        if let Some(doc) = state.tab_manager.get_document_mut(idx) {
            let len = doc.buffer.len_bytes();
            if len > 0 {
                doc.buffer.delete(0, len);
            }
            if !new_text.is_empty() {
                doc.buffer.insert(0, &new_text);
            }
        }
    }
}

fn sync_split_content_to_doc(state: &mut NotepadIced, idx: usize) {
    if let Some(tc) = state.split_tab_contents.get(idx) {
        let new_text = tc.content.text();
        if let Some(doc) = state.split_tab_manager.get_document_mut(idx) {
            let len = doc.buffer.len_bytes();
            if len > 0 {
                doc.buffer.delete(0, len);
            }
            if !new_text.is_empty() {
                doc.buffer.insert(0, &new_text);
            }
        }
    }
}

fn save_as_dialog() -> Task<Message> {
    Task::perform(
        async {
            let handle = rfd::AsyncFileDialog::new()
                .set_title("Save As")
                .save_file()
                .await;
            match handle {
                Some(h) => Ok(h.path().to_path_buf()),
                None => Err("Cancelled".to_string()),
            }
        },
        Message::FileSaved,
    )
}

fn get_buffer_text(state: &NotepadIced) -> String {
    state.tab_manager.active_document().buffer.text()
}

fn set_buffer_text(state: &mut NotepadIced, text: &str) {
    let doc = state.tab_manager.active_document_mut();
    let len = doc.buffer.len_bytes();
    if len > 0 {
        doc.buffer.delete(0, len);
    }
    if !text.is_empty() {
        doc.buffer.insert(0, text);
    }
    let idx = state.tab_manager.active_index();
    state.tab_contents[idx] = TabContent::with_text(text);
}

/// Apply a line-based transform: split text into lines, call the closure, rejoin, update buffer.
fn apply_line_op(state: &mut NotepadIced, op: impl FnOnce(&mut Vec<String>, usize)) {
    if state.tab_manager.active_document().read_only {
        return;
    }
    let text = get_buffer_text(state);
    let cursor_line = state
        .tab_contents
        .get(state.tab_manager.active_index())
        .map(|tc| tc.content.cursor_position().0)
        .unwrap_or(0);

    let mut lines: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();
    op(&mut lines, cursor_line);
    let new_text = lines.join("\n");
    set_buffer_text(state, &new_text);
}

/// Apply a whole-text transform.
fn apply_text_transform(state: &mut NotepadIced, transform: impl FnOnce(&str) -> String) {
    if state.tab_manager.active_document().read_only {
        return;
    }
    let text = get_buffer_text(state);
    let new_text = transform(&text);
    set_buffer_text(state, &new_text);
}

/// Apply a hash function to selected text (or full buffer) and replace with result.
fn apply_hash_transform(state: &mut NotepadIced, label: &str, hash_fn: fn(&[u8]) -> String) {
    if state.tab_manager.active_document().read_only {
        return;
    }
    let active = state.tab_manager.active_index();
    let selected = state.tab_contents.get(active).and_then(|tc| tc.content.selection());
    let input = selected.unwrap_or_else(|| get_buffer_text(state));
    let result = hash_fn(input.as_bytes());
    let display = if result.len() > 40 {
        format!("{}: {}…", label, &result[..40])
    } else {
        format!("{}: {}", label, result)
    };
    state.status_message = Some((display, std::time::Instant::now()));
    set_buffer_text(state, &result);
}

fn view_tab_bar<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let count = state.tab_manager.tab_count();
    let active = state.tab_manager.active_index();
    let is_active_pane = state.active_pane == 0;

    let t_tab_active = state.theme.tab_active_bg;
    let t_tab_inactive = state.theme.tab_inactive_bg;
    let t_text = state.theme.text;
    let t_text_dim = state.theme.text_dim;
    let t_accent = state.theme.accent;
    let t_tab_bar = state.theme.tab_bar_bg;

    let has_split = state.split_mode != SplitMode::None;

    let mut tabs = row![].spacing(1).padding([1, 2]).align_y(iced::Alignment::Center);

    for i in 0..count {
        let tab_title = state.tab_manager.get_tab_title(i);
        let is_active = i == active;

        let bg_color = if is_active {
            t_tab_active
        } else {
            t_tab_inactive
        };

        let label = text(tab_title).size(13);
        let close = button(text("x").size(11))
            .on_press(Message::CloseTab(i))
            .padding([0, 2])
            .style(move |_theme: &Theme, status| {
                let bg = match status {
                    button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_accent)),
                    _ => None,
                };
                button::Style {
                    background: bg,
                    text_color: t_text_dim,
                    border: iced::Border {
                        radius: 2.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            });

        let mut tab_row = row![label].spacing(4).padding([2, 6]).align_y(iced::Alignment::Center);
        if has_split {
            let move_btn = button(text(">").size(11))
                .on_press(Message::MoveTabToSplit(i))
                .padding([0, 2])
                .style(move |_theme: &Theme, _status| button::Style {
                    background: None,
                    text_color: t_text_dim,
                    ..Default::default()
                });
            tab_row = tab_row.push(move_btn);
        }
        tab_row = tab_row.push(close);

        let tab = button(tab_row)
            .on_press(Message::SelectTab(i))
            .style(move |_theme: &Theme, _status| button::Style {
                background: Some(iced::Background::Color(bg_color)),
                text_color: t_text,
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        // Check if this tab is the drag target
        let is_drag_target = state.tab_drag.as_ref().map_or(false, |d| {
            !d.is_split && d.target_index == Some(i) && d.from_index != i
        });

        let tab_with_context: Element<'a, Message> = if is_drag_target {
            // Show drop indicator: a left border highlight
            let indicator = container(Space::new(Length::Fixed(2.0), Length::Fill))
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(t_accent)),
                    ..Default::default()
                });
            let tab_with_indicator = row![indicator, mouse_area(tab)
                .on_right_press(Message::TabContextMenu(i, false))
                .on_middle_press(Message::MiddleClickTab(i, false))
                .on_enter(Message::TabDragOver(i, false))];
            tab_with_indicator.into()
        } else {
            mouse_area(tab)
                .on_right_press(Message::TabContextMenu(i, false))
                .on_middle_press(Message::MiddleClickTab(i, false))
                .on_enter(Message::TabDragOver(i, false))
                .into()
        };

        tabs = tabs.push(tab_with_context);
    }

    // "+" add-tab button with gap from tabs
    let t_border = state.theme.border;
    let t_button_bg = state.theme.button_bg;
    let add_btn = button(text("+").size(14))
        .on_press(Message::NewTab)
        .padding([2, 8])
        .style(move |_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => t_tab_active,
                _ => t_button_bg,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: t_text_dim,
                border: iced::Border {
                    color: t_border,
                    width: 1.0,
                    radius: 3.0.into(),
                },
                ..Default::default()
            }
        });
    tabs = tabs.push(iced::widget::horizontal_space().width(4));
    tabs = tabs.push(add_btn);

    scrollable(tabs)
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::new().width(3).scroller_width(3),
        ))
        .width(Length::Fill)
        .style(move |theme: &Theme, status| {
            let mut style = iced::widget::scrollable::default(theme, status);
            style.container.background = Some(iced::Background::Color(t_tab_bar));
            style
        })
        .into()
}

fn view_editor<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let active = state.tab_manager.active_index();
    let t_text_dim = state.theme.text_dim;
    let t_bg = state.theme.background;
    let t_editor_bg = state.theme.editor_bg;
    let t_text = state.theme.text;
    let t_selection = state.theme.selection_bg;

    if let Some(tc) = state.tab_contents.get(active) {
        let wrapping = if state.word_wrap {
            Wrapping::Word
        } else {
            Wrapping::None
        };

        let editor_font = Font {
            family: iced::font::Family::Name(Box::leak(state.font_family.clone().into_boxed_str())),
            ..Font::MONOSPACE
        };

        let editor = text_editor(&tc.content)
            .on_action(Message::EditorAction)
            .size(state.font_size)
            .font(editor_font)
            .line_height(iced::advanced::text::LineHeight::Relative(state.line_spacing))
            .height(Length::Fill)
            .wrapping(wrapping)
            .style(move |_theme: &Theme, _status| {
                let bg = iced::Background::Color(t_editor_bg);
                text_editor::Style {
                    background: bg,
                    border: iced::Border::default(),
                    icon: t_text_dim,
                    placeholder: t_text_dim,
                    value: t_text,
                    selection: t_selection,
                }
            })
            .highlight_with::<SyntectHighlighter>(
                SyntectSettings {
                    extension: state.file_extension.clone(),
                    theme: state.theme.syntect_theme().to_string(),
                },
                |highlight, _theme| highlight.to_format(),
            );

        container(editor)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(t_bg)),
                    ..Default::default()
                })
                .into()
    } else {
        container(text("No document open"))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

/// Tab bar for split pane, rendered at the same level as the primary tab bar.
fn view_split_tab_bar<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let t_tab_active = state.theme.tab_active_bg;
    let t_tab_inactive = state.theme.tab_inactive_bg;
    let t_text = state.theme.text;
    let t_text_dim = state.theme.text_dim;
    let t_accent = state.theme.accent;
    let t_tab_bar = state.theme.tab_bar_bg;

    let count = state.split_tab_manager.tab_count();
    let active = state.split_tab_manager.active_index();

    let mut tabs = row![].spacing(1).padding([1, 2]).align_y(iced::Alignment::Center);

    for i in 0..count {
        let tab_title = state.split_tab_manager.get_tab_title(i);
        let is_active_tab = i == active;

        let bg_color = if is_active_tab {
            t_tab_active
        } else {
            t_tab_inactive
        };

        let label = text(tab_title).size(13);
        let close = button(text("x").size(11))
            .on_press(Message::SplitCloseTab(i))
            .padding([0, 2])
            .style(move |_theme: &Theme, status| {
                let bg = match status {
                    button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_accent)),
                    _ => None,
                };
                button::Style {
                    background: bg,
                    text_color: t_text_dim,
                    border: iced::Border {
                        radius: 2.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            });

        let move_btn = button(text("<").size(11))
            .on_press(Message::MoveTabFromSplit(i))
            .padding([0, 2])
            .style(move |_theme: &Theme, _status| button::Style {
                background: None,
                text_color: t_text_dim,
                ..Default::default()
            });

        let tab = button(row![label, move_btn, close].spacing(4).padding([2, 6]).align_y(iced::Alignment::Center))
            .on_press(Message::SplitSelectTab(i))
            .style(move |_theme: &Theme, _status| button::Style {
                background: Some(iced::Background::Color(bg_color)),
                text_color: t_text,
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        let is_drag_target = state.tab_drag.as_ref().map_or(false, |d| {
            d.is_split && d.target_index == Some(i) && d.from_index != i
        });

        let tab_with_context: Element<'a, Message> = if is_drag_target {
            let indicator = container(Space::new(Length::Fixed(2.0), Length::Fill))
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(t_accent)),
                    ..Default::default()
                });
            let tab_with_indicator = row![indicator, mouse_area(tab)
                .on_right_press(Message::TabContextMenu(i, true))
                .on_middle_press(Message::MiddleClickTab(i, true))
                .on_enter(Message::TabDragOver(i, true))];
            tab_with_indicator.into()
        } else {
            mouse_area(tab)
                .on_right_press(Message::TabContextMenu(i, true))
                .on_middle_press(Message::MiddleClickTab(i, true))
                .on_enter(Message::TabDragOver(i, true))
                .into()
        };

        tabs = tabs.push(tab_with_context);
    }

    // "+" add-tab button with gap from tabs
    let t_border = state.theme.border;
    let t_button_bg = state.theme.button_bg;
    let add_btn = button(text("+").size(14))
        .on_press(Message::SplitNewTab)
        .padding([2, 8])
        .style(move |_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => t_tab_active,
                _ => t_button_bg,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: t_text_dim,
                border: iced::Border {
                    color: t_border,
                    width: 1.0,
                    radius: 3.0.into(),
                },
                ..Default::default()
            }
        });
    tabs = tabs.push(iced::widget::horizontal_space().width(4));
    tabs = tabs.push(add_btn);

    scrollable(tabs)
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::new().width(3).scroller_width(3),
        ))
        .width(Length::Fill)
        .style(move |theme: &Theme, status| {
            let mut style = iced::widget::scrollable::default(theme, status);
            style.container.background = Some(iced::Background::Color(t_tab_bar));
            style
        })
        .into()
}

/// Full editing secondary pane for split view (editor + status only, tab bar rendered separately).
fn view_split_pane<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let t_text_dim = state.theme.text_dim;
    let t_bg = state.theme.background;
    let t_status = state.theme.status_bar_bg;

    let t_editor_bg_split = state.theme.editor_bg;
    let t_text_split = state.theme.text;
    let t_selection_split = state.theme.selection_bg;

    let active = state.split_tab_manager.active_index();

    // Editor for split pane
    let split_editor: Element<'_, Message> = if let Some(tc) = state.split_tab_contents.get(active) {
        let wrapping = if state.word_wrap {
            Wrapping::Word
        } else {
            Wrapping::None
        };

        let split_font = Font {
            family: iced::font::Family::Name(Box::leak(state.font_family.clone().into_boxed_str())),
            ..Font::MONOSPACE
        };

        let editor_widget = text_editor(&tc.content)
            .on_action(Message::SplitEditorAction)
            .size(state.split_font_size)
            .font(split_font)
            .line_height(iced::advanced::text::LineHeight::Relative(state.line_spacing))
            .height(Length::Fill)
            .wrapping(wrapping)
            .style(move |_theme: &Theme, _status| {
                text_editor::Style {
                    background: iced::Background::Color(t_editor_bg_split),
                    border: iced::Border::default(),
                    icon: t_text_dim,
                    placeholder: t_text_dim,
                    value: t_text_split,
                    selection: t_selection_split,
                }
            })
            .highlight_with::<SyntectHighlighter>(
                SyntectSettings {
                    extension: state.split_file_extension.clone(),
                    theme: state.theme.syntect_theme().to_string(),
                },
                |highlight, _theme| highlight.to_format(),
            );

        if state.show_line_numbers {
            let content_text = tc.content.text();
            let line_count = content_text.lines().count().max(1);
            let gutter_width = 55.0;

            let mut gutter_col = column![];
            for i in 1..=line_count {
                let line_label = text(format!("    {}", i))
                    .size(state.split_font_size)
                    .color(t_text_dim);

                let line_widget: Element<'_, Message> = container(line_label)
                    .width(Length::Fixed(gutter_width))
                    .align_x(iced::alignment::Horizontal::Right)
                    .padding([0, 4])
                    .into();

                gutter_col = gutter_col.push(line_widget);
            }

            let gutter = container(gutter_col)
                .height(Length::Fill)
                .clip(true)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(t_bg)),
                    ..Default::default()
                });

            let editor_row = row![gutter, editor_widget].height(Length::Fill);

            container(editor_row)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(t_bg)),
                    ..Default::default()
                })
                .into()
        } else {
            container(editor_widget)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(t_bg)),
                    ..Default::default()
                })
                .into()
        }
    } else {
        container(text("No document open"))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

    // Status bar for split pane
    let (line, col) = state
        .split_tab_contents
        .get(active)
        .map(|tc| {
            let (l, c) = tc.content.cursor_position();
            (l + 1, c + 1)
        })
        .unwrap_or((1, 1));

    let split_status: Element<'_, Message> = container(
        text(format!("  Ln {}, Col {}", line, col)).size(12),
    )
    .width(Length::Fill)
    .padding([2, 8])
    .style(move |_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(t_status)),
        ..Default::default()
    })
    .into();

    let pane_content = column![split_editor, split_status]
        .width(Length::Fill)
        .height(Length::Fill);

    mouse_area(pane_content)
        .on_press(Message::SetActivePane(1))
        .into()
}

fn view_status_bar<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let active = state.tab_manager.active_index();
    let t_status = state.theme.status_bar_bg;
    let (line, col, encoding_str, line_ending_str, language) =
        if let Some(tc) = state.tab_contents.get(active) {
            let (cursor_line, cursor_col) = tc.content.cursor_position();
            let doc = state.tab_manager.active_document();
            let enc = crate::editor::text_transforms::encoding_display_name(&doc.encoding);
            let le = crate::editor::text_transforms::line_ending_display_name(&doc.line_ending);
            let lang = if doc.language.is_empty() {
                "Plain Text"
            } else {
                &doc.language
            };
            (
                cursor_line + 1,
                cursor_col + 1,
                enc.to_string(),
                le.to_string(),
                lang.to_string(),
            )
        } else {
            (
                1,
                1,
                "UTF-8".to_string(),
                "LF".to_string(),
                "Plain Text".to_string(),
            )
        };

    let is_read_only = state.tab_manager.active_document().read_only;
    let mut status_string = crate::editor::text_transforms::format_status_bar(
        line, col, &encoding_str, &line_ending_str, &language, state.show_whitespace,
    );
    if is_read_only {
        status_string.push_str("    [READ ONLY]");
    }

    // Show transient status message (auto-expires after 5 seconds)
    if let Some((ref msg, instant)) = state.status_message {
        if instant.elapsed().as_secs() < 5 {
            status_string.push_str("    ");
            status_string.push_str(msg);
        }
    }

    let status_text = text(status_string).size(12);

    container(status_text)
        .width(Length::Fill)
        .padding([2, 8])
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_status)),
            ..Default::default()
        })
        .into()
}


fn run_search(state: &mut NotepadIced) {
    if state.search_query.is_empty() {
        state.search_matches.clear();
        state.current_match_index = None;
        return;
    }

    let mut engine = SearchEngine::new();
    engine.query = state.search_query.clone();
    engine.case_sensitive = state.case_sensitive;
    engine.whole_word = state.whole_word;
    engine.use_regex = state.use_regex;
    engine.search_mode = if state.use_regex {
        SearchMode::Regex
    } else {
        SearchMode::Normal
    };

    let text = get_buffer_text(state);
    state.search_matches = engine.find_all(&text);

    // Keep current_match_index in bounds
    if state.search_matches.is_empty() {
        state.current_match_index = None;
    } else if let Some(idx) = state.current_match_index {
        if idx >= state.search_matches.len() {
            state.current_match_index = Some(0);
        }
    } else {
        state.current_match_index = Some(0);
    }
}

fn rebuild_content(state: &mut NotepadIced) {
    let idx = state.tab_manager.active_index();
    let buf_text = state.tab_manager.active_document().buffer.text();
    if idx < state.tab_contents.len() {
        state.tab_contents[idx] = TabContent::with_text(&buf_text);
    }
}
