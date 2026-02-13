//! Native Win32 + Scintilla backend for Notepad+++.

#![allow(unsafe_op_in_unsafe_fn)]

pub mod scintilla;

use std::path::PathBuf;

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::*;
use windows_sys::Win32::UI::Controls::Dialogs::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{SetFocus, EnableWindow};
use windows_sys::Win32::UI::Shell::{DragAcceptFiles, DragQueryFileW, DragFinish, HDROP};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use scintilla::*;

// ── Scintilla registration (from static lib) ──
unsafe extern "C" {
    fn Scintilla_RegisterClasses(hInstance: HINSTANCE) -> i32;
}

// ── Lexilla (from static lib) ──
#[repr(C)]
pub struct ILexer5 {
    _opaque: [u8; 0],
}

unsafe extern "C" {
    fn CreateLexer(name: *const u8) -> *mut ILexer5;
}

// ── Menu ID Constants ──

// File menu
const IDM_FILE_NEW: u16 = 101;
const IDM_FILE_OPEN: u16 = 102;
const IDM_FILE_SAVE: u16 = 103;
const IDM_FILE_SAVE_AS: u16 = 104;
const IDM_FILE_SAVE_ALL: u16 = 105;
const IDM_FILE_CLOSE: u16 = 106;
const IDM_FILE_CLOSE_ALL: u16 = 107;
const IDM_FILE_SAVE_SESSION: u16 = 108;
const IDM_FILE_LOAD_SESSION: u16 = 109;
const IDM_FILE_EXPORT_HTML: u16 = 110;
const IDM_FILE_EXPORT_RTF: u16 = 111;
const IDM_FILE_EXIT: u16 = 112;

// Edit menu
const IDM_EDIT_UNDO: u16 = 201;
const IDM_EDIT_REDO: u16 = 202;
const IDM_EDIT_CUT: u16 = 203;
const IDM_EDIT_COPY: u16 = 204;
const IDM_EDIT_PASTE: u16 = 205;
const IDM_EDIT_DELETE: u16 = 206;
const IDM_EDIT_SELECT_ALL: u16 = 207;
const IDM_EDIT_TOGGLE_COMMENT: u16 = 208;

// Line Operations submenu
const IDM_LINE_DUPLICATE: u16 = 220;
const IDM_LINE_DELETE: u16 = 221;
const IDM_LINE_MOVE_UP: u16 = 222;
const IDM_LINE_MOVE_DOWN: u16 = 223;
const IDM_LINE_SORT_ASC: u16 = 224;
const IDM_LINE_SORT_DESC: u16 = 225;
const IDM_LINE_REMOVE_EMPTY: u16 = 226;
const IDM_LINE_REMOVE_DUPLICATE: u16 = 227;
const IDM_LINE_TRIM_TRAILING: u16 = 228;
const IDM_LINE_JOIN: u16 = 229;
const IDM_LINE_SPLIT: u16 = 230;
const IDM_LINE_INSERT_ABOVE: u16 = 231;
const IDM_LINE_INSERT_BELOW: u16 = 232;
const IDM_LINE_REVERSE: u16 = 233;
const IDM_LINE_SORT_NOCASE: u16 = 234;
const IDM_LINE_SORT_NUMERIC: u16 = 235;
const IDM_LINE_TRIM_LEADING: u16 = 236;
const IDM_LINE_TRIM_BOTH: u16 = 237;

// Case Conversion submenu
const IDM_CASE_UPPER: u16 = 250;
const IDM_CASE_LOWER: u16 = 251;
const IDM_CASE_TITLE: u16 = 252;
const IDM_CASE_SENTENCE: u16 = 253;
const IDM_CASE_INVERSE: u16 = 254;

// Encoding submenu
const IDM_ENC_UTF8: u16 = 260;
const IDM_ENC_UTF8_BOM: u16 = 261;
const IDM_ENC_UTF16_LE: u16 = 262;
const IDM_ENC_UTF16_BE: u16 = 263;
const IDM_ENC_ANSI: u16 = 264;

// Line Endings submenu
const IDM_EOL_CRLF: u16 = 270;
const IDM_EOL_LF: u16 = 271;
const IDM_EOL_CR: u16 = 272;

// Search menu
const IDM_SEARCH_FIND: u16 = 301;
const IDM_SEARCH_REPLACE: u16 = 302;
const IDM_SEARCH_FIND_IN_FILES: u16 = 303;
const IDM_SEARCH_SELECT_ALL_OCCURRENCES: u16 = 304;
const IDM_SEARCH_GOTO_LINE: u16 = 305;
const IDM_SEARCH_BOOKMARK_TOGGLE: u16 = 310;
const IDM_SEARCH_BOOKMARK_NEXT: u16 = 311;
const IDM_SEARCH_BOOKMARK_PREV: u16 = 312;
const IDM_SEARCH_BOOKMARK_CLEAR: u16 = 313;

// View menu
const IDM_VIEW_WORDWRAP: u16 = 401;
const IDM_VIEW_LINENUMBERS: u16 = 402;
const IDM_VIEW_WHITESPACE: u16 = 403;
const IDM_VIEW_SPLIT_HORIZ: u16 = 410;
const IDM_VIEW_SPLIT_VERT: u16 = 411;
const IDM_VIEW_REMOVE_SPLIT: u16 = 412;
const IDM_VIEW_ZOOM_IN: u16 = 420;
const IDM_VIEW_ZOOM_OUT: u16 = 421;
const IDM_VIEW_ZOOM_RESET: u16 = 422;
const IDM_VIEW_FOLD_TOGGLE: u16 = 430;
const IDM_VIEW_FOLD_ALL: u16 = 431;
const IDM_VIEW_UNFOLD_ALL: u16 = 432;

// Language menu (base ID; items are IDM_LANG_BASE + index)
const IDM_LANG_BASE: u16 = 500;

// Tools menu
const IDM_TOOL_JSON_FORMAT: u16 = 601;
const IDM_TOOL_JSON_COMPACT: u16 = 602;
const IDM_TOOL_JSON_VALIDATE: u16 = 603;
const IDM_TOOL_JSON_SORT_KEYS: u16 = 604;
const IDM_TOOL_BASE64_ENCODE: u16 = 611;
const IDM_TOOL_BASE64_DECODE: u16 = 612;
const IDM_TOOL_URL_ENCODE: u16 = 613;
const IDM_TOOL_URL_DECODE: u16 = 614;
const IDM_TOOL_HEX_VIEWER: u16 = 620;

// Macro menu
const IDM_MACRO_RECORD: u16 = 701;
const IDM_MACRO_PLAY: u16 = 702;

// Settings menu
const IDM_SETTINGS_PREFERENCES: u16 = 801;
const IDM_SETTINGS_SHORTCUTS: u16 = 802;

// Help menu
const IDM_HELP_ABOUT: u16 = 901;

// Recent files menu IDs
const IDM_RECENT_BASE: u16 = 1100;

// Control IDs
const ID_TAB_CONTROL: i32 = 1000;
const ID_STATUS_BAR: i32 = 1001;
const ID_SCINTILLA: i32 = 1002;
const ID_SCINTILLA2: i32 = 1003;

// WM_NOTIFY codes for tab control
const TCN_FIRST: i32 = -550;
const TCN_SELCHANGE: i32 = TCN_FIRST - 1;

// Bookmark marker number (0 is available; folding uses 25-31)
const BOOKMARK_MARKER: usize = 1;

/// A single open document/tab.
struct TabDocument {
    path: Option<PathBuf>,
    title: String,
    text: Vec<u8>,
    language: String,
}

impl TabDocument {
    fn new_untitled(n: usize) -> Self {
        Self {
            path: None,
            title: format!("Untitled-{n}"),
            text: Vec::new(),
            language: "Plain Text".to_string(),
        }
    }
}

// ── App State (global, since Win32 WndProc is a C callback) ──
struct AppState {
    hwnd_main: HWND,
    hwnd_tab: HWND,
    hwnd_scintilla: HWND,
    hwnd_scintilla2: HWND,
    hwnd_status: HWND,
    h_accel: HACCEL,
    tabs: Vec<TabDocument>,
    active_tab: usize,
    untitled_counter: usize,
    word_wrap: bool,
    line_numbers: bool,
    show_whitespace: bool,
    encoding: String,
    line_ending: String,
    split_mode: u8, // 0=none, 1=horizontal, 2=vertical
    recent_files: Vec<PathBuf>,
    hex_mode: bool,
    hex_original: Vec<u8>,
    recording: bool,
    macro_buffer: Vec<(u32, usize, isize)>,
}

static mut APP: *mut AppState = std::ptr::null_mut();

unsafe fn app() -> &'static mut AppState {
    &mut *APP
}

// ── Wide-string helpers ──
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

// ── Entry point ──
pub fn run() {
    unsafe {
        let hinstance = GetModuleHandleW(std::ptr::null());

        // Init common controls (tabs + status bar)
        let icc = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_TAB_CLASSES | ICC_BAR_CLASSES,
        };
        InitCommonControlsEx(&icc);

        // Register Scintilla
        Scintilla_RegisterClasses(hinstance);

        // Register window class
        let class_name = wide("NotepadPPPMain");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: LoadIconW(std::ptr::null_mut(), IDI_APPLICATION),
            hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };
        RegisterClassExW(&wc);

        // Create menu bar
        let hmenu = create_menu_bar();

        // Create main window
        let title = wide("Notepad+++");
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1200,
            800,
            std::ptr::null_mut(),
            hmenu,
            hinstance,
            std::ptr::null(),
        );

        // Build accelerator table
        let h_accel = create_accelerators();

        // Init app state
        let state = Box::new(AppState {
            hwnd_main: hwnd,
            hwnd_tab: std::ptr::null_mut(),
            hwnd_scintilla: std::ptr::null_mut(),
            hwnd_scintilla2: std::ptr::null_mut(),
            hwnd_status: std::ptr::null_mut(),
            h_accel,
            tabs: Vec::new(),
            active_tab: 0,
            untitled_counter: 0,
            word_wrap: false,
            line_numbers: true,
            show_whitespace: false,
            encoding: "UTF-8".to_string(),
            line_ending: "CRLF".to_string(),
            split_mode: 0,
            recent_files: Vec::new(),
            hex_mode: false,
            hex_original: Vec::new(),
            recording: false,
            macro_buffer: Vec::new(),
        });
        APP = Box::into_raw(state);

        // Load recent files list
        load_recent_files();

        // Create child controls
        create_controls(hwnd, hinstance);

        // Accept drag-and-drop files
        DragAcceptFiles(hwnd, TRUE);

        // Set initial menu check marks
        let menu = GetMenu(hwnd);
        CheckMenuItem(menu, IDM_VIEW_LINENUMBERS as u32, MF_CHECKED);

        // Add initial tab
        cmd_new_tab();

        // Open files from command line arguments
        let args: Vec<String> = std::env::args().skip(1).collect();
        for arg in &args {
            if !arg.starts_with('-') {
                let path = std::path::Path::new(arg);
                if path.exists() {
                    open_file_in_tab(&arg);
                }
            }
        }
        // If we opened files, close the initial empty tab
        if !args.is_empty() && (*APP).tabs.len() > 1 {
            SendMessageW((*APP).hwnd_tab, TCM_SETCURSEL, 1, 0);
            switch_tab(1);
            (*APP).tabs.remove(0);
            SendMessageW((*APP).hwnd_tab, TCM_DELETEITEM, 0, 0);
            (*APP).active_tab = 0;
            SendMessageW((*APP).hwnd_tab, TCM_SETCURSEL, 0, 0);
        }

        ShowWindow(hwnd, SW_SHOWMAXIMIZED);
        UpdateWindow(hwnd);

        // Message loop with accelerator support and modeless dialog handling
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            // Let modeless Find/Replace dialog process its messages
            let find_dlg = FIND_DLG_HWND;
            if !find_dlg.is_null() && IsDialogMessageW(find_dlg, &msg) != 0 {
                continue;
            }
            if !h_accel.is_null()
                && TranslateAcceleratorW(hwnd, h_accel, &msg) != 0
            {
                continue;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Cleanup
        if !h_accel.is_null() {
            DestroyAcceleratorTable(h_accel);
        }
        let _ = Box::from_raw(APP);
        APP = std::ptr::null_mut();
    }
}

unsafe fn create_menu_bar() -> HMENU {
    let menu_bar = CreateMenu();

    // ── File ──
    let file_menu = CreatePopupMenu();
    append_menu(file_menu, IDM_FILE_NEW, "&New\tCtrl+N");
    append_menu(file_menu, IDM_FILE_OPEN, "&Open...\tCtrl+O");
    append_menu(file_menu, IDM_FILE_SAVE, "&Save\tCtrl+S");
    append_menu(file_menu, IDM_FILE_SAVE_AS, "Save &As...\tCtrl+Shift+S");
    append_menu(file_menu, IDM_FILE_SAVE_ALL, "Save A&ll");
    AppendMenuW(file_menu, MF_SEPARATOR, 0, std::ptr::null());
    append_menu(file_menu, IDM_FILE_CLOSE, "&Close\tCtrl+W");
    append_menu(file_menu, IDM_FILE_CLOSE_ALL, "Close All");
    AppendMenuW(file_menu, MF_SEPARATOR, 0, std::ptr::null());
    // Recent Files placeholder
    let recent_menu = CreatePopupMenu();
    append_menu(recent_menu, 0, "(empty)");
    let lbl = wide("Recent &Files");
    AppendMenuW(file_menu, MF_POPUP, recent_menu as usize, lbl.as_ptr());
    AppendMenuW(file_menu, MF_SEPARATOR, 0, std::ptr::null());
    append_menu(file_menu, IDM_FILE_SAVE_SESSION, "Save Session");
    append_menu(file_menu, IDM_FILE_LOAD_SESSION, "Load Session");
    AppendMenuW(file_menu, MF_SEPARATOR, 0, std::ptr::null());
    // Export submenu
    let export_menu = CreatePopupMenu();
    append_menu(export_menu, IDM_FILE_EXPORT_HTML, "As &HTML");
    append_menu(export_menu, IDM_FILE_EXPORT_RTF, "As &RTF");
    let lbl = wide("&Export");
    AppendMenuW(file_menu, MF_POPUP, export_menu as usize, lbl.as_ptr());
    AppendMenuW(file_menu, MF_SEPARATOR, 0, std::ptr::null());
    append_menu(file_menu, IDM_FILE_EXIT, "E&xit\tAlt+F4");
    let lbl = wide("&File");
    AppendMenuW(menu_bar, MF_POPUP, file_menu as usize, lbl.as_ptr());

    // ── Edit ──
    let edit_menu = CreatePopupMenu();
    append_menu(edit_menu, IDM_EDIT_UNDO, "&Undo\tCtrl+Z");
    append_menu(edit_menu, IDM_EDIT_REDO, "&Redo\tCtrl+Y");
    AppendMenuW(edit_menu, MF_SEPARATOR, 0, std::ptr::null());
    append_menu(edit_menu, IDM_EDIT_CUT, "Cu&t\tCtrl+X");
    append_menu(edit_menu, IDM_EDIT_COPY, "&Copy\tCtrl+C");
    append_menu(edit_menu, IDM_EDIT_PASTE, "&Paste\tCtrl+V");
    append_menu(edit_menu, IDM_EDIT_DELETE, "&Delete\tDel");
    AppendMenuW(edit_menu, MF_SEPARATOR, 0, std::ptr::null());
    append_menu(edit_menu, IDM_EDIT_SELECT_ALL, "Select &All\tCtrl+A");
    AppendMenuW(edit_menu, MF_SEPARATOR, 0, std::ptr::null());
    append_menu(edit_menu, IDM_EDIT_TOGGLE_COMMENT, "Toggle &Comment\tCtrl+/");
    AppendMenuW(edit_menu, MF_SEPARATOR, 0, std::ptr::null());

    // Line Operations submenu
    let line_menu = CreatePopupMenu();
    append_menu(line_menu, IDM_LINE_DUPLICATE, "&Duplicate Line");
    append_menu(line_menu, IDM_LINE_DELETE, "De&lete Line");
    append_menu(line_menu, IDM_LINE_MOVE_UP, "Move Line &Up");
    append_menu(line_menu, IDM_LINE_MOVE_DOWN, "Move Line &Down");
    AppendMenuW(line_menu, MF_SEPARATOR, 0, std::ptr::null());
    append_menu(line_menu, IDM_LINE_SORT_ASC, "Sort Lines &Ascending");
    append_menu(line_menu, IDM_LINE_SORT_DESC, "Sort Lines D&escending");
    AppendMenuW(line_menu, MF_SEPARATOR, 0, std::ptr::null());
    append_menu(line_menu, IDM_LINE_REMOVE_EMPTY, "Remove &Empty Lines");
    append_menu(line_menu, IDM_LINE_REMOVE_DUPLICATE, "Remove Duplicate Lines");
    append_menu(line_menu, IDM_LINE_TRIM_TRAILING, "&Trim Trailing Whitespace");
    AppendMenuW(line_menu, MF_SEPARATOR, 0, std::ptr::null());
    append_menu(line_menu, IDM_LINE_JOIN, "&Join Lines");
    append_menu(line_menu, IDM_LINE_SPLIT, "&Split Line");
    append_menu(line_menu, IDM_LINE_INSERT_ABOVE, "Insert Blank Line A&bove");
    append_menu(line_menu, IDM_LINE_INSERT_BELOW, "Insert Blank Line Belo&w");
    append_menu(line_menu, IDM_LINE_REVERSE, "&Reverse Line Order");
    append_menu(line_menu, IDM_LINE_SORT_NOCASE, "Sort Lines (Case &Insensitive)");
    append_menu(line_menu, IDM_LINE_SORT_NUMERIC, "Sort Lines (&Numeric)");
    append_menu(line_menu, IDM_LINE_TRIM_LEADING, "Trim Leadin&g Whitespace");
    append_menu(line_menu, IDM_LINE_TRIM_BOTH, "Trim Bot&h Whitespace");
    let lbl = wide("&Line Operations");
    AppendMenuW(edit_menu, MF_POPUP, line_menu as usize, lbl.as_ptr());

    // Case Conversion submenu
    let case_menu = CreatePopupMenu();
    append_menu(case_menu, IDM_CASE_UPPER, "&UPPERCASE");
    append_menu(case_menu, IDM_CASE_LOWER, "&lowercase");
    append_menu(case_menu, IDM_CASE_TITLE, "&Title Case");
    append_menu(case_menu, IDM_CASE_SENTENCE, "&Sentence case");
    append_menu(case_menu, IDM_CASE_INVERSE, "&iNVERSE Case");
    let lbl = wide("Case &Conversion");
    AppendMenuW(edit_menu, MF_POPUP, case_menu as usize, lbl.as_ptr());
    AppendMenuW(edit_menu, MF_SEPARATOR, 0, std::ptr::null());

    // Encoding submenu
    let enc_menu = CreatePopupMenu();
    append_menu(enc_menu, IDM_ENC_UTF8, "&UTF-8");
    append_menu(enc_menu, IDM_ENC_UTF8_BOM, "UTF-8 &BOM");
    append_menu(enc_menu, IDM_ENC_UTF16_LE, "UTF-16 &LE");
    append_menu(enc_menu, IDM_ENC_UTF16_BE, "UTF-16 B&E");
    append_menu(enc_menu, IDM_ENC_ANSI, "&ANSI");
    let lbl = wide("Encodin&g");
    AppendMenuW(edit_menu, MF_POPUP, enc_menu as usize, lbl.as_ptr());

    // Line Endings submenu
    let eol_menu = CreatePopupMenu();
    append_menu(eol_menu, IDM_EOL_CRLF, "&Windows (CRLF)");
    append_menu(eol_menu, IDM_EOL_LF, "&Unix (LF)");
    append_menu(eol_menu, IDM_EOL_CR, "&Mac (CR)");
    let lbl = wide("Line &Endings");
    AppendMenuW(edit_menu, MF_POPUP, eol_menu as usize, lbl.as_ptr());

    let lbl = wide("&Edit");
    AppendMenuW(menu_bar, MF_POPUP, edit_menu as usize, lbl.as_ptr());

    // ── Search ──
    let search_menu = CreatePopupMenu();
    append_menu(search_menu, IDM_SEARCH_FIND, "&Find...\tCtrl+F");
    append_menu(search_menu, IDM_SEARCH_REPLACE, "&Replace...\tCtrl+H");
    append_menu(search_menu, IDM_SEARCH_FIND_IN_FILES, "Find in &Files...");
    append_menu(search_menu, IDM_SEARCH_SELECT_ALL_OCCURRENCES, "Select All &Occurrences\tCtrl+Shift+L");
    AppendMenuW(search_menu, MF_SEPARATOR, 0, std::ptr::null());
    append_menu(search_menu, IDM_SEARCH_GOTO_LINE, "&Go to Line...\tCtrl+G");
    AppendMenuW(search_menu, MF_SEPARATOR, 0, std::ptr::null());

    // Bookmarks submenu
    let bm_menu = CreatePopupMenu();
    append_menu(bm_menu, IDM_SEARCH_BOOKMARK_TOGGLE, "&Toggle Bookmark");
    append_menu(bm_menu, IDM_SEARCH_BOOKMARK_NEXT, "&Next Bookmark");
    append_menu(bm_menu, IDM_SEARCH_BOOKMARK_PREV, "&Previous Bookmark");
    AppendMenuW(bm_menu, MF_SEPARATOR, 0, std::ptr::null());
    append_menu(bm_menu, IDM_SEARCH_BOOKMARK_CLEAR, "&Clear All Bookmarks");
    let lbl = wide("&Bookmarks");
    AppendMenuW(search_menu, MF_POPUP, bm_menu as usize, lbl.as_ptr());

    let lbl = wide("&Search");
    AppendMenuW(menu_bar, MF_POPUP, search_menu as usize, lbl.as_ptr());

    // ── View ──
    let view_menu = CreatePopupMenu();
    append_menu(view_menu, IDM_VIEW_WORDWRAP, "&Word Wrap");
    append_menu(view_menu, IDM_VIEW_LINENUMBERS, "&Line Numbers");
    append_menu(view_menu, IDM_VIEW_WHITESPACE, "Show &Whitespace");
    AppendMenuW(view_menu, MF_SEPARATOR, 0, std::ptr::null());
    append_menu(view_menu, IDM_VIEW_SPLIT_HORIZ, "Split &Horizontal");
    append_menu(view_menu, IDM_VIEW_SPLIT_VERT, "Split &Vertical");
    append_menu(view_menu, IDM_VIEW_REMOVE_SPLIT, "&Remove Split");
    AppendMenuW(view_menu, MF_SEPARATOR, 0, std::ptr::null());
    append_menu(view_menu, IDM_VIEW_ZOOM_IN, "Zoom &In\tCtrl+=");
    append_menu(view_menu, IDM_VIEW_ZOOM_OUT, "Zoom &Out\tCtrl+-");
    append_menu(view_menu, IDM_VIEW_ZOOM_RESET, "Reset &Zoom\tCtrl+0");
    AppendMenuW(view_menu, MF_SEPARATOR, 0, std::ptr::null());

    // Folding submenu
    let fold_menu = CreatePopupMenu();
    append_menu(fold_menu, IDM_VIEW_FOLD_TOGGLE, "&Toggle Fold");
    append_menu(fold_menu, IDM_VIEW_FOLD_ALL, "&Fold All");
    append_menu(fold_menu, IDM_VIEW_UNFOLD_ALL, "&Unfold All");
    let lbl = wide("&Folding");
    AppendMenuW(view_menu, MF_POPUP, fold_menu as usize, lbl.as_ptr());

    let lbl = wide("&View");
    AppendMenuW(menu_bar, MF_POPUP, view_menu as usize, lbl.as_ptr());

    // ── Language ──
    let lang_menu = CreatePopupMenu();
    for (i, (name, _lexer)) in all_languages().iter().enumerate() {
        let id = IDM_LANG_BASE + i as u16;
        append_menu(lang_menu, id, name);
    }
    let lbl = wide("&Language");
    AppendMenuW(menu_bar, MF_POPUP, lang_menu as usize, lbl.as_ptr());

    // ── Tools ──
    let tools_menu = CreatePopupMenu();
    // JSON submenu
    let json_menu = CreatePopupMenu();
    append_menu(json_menu, IDM_TOOL_JSON_FORMAT, "&Format JSON");
    append_menu(json_menu, IDM_TOOL_JSON_COMPACT, "&Compact JSON");
    append_menu(json_menu, IDM_TOOL_JSON_VALIDATE, "&Validate JSON");
    append_menu(json_menu, IDM_TOOL_JSON_SORT_KEYS, "&Sort JSON Keys");
    let lbl = wide("&JSON");
    AppendMenuW(tools_menu, MF_POPUP, json_menu as usize, lbl.as_ptr());
    AppendMenuW(tools_menu, MF_SEPARATOR, 0, std::ptr::null());

    // MIME Tools submenu
    let mime_menu = CreatePopupMenu();
    append_menu(mime_menu, IDM_TOOL_BASE64_ENCODE, "&Base64 Encode");
    append_menu(mime_menu, IDM_TOOL_BASE64_DECODE, "Base64 &Decode");
    append_menu(mime_menu, IDM_TOOL_URL_ENCODE, "&URL Encode");
    append_menu(mime_menu, IDM_TOOL_URL_DECODE, "URL D&ecode");
    let lbl = wide("&MIME Tools");
    AppendMenuW(tools_menu, MF_POPUP, mime_menu as usize, lbl.as_ptr());
    AppendMenuW(tools_menu, MF_SEPARATOR, 0, std::ptr::null());

    append_menu(tools_menu, IDM_TOOL_HEX_VIEWER, "&Hex Viewer");
    let lbl = wide("&Tools");
    AppendMenuW(menu_bar, MF_POPUP, tools_menu as usize, lbl.as_ptr());

    // ── Macro ──
    let macro_menu = CreatePopupMenu();
    append_menu(macro_menu, IDM_MACRO_RECORD, "Start/Stop &Recording\tCtrl+Shift+R");
    append_menu(macro_menu, IDM_MACRO_PLAY, "&Play Last Macro\tCtrl+Shift+P");
    let lbl = wide("&Macro");
    AppendMenuW(menu_bar, MF_POPUP, macro_menu as usize, lbl.as_ptr());

    // ── Settings ──
    let settings_menu = CreatePopupMenu();
    append_menu(settings_menu, IDM_SETTINGS_PREFERENCES, "&Preferences...");
    append_menu(settings_menu, IDM_SETTINGS_SHORTCUTS, "&Keyboard Shortcuts...");
    let lbl = wide("S&ettings");
    AppendMenuW(menu_bar, MF_POPUP, settings_menu as usize, lbl.as_ptr());

    // ── Help ──
    let help_menu = CreatePopupMenu();
    append_menu(help_menu, IDM_HELP_ABOUT, "&About");
    let lbl = wide("&Help");
    AppendMenuW(menu_bar, MF_POPUP, help_menu as usize, lbl.as_ptr());

    menu_bar
}

unsafe fn append_menu(menu: HMENU, id: u16, text: &str) {
    let w = wide(text);
    AppendMenuW(menu, MF_STRING, id as usize, w.as_ptr());
}

unsafe fn create_controls(hwnd: HWND, hinstance: HINSTANCE) {
    let s = app();

    // Tab control
    let tab_class = wide("SysTabControl32");
    s.hwnd_tab = CreateWindowExW(
        0,
        tab_class.as_ptr(),
        std::ptr::null(),
        WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS,
        0,
        0,
        800,
        28,
        hwnd,
        ID_TAB_CONTROL as isize as HMENU,
        hinstance,
        std::ptr::null(),
    );

    // Scintilla editor — no WS_VSCROLL/WS_HSCROLL (Scintilla manages its own)
    let sci_class = wide("Scintilla");
    s.hwnd_scintilla = CreateWindowExW(
        0,
        sci_class.as_ptr(),
        std::ptr::null(),
        WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN,
        0,
        28,
        800,
        500,
        hwnd,
        ID_SCINTILLA as isize as HMENU,
        hinstance,
        std::ptr::null(),
    );
    sci_configure_dark(s.hwnd_scintilla);

    // Configure bookmark marker appearance
    sci_send(s.hwnd_scintilla, SCI_MARKERDEFINE, BOOKMARK_MARKER, SC_MARK_CIRCLE as isize);
    sci_send(s.hwnd_scintilla, SCI_MARKERSETFORE, BOOKMARK_MARKER, rgb(255, 255, 255) as isize);
    sci_send(s.hwnd_scintilla, SCI_MARKERSETBACK, BOOKMARK_MARKER, rgb(30, 120, 220) as isize);

    // Status bar
    let sb_class = wide("msctls_statusbar32");
    s.hwnd_status = CreateWindowExW(
        0,
        sb_class.as_ptr(),
        std::ptr::null(),
        WS_CHILD | WS_VISIBLE | SBARS_SIZEGRIP,
        0,
        0,
        0,
        0,
        hwnd,
        ID_STATUS_BAR as isize as HMENU,
        hinstance,
        std::ptr::null(),
    );

    // Set status bar parts: pos | encoding | eol | language
    let parts: [i32; 4] = [200, 350, 450, -1];
    SendMessageW(
        s.hwnd_status,
        SB_SETPARTS,
        parts.len(),
        parts.as_ptr() as LPARAM,
    );

    update_status_bar();
}

/// Build the Win32 keyboard accelerator table.
unsafe fn create_accelerators() -> HACCEL {
    // FVIRTKEY = 1, FSHIFT = 4, FCONTROL = 8
    const FVIRTKEY: u8 = 1;
    const FSHIFT: u8 = 4;
    const FCONTROL: u8 = 8;

    #[repr(C, packed)]
    #[derive(Copy, Clone)]
    struct ACCEL {
        f_virt: u8,
        key: u16,
        cmd: u16,
    }

    let entries: Vec<ACCEL> = vec![
        // File
        ACCEL { f_virt: FVIRTKEY | FCONTROL, key: b'N' as u16, cmd: IDM_FILE_NEW },
        ACCEL { f_virt: FVIRTKEY | FCONTROL, key: b'O' as u16, cmd: IDM_FILE_OPEN },
        ACCEL { f_virt: FVIRTKEY | FCONTROL, key: b'S' as u16, cmd: IDM_FILE_SAVE },
        ACCEL { f_virt: FVIRTKEY | FCONTROL | FSHIFT, key: b'S' as u16, cmd: IDM_FILE_SAVE_AS },
        ACCEL { f_virt: FVIRTKEY | FCONTROL, key: b'W' as u16, cmd: IDM_FILE_CLOSE },
        // Edit
        ACCEL { f_virt: FVIRTKEY | FCONTROL, key: b'Z' as u16, cmd: IDM_EDIT_UNDO },
        ACCEL { f_virt: FVIRTKEY | FCONTROL, key: b'Y' as u16, cmd: IDM_EDIT_REDO },
        ACCEL { f_virt: FVIRTKEY | FCONTROL, key: b'X' as u16, cmd: IDM_EDIT_CUT },
        ACCEL { f_virt: FVIRTKEY | FCONTROL, key: b'C' as u16, cmd: IDM_EDIT_COPY },
        ACCEL { f_virt: FVIRTKEY | FCONTROL, key: b'V' as u16, cmd: IDM_EDIT_PASTE },
        ACCEL { f_virt: FVIRTKEY | FCONTROL, key: b'A' as u16, cmd: IDM_EDIT_SELECT_ALL },
        // Search
        ACCEL { f_virt: FVIRTKEY | FCONTROL, key: b'F' as u16, cmd: IDM_SEARCH_FIND },
        ACCEL { f_virt: FVIRTKEY | FCONTROL, key: b'H' as u16, cmd: IDM_SEARCH_REPLACE },
        ACCEL { f_virt: FVIRTKEY | FCONTROL, key: b'G' as u16, cmd: IDM_SEARCH_GOTO_LINE },
        ACCEL { f_virt: FVIRTKEY | FCONTROL | FSHIFT, key: b'L' as u16, cmd: IDM_SEARCH_SELECT_ALL_OCCURRENCES },
        // View / Zoom
        ACCEL { f_virt: FVIRTKEY | FCONTROL, key: 0xBB, cmd: IDM_VIEW_ZOOM_IN },   // VK_OEM_PLUS (=)
        ACCEL { f_virt: FVIRTKEY | FCONTROL, key: 0xBD, cmd: IDM_VIEW_ZOOM_OUT },  // VK_OEM_MINUS (-)
        ACCEL { f_virt: FVIRTKEY | FCONTROL, key: b'0' as u16, cmd: IDM_VIEW_ZOOM_RESET },
        // Macro
        ACCEL { f_virt: FVIRTKEY | FCONTROL | FSHIFT, key: b'R' as u16, cmd: IDM_MACRO_RECORD },
        ACCEL { f_virt: FVIRTKEY | FCONTROL | FSHIFT, key: b'P' as u16, cmd: IDM_MACRO_PLAY },
    ];

    CreateAcceleratorTableW(
        entries.as_ptr() as *const _,
        entries.len() as i32,
    ) as HACCEL
}

// ── Window procedure ──
unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_SIZE => {
            on_size(hwnd);
            0
        }
        WM_COMMAND => {
            let cmd_id = (wparam & 0xFFFF) as u16;
            on_command(cmd_id);
            0
        }
        WM_NOTIFY => {
            let nmhdr = &*(lparam as *const NMHDR);
            on_notify(nmhdr);
            0
        }
        WM_CLOSE => {
            DestroyWindow(hwnd);
            0
        }
        WM_DROPFILES => {
            on_drop_files(wparam as HDROP);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn on_size(hwnd: HWND) {
    if APP.is_null() {
        return;
    }
    let s = app();
    let mut rc: RECT = std::mem::zeroed();
    GetClientRect(hwnd, &mut rc);
    let w = rc.right - rc.left;
    let h = rc.bottom - rc.top;

    // Status bar auto-sizes
    SendMessageW(s.hwnd_status, WM_SIZE, 0, 0);
    let mut sb_rect: RECT = std::mem::zeroed();
    GetWindowRect(s.hwnd_status, &mut sb_rect);
    let sb_h = sb_rect.bottom - sb_rect.top;

    // Tab control
    let tab_h: i32 = 28;
    MoveWindow(s.hwnd_tab, 0, 0, w, tab_h, TRUE);

    // Scintilla fills between tab and status bar
    let sci_y = tab_h;
    let sci_h = h - tab_h - sb_h;
    if sci_h > 0 {
        if s.split_mode == 0 || s.hwnd_scintilla2.is_null() {
            // No split — primary editor fills all
            MoveWindow(s.hwnd_scintilla, 0, sci_y, w, sci_h, TRUE);
        } else if s.split_mode == 1 {
            // Horizontal split — stacked
            let half = sci_h / 2;
            MoveWindow(s.hwnd_scintilla, 0, sci_y, w, half, TRUE);
            MoveWindow(s.hwnd_scintilla2, 0, sci_y + half, w, sci_h - half, TRUE);
        } else {
            // Vertical split — side by side
            let half = w / 2;
            MoveWindow(s.hwnd_scintilla, 0, sci_y, half, sci_h, TRUE);
            MoveWindow(s.hwnd_scintilla2, half, sci_y, w - half, sci_h, TRUE);
        }
    }
}

unsafe fn on_command(id: u16) {
    let langs = all_languages();
    let lang_end = IDM_LANG_BASE + langs.len() as u16;

    // Language menu range
    if id >= IDM_LANG_BASE && id < lang_end {
        cmd_set_language((id - IDM_LANG_BASE) as usize);
        return;
    }

    // Recent files range
    if id >= IDM_RECENT_BASE && id < IDM_RECENT_BASE + 10 {
        cmd_open_recent((id - IDM_RECENT_BASE) as usize);
        return;
    }

    match id {
        // ── File ──
        IDM_FILE_NEW => cmd_new_tab(),
        IDM_FILE_OPEN => cmd_open_file(),
        IDM_FILE_SAVE => cmd_save_file(false),
        IDM_FILE_SAVE_AS => cmd_save_file(true),
        IDM_FILE_SAVE_ALL => cmd_save_all(),
        IDM_FILE_CLOSE => cmd_close_tab(),
        IDM_FILE_CLOSE_ALL => cmd_close_all(),
        IDM_FILE_EXIT => {
            DestroyWindow(app().hwnd_main);
        }
        IDM_FILE_SAVE_SESSION => cmd_save_session(),
        IDM_FILE_LOAD_SESSION => cmd_load_session(),
        IDM_FILE_EXPORT_HTML => cmd_export_html(),
        IDM_FILE_EXPORT_RTF => cmd_export_rtf(),

        // ── Edit ──
        IDM_EDIT_UNDO => { sci_send(app().hwnd_scintilla, SCI_UNDO, 0, 0); }
        IDM_EDIT_REDO => { sci_send(app().hwnd_scintilla, SCI_REDO, 0, 0); }
        IDM_EDIT_CUT => { sci_send(app().hwnd_scintilla, SCI_CUT, 0, 0); }
        IDM_EDIT_COPY => { sci_send(app().hwnd_scintilla, SCI_COPY, 0, 0); }
        IDM_EDIT_PASTE => { sci_send(app().hwnd_scintilla, SCI_PASTE, 0, 0); }
        IDM_EDIT_DELETE => { sci_send(app().hwnd_scintilla, SCI_CLEAR, 0, 0); }
        IDM_EDIT_SELECT_ALL => { sci_send(app().hwnd_scintilla, SCI_SELECTALL, 0, 0); }
        IDM_EDIT_TOGGLE_COMMENT => { cmd_toggle_comment(); }

        // Line operations
        IDM_LINE_DUPLICATE => { sci_send(app().hwnd_scintilla, SCI_LINEDUP, 0, 0); }
        IDM_LINE_DELETE => { sci_send(app().hwnd_scintilla, SCI_LINEDELETE, 0, 0); }
        IDM_LINE_MOVE_UP => { sci_send(app().hwnd_scintilla, SCI_LINETRANSPOSE, 0, 0); /* swap with above */ }
        IDM_LINE_MOVE_DOWN => {
            // Move down: go to next line first, then transpose
            let s = app();
            let ln = sci_current_line(s.hwnd_scintilla);
            let total = sci_send(s.hwnd_scintilla, SCI_GETLINECOUNT, 0, 0) as usize;
            if ln < total {
                sci_send(s.hwnd_scintilla, SCI_GOTOLINE, ln, 0); // move to next (0-based)
                sci_send(s.hwnd_scintilla, SCI_LINETRANSPOSE, 0, 0);
            }
        }
        IDM_LINE_SORT_ASC | IDM_LINE_SORT_DESC | IDM_LINE_SORT_NOCASE | IDM_LINE_SORT_NUMERIC => {
            cmd_sort_lines(id);
        }
        IDM_LINE_REMOVE_EMPTY | IDM_LINE_REMOVE_DUPLICATE | IDM_LINE_TRIM_TRAILING
        | IDM_LINE_TRIM_LEADING | IDM_LINE_TRIM_BOTH | IDM_LINE_JOIN | IDM_LINE_SPLIT
        | IDM_LINE_INSERT_ABOVE | IDM_LINE_INSERT_BELOW | IDM_LINE_REVERSE => {
            cmd_line_operation(id);
        }

        // Case conversion
        IDM_CASE_UPPER => { sci_send(app().hwnd_scintilla, SCI_UPPERCASE, 0, 0); }
        IDM_CASE_LOWER => { sci_send(app().hwnd_scintilla, SCI_LOWERCASE, 0, 0); }
        IDM_CASE_TITLE | IDM_CASE_SENTENCE | IDM_CASE_INVERSE => { cmd_case_conversion(id); }

        // Encoding
        IDM_ENC_UTF8 => { cmd_set_encoding("UTF-8"); }
        IDM_ENC_UTF8_BOM => { cmd_set_encoding("UTF-8 BOM"); }
        IDM_ENC_UTF16_LE => { cmd_set_encoding("UTF-16 LE"); }
        IDM_ENC_UTF16_BE => { cmd_set_encoding("UTF-16 BE"); }
        IDM_ENC_ANSI => { cmd_set_encoding("ANSI"); }

        // Line endings
        IDM_EOL_CRLF => { cmd_set_eol(SC_EOL_CRLF, "CRLF"); }
        IDM_EOL_LF => { cmd_set_eol(SC_EOL_LF, "LF"); }
        IDM_EOL_CR => { cmd_set_eol(SC_EOL_CR, "CR"); }

        // ── Search ──
        IDM_SEARCH_FIND => { cmd_open_find_replace(false); }
        IDM_SEARCH_REPLACE => { cmd_open_find_replace(true); }
        IDM_SEARCH_FIND_IN_FILES => {
            cmd_find_in_files_dialog();
        }
        IDM_SEARCH_SELECT_ALL_OCCURRENCES => { cmd_select_all_occurrences(); }
        IDM_SEARCH_GOTO_LINE => { cmd_goto_line(); }
        IDM_SEARCH_BOOKMARK_TOGGLE => { cmd_bookmark_toggle(); }
        IDM_SEARCH_BOOKMARK_NEXT => { cmd_bookmark_next(); }
        IDM_SEARCH_BOOKMARK_PREV => { cmd_bookmark_prev(); }
        IDM_SEARCH_BOOKMARK_CLEAR => { cmd_bookmark_clear_all(); }

        // ── View ──
        IDM_VIEW_WORDWRAP => cmd_toggle_word_wrap(),
        IDM_VIEW_LINENUMBERS => cmd_toggle_line_numbers(),
        IDM_VIEW_WHITESPACE => cmd_toggle_whitespace(),
        IDM_VIEW_SPLIT_HORIZ => cmd_split_view(1),
        IDM_VIEW_SPLIT_VERT => cmd_split_view(2),
        IDM_VIEW_REMOVE_SPLIT => cmd_remove_split(),
        IDM_VIEW_ZOOM_IN => { sci_send(app().hwnd_scintilla, SCI_ZOOMIN, 0, 0); }
        IDM_VIEW_ZOOM_OUT => { sci_send(app().hwnd_scintilla, SCI_ZOOMOUT, 0, 0); }
        IDM_VIEW_ZOOM_RESET => { sci_send(app().hwnd_scintilla, SCI_SETZOOM, 0, 0); }
        IDM_VIEW_FOLD_TOGGLE => {
            let s = app();
            let ln = sci_current_line(s.hwnd_scintilla) - 1; // 0-based
            sci_send(s.hwnd_scintilla, SCI_TOGGLEFOLD, ln, 0);
        }
        IDM_VIEW_FOLD_ALL => {
            sci_send(app().hwnd_scintilla, SCI_FOLDALL, SC_FOLDACTION_CONTRACT as usize, 0);
        }
        IDM_VIEW_UNFOLD_ALL => {
            sci_send(app().hwnd_scintilla, SCI_FOLDALL, SC_FOLDACTION_EXPAND as usize, 0);
        }

        // ── Tools ──
        IDM_TOOL_JSON_FORMAT => { cmd_json_tool(crate::tools::json_tools::format_json); }
        IDM_TOOL_JSON_COMPACT => { cmd_json_tool(crate::tools::json_tools::compact_json); }
        IDM_TOOL_JSON_VALIDATE => { cmd_json_validate(); }
        IDM_TOOL_JSON_SORT_KEYS => { cmd_json_tool(crate::tools::json_tools::sort_json_keys); }
        IDM_TOOL_BASE64_ENCODE => { cmd_text_transform(crate::tools::mime_tools::base64_encode); }
        IDM_TOOL_BASE64_DECODE => { cmd_text_transform_result(crate::tools::mime_tools::base64_decode); }
        IDM_TOOL_URL_ENCODE => { cmd_text_transform(crate::tools::mime_tools::url_encode); }
        IDM_TOOL_URL_DECODE => { cmd_text_transform_result(crate::tools::mime_tools::url_decode); }
        IDM_TOOL_HEX_VIEWER => { cmd_hex_viewer(); }

        // ── Macro ──
        IDM_MACRO_RECORD => { cmd_macro_toggle_record(); }
        IDM_MACRO_PLAY => { cmd_macro_play(); }

        IDM_SETTINGS_PREFERENCES => cmd_preferences_dialog(),
        IDM_SETTINGS_SHORTCUTS => {
            let title = wide("Keyboard Shortcuts");
            let text = wide("Keyboard shortcuts can be customized in a future update.");
            MessageBoxW(app().hwnd_main, text.as_ptr(), title.as_ptr(), MB_OK | 0x40);
        }

        // ── Help ──
        IDM_HELP_ABOUT => cmd_about(),
        _ => {}
    }
}

unsafe fn on_notify(nmhdr: &NMHDR) {
    let s = app();

    // Tab change
    if nmhdr.hwndFrom == s.hwnd_tab && nmhdr.code == TCN_SELCHANGE as u32 {
        let new_idx = SendMessageW(s.hwnd_tab, TCM_GETCURSEL, 0, 0) as usize;
        switch_tab(new_idx);
    }

    // Scintilla notification (from either editor)
    if (nmhdr.hwndFrom == s.hwnd_scintilla || nmhdr.hwndFrom == s.hwnd_scintilla2)
        && nmhdr.code == SCN_UPDATEUI
    {
        update_status_bar();
    }

    // Macro recording notification
    if nmhdr.hwndFrom == s.hwnd_scintilla && nmhdr.code == SCN_MACRORECORD && s.recording {
        // The SCNotification struct has: code, ..., message, wParam, lParam at known offsets.
        // SCNotification layout: NMHDR (12 or 24 bytes on 64-bit), then fields.
        // On 64-bit: NMHDR is 24 bytes, then position(isize=8), ch(i32=4), modifiers(i32=4),
        //   modificationType(i32=4), padding(4), text(ptr=8), length(isize=8), linesAdded(isize=8),
        //   message(i32=4), wParam(usize=8), lParam(isize=8)
        // We use a simpler approach: cast to a raw pointer and offset.
        #[repr(C)]
        struct SCNotification {
            nmhdr: NMHDR,            // 24 bytes on x86_64
            position: isize,         // 8
            ch: i32,                 // 4
            modifiers: i32,          // 4
            modification_type: i32,  // 4
            _pad0: i32,              // 4 (padding)
            text: *const u8,         // 8
            length: isize,           // 8
            lines_added: isize,      // 8
            message: i32,            // 4
            _pad1: i32,              // 4 (padding)
            w_param: usize,          // 8
            l_param: isize,          // 8
        }
        let scn = &*(nmhdr as *const NMHDR as *const SCNotification);
        s.macro_buffer.push((scn.message as u32, scn.w_param, scn.l_param));
    }
}

unsafe fn on_drop_files(hdrop: HDROP) {
    let count = DragQueryFileW(hdrop, 0xFFFFFFFF, std::ptr::null_mut(), 0);
    for i in 0..count {
        let len = DragQueryFileW(hdrop, i, std::ptr::null_mut(), 0);
        let mut buf = vec![0u16; (len + 1) as usize];
        DragQueryFileW(hdrop, i, buf.as_mut_ptr(), buf.len() as u32);
        let path = wchar_to_string(&buf);
        if !path.is_empty() {
            open_file_in_tab(&path);
        }
    }
    DragFinish(hdrop);
}

// ── Tab management ──
unsafe fn cmd_new_tab() {
    let s = app();
    s.untitled_counter += 1;
    let doc = TabDocument::new_untitled(s.untitled_counter);
    let title = doc.title.clone();
    s.tabs.push(doc);

    // Add to tab control
    let idx = s.tabs.len() - 1;
    insert_tab_item(idx, &title);

    // Switch to new tab
    SendMessageW(s.hwnd_tab, TCM_SETCURSEL, idx, 0);
    switch_tab(idx);
}

unsafe fn insert_tab_item(idx: usize, title: &str) {
    let s = app();
    let w = wide(title);
    let mut item: TCITEMW = std::mem::zeroed();
    item.mask = TCIF_TEXT;
    item.pszText = w.as_ptr() as *mut u16;
    SendMessageW(
        s.hwnd_tab,
        TCM_INSERTITEMW,
        idx,
        &item as *const TCITEMW as LPARAM,
    );
}

unsafe fn switch_tab(new_idx: usize) {
    let s = app();
    if s.tabs.is_empty() || new_idx >= s.tabs.len() {
        return;
    }

    // Save current tab content
    if s.active_tab < s.tabs.len() {
        s.tabs[s.active_tab].text = sci_get_text(s.hwnd_scintilla);
    }

    s.active_tab = new_idx;

    // Load new tab content
    let text = s.tabs[new_idx].text.clone();
    sci_set_text(s.hwnd_scintilla, &text);
    sci_send(s.hwnd_scintilla, SCI_EMPTYUNDOBUFFER, 0, 0);
    sci_send(s.hwnd_scintilla, SCI_SETSAVEPOINT, 0, 0);
    sci_send(s.hwnd_scintilla, SCI_GOTOPOS, 0, 0);

    // Apply lexer based on extension
    apply_lexer_for_tab(new_idx);

    update_status_bar();
}

unsafe fn apply_lexer_for_tab(idx: usize) {
    let s = app();
    let ext = s.tabs[idx]
        .path
        .as_ref()
        .and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .unwrap_or("");

    if let Some(name) = lexer_for_extension(ext) {
        let mut cname = name.as_bytes().to_vec();
        cname.push(0);
        let lexer = CreateLexer(cname.as_ptr());
        if !lexer.is_null() {
            sci_send(s.hwnd_scintilla, SCI_SETILEXER, 0, lexer as isize);
            // Force re-colourise
            let len = sci_send(s.hwnd_scintilla, SCI_GETLENGTH, 0, 0);
            sci_send(s.hwnd_scintilla, SCI_COLOURISE, 0, len);
        }
    } else {
        // No lexer — plain text
        sci_send(s.hwnd_scintilla, SCI_SETILEXER, 0, 0);
    }
}

// ── File I/O ──
unsafe fn cmd_open_file() {
    let mut filename = [0u16; 1024];
    let filter = wide("All Files (*.*)\0*.*\0Text Files (*.txt)\0*.txt\0\0");
    let title = wide("Open File");

    let mut ofn: OPENFILENAMEW = std::mem::zeroed();
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = app().hwnd_main;
    ofn.lpstrFilter = filter.as_ptr();
    ofn.lpstrFile = filename.as_mut_ptr();
    ofn.nMaxFile = filename.len() as u32;
    ofn.lpstrTitle = title.as_ptr();
    ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST;

    if GetOpenFileNameW(&mut ofn) != 0 {
        let path = wchar_to_string(&filename);
        open_file_in_tab(&path);
    }
}

unsafe fn open_file_in_tab(path_str: &str) {
    let path = PathBuf::from(path_str);
    let Ok(content) = std::fs::read(&path) else {
        return;
    };
    let title = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| path_str.to_string());

    let s = app();
    let lang = {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        language_name_for_extension(ext)
    };
    let doc = TabDocument {
        path: Some(path.clone()),
        title: title.clone(),
        text: content,
        language: lang,
    };
    s.tabs.push(doc);
    let idx = s.tabs.len() - 1;
    insert_tab_item(idx, &title);
    SendMessageW(s.hwnd_tab, TCM_SETCURSEL, idx, 0);
    switch_tab(idx);

    // Add to recent files
    add_recent_file(path);
}

unsafe fn cmd_save_file(save_as: bool) {
    let s = app();
    if s.tabs.is_empty() {
        return;
    }

    // Update text from editor
    s.tabs[s.active_tab].text = sci_get_text(s.hwnd_scintilla);

    let need_dialog = save_as || s.tabs[s.active_tab].path.is_none();
    if need_dialog {
        let mut filename = [0u16; 1024];

        // Pre-fill with current filename if it exists
        if let Some(ref p) = s.tabs[s.active_tab].path {
            let w = wide(&p.to_string_lossy());
            let copy_len = w.len().min(filename.len());
            filename[..copy_len].copy_from_slice(&w[..copy_len]);
        }

        let filter = wide("All Files (*.*)\0*.*\0\0");
        let title = wide("Save File");

        let mut ofn: OPENFILENAMEW = std::mem::zeroed();
        ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.hwndOwner = s.hwnd_main;
        ofn.lpstrFilter = filter.as_ptr();
        ofn.lpstrFile = filename.as_mut_ptr();
        ofn.nMaxFile = filename.len() as u32;
        ofn.lpstrTitle = title.as_ptr();
        ofn.Flags = OFN_OVERWRITEPROMPT;

        if GetSaveFileNameW(&mut ofn) == 0 {
            return;
        }
        let path_str = wchar_to_string(&filename);
        let path = PathBuf::from(&path_str);
        let new_title = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or(path_str);
        s.tabs[s.active_tab].path = Some(path);
        s.tabs[s.active_tab].title = new_title.clone();

        // Update tab label
        let w = wide(&new_title);
        let mut item: TCITEMW = std::mem::zeroed();
        item.mask = TCIF_TEXT;
        item.pszText = w.as_ptr() as *mut u16;
        SendMessageW(
            s.hwnd_tab,
            TCM_SETITEMW,
            s.active_tab,
            &item as *const TCITEMW as LPARAM,
        );
    }

    if let Some(ref path) = s.tabs[s.active_tab].path {
        let _ = std::fs::write(path, &s.tabs[s.active_tab].text);
        sci_send(s.hwnd_scintilla, SCI_SETSAVEPOINT, 0, 0);
    }
    update_status_bar();
}

unsafe fn cmd_close_tab() {
    let s = app();
    if s.tabs.is_empty() {
        return;
    }
    let idx = s.active_tab;
    s.tabs.remove(idx);
    SendMessageW(s.hwnd_tab, TCM_DELETEITEM, idx, 0);

    if s.tabs.is_empty() {
        sci_set_text(s.hwnd_scintilla, b"");
        s.active_tab = 0;
        // Create a new untitled tab
        cmd_new_tab();
    } else {
        let new_idx = if idx >= s.tabs.len() {
            s.tabs.len() - 1
        } else {
            idx
        };
        SendMessageW(s.hwnd_tab, TCM_SETCURSEL, new_idx, 0);
        switch_tab(new_idx);
    }
}

// ── View toggles ──
unsafe fn cmd_toggle_word_wrap() {
    let s = app();
    s.word_wrap = !s.word_wrap;
    let mode = if s.word_wrap {
        SC_WRAP_WORD
    } else {
        SC_WRAP_NONE
    };
    sci_send(s.hwnd_scintilla, SCI_SETWRAPMODE, mode as usize, 0);

    // Update menu check
    let menu = GetMenu(s.hwnd_main);
    let flags = if s.word_wrap {
        MF_CHECKED
    } else {
        MF_UNCHECKED
    };
    CheckMenuItem(menu, IDM_VIEW_WORDWRAP as u32, flags);
}

unsafe fn cmd_toggle_line_numbers() {
    let s = app();
    s.line_numbers = !s.line_numbers;
    let width = if s.line_numbers { 48 } else { 0 };
    sci_send(s.hwnd_scintilla, SCI_SETMARGINWIDTHN, 0, width);

    let menu = GetMenu(s.hwnd_main);
    let flags = if s.line_numbers {
        MF_CHECKED
    } else {
        MF_UNCHECKED
    };
    CheckMenuItem(menu, IDM_VIEW_LINENUMBERS as u32, flags);
}

unsafe fn cmd_about() {
    let title = wide("About Notepad+++");
    let text = wide("Notepad+++\nA fast, native text editor for programmers\nBuilt with Rust + Scintilla\n\nVersion 0.1.0");
    MessageBoxW(app().hwnd_main, text.as_ptr(), title.as_ptr(), MB_OK);
}

// ── New file commands ──

unsafe fn cmd_save_all() {
    let s = app();
    // Save current editor content to active tab
    if !s.tabs.is_empty() {
        s.tabs[s.active_tab].text = sci_get_text(s.hwnd_scintilla);
    }
    for i in 0..s.tabs.len() {
        if let Some(ref path) = s.tabs[i].path {
            let _ = std::fs::write(path, &s.tabs[i].text);
        }
    }
    sci_send(s.hwnd_scintilla, SCI_SETSAVEPOINT, 0, 0);
}

unsafe fn cmd_close_all() {
    let s = app();
    while s.tabs.len() > 1 {
        s.tabs.pop();
        SendMessageW(s.hwnd_tab, TCM_DELETEITEM, s.tabs.len(), 0);
    }
    if !s.tabs.is_empty() {
        s.tabs.remove(0);
        SendMessageW(s.hwnd_tab, TCM_DELETEITEM, 0, 0);
    }
    s.active_tab = 0;
    sci_set_text(s.hwnd_scintilla, b"");
    cmd_new_tab();
}

// ── View toggles ──

unsafe fn cmd_toggle_whitespace() {
    let s = app();
    s.show_whitespace = !s.show_whitespace;
    let mode = if s.show_whitespace {
        SCWS_VISIBLEALWAYS
    } else {
        SCWS_INVISIBLE
    };
    sci_send(s.hwnd_scintilla, SCI_SETVIEWWS, mode as usize, 0);

    let menu = GetMenu(s.hwnd_main);
    let flags = if s.show_whitespace { MF_CHECKED } else { MF_UNCHECKED };
    CheckMenuItem(menu, IDM_VIEW_WHITESPACE as u32, flags);
}

// ── Encoding ──

unsafe fn cmd_set_encoding(name: &str) {
    let s = app();
    s.encoding = name.to_string();
    update_status_bar();
}

// ── Line endings ──

unsafe fn cmd_set_eol(mode: i32, label: &str) {
    let s = app();
    sci_send(s.hwnd_scintilla, SCI_SETEOLMODE, mode as usize, 0);
    s.line_ending = label.to_string();
    update_status_bar();
}

// ── Language ──

unsafe fn cmd_set_language(idx: usize) {
    let langs = all_languages();
    if idx >= langs.len() {
        return;
    }
    let (name, lexer_name) = langs[idx];
    let s = app();

    if !s.tabs.is_empty() {
        s.tabs[s.active_tab].language = name.to_string();
    }

    if lexer_name.is_empty() {
        // Plain text
        sci_send(s.hwnd_scintilla, SCI_SETILEXER, 0, 0);
    } else {
        let mut cname = lexer_name.as_bytes().to_vec();
        cname.push(0);
        let lexer = CreateLexer(cname.as_ptr());
        if !lexer.is_null() {
            sci_send(s.hwnd_scintilla, SCI_SETILEXER, 0, lexer as isize);
            let len = sci_send(s.hwnd_scintilla, SCI_GETLENGTH, 0, 0);
            sci_send(s.hwnd_scintilla, SCI_COLOURISE, 0, len);
        }
    }
    update_status_bar();
}

/// Map file extension to a human-readable language name.
fn language_name_for_extension(ext: &str) -> String {
    let ext_lower = ext.to_ascii_lowercase();
    for &(name, lexer) in all_languages() {
        if lexer.is_empty() {
            continue;
        }
        // Check if the lexer matches what lexer_for_extension returns
        if let Some(lex) = lexer_for_extension(&ext_lower) {
            if lex == lexer {
                return name.to_string();
            }
        }
    }
    "Plain Text".to_string()
}

// ── Go to line ──

unsafe fn cmd_goto_line() {
    let s = app();
    // Simple input via a prompt using a message box with input
    // Win32 doesn't have a built-in input dialog, so use a basic approach:
    // We'll use a GetDlgItemText pattern with a simple dialog.
    // For simplicity, use the clipboard trick or just prompt with InputBox.
    // Simplest: use a tiny dialog created inline.
    let total_lines = sci_send(s.hwnd_scintilla, SCI_GETLINECOUNT, 0, 0);
    let prompt = wide(&format!("Enter line number (1-{total_lines}):"));
    let title = wide("Go to Line");

    // Create a simple input buffer and use a message box
    // Since Win32 doesn't have InputBox, we'll prompt and accept a typed number
    // Using FindText as a workaround is complex. Use a simple approach:
    // Prompt with GetSaveFileNameW is hacky. Instead, just go to a line number
    // via an edit control in a custom dialog.
    //
    // For now, use a minimal approach: prompt the user via a simple dialog.
    // We'll create a modal dialog on the fly.
    goto_line_dialog(s.hwnd_main, s.hwnd_scintilla, total_lines as usize);
}

/// Show a simple "Go to Line" dialog.
unsafe fn goto_line_dialog(parent: HWND, hwnd_sci: HWND, max_line: usize) {
    // We use a simple approach: create a window class for the dialog
    let hinstance = GetModuleHandleW(std::ptr::null());

    // Store targets in statics for the dialog proc
    static mut DLG_SCI: HWND = std::ptr::null_mut();
    static mut DLG_EDIT: HWND = std::ptr::null_mut();
    static mut DLG_MAX: usize = 0;
    DLG_SCI = hwnd_sci;
    DLG_MAX = max_line;

    let class_name = wide("NPPPGoToLine");
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: 0,
        lpfnWndProc: Some(goto_dlg_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: std::ptr::null_mut(),
        hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
        hbrBackground: (COLOR_BTNFACE + 1) as *mut _,
        lpszMenuName: std::ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: std::ptr::null_mut(),
    };
    RegisterClassExW(&wc);

    let title = wide("Go to Line");
    let dlg = CreateWindowExW(
        WS_EX_DLGMODALFRAME,
        class_name.as_ptr(),
        title.as_ptr(),
        WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        280,
        130,
        parent,
        std::ptr::null_mut(),
        hinstance,
        std::ptr::null(),
    );

    // Create child controls
    let label_class = wide("STATIC");
    let label_text = wide(&format!("Line number (1-{max_line}):"));
    CreateWindowExW(
        0,
        label_class.as_ptr(),
        label_text.as_ptr(),
        WS_CHILD | WS_VISIBLE,
        10, 10, 250, 20,
        dlg,
        std::ptr::null_mut(),
        hinstance,
        std::ptr::null(),
    );

    let edit_class = wide("EDIT");
    DLG_EDIT = CreateWindowExW(
        WS_EX_CLIENTEDGE,
        edit_class.as_ptr(),
        std::ptr::null(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x2000, // ES_NUMBER
        10, 35, 250, 22,
        dlg,
        100 as isize as HMENU,
        hinstance,
        std::ptr::null(),
    );

    let btn_class = wide("BUTTON");
    let btn_text = wide("Go");
    CreateWindowExW(
        0,
        btn_class.as_ptr(),
        btn_text.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0001, // BS_DEFPUSHBUTTON
        100, 65, 80, 28,
        dlg,
        1 as isize as HMENU, // IDOK
        hinstance,
        std::ptr::null(),
    );

    SetFocus(DLG_EDIT);

    // Enable parent window disabled for modality
    EnableWindow(parent, FALSE);

    // Run a local message loop for this dialog
    let mut msg: MSG = std::mem::zeroed();
    while IsWindow(dlg) != 0 && GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
        if msg.message == WM_KEYDOWN && msg.wParam == 0x0D {
            // Enter key — act as OK
            SendMessageW(dlg, WM_COMMAND, 1, 0);
            continue;
        }
        if msg.message == WM_KEYDOWN && msg.wParam == 0x1B {
            // Escape
            DestroyWindow(dlg);
            break;
        }
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    EnableWindow(parent, TRUE);
    SetForegroundWindow(parent);

    unsafe extern "system" fn goto_dlg_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_COMMAND => {
                let cmd = (wparam & 0xFFFF) as u16;
                if cmd == 1 {
                    // OK button
                    let mut buf = [0u16; 32];
                    GetWindowTextW(DLG_EDIT, buf.as_mut_ptr(), buf.len() as i32);
                    let text = wchar_to_string(&buf);
                    if let Ok(line) = text.trim().parse::<usize>() {
                        if line >= 1 && line <= DLG_MAX {
                            sci_send(DLG_SCI, SCI_GOTOLINE, line - 1, 0);
                        }
                    }
                    DestroyWindow(hwnd);
                }
                0
            }
            WM_CLOSE => {
                DestroyWindow(hwnd);
                0
            }
            WM_DESTROY => {
                PostMessageW(hwnd, WM_QUIT, 0, 0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

// ── Bookmarks ──

unsafe fn cmd_bookmark_toggle() {
    let s = app();
    let line = sci_current_line(s.hwnd_scintilla) - 1; // 0-based
    let markers = sci_send(s.hwnd_scintilla, SCI_MARKERGET, line, 0);
    if markers & (1 << BOOKMARK_MARKER) != 0 {
        sci_send(s.hwnd_scintilla, SCI_MARKERDELETE, line, BOOKMARK_MARKER as isize);
    } else {
        sci_send(s.hwnd_scintilla, SCI_MARKERADD, line, BOOKMARK_MARKER as isize);
    }
}

unsafe fn cmd_bookmark_next() {
    let s = app();
    let line = sci_current_line(s.hwnd_scintilla); // 1-based, search from next
    let found = sci_send(s.hwnd_scintilla, SCI_MARKERNEXT, line, 1 << BOOKMARK_MARKER);
    if found >= 0 {
        sci_send(s.hwnd_scintilla, SCI_GOTOLINE, found as usize, 0);
    } else {
        // Wrap around from beginning
        let found = sci_send(s.hwnd_scintilla, SCI_MARKERNEXT, 0, 1 << BOOKMARK_MARKER);
        if found >= 0 {
            sci_send(s.hwnd_scintilla, SCI_GOTOLINE, found as usize, 0);
        }
    }
}

unsafe fn cmd_bookmark_prev() {
    let s = app();
    let line = sci_current_line(s.hwnd_scintilla) as isize - 2; // search from previous (0-based)
    let search_from = if line >= 0 { line as usize } else { 0 };
    let found = sci_send(s.hwnd_scintilla, SCI_MARKERPREVIOUS, search_from, 1 << BOOKMARK_MARKER);
    if found >= 0 {
        sci_send(s.hwnd_scintilla, SCI_GOTOLINE, found as usize, 0);
    } else {
        // Wrap around from end
        let total = sci_send(s.hwnd_scintilla, SCI_GETLINECOUNT, 0, 0) as usize;
        let found = sci_send(s.hwnd_scintilla, SCI_MARKERPREVIOUS, total, 1 << BOOKMARK_MARKER);
        if found >= 0 {
            sci_send(s.hwnd_scintilla, SCI_GOTOLINE, found as usize, 0);
        }
    }
}

unsafe fn cmd_bookmark_clear_all() {
    let s = app();
    sci_send(s.hwnd_scintilla, SCI_MARKERDELETEALL, BOOKMARK_MARKER, 0);
}

// ── Line operations (text-based) ──

unsafe fn cmd_sort_lines(id: u16) {
    let s = app();
    let text = sci_get_text(s.hwnd_scintilla);
    let content = String::from_utf8_lossy(&text);
    let mut lines: Vec<&str> = content.lines().collect();

    match id {
        IDM_LINE_SORT_ASC => lines.sort(),
        IDM_LINE_SORT_DESC => {
            lines.sort();
            lines.reverse();
        }
        IDM_LINE_SORT_NOCASE => lines.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase())),
        IDM_LINE_SORT_NUMERIC => lines.sort_by(|a, b| {
            let na: f64 = a.trim().parse().unwrap_or(f64::MAX);
            let nb: f64 = b.trim().parse().unwrap_or(f64::MAX);
            na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
        }),
        _ => {}
    }

    let result = lines.join("\n");
    sci_set_text(s.hwnd_scintilla, result.as_bytes());
}

unsafe fn cmd_line_operation(id: u16) {
    let s = app();
    let text = sci_get_text(s.hwnd_scintilla);
    let content = String::from_utf8_lossy(&text).to_string();

    let result = match id {
        IDM_LINE_REMOVE_EMPTY => {
            content.lines().filter(|l| !l.trim().is_empty()).collect::<Vec<_>>().join("\n")
        }
        IDM_LINE_REMOVE_DUPLICATE => {
            let mut seen = std::collections::HashSet::new();
            content.lines().filter(|l| seen.insert(l.to_string())).collect::<Vec<_>>().join("\n")
        }
        IDM_LINE_TRIM_TRAILING => {
            content.lines().map(|l| l.trim_end()).collect::<Vec<_>>().join("\n")
        }
        IDM_LINE_TRIM_LEADING => {
            content.lines().map(|l| l.trim_start()).collect::<Vec<_>>().join("\n")
        }
        IDM_LINE_TRIM_BOTH => {
            content.lines().map(|l| l.trim()).collect::<Vec<_>>().join("\n")
        }
        IDM_LINE_JOIN => {
            content.lines().collect::<Vec<_>>().join(" ")
        }
        IDM_LINE_SPLIT => {
            // Split at spaces
            content.replace(' ', "\n")
        }
        IDM_LINE_INSERT_ABOVE => {
            // Insert blank line above current line — operate on whole text for simplicity
            let line_idx = sci_current_line(s.hwnd_scintilla) - 1;
            let mut lines: Vec<&str> = content.lines().collect();
            if line_idx <= lines.len() {
                lines.insert(line_idx, "");
            }
            lines.join("\n")
        }
        IDM_LINE_INSERT_BELOW => {
            let line_idx = sci_current_line(s.hwnd_scintilla);
            let mut lines: Vec<&str> = content.lines().collect();
            if line_idx <= lines.len() {
                lines.insert(line_idx, "");
            }
            lines.join("\n")
        }
        IDM_LINE_REVERSE => {
            let mut lines: Vec<&str> = content.lines().collect();
            lines.reverse();
            lines.join("\n")
        }
        _ => return,
    };

    sci_set_text(s.hwnd_scintilla, result.as_bytes());
}

// ── Case conversion (non-Scintilla) ──

unsafe fn cmd_case_conversion(id: u16) {
    let s = app();
    let text = sci_get_text(s.hwnd_scintilla);
    let content = String::from_utf8_lossy(&text).to_string();

    let result = match id {
        IDM_CASE_TITLE => {
            content
                .split_whitespace()
                .map(|w| {
                    let mut chars = w.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(c) => {
                            let upper: String = c.to_uppercase().collect();
                            let lower: String = chars.as_str().to_lowercase();
                            format!("{upper}{lower}")
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
        IDM_CASE_SENTENCE => {
            let mut result = String::with_capacity(content.len());
            let mut capitalize_next = true;
            for ch in content.chars() {
                if capitalize_next && ch.is_alphabetic() {
                    for c in ch.to_uppercase() {
                        result.push(c);
                    }
                    capitalize_next = false;
                } else {
                    for c in ch.to_lowercase() {
                        result.push(c);
                    }
                    if ch == '.' || ch == '!' || ch == '?' {
                        capitalize_next = true;
                    }
                }
            }
            result
        }
        IDM_CASE_INVERSE => {
            content
                .chars()
                .map(|c| {
                    if c.is_uppercase() {
                        c.to_lowercase().collect::<String>()
                    } else {
                        c.to_uppercase().collect::<String>()
                    }
                })
                .collect::<String>()
        }
        _ => return,
    };

    sci_set_text(s.hwnd_scintilla, result.as_bytes());
}

// ── JSON tools ──

unsafe fn cmd_json_tool(f: fn(&str) -> Result<String, String>) {
    let s = app();
    let text = sci_get_text(s.hwnd_scintilla);
    let content = String::from_utf8_lossy(&text);
    match f(&content) {
        Ok(result) => {
            sci_set_text(s.hwnd_scintilla, result.as_bytes());
        }
        Err(e) => {
            let title = wide("JSON Error");
            let msg = wide(&e);
            MessageBoxW(s.hwnd_main, msg.as_ptr(), title.as_ptr(), MB_OK | 0x10);
        }
    }
}

unsafe fn cmd_json_validate() {
    let s = app();
    let text = sci_get_text(s.hwnd_scintilla);
    let content = String::from_utf8_lossy(&text);
    match crate::tools::json_tools::validate_json(&content) {
        Ok(()) => {
            let title = wide("JSON Validation");
            let msg = wide("Valid JSON");
            MessageBoxW(s.hwnd_main, msg.as_ptr(), title.as_ptr(), MB_OK | 0x40);
        }
        Err(e) => {
            let title = wide("JSON Validation");
            let msg = wide(&e);
            MessageBoxW(s.hwnd_main, msg.as_ptr(), title.as_ptr(), MB_OK | 0x10);
        }
    }
}

// ── Text transform tools ──

/// Infallible transform (e.g. base64_encode, url_encode).
unsafe fn cmd_text_transform(f: fn(&str) -> String) {
    let s = app();
    let text = sci_get_text(s.hwnd_scintilla);
    let content = String::from_utf8_lossy(&text);
    let result = f(&content);
    sci_set_text(s.hwnd_scintilla, result.as_bytes());
}

/// Fallible transform (e.g. base64_decode, url_decode).
unsafe fn cmd_text_transform_result(f: fn(&str) -> Result<String, String>) {
    let s = app();
    let text = sci_get_text(s.hwnd_scintilla);
    let content = String::from_utf8_lossy(&text);
    match f(&content) {
        Ok(result) => {
            sci_set_text(s.hwnd_scintilla, result.as_bytes());
        }
        Err(e) => {
            let title = wide("Error");
            let msg = wide(&e);
            MessageBoxW(s.hwnd_main, msg.as_ptr(), title.as_ptr(), MB_OK | 0x10);
        }
    }
}

// ── Find / Replace dialog (modeless) ──

// Control IDs for Find/Replace dialog
const IDC_FIND_EDIT: i32 = 2001;
const IDC_REPLACE_EDIT: i32 = 2002;
const IDC_FIND_NEXT: i32 = 2003;
const IDC_COUNT: i32 = 2004;
const IDC_FIND_ALL: i32 = 2005;
const IDC_REPLACE_BTN: i32 = 2006;
const IDC_REPLACE_ALL: i32 = 2007;
const IDC_MATCH_CASE: i32 = 2010;
const IDC_WHOLE_WORD: i32 = 2011;
const IDC_REGEX: i32 = 2012;
const IDC_WRAP_AROUND: i32 = 2013;
const IDC_DIR_UP: i32 = 2020;
const IDC_DIR_DOWN: i32 = 2021;
const IDC_REPLACE_LABEL: i32 = 2030;
const IDC_STATUS_LABEL: i32 = 2031;

// Find indicator number (avoid folding markers 25-31 and bookmark 1)
const FIND_INDICATOR: usize = 8;

static mut FIND_DLG_HWND: HWND = std::ptr::null_mut();
static mut FIND_DLG_SHOW_REPLACE: bool = false;

unsafe fn cmd_open_find_replace(show_replace: bool) {
    FIND_DLG_SHOW_REPLACE = show_replace;

    if !FIND_DLG_HWND.is_null() && IsWindow(FIND_DLG_HWND) != 0 {
        // Dialog already open — update replace visibility and focus
        find_dlg_update_replace_visibility();
        let edit = GetDlgItem(FIND_DLG_HWND, IDC_FIND_EDIT);
        SetFocus(edit);
        SendMessageW(edit, 0x00B1 /*EM_SETSEL*/, 0, -1isize); // select all
        SetForegroundWindow(FIND_DLG_HWND);
        return;
    }

    // Pre-fill find text from selection
    let s = app();
    let sel_start = sci_send(s.hwnd_scintilla, SCI_GETSELECTIONSTART, 0, 0) as usize;
    let sel_end = sci_send(s.hwnd_scintilla, SCI_GETSELECTIONEND, 0, 0) as usize;
    let sel_text = if sel_end > sel_start && (sel_end - sel_start) < 1024 {
        let mut buf = vec![0u8; sel_end - sel_start + 1];
        sci_send(s.hwnd_scintilla, SCI_GETSELTEXT, 0, buf.as_mut_ptr() as isize);
        let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        String::from_utf8_lossy(&buf[..len]).to_string()
    } else {
        String::new()
    };

    let hinstance = GetModuleHandleW(std::ptr::null());

    // Register class (once)
    let class_name = wide("NPPPFindReplace");
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: 0,
        lpfnWndProc: Some(find_dlg_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: std::ptr::null_mut(),
        hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
        hbrBackground: (COLOR_BTNFACE + 1) as *mut _,
        lpszMenuName: std::ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: std::ptr::null_mut(),
    };
    RegisterClassExW(&wc);

    let dlg_h = if show_replace { 280 } else { 250 };
    let title_str = if show_replace { "Replace" } else { "Find" };
    let title = wide(title_str);
    let dlg = CreateWindowExW(
        WS_EX_TOOLWINDOW,
        class_name.as_ptr(),
        title.as_ptr(),
        WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
        CW_USEDEFAULT, CW_USEDEFAULT, 420, dlg_h,
        s.hwnd_main,
        std::ptr::null_mut(),
        hinstance,
        std::ptr::null(),
    );
    FIND_DLG_HWND = dlg;

    let static_c = wide("STATIC");
    let edit_c = wide("EDIT");
    let btn_c = wide("BUTTON");

    // Find label + edit
    let lbl = wide("Find what:");
    CreateWindowExW(0, static_c.as_ptr(), lbl.as_ptr(),
        WS_CHILD | WS_VISIBLE, 10, 10, 80, 20, dlg,
        std::ptr::null_mut(), hinstance, std::ptr::null());

    let find_edit = CreateWindowExW(WS_EX_CLIENTEDGE, edit_c.as_ptr(), std::ptr::null(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0080/*ES_AUTOHSCROLL*/,
        95, 8, 305, 22, dlg,
        IDC_FIND_EDIT as isize as HMENU, hinstance, std::ptr::null());

    // Replace label + edit
    let lbl = wide("Replace with:");
    CreateWindowExW(0, static_c.as_ptr(), lbl.as_ptr(),
        WS_CHILD | WS_VISIBLE, 10, 38, 80, 20, dlg,
        IDC_REPLACE_LABEL as isize as HMENU, hinstance, std::ptr::null());

    CreateWindowExW(WS_EX_CLIENTEDGE, edit_c.as_ptr(), std::ptr::null(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0080,
        95, 36, 305, 22, dlg,
        IDC_REPLACE_EDIT as isize as HMENU, hinstance, std::ptr::null());

    // Buttons row
    let y_btns = 66;
    let mk_btn = |text: &str, x: i32, id: i32, def: bool| {
        let w = wide(text);
        let style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | if def { 0x0001 } else { 0 };
        CreateWindowExW(0, btn_c.as_ptr(), w.as_ptr(), style,
            x, y_btns, 90, 26, dlg, id as isize as HMENU, hinstance, std::ptr::null());
    };
    mk_btn("Find Next", 10, IDC_FIND_NEXT, true);
    mk_btn("Count", 105, IDC_COUNT, false);
    mk_btn("Find All", 200, IDC_FIND_ALL, false);

    // Replace buttons row
    let y_rep = y_btns + 30;
    mk_btn("Replace", 10, IDC_REPLACE_BTN, false);
    mk_btn("Replace All", 105, IDC_REPLACE_ALL, false);

    // Checkboxes
    let y_chk = y_rep + 36;
    let mk_chk = |text: &str, x: i32, y: i32, id: i32, checked: bool| {
        let w = wide(text);
        let style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0003/*BS_AUTOCHECKBOX*/;
        let h = CreateWindowExW(0, btn_c.as_ptr(), w.as_ptr(), style,
            x, y, 130, 20, dlg, id as isize as HMENU, hinstance, std::ptr::null());
        if checked {
            SendMessageW(h, 0x00F1/*BM_SETCHECK*/, 1/*BST_CHECKED*/, 0);
        }
    };
    mk_chk("Match case", 10, y_chk, IDC_MATCH_CASE, false);
    mk_chk("Whole word", 145, y_chk, IDC_WHOLE_WORD, false);
    mk_chk("Regular expression", 10, y_chk + 22, IDC_REGEX, false);
    mk_chk("Wrap around", 145, y_chk + 22, IDC_WRAP_AROUND, true);

    // Direction radio buttons
    let y_dir = y_chk + 50;
    let lbl = wide("Direction:");
    CreateWindowExW(0, static_c.as_ptr(), lbl.as_ptr(),
        WS_CHILD | WS_VISIBLE, 10, y_dir, 70, 20, dlg,
        std::ptr::null_mut(), hinstance, std::ptr::null());

    let w = wide("Up");
    CreateWindowExW(0, btn_c.as_ptr(), w.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0009/*BS_AUTORADIOBUTTON | WS_GROUP*/,
        80, y_dir, 50, 20, dlg, IDC_DIR_UP as isize as HMENU, hinstance, std::ptr::null());

    let w = wide("Down");
    let h_down = CreateWindowExW(0, btn_c.as_ptr(), w.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0009,
        135, y_dir, 60, 20, dlg, IDC_DIR_DOWN as isize as HMENU, hinstance, std::ptr::null());
    SendMessageW(h_down, 0x00F1/*BM_SETCHECK*/, 1, 0);

    // Status label
    let lbl = wide("");
    CreateWindowExW(0, static_c.as_ptr(), lbl.as_ptr(),
        WS_CHILD | WS_VISIBLE, 200, y_dir, 200, 20, dlg,
        IDC_STATUS_LABEL as isize as HMENU, hinstance, std::ptr::null());

    // Pre-fill find text if there was a selection
    if !sel_text.is_empty() {
        let w = wide(&sel_text);
        SetWindowTextW(find_edit, w.as_ptr());
    }

    find_dlg_update_replace_visibility();
    SetFocus(find_edit);
}

unsafe fn find_dlg_update_replace_visibility() {
    let show = FIND_DLG_SHOW_REPLACE;
    let dlg = FIND_DLG_HWND;
    let show_cmd = if show { SW_SHOW } else { SW_HIDE };

    let ids = [IDC_REPLACE_LABEL, IDC_REPLACE_EDIT, IDC_REPLACE_BTN, IDC_REPLACE_ALL];
    for &id in &ids {
        let h = GetDlgItem(dlg, id);
        if !h.is_null() {
            ShowWindow(h, show_cmd);
        }
    }

    // Resize dialog
    let h = if show { 280 } else { 250 };
    let mut rc: RECT = std::mem::zeroed();
    GetWindowRect(dlg, &mut rc);
    MoveWindow(dlg, rc.left, rc.top, 420, h, TRUE);

    let title = if show { "Replace" } else { "Find" };
    let w = wide(title);
    SetWindowTextW(dlg, w.as_ptr());
}

/// Get search flags from the Find dialog checkboxes.
unsafe fn find_dlg_search_flags() -> i32 {
    let dlg = FIND_DLG_HWND;
    let mut flags = 0i32;
    if SendMessageW(GetDlgItem(dlg, IDC_MATCH_CASE), 0x00F0/*BM_GETCHECK*/, 0, 0) != 0 {
        flags |= SCFIND_MATCHCASE;
    }
    if SendMessageW(GetDlgItem(dlg, IDC_WHOLE_WORD), 0x00F0, 0, 0) != 0 {
        flags |= SCFIND_WHOLEWORD;
    }
    if SendMessageW(GetDlgItem(dlg, IDC_REGEX), 0x00F0, 0, 0) != 0 {
        flags |= SCFIND_REGEXP;
    }
    flags
}

unsafe fn find_dlg_wrap_around() -> bool {
    SendMessageW(GetDlgItem(FIND_DLG_HWND, IDC_WRAP_AROUND), 0x00F0, 0, 0) != 0
}

unsafe fn find_dlg_direction_down() -> bool {
    SendMessageW(GetDlgItem(FIND_DLG_HWND, IDC_DIR_DOWN), 0x00F0, 0, 0) != 0
}

unsafe fn find_dlg_get_text(id: i32) -> String {
    let h = GetDlgItem(FIND_DLG_HWND, id);
    let mut buf = [0u16; 1024];
    GetWindowTextW(h, buf.as_mut_ptr(), buf.len() as i32);
    wchar_to_string(&buf)
}

unsafe fn find_dlg_set_status(msg: &str) {
    let h = GetDlgItem(FIND_DLG_HWND, IDC_STATUS_LABEL);
    if !h.is_null() {
        let w = wide(msg);
        SetWindowTextW(h, w.as_ptr());
    }
}

/// Find next/previous occurrence from current position.
unsafe fn cmd_find_next_in_dlg() {
    let search_text = find_dlg_get_text(IDC_FIND_EDIT);
    if search_text.is_empty() {
        return;
    }
    let s = app();
    let hwnd = s.hwnd_scintilla;
    let flags = find_dlg_search_flags();
    let down = find_dlg_direction_down();
    let wrap = find_dlg_wrap_around();
    let doc_len = sci_send(hwnd, SCI_GETLENGTH, 0, 0) as usize;
    let search_bytes = search_text.as_bytes();
    let mut needle = search_bytes.to_vec();
    needle.push(0);

    sci_send(hwnd, SCI_SETSEARCHFLAGS, flags as usize, 0);

    let (start, end) = if down {
        let sel_end = sci_send(hwnd, SCI_GETSELECTIONEND, 0, 0) as usize;
        (sel_end, doc_len)
    } else {
        let sel_start = sci_send(hwnd, SCI_GETSELECTIONSTART, 0, 0) as usize;
        (sel_start, 0)
    };

    sci_send(hwnd, SCI_SETTARGETSTART, start, 0);
    sci_send(hwnd, SCI_SETTARGETEND, end, 0);
    let pos = sci_send(hwnd, SCI_SEARCHINTARGET, search_bytes.len(), needle.as_ptr() as isize);

    if pos >= 0 {
        let match_end = sci_send(hwnd, SCI_GETTARGETEND, 0, 0) as usize;
        sci_send(hwnd, SCI_SETSEL, pos as usize, match_end as isize);
        sci_send(hwnd, SCI_SCROLLCARET, 0, 0);
        find_dlg_set_status("");
    } else if wrap {
        // Wrap around
        let (ws, we) = if down { (0, doc_len) } else { (doc_len, 0) };
        sci_send(hwnd, SCI_SETTARGETSTART, ws, 0);
        sci_send(hwnd, SCI_SETTARGETEND, we, 0);
        let pos = sci_send(hwnd, SCI_SEARCHINTARGET, search_bytes.len(), needle.as_ptr() as isize);
        if pos >= 0 {
            let match_end = sci_send(hwnd, SCI_GETTARGETEND, 0, 0) as usize;
            sci_send(hwnd, SCI_SETSEL, pos as usize, match_end as isize);
            sci_send(hwnd, SCI_SCROLLCARET, 0, 0);
            find_dlg_set_status("Wrapped");
        } else {
            find_dlg_set_status("Not found");
        }
    } else {
        find_dlg_set_status("Not found");
    }
}

/// Count all matches in the document.
unsafe fn cmd_count_matches() {
    let search_text = find_dlg_get_text(IDC_FIND_EDIT);
    if search_text.is_empty() {
        return;
    }
    let s = app();
    let hwnd = s.hwnd_scintilla;
    let flags = find_dlg_search_flags();
    let doc_len = sci_send(hwnd, SCI_GETLENGTH, 0, 0) as usize;
    let search_bytes = search_text.as_bytes();
    let mut needle = search_bytes.to_vec();
    needle.push(0);

    sci_send(hwnd, SCI_SETSEARCHFLAGS, flags as usize, 0);
    let mut count = 0usize;
    let mut pos = 0usize;

    loop {
        sci_send(hwnd, SCI_SETTARGETSTART, pos, 0);
        sci_send(hwnd, SCI_SETTARGETEND, doc_len, 0);
        let found = sci_send(hwnd, SCI_SEARCHINTARGET, search_bytes.len(), needle.as_ptr() as isize);
        if found < 0 {
            break;
        }
        count += 1;
        let match_end = sci_send(hwnd, SCI_GETTARGETEND, 0, 0) as usize;
        if match_end <= pos {
            break; // prevent infinite loop
        }
        pos = match_end;
    }

    find_dlg_set_status(&format!("{count} matches"));
}

/// Find All: highlight all matches with an indicator.
unsafe fn cmd_find_all() {
    let search_text = find_dlg_get_text(IDC_FIND_EDIT);
    if search_text.is_empty() {
        return;
    }
    let s = app();
    let hwnd = s.hwnd_scintilla;
    let flags = find_dlg_search_flags();
    let doc_len = sci_send(hwnd, SCI_GETLENGTH, 0, 0) as usize;
    let search_bytes = search_text.as_bytes();
    let mut needle = search_bytes.to_vec();
    needle.push(0);

    // Configure indicator
    sci_send(hwnd, SCI_INDICSETSTYLE, FIND_INDICATOR, INDIC_ROUNDBOX as isize);
    sci_send(hwnd, SCI_INDICSETFORE, FIND_INDICATOR, rgb(255, 150, 50) as isize);
    sci_send(hwnd, SCI_INDICSETALPHA, FIND_INDICATOR, 100);
    sci_send(hwnd, SCI_SETINDICATORCURRENT, FIND_INDICATOR, 0);

    // Clear previous highlights
    sci_send(hwnd, SCI_INDICATORCLEARRANGE, 0, doc_len as isize);

    sci_send(hwnd, SCI_SETSEARCHFLAGS, flags as usize, 0);
    let mut count = 0usize;
    let mut pos = 0usize;

    loop {
        sci_send(hwnd, SCI_SETTARGETSTART, pos, 0);
        sci_send(hwnd, SCI_SETTARGETEND, doc_len, 0);
        let found = sci_send(hwnd, SCI_SEARCHINTARGET, search_bytes.len(), needle.as_ptr() as isize);
        if found < 0 {
            break;
        }
        let match_end = sci_send(hwnd, SCI_GETTARGETEND, 0, 0) as usize;
        let match_len = match_end - found as usize;
        sci_send(hwnd, SCI_INDICATORFILLRANGE, found as usize, match_len as isize);
        count += 1;
        if match_end <= pos {
            break;
        }
        pos = match_end;
    }

    find_dlg_set_status(&format!("{count} matches highlighted"));
}

/// Replace the current selection (if it matches the find text) and find next.
unsafe fn cmd_replace_single() {
    let search_text = find_dlg_get_text(IDC_FIND_EDIT);
    let replace_text = find_dlg_get_text(IDC_REPLACE_EDIT);
    if search_text.is_empty() {
        return;
    }
    let s = app();
    let hwnd = s.hwnd_scintilla;
    let flags = find_dlg_search_flags();
    let search_bytes = search_text.as_bytes();
    let mut needle = search_bytes.to_vec();
    needle.push(0);

    // Check if current selection matches the find text
    let sel_start = sci_send(hwnd, SCI_GETSELECTIONSTART, 0, 0) as usize;
    let sel_end = sci_send(hwnd, SCI_GETSELECTIONEND, 0, 0) as usize;

    if sel_end > sel_start {
        sci_send(hwnd, SCI_SETSEARCHFLAGS, flags as usize, 0);
        sci_send(hwnd, SCI_SETTARGETSTART, sel_start, 0);
        sci_send(hwnd, SCI_SETTARGETEND, sel_end, 0);
        let found = sci_send(hwnd, SCI_SEARCHINTARGET, search_bytes.len(), needle.as_ptr() as isize);
        if found >= 0 && found as usize == sel_start {
            let target_end = sci_send(hwnd, SCI_GETTARGETEND, 0, 0) as usize;
            if target_end == sel_end {
                // Current selection matches — replace it
                let mut rep = replace_text.as_bytes().to_vec();
                rep.push(0);
                sci_send(hwnd, SCI_REPLACETARGET, replace_text.len(), rep.as_ptr() as isize);
            }
        }
    }

    // Find next
    cmd_find_next_in_dlg();
}

/// Replace all occurrences.
unsafe fn cmd_replace_all() {
    let search_text = find_dlg_get_text(IDC_FIND_EDIT);
    let replace_text = find_dlg_get_text(IDC_REPLACE_EDIT);
    if search_text.is_empty() {
        return;
    }
    let s = app();
    let hwnd = s.hwnd_scintilla;
    let flags = find_dlg_search_flags();
    let search_bytes = search_text.as_bytes();
    let mut needle = search_bytes.to_vec();
    needle.push(0);
    let mut rep = replace_text.as_bytes().to_vec();
    rep.push(0);

    sci_send(hwnd, SCI_SETSEARCHFLAGS, flags as usize, 0);
    sci_send(hwnd, SCI_BEGINUNDOACTION, 0, 0);

    let mut count = 0usize;
    let mut pos = 0usize;

    loop {
        let current_len = sci_send(hwnd, SCI_GETLENGTH, 0, 0) as usize;
        sci_send(hwnd, SCI_SETTARGETSTART, pos, 0);
        sci_send(hwnd, SCI_SETTARGETEND, current_len, 0);
        let found = sci_send(hwnd, SCI_SEARCHINTARGET, search_bytes.len(), needle.as_ptr() as isize);
        if found < 0 {
            break;
        }
        sci_send(hwnd, SCI_REPLACETARGET, replace_text.len(), rep.as_ptr() as isize);
        let new_end = sci_send(hwnd, SCI_GETTARGETEND, 0, 0) as usize;
        count += 1;
        if new_end <= pos {
            break;
        }
        pos = new_end;
    }

    sci_send(hwnd, SCI_ENDUNDOACTION, 0, 0);
    find_dlg_set_status(&format!("{count} replaced"));
}

unsafe extern "system" fn find_dlg_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let cmd = (wparam & 0xFFFF) as i32;
            match cmd {
                IDC_FIND_NEXT => { cmd_find_next_in_dlg(); }
                IDC_COUNT => { cmd_count_matches(); }
                IDC_FIND_ALL => { cmd_find_all(); }
                IDC_REPLACE_BTN => { cmd_replace_single(); }
                IDC_REPLACE_ALL => { cmd_replace_all(); }
                _ => {}
            }
            0
        }
        WM_CLOSE => {
            // Clear find indicators when closing
            let s = app();
            let hwnd_sci = s.hwnd_scintilla;
            let doc_len = sci_send(hwnd_sci, SCI_GETLENGTH, 0, 0) as usize;
            sci_send(hwnd_sci, SCI_SETINDICATORCURRENT, FIND_INDICATOR, 0);
            sci_send(hwnd_sci, SCI_INDICATORCLEARRANGE, 0, doc_len as isize);

            DestroyWindow(hwnd);
            FIND_DLG_HWND = std::ptr::null_mut();
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// ── Toggle Comment ──

/// Get comment prefix/suffix for a language name.
fn comment_style_for_language(lang: &str) -> (&'static str, &'static str) {
    match lang {
        "C" | "C++" | "C#" | "Java" | "JavaScript" | "TypeScript" | "Rust" | "Go"
        | "Swift" | "Kotlin" | "Scala" | "D" | "Objective-C" => ("//", ""),
        "Python" | "Ruby" | "Perl" | "Bash" | "PowerShell" | "R" | "Nim"
        | "YAML" | "TOML" | "INI/Properties" | "CMake" | "Makefile" => ("#", ""),
        "SQL" | "Lua" | "Haskell" => ("--", ""),
        "HTML" | "XML" => ("<!-- ", " -->"),
        "CSS" => ("/* ", " */"),
        "VB" => ("'", ""),
        "Batch" => ("REM ", ""),
        "LaTeX" => ("%", ""),
        "Julia" => ("#", ""),
        _ => ("//", ""),
    }
}

unsafe fn cmd_toggle_comment() {
    let s = app();
    let hwnd = s.hwnd_scintilla;

    let lang = if !s.tabs.is_empty() {
        s.tabs[s.active_tab].language.clone()
    } else {
        "Plain Text".to_string()
    };
    let (prefix, suffix) = comment_style_for_language(&lang);

    let sel_start = sci_send(hwnd, SCI_GETSELECTIONSTART, 0, 0) as usize;
    let sel_end = sci_send(hwnd, SCI_GETSELECTIONEND, 0, 0) as usize;

    let line_start = sci_send(hwnd, SCI_LINEFROMPOSITION, sel_start, 0) as usize;
    let line_end_pos = if sel_end > sel_start {
        sci_send(hwnd, SCI_LINEFROMPOSITION, sel_end, 0) as usize
    } else {
        line_start
    };

    // Collect lines
    let mut all_commented = true;
    let mut line_contents: Vec<String> = Vec::new();

    for line in line_start..=line_end_pos {
        let line_len = sci_send(hwnd, SCI_LINELENGTH, line, 0) as usize;
        if line_len == 0 {
            line_contents.push(String::new());
            continue;
        }
        let mut buf = vec![0u8; line_len + 1];
        sci_send(hwnd, SCI_GETLINE, line, buf.as_mut_ptr() as isize);
        buf.truncate(line_len);
        let text = String::from_utf8_lossy(&buf).to_string();
        let trimmed = text.trim_start();
        if !trimmed.is_empty() && !trimmed.starts_with(prefix) {
            all_commented = false;
        }
        line_contents.push(text);
    }

    sci_send(hwnd, SCI_BEGINUNDOACTION, 0, 0);

    // Apply toggle
    for (i, line) in line_contents.iter().enumerate() {
        let line_num = line_start + i;
        let pos_start = sci_send(hwnd, SCI_POSITIONFROMLINE, line_num, 0) as usize;

        if all_commented {
            // Remove comment
            let trimmed_start = line.len() - line.trim_start().len();
            if line.trim_start().starts_with(prefix) {
                // Remove prefix
                let prefix_pos = pos_start + trimmed_start;
                sci_send(hwnd, SCI_SETTARGETSTART, prefix_pos, 0);
                sci_send(hwnd, SCI_SETTARGETEND, prefix_pos + prefix.len(), 0);
                let empty = b"\0";
                sci_send(hwnd, SCI_REPLACETARGET, 0, empty.as_ptr() as isize);

                // Remove suffix if present
                if !suffix.is_empty() {
                    // Re-read line after prefix removal
                    let new_len = sci_send(hwnd, SCI_LINELENGTH, line_num, 0) as usize;
                    let mut buf2 = vec![0u8; new_len + 1];
                    sci_send(hwnd, SCI_GETLINE, line_num, buf2.as_mut_ptr() as isize);
                    buf2.truncate(new_len);
                    let text2 = String::from_utf8_lossy(&buf2).to_string();
                    let trimmed_end = text2.trim_end_matches(|c: char| c == '\r' || c == '\n');
                    if trimmed_end.ends_with(suffix) {
                        let suffix_start = pos_start + trimmed_end.len() - suffix.len();
                        // Account for prefix already removed
                        let adj_start = suffix_start;
                        sci_send(hwnd, SCI_SETTARGETSTART, adj_start, 0);
                        sci_send(hwnd, SCI_SETTARGETEND, adj_start + suffix.len(), 0);
                        sci_send(hwnd, SCI_REPLACETARGET, 0, empty.as_ptr() as isize);
                    }
                }
            }
        } else {
            // Add comment
            let trimmed = line.trim_start();
            if trimmed.is_empty() && line_contents.len() > 1 {
                continue; // skip empty lines in multi-line selection
            }
            let trimmed_start = line.len() - trimmed.len();
            let insert_pos = pos_start + trimmed_start;

            // Insert prefix
            let mut pfx = prefix.as_bytes().to_vec();
            pfx.push(0);
            sci_send(hwnd, SCI_INSERTTEXT, insert_pos, pfx.as_ptr() as isize);

            // Insert suffix at end of line if needed
            if !suffix.is_empty() {
                let new_len = sci_send(hwnd, SCI_LINELENGTH, line_num, 0) as usize;
                let new_pos_start = sci_send(hwnd, SCI_POSITIONFROMLINE, line_num, 0) as usize;
                let mut buf2 = vec![0u8; new_len + 1];
                sci_send(hwnd, SCI_GETLINE, line_num, buf2.as_mut_ptr() as isize);
                buf2.truncate(new_len);
                let text2 = String::from_utf8_lossy(&buf2).to_string();
                let trimmed_end = text2.trim_end_matches(|c: char| c == '\r' || c == '\n');
                let suffix_pos = new_pos_start + trimmed_end.len();
                let mut sfx = suffix.as_bytes().to_vec();
                sfx.push(0);
                sci_send(hwnd, SCI_INSERTTEXT, suffix_pos, sfx.as_ptr() as isize);
            }
        }
    }

    sci_send(hwnd, SCI_ENDUNDOACTION, 0, 0);
}

// ── Select All Occurrences ──

unsafe fn cmd_select_all_occurrences() {
    let s = app();
    let hwnd = s.hwnd_scintilla;

    // Get current selection as search term
    let sel_start = sci_send(hwnd, SCI_GETSELECTIONSTART, 0, 0) as usize;
    let sel_end = sci_send(hwnd, SCI_GETSELECTIONEND, 0, 0) as usize;
    if sel_end <= sel_start {
        return; // nothing selected
    }
    let sel_len = sel_end - sel_start;
    let mut buf = vec![0u8; sel_len + 1];
    sci_send(hwnd, SCI_GETSELTEXT, 0, buf.as_mut_ptr() as isize);
    let search_len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    if search_len == 0 {
        return;
    }
    let mut needle = buf[..search_len].to_vec();
    needle.push(0);

    // Enable multiple selection
    sci_send(hwnd, SCI_SETMULTIPLESELECTION, 1, 0);
    sci_send(hwnd, SCI_SETADDITIONALSELECTIONTYPING, 1, 0);

    // Search entire document for all occurrences
    let doc_len = sci_send(hwnd, SCI_GETLENGTH, 0, 0) as usize;
    sci_send(hwnd, SCI_SETSEARCHFLAGS, SCFIND_MATCHCASE as usize, 0);

    let mut first = true;
    let mut pos = 0usize;
    let mut main_idx = 0usize;
    let mut idx = 0usize;

    loop {
        sci_send(hwnd, SCI_SETTARGETSTART, pos, 0);
        sci_send(hwnd, SCI_SETTARGETEND, doc_len, 0);
        let found = sci_send(hwnd, SCI_SEARCHINTARGET, search_len, needle.as_ptr() as isize);
        if found < 0 {
            break;
        }
        let match_end = sci_send(hwnd, SCI_GETTARGETEND, 0, 0) as usize;

        if first {
            sci_send(hwnd, SCI_SETSEL, found as usize, match_end as isize);
            first = false;
        } else {
            sci_send(hwnd, SCI_ADDSELECTION, found as usize, match_end as isize);
        }

        if found as usize == sel_start {
            main_idx = idx;
        }
        idx += 1;

        if match_end <= pos {
            break;
        }
        pos = match_end;
    }

    if idx > 0 {
        sci_send(hwnd, SCI_SETMAINSELECTION, main_idx, 0);
    }
}

// ── Split View ──

unsafe fn cmd_split_view(mode: u8) {
    let s = app();
    // If already split, remove first
    if !s.hwnd_scintilla2.is_null() {
        cmd_remove_split();
    }
    s.split_mode = mode;

    let hinstance = GetModuleHandleW(std::ptr::null());
    let sci_class = wide("Scintilla");
    s.hwnd_scintilla2 = CreateWindowExW(
        0,
        sci_class.as_ptr(),
        std::ptr::null(),
        WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN,
        0, 0, 100, 100,
        s.hwnd_main,
        ID_SCINTILLA2 as isize as HMENU,
        hinstance,
        std::ptr::null(),
    );

    // Apply dark theme
    sci_configure_dark(s.hwnd_scintilla2);

    // Share the document from primary editor
    let doc_ptr = sci_send(s.hwnd_scintilla, SCI_GETDOCPOINTER, 0, 0);
    sci_send(s.hwnd_scintilla, SCI_ADDREFDOCUMENT, 0, doc_ptr);
    sci_send(s.hwnd_scintilla2, SCI_SETDOCPOINTER, 0, doc_ptr);

    // Apply same lexer
    if !s.tabs.is_empty() {
        apply_lexer_to_hwnd(s.hwnd_scintilla2, s.active_tab);
    }

    // Configure bookmark marker on secondary
    sci_send(s.hwnd_scintilla2, SCI_MARKERDEFINE, BOOKMARK_MARKER, SC_MARK_CIRCLE as isize);
    sci_send(s.hwnd_scintilla2, SCI_MARKERSETFORE, BOOKMARK_MARKER, rgb(255, 255, 255) as isize);
    sci_send(s.hwnd_scintilla2, SCI_MARKERSETBACK, BOOKMARK_MARKER, rgb(30, 120, 220) as isize);

    // Apply same word wrap / line number / whitespace settings
    let wrap_mode = if s.word_wrap { SC_WRAP_WORD } else { SC_WRAP_NONE };
    sci_send(s.hwnd_scintilla2, SCI_SETWRAPMODE, wrap_mode as usize, 0);
    let ln_width = if s.line_numbers { 48 } else { 0 };
    sci_send(s.hwnd_scintilla2, SCI_SETMARGINWIDTHN, 0, ln_width);
    let ws = if s.show_whitespace { SCWS_VISIBLEALWAYS } else { SCWS_INVISIBLE };
    sci_send(s.hwnd_scintilla2, SCI_SETVIEWWS, ws as usize, 0);

    // Trigger resize
    on_size(s.hwnd_main);
}

unsafe fn cmd_remove_split() {
    let s = app();
    if s.hwnd_scintilla2.is_null() {
        return;
    }
    // Release the shared document reference
    let doc_ptr = sci_send(s.hwnd_scintilla2, SCI_GETDOCPOINTER, 0, 0);
    sci_send(s.hwnd_scintilla2, SCI_SETDOCPOINTER, 0, 0); // detach
    sci_send(s.hwnd_scintilla, SCI_ADDREFDOCUMENT, 0, doc_ptr); // won't hurt if already owned
    // Actually we need to release. The primary still owns it. Just destroy.
    DestroyWindow(s.hwnd_scintilla2);
    s.hwnd_scintilla2 = std::ptr::null_mut();
    s.split_mode = 0;
    on_size(s.hwnd_main);
}

/// Apply lexer to a specific scintilla hwnd for a given tab index.
unsafe fn apply_lexer_to_hwnd(hwnd: HWND, idx: usize) {
    let s = app();
    let ext = s.tabs[idx]
        .path
        .as_ref()
        .and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .unwrap_or("");

    if let Some(name) = lexer_for_extension(ext) {
        let mut cname = name.as_bytes().to_vec();
        cname.push(0);
        let lexer = CreateLexer(cname.as_ptr());
        if !lexer.is_null() {
            sci_send(hwnd, SCI_SETILEXER, 0, lexer as isize);
            let len = sci_send(hwnd, SCI_GETLENGTH, 0, 0);
            sci_send(hwnd, SCI_COLOURISE, 0, len);
        }
    } else {
        sci_send(hwnd, SCI_SETILEXER, 0, 0);
    }
}

// ── Recent Files ──

fn recent_files_path() -> PathBuf {
    // Store next to the executable, or fallback to temp
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.join("notepadppp_recent.txt");
        }
    }
    std::env::temp_dir().join("notepadppp_recent.txt")
}

unsafe fn load_recent_files() {
    let path = recent_files_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        let s = app();
        s.recent_files = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .take(10)
            .map(PathBuf::from)
            .collect();
    }
}

unsafe fn save_recent_files() {
    let s = app();
    let content: String = s.recent_files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("\n");
    let _ = std::fs::write(recent_files_path(), content);
}

unsafe fn add_recent_file(path: PathBuf) {
    let s = app();
    // Remove duplicates
    s.recent_files.retain(|p| p != &path);
    // Insert at front
    s.recent_files.insert(0, path);
    // Keep max 10
    s.recent_files.truncate(10);
    save_recent_files();
    rebuild_recent_menu();
}

unsafe fn rebuild_recent_menu() {
    let s = app();
    let menu_bar = GetMenu(s.hwnd_main);
    // The Recent Files submenu is inside the File menu (first popup).
    // File menu is the first item in menu_bar.
    let file_menu = GetSubMenu(menu_bar, 0);
    // Find the Recent Files submenu by iterating menu items
    let count = GetMenuItemCount(file_menu);
    for i in 0..count {
        let sub = GetSubMenu(file_menu, i);
        if !sub.is_null() {
            // Check if this is the recent files submenu by checking item IDs
            let item_id = GetMenuItemID(sub, 0);
            // Our recent menu has items IDM_RECENT_BASE or the "(empty)" placeholder (id 0)
            if item_id == 0 || (item_id >= IDM_RECENT_BASE as u32 && item_id < (IDM_RECENT_BASE + 10) as u32) {
                // Clear existing items
                while GetMenuItemCount(sub) > 0 {
                    DeleteMenu(sub, 0, MF_BYPOSITION);
                }
                // Add recent files
                if s.recent_files.is_empty() {
                    append_menu(sub, 0, "(empty)");
                } else {
                    for (j, path) in s.recent_files.iter().enumerate() {
                        let label = format!("&{} {}", j + 1,
                            path.file_name()
                                .map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_else(|| path.to_string_lossy().to_string()));
                        append_menu(sub, IDM_RECENT_BASE + j as u16, &label);
                    }
                }
                break;
            }
        }
    }
}

unsafe fn cmd_open_recent(idx: usize) {
    let s = app();
    if idx >= s.recent_files.len() {
        return;
    }
    let path = s.recent_files[idx].clone();
    let path_str = path.to_string_lossy().to_string();
    if path.exists() {
        open_file_in_tab(&path_str);
    } else {
        let title = wide("File Not Found");
        let msg = wide(&format!("The file no longer exists:\n{}", path_str));
        MessageBoxW(s.hwnd_main, msg.as_ptr(), title.as_ptr(), MB_OK | 0x10);
        // Remove from list
        s.recent_files.remove(idx);
        save_recent_files();
        rebuild_recent_menu();
    }
}

// ── Hex Viewer ──

unsafe fn cmd_hex_viewer() {
    let s = app();

    if s.hex_mode {
        // Toggle off — restore original content
        sci_send(s.hwnd_scintilla, SCI_SETREADONLY, 0, 0);
        sci_set_text(s.hwnd_scintilla, &s.hex_original);
        s.hex_original.clear();
        s.hex_mode = false;
        // Re-apply lexer
        if !s.tabs.is_empty() {
            apply_lexer_for_tab(s.active_tab);
        }
    } else {
        // Toggle on — save original and show hex dump
        let raw = sci_get_text(s.hwnd_scintilla);
        s.hex_original = raw.clone();
        s.hex_mode = true;

        let hex_dump = format_hex_dump(&raw);
        // Set to plain text lexer
        sci_send(s.hwnd_scintilla, SCI_SETILEXER, 0, 0);
        sci_set_text(s.hwnd_scintilla, hex_dump.as_bytes());
        sci_send(s.hwnd_scintilla, SCI_SETREADONLY, 1, 0);
    }

    // Update menu check
    let menu = GetMenu(s.hwnd_main);
    let flags = if s.hex_mode { MF_CHECKED } else { MF_UNCHECKED };
    CheckMenuItem(menu, IDM_TOOL_HEX_VIEWER as u32, flags);
}

fn format_hex_dump(data: &[u8]) -> String {
    let mut result = String::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        let offset = i * 16;
        // Offset
        result.push_str(&format!("{:08X}  ", offset));
        // Hex bytes (two groups of 8)
        for j in 0..16 {
            if j == 8 {
                result.push(' ');
            }
            if j < chunk.len() {
                result.push_str(&format!("{:02X} ", chunk[j]));
            } else {
                result.push_str("   ");
            }
        }
        // ASCII
        result.push_str(" |");
        for &b in chunk {
            if b >= 0x20 && b <= 0x7E {
                result.push(b as char);
            } else {
                result.push('.');
            }
        }
        // Pad ASCII column
        for _ in chunk.len()..16 {
            result.push(' ');
        }
        result.push_str("|\n");
    }
    result
}

// ── Macro Recording/Playback ──

unsafe fn cmd_macro_toggle_record() {
    let s = app();
    if s.recording {
        // Stop recording
        sci_send(s.hwnd_scintilla, SCI_STOPRECORD, 0, 0);
        s.recording = false;
        // Update menu text
        let menu = GetMenu(s.hwnd_main);
        let w = wide("Start/Stop &Recording\tCtrl+Shift+R");
        let mut info: MENUITEMINFOW = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MENUITEMINFOW>() as u32;
        info.fMask = 0x0040; // MIIM_STRING
        info.dwTypeData = w.as_ptr() as *mut u16;
        SetMenuItemInfoW(menu, IDM_MACRO_RECORD as u32, FALSE, &info);
    } else {
        // Start recording
        s.macro_buffer.clear();
        sci_send(s.hwnd_scintilla, SCI_STARTRECORD, 0, 0);
        s.recording = true;
        // Update menu text to indicate recording
        let menu = GetMenu(s.hwnd_main);
        let w = wide("Stop &Recording\tCtrl+Shift+R");
        let mut info: MENUITEMINFOW = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MENUITEMINFOW>() as u32;
        info.fMask = 0x0040; // MIIM_STRING
        info.dwTypeData = w.as_ptr() as *mut u16;
        SetMenuItemInfoW(menu, IDM_MACRO_RECORD as u32, FALSE, &info);
    }
}

unsafe fn cmd_macro_play() {
    let s = app();
    if s.recording || s.macro_buffer.is_empty() {
        return;
    }
    let buffer = s.macro_buffer.clone();
    for &(msg, wparam, lparam) in &buffer {
        sci_send(s.hwnd_scintilla, msg, wparam, lparam);
    }
}

// ── Session Save/Load ──

unsafe fn cmd_save_session() {
    let s = app();
    // Update current tab text
    if !s.tabs.is_empty() {
        s.tabs[s.active_tab].text = sci_get_text(s.hwnd_scintilla);
    }

    let mut filename = [0u16; 1024];
    let filter = wide("Session Files (*.json)\0*.json\0All Files (*.*)\0*.*\0\0");
    let title = wide("Save Session");

    let mut ofn: OPENFILENAMEW = std::mem::zeroed();
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = s.hwnd_main;
    ofn.lpstrFilter = filter.as_ptr();
    ofn.lpstrFile = filename.as_mut_ptr();
    ofn.nMaxFile = filename.len() as u32;
    ofn.lpstrTitle = title.as_ptr();
    ofn.Flags = OFN_OVERWRITEPROMPT;

    if GetSaveFileNameW(&mut ofn) == 0 {
        return;
    }
    let path_str = wchar_to_string(&filename);

    let tabs: Vec<String> = s.tabs.iter().filter_map(|t| {
        t.path.as_ref().map(|p| p.to_string_lossy().to_string())
    }).collect();
    let session = serde_json::json!({
        "tabs": tabs,
        "active": s.active_tab,
    });
    let _ = std::fs::write(&path_str, serde_json::to_string_pretty(&session).unwrap_or_default());
}

unsafe fn cmd_load_session() {
    let mut filename = [0u16; 1024];
    let filter = wide("Session Files (*.json)\0*.json\0All Files (*.*)\0*.*\0\0");
    let title = wide("Load Session");

    let mut ofn: OPENFILENAMEW = std::mem::zeroed();
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = app().hwnd_main;
    ofn.lpstrFilter = filter.as_ptr();
    ofn.lpstrFile = filename.as_mut_ptr();
    ofn.nMaxFile = filename.len() as u32;
    ofn.lpstrTitle = title.as_ptr();
    ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST;

    if GetOpenFileNameW(&mut ofn) == 0 {
        return;
    }
    let path_str = wchar_to_string(&filename);

    let Ok(content) = std::fs::read_to_string(&path_str) else { return };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) else { return };

    let Some(tabs_arr) = val.get("tabs").and_then(|v| v.as_array()) else { return };
    let active = val.get("active").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

    cmd_close_all();

    // Close the default Untitled tab that cmd_close_all creates
    let s = app();
    if !s.tabs.is_empty() {
        s.tabs.remove(0);
        SendMessageW(s.hwnd_tab, TCM_DELETEITEM, 0, 0);
    }

    for entry in tabs_arr {
        if let Some(path) = entry.as_str() {
            open_file_in_tab(path);
        }
    }

    // If no tabs were opened, create an untitled one
    if app().tabs.is_empty() {
        cmd_new_tab();
    }

    let s = app();
    let target = if active < s.tabs.len() { active } else { 0 };
    SendMessageW(s.hwnd_tab, TCM_SETCURSEL, target, 0);
    switch_tab(target);
}

// ── Export HTML/RTF ──

unsafe fn cmd_export_html() {
    let s = app();
    if s.tabs.is_empty() { return; }

    let text_bytes = sci_get_text(s.hwnd_scintilla);
    let text = String::from_utf8_lossy(&text_bytes);
    let lang_class = s.tabs[s.active_tab].language.to_lowercase().replace(' ', "-");

    let html_escaped = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>{title}</title>
<style>
body {{ background: #1e1e1e; color: #d4d4d4; margin: 0; padding: 16px; font-family: Consolas, monospace; }}
pre {{ font-size: 11pt; line-height: 1.4; white-space: pre-wrap; word-wrap: break-word; }}
</style>
</head>
<body>
<pre class="{lang_class}">{content}</pre>
</body>
</html>"#,
        title = s.tabs[s.active_tab].title,
        lang_class = lang_class,
        content = html_escaped,
    );

    let mut filename = [0u16; 1024];
    let filter = wide("HTML Files (*.html)\0*.html\0All Files (*.*)\0*.*\0\0");
    let title = wide("Export as HTML");

    let mut ofn: OPENFILENAMEW = std::mem::zeroed();
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = s.hwnd_main;
    ofn.lpstrFilter = filter.as_ptr();
    ofn.lpstrFile = filename.as_mut_ptr();
    ofn.nMaxFile = filename.len() as u32;
    ofn.lpstrTitle = title.as_ptr();
    ofn.Flags = OFN_OVERWRITEPROMPT;

    if GetSaveFileNameW(&mut ofn) == 0 { return; }
    let path = wchar_to_string(&filename);
    let _ = std::fs::write(&path, html.as_bytes());
}

unsafe fn cmd_export_rtf() {
    let s = app();
    if s.tabs.is_empty() { return; }

    let text_bytes = sci_get_text(s.hwnd_scintilla);
    let text = String::from_utf8_lossy(&text_bytes);

    let mut rtf_body = String::new();
    for ch in text.chars() {
        match ch {
            '\\' => rtf_body.push_str("\\\\"),
            '{' => rtf_body.push_str("\\{"),
            '}' => rtf_body.push_str("\\}"),
            '\n' => rtf_body.push_str("\\par\n"),
            '\r' => {} // skip CR, handled with LF
            c if (c as u32) > 127 => {
                rtf_body.push_str(&format!("\\u{}?", c as i16));
            }
            c => rtf_body.push(c),
        }
    }

    let rtf = format!(
        "{{\\rtf1\\ansi\\deff0{{\\fonttbl{{\\f0 Consolas;}}}}\\f0\\fs20 {rtf_body}}}",
        rtf_body = rtf_body,
    );

    let mut filename = [0u16; 1024];
    let filter = wide("RTF Files (*.rtf)\0*.rtf\0All Files (*.*)\0*.*\0\0");
    let title = wide("Export as RTF");

    let mut ofn: OPENFILENAMEW = std::mem::zeroed();
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = s.hwnd_main;
    ofn.lpstrFilter = filter.as_ptr();
    ofn.lpstrFile = filename.as_mut_ptr();
    ofn.nMaxFile = filename.len() as u32;
    ofn.lpstrTitle = title.as_ptr();
    ofn.Flags = OFN_OVERWRITEPROMPT;

    if GetSaveFileNameW(&mut ofn) == 0 { return; }
    let path = wchar_to_string(&filename);
    let _ = std::fs::write(&path, rtf.as_bytes());
}

// ── Find in Files Dialog ──

const IDC_FIF_FIND_EDIT: i32 = 3001;
const IDC_FIF_DIR_EDIT: i32 = 3002;
const IDC_FIF_FILTER_EDIT: i32 = 3003;
const IDC_FIF_FIND_ALL: i32 = 3004;
const IDC_FIF_BROWSE: i32 = 3005;
const IDC_FIF_MATCH_CASE: i32 = 3010;
const IDC_FIF_WHOLE_WORD: i32 = 3011;
const IDC_FIF_REGEX: i32 = 3012;
const IDC_FIF_RECURSIVE: i32 = 3013;

static mut FIF_DLG_HWND: HWND = std::ptr::null_mut();

unsafe fn cmd_find_in_files_dialog() {
    if !FIF_DLG_HWND.is_null() && IsWindow(FIF_DLG_HWND) != 0 {
        SetForegroundWindow(FIF_DLG_HWND);
        return;
    }

    let s = app();
    let hinstance = GetModuleHandleW(std::ptr::null());

    let class_name = wide("NPPPFindInFiles");
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: 0,
        lpfnWndProc: Some(fif_dlg_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: std::ptr::null_mut(),
        hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
        hbrBackground: (COLOR_BTNFACE + 1) as *mut _,
        lpszMenuName: std::ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: std::ptr::null_mut(),
    };
    RegisterClassExW(&wc);

    let title = wide("Find in Files");
    let dlg = CreateWindowExW(
        WS_EX_TOOLWINDOW,
        class_name.as_ptr(),
        title.as_ptr(),
        WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
        CW_USEDEFAULT, CW_USEDEFAULT, 480, 260,
        s.hwnd_main,
        std::ptr::null_mut(),
        hinstance,
        std::ptr::null(),
    );
    FIF_DLG_HWND = dlg;

    let static_c = wide("STATIC");
    let edit_c = wide("EDIT");
    let btn_c = wide("BUTTON");

    // Find what
    let lbl = wide("Find what:");
    CreateWindowExW(0, static_c.as_ptr(), lbl.as_ptr(),
        WS_CHILD | WS_VISIBLE, 10, 12, 70, 20, dlg,
        std::ptr::null_mut(), hinstance, std::ptr::null());
    CreateWindowExW(WS_EX_CLIENTEDGE, edit_c.as_ptr(), std::ptr::null(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0080,
        85, 10, 375, 22, dlg,
        IDC_FIF_FIND_EDIT as isize as HMENU, hinstance, std::ptr::null());

    // Directory
    let lbl = wide("Directory:");
    CreateWindowExW(0, static_c.as_ptr(), lbl.as_ptr(),
        WS_CHILD | WS_VISIBLE, 10, 42, 70, 20, dlg,
        std::ptr::null_mut(), hinstance, std::ptr::null());
    // Pre-fill with the directory of the current file if available
    let dir_default = {
        let st = app();
        if !st.tabs.is_empty() {
            st.tabs[st.active_tab].path.as_ref()
                .and_then(|p| p.parent().map(|d| d.to_string_lossy().to_string()))
                .unwrap_or_default()
        } else {
            String::new()
        }
    };
    let dir_w = wide(&dir_default);
    CreateWindowExW(WS_EX_CLIENTEDGE, edit_c.as_ptr(), dir_w.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0080,
        85, 40, 295, 22, dlg,
        IDC_FIF_DIR_EDIT as isize as HMENU, hinstance, std::ptr::null());

    let lbl = wide("Browse...");
    CreateWindowExW(0, btn_c.as_ptr(), lbl.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP,
        385, 39, 75, 24, dlg,
        IDC_FIF_BROWSE as isize as HMENU, hinstance, std::ptr::null());

    // Filter
    let lbl = wide("Filter:");
    CreateWindowExW(0, static_c.as_ptr(), lbl.as_ptr(),
        WS_CHILD | WS_VISIBLE, 10, 72, 70, 20, dlg,
        std::ptr::null_mut(), hinstance, std::ptr::null());
    let default_filter = wide("*.*");
    CreateWindowExW(WS_EX_CLIENTEDGE, edit_c.as_ptr(), default_filter.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0080,
        85, 70, 375, 22, dlg,
        IDC_FIF_FILTER_EDIT as isize as HMENU, hinstance, std::ptr::null());

    // Checkboxes
    let y_chk = 102;
    let mk_chk = |text: &str, x: i32, y: i32, id: i32, checked: bool| {
        let w = wide(text);
        let style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0003;
        let h = CreateWindowExW(0, btn_c.as_ptr(), w.as_ptr(), style,
            x, y, 130, 20, dlg, id as isize as HMENU, hinstance, std::ptr::null());
        if checked {
            SendMessageW(h, 0x00F1, 1, 0);
        }
    };
    mk_chk("Match case", 10, y_chk, IDC_FIF_MATCH_CASE, false);
    mk_chk("Whole word", 145, y_chk, IDC_FIF_WHOLE_WORD, false);
    mk_chk("Regular expression", 280, y_chk, IDC_FIF_REGEX, false);
    mk_chk("Recursive", 10, y_chk + 24, IDC_FIF_RECURSIVE, true);

    // Find All button
    let lbl = wide("Find All");
    CreateWindowExW(0, btn_c.as_ptr(), lbl.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0001,
        190, y_chk + 55, 100, 30, dlg,
        IDC_FIF_FIND_ALL as isize as HMENU, hinstance, std::ptr::null());

    SetFocus(GetDlgItem(dlg, IDC_FIF_FIND_EDIT));
}

unsafe fn fif_get_text(id: i32) -> String {
    let h = GetDlgItem(FIF_DLG_HWND, id);
    let mut buf = [0u16; 1024];
    GetWindowTextW(h, buf.as_mut_ptr(), buf.len() as i32);
    wchar_to_string(&buf)
}

unsafe fn cmd_fif_execute() {
    let query = fif_get_text(IDC_FIF_FIND_EDIT);
    let dir = fif_get_text(IDC_FIF_DIR_EDIT);
    let filter = fif_get_text(IDC_FIF_FILTER_EDIT);

    if query.is_empty() || dir.is_empty() { return; }

    let case_sensitive = SendMessageW(GetDlgItem(FIF_DLG_HWND, IDC_FIF_MATCH_CASE), 0x00F0, 0, 0) != 0;
    let _whole_word = SendMessageW(GetDlgItem(FIF_DLG_HWND, IDC_FIF_WHOLE_WORD), 0x00F0, 0, 0) != 0;
    let use_regex = SendMessageW(GetDlgItem(FIF_DLG_HWND, IDC_FIF_REGEX), 0x00F0, 0, 0) != 0;
    let recursive = SendMessageW(GetDlgItem(FIF_DLG_HWND, IDC_FIF_RECURSIVE), 0x00F0, 0, 0) != 0;

    let dir_path = std::path::Path::new(&dir);
    match crate::search::find_in_files::FindInFiles::search(
        dir_path, &query, &filter, recursive, case_sensitive, use_regex,
    ) {
        Ok(results) => {
            let mut output = String::new();
            let mut total = 0usize;
            for file_result in &results {
                for m in &file_result.matches {
                    if total < 50 {
                        output.push_str(&format!(
                            "{}:{}: {}\n",
                            file_result.path.display(),
                            m.line + 1,
                            m.line_text.trim_end(),
                        ));
                    }
                    total += 1;
                }
            }
            if total == 0 {
                output = "No matches found.".to_string();
            } else if total > 50 {
                output.push_str(&format!("\n... and {} more matches", total - 50));
            }
            let header = format!("Found {} match(es) in {} file(s):\n\n", total, results.len());
            let msg_text = wide(&format!("{header}{output}"));
            let msg_title = wide("Find in Files Results");
            MessageBoxW(app().hwnd_main, msg_text.as_ptr(), msg_title.as_ptr(), MB_OK);
        }
        Err(e) => {
            let msg = wide(&format!("Search error: {e}"));
            let title = wide("Find in Files");
            MessageBoxW(app().hwnd_main, msg.as_ptr(), title.as_ptr(), MB_OK | 0x10);
        }
    }
}

unsafe fn fif_browse_directory() {
    // Use a simple folder selection via a Save dialog trick:
    // Open a file dialog and extract the directory from it.
    // Alternatively, use SHBrowseForFolderW — but that requires shell32 imports.
    // Simplest approach: use GetOpenFileNameW and strip the filename.
    let mut filename = [0u16; 1024];
    let filter = wide("All Files (*.*)\0*.*\0\0");
    let title = wide("Select any file in the target directory");

    let mut ofn: OPENFILENAMEW = std::mem::zeroed();
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = FIF_DLG_HWND;
    ofn.lpstrFilter = filter.as_ptr();
    ofn.lpstrFile = filename.as_mut_ptr();
    ofn.nMaxFile = filename.len() as u32;
    ofn.lpstrTitle = title.as_ptr();
    ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST;

    if GetOpenFileNameW(&mut ofn) != 0 {
        let path = wchar_to_string(&filename);
        if let Some(parent) = std::path::Path::new(&path).parent() {
            let dir_str = parent.to_string_lossy();
            let w = wide(&dir_str);
            SetWindowTextW(GetDlgItem(FIF_DLG_HWND, IDC_FIF_DIR_EDIT), w.as_ptr());
        }
    }
}

unsafe extern "system" fn fif_dlg_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let cmd = (wparam & 0xFFFF) as i32;
            match cmd {
                IDC_FIF_FIND_ALL => { cmd_fif_execute(); }
                IDC_FIF_BROWSE => { fif_browse_directory(); }
                _ => {}
            }
            0
        }
        WM_CLOSE => {
            DestroyWindow(hwnd);
            FIF_DLG_HWND = std::ptr::null_mut();
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// ── Settings / Preferences Dialog ──

const IDC_PREF_FONT_SIZE: i32 = 4001;
const IDC_PREF_TAB_SIZE: i32 = 4002;
const IDC_PREF_USE_SPACES: i32 = 4003;
const IDC_PREF_FOLD_MARGIN: i32 = 4004;
const IDC_PREF_OK: i32 = 4010;
const IDC_PREF_CANCEL: i32 = 4011;

static mut PREF_DLG_HWND: HWND = std::ptr::null_mut();

unsafe fn cmd_preferences_dialog() {
    if !PREF_DLG_HWND.is_null() && IsWindow(PREF_DLG_HWND) != 0 {
        SetForegroundWindow(PREF_DLG_HWND);
        return;
    }

    let s = app();
    let hinstance = GetModuleHandleW(std::ptr::null());

    let class_name = wide("NPPPPreferences");
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: 0,
        lpfnWndProc: Some(pref_dlg_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: std::ptr::null_mut(),
        hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
        hbrBackground: (COLOR_BTNFACE + 1) as *mut _,
        lpszMenuName: std::ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: std::ptr::null_mut(),
    };
    RegisterClassExW(&wc);

    let title = wide("Preferences");
    let dlg = CreateWindowExW(
        WS_EX_DLGMODALFRAME,
        class_name.as_ptr(),
        title.as_ptr(),
        WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
        CW_USEDEFAULT, CW_USEDEFAULT, 320, 230,
        s.hwnd_main,
        std::ptr::null_mut(),
        hinstance,
        std::ptr::null(),
    );
    PREF_DLG_HWND = dlg;

    let static_c = wide("STATIC");
    let edit_c = wide("EDIT");
    let btn_c = wide("BUTTON");

    // Get current values
    let cur_font_size = sci_send(s.hwnd_scintilla, SCI_STYLEGETSIZE, STYLE_DEFAULT, 0) as i32;
    let cur_tab_width = sci_send(s.hwnd_scintilla, SCI_GETTABWIDTH, 0, 0) as i32;
    let cur_use_tabs = sci_send(s.hwnd_scintilla, SCI_GETUSETABS, 0, 0) != 0;
    let cur_fold_margin = sci_send(s.hwnd_scintilla, SCI_GETMARGINWIDTHN, 2, 0) > 0;

    // Font size
    let lbl = wide("Font size:");
    CreateWindowExW(0, static_c.as_ptr(), lbl.as_ptr(),
        WS_CHILD | WS_VISIBLE, 10, 14, 90, 20, dlg,
        std::ptr::null_mut(), hinstance, std::ptr::null());
    let val = wide(&cur_font_size.to_string());
    CreateWindowExW(WS_EX_CLIENTEDGE, edit_c.as_ptr(), val.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x2000,
        110, 12, 60, 22, dlg,
        IDC_PREF_FONT_SIZE as isize as HMENU, hinstance, std::ptr::null());

    // Tab size
    let lbl = wide("Tab size:");
    CreateWindowExW(0, static_c.as_ptr(), lbl.as_ptr(),
        WS_CHILD | WS_VISIBLE, 10, 44, 90, 20, dlg,
        std::ptr::null_mut(), hinstance, std::ptr::null());
    let val = wide(&cur_tab_width.to_string());
    CreateWindowExW(WS_EX_CLIENTEDGE, edit_c.as_ptr(), val.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x2000,
        110, 42, 60, 22, dlg,
        IDC_PREF_TAB_SIZE as isize as HMENU, hinstance, std::ptr::null());

    // Use spaces for tabs
    let lbl = wide("Use spaces for tabs");
    let h = CreateWindowExW(0, btn_c.as_ptr(), lbl.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0003,
        10, 76, 180, 20, dlg,
        IDC_PREF_USE_SPACES as isize as HMENU, hinstance, std::ptr::null());
    if !cur_use_tabs {
        SendMessageW(h, 0x00F1, 1, 0);
    }

    // Show fold margin
    let lbl = wide("Show fold margin");
    let h = CreateWindowExW(0, btn_c.as_ptr(), lbl.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0003,
        10, 102, 180, 20, dlg,
        IDC_PREF_FOLD_MARGIN as isize as HMENU, hinstance, std::ptr::null());
    if cur_fold_margin {
        SendMessageW(h, 0x00F1, 1, 0);
    }

    // OK / Cancel
    let lbl = wide("OK");
    CreateWindowExW(0, btn_c.as_ptr(), lbl.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0001,
        60, 145, 80, 28, dlg,
        IDC_PREF_OK as isize as HMENU, hinstance, std::ptr::null());

    let lbl = wide("Cancel");
    CreateWindowExW(0, btn_c.as_ptr(), lbl.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP,
        170, 145, 80, 28, dlg,
        IDC_PREF_CANCEL as isize as HMENU, hinstance, std::ptr::null());

    EnableWindow(s.hwnd_main, FALSE);
}

unsafe fn pref_apply() {
    let dlg = PREF_DLG_HWND;
    let s = app();

    // Font size
    let mut buf = [0u16; 32];
    GetWindowTextW(GetDlgItem(dlg, IDC_PREF_FONT_SIZE), buf.as_mut_ptr(), buf.len() as i32);
    let text = wchar_to_string(&buf);
    if let Ok(size) = text.trim().parse::<isize>() {
        if size >= 6 && size <= 72 {
            sci_send(s.hwnd_scintilla, SCI_STYLESETSIZE, STYLE_DEFAULT, size);
            sci_send(s.hwnd_scintilla, SCI_STYLECLEARALL, 0, 0);
        }
    }

    // Tab size
    GetWindowTextW(GetDlgItem(dlg, IDC_PREF_TAB_SIZE), buf.as_mut_ptr(), buf.len() as i32);
    let text = wchar_to_string(&buf);
    if let Ok(size) = text.trim().parse::<usize>() {
        if size >= 1 && size <= 16 {
            sci_send(s.hwnd_scintilla, SCI_SETTABWIDTH, size, 0);
        }
    }

    // Use spaces for tabs (checkbox checked = use spaces = SCI_SETUSETABS(false))
    let use_spaces = SendMessageW(GetDlgItem(dlg, IDC_PREF_USE_SPACES), 0x00F0, 0, 0) != 0;
    sci_send(s.hwnd_scintilla, SCI_SETUSETABS, if use_spaces { 0 } else { 1 }, 0);

    // Fold margin
    let show_fold = SendMessageW(GetDlgItem(dlg, IDC_PREF_FOLD_MARGIN), 0x00F0, 0, 0) != 0;
    sci_send(s.hwnd_scintilla, SCI_SETMARGINWIDTHN, 2, if show_fold { 16 } else { 0 });
}

unsafe extern "system" fn pref_dlg_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let cmd = (wparam & 0xFFFF) as i32;
            match cmd {
                IDC_PREF_OK => {
                    pref_apply();
                    EnableWindow(app().hwnd_main, TRUE);
                    SetForegroundWindow(app().hwnd_main);
                    DestroyWindow(hwnd);
                    PREF_DLG_HWND = std::ptr::null_mut();
                }
                IDC_PREF_CANCEL => {
                    EnableWindow(app().hwnd_main, TRUE);
                    SetForegroundWindow(app().hwnd_main);
                    DestroyWindow(hwnd);
                    PREF_DLG_HWND = std::ptr::null_mut();
                }
                _ => {}
            }
            0
        }
        WM_CLOSE => {
            EnableWindow(app().hwnd_main, TRUE);
            SetForegroundWindow(app().hwnd_main);
            DestroyWindow(hwnd);
            PREF_DLG_HWND = std::ptr::null_mut();
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// ── Status bar ──
unsafe fn update_status_bar() {
    if APP.is_null() {
        return;
    }
    let s = app();
    if s.hwnd_status.is_null() || s.hwnd_scintilla.is_null() {
        return;
    }

    let ln = sci_current_line(s.hwnd_scintilla);
    let col = sci_current_col(s.hwnd_scintilla);
    set_sb_text(s.hwnd_status, 0, &format!("Ln {ln}, Col {col}"));

    set_sb_text(s.hwnd_status, 1, &s.encoding);

    set_sb_text(s.hwnd_status, 2, &s.line_ending);

    // Language from current tab
    let lang = if !s.tabs.is_empty() {
        s.tabs[s.active_tab].language.clone()
    } else {
        "Plain Text".to_string()
    };
    set_sb_text(s.hwnd_status, 3, &lang);
}

unsafe fn set_sb_text(hwnd: HWND, part: usize, text: &str) {
    let w = wide(text);
    SendMessageW(hwnd, SB_SETTEXTW, part, w.as_ptr() as LPARAM);
}

// ── Utility ──
fn wchar_to_string(buf: &[u16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}
