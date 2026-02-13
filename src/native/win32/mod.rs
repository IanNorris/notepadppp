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

// Control IDs
const ID_TAB_CONTROL: i32 = 1000;
const ID_STATUS_BAR: i32 = 1001;
const ID_SCINTILLA: i32 = 1002;

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
        });
        APP = Box::into_raw(state);

        // Create child controls
        create_controls(hwnd, hinstance);

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

        // Message loop with accelerator support
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
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
        MoveWindow(s.hwnd_scintilla, 0, sci_y, w, sci_h, TRUE);
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
        // Placeholders
        IDM_FILE_SAVE_SESSION | IDM_FILE_LOAD_SESSION
        | IDM_FILE_EXPORT_HTML | IDM_FILE_EXPORT_RTF => {
            show_todo("This feature is not yet implemented.");
        }

        // ── Edit ──
        IDM_EDIT_UNDO => { sci_send(app().hwnd_scintilla, SCI_UNDO, 0, 0); }
        IDM_EDIT_REDO => { sci_send(app().hwnd_scintilla, SCI_REDO, 0, 0); }
        IDM_EDIT_CUT => { sci_send(app().hwnd_scintilla, SCI_CUT, 0, 0); }
        IDM_EDIT_COPY => { sci_send(app().hwnd_scintilla, SCI_COPY, 0, 0); }
        IDM_EDIT_PASTE => { sci_send(app().hwnd_scintilla, SCI_PASTE, 0, 0); }
        IDM_EDIT_DELETE => { sci_send(app().hwnd_scintilla, SCI_CLEAR, 0, 0); }
        IDM_EDIT_SELECT_ALL => { sci_send(app().hwnd_scintilla, SCI_SELECTALL, 0, 0); }
        IDM_EDIT_TOGGLE_COMMENT => { show_todo("Toggle comment not yet implemented."); }

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
        IDM_SEARCH_FIND | IDM_SEARCH_REPLACE | IDM_SEARCH_FIND_IN_FILES
        | IDM_SEARCH_SELECT_ALL_OCCURRENCES => {
            show_todo("Find/Replace is not yet implemented.");
        }
        IDM_SEARCH_GOTO_LINE => { cmd_goto_line(); }
        IDM_SEARCH_BOOKMARK_TOGGLE => { cmd_bookmark_toggle(); }
        IDM_SEARCH_BOOKMARK_NEXT => { cmd_bookmark_next(); }
        IDM_SEARCH_BOOKMARK_PREV => { cmd_bookmark_prev(); }
        IDM_SEARCH_BOOKMARK_CLEAR => { cmd_bookmark_clear_all(); }

        // ── View ──
        IDM_VIEW_WORDWRAP => cmd_toggle_word_wrap(),
        IDM_VIEW_LINENUMBERS => cmd_toggle_line_numbers(),
        IDM_VIEW_WHITESPACE => cmd_toggle_whitespace(),
        IDM_VIEW_SPLIT_HORIZ | IDM_VIEW_SPLIT_VERT | IDM_VIEW_REMOVE_SPLIT => {
            show_todo("Split view is not yet implemented.");
        }
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
        IDM_TOOL_HEX_VIEWER => { show_todo("Hex viewer is not yet implemented."); }

        // ── Macro (placeholder) ──
        IDM_MACRO_RECORD | IDM_MACRO_PLAY => {
            show_todo("Macro recording is not yet implemented.");
        }

        // ── Settings (placeholder) ──
        IDM_SETTINGS_PREFERENCES | IDM_SETTINGS_SHORTCUTS => {
            show_todo("Settings dialogs are not yet implemented.");
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

    // Scintilla notification
    if nmhdr.hwndFrom == s.hwnd_scintilla && nmhdr.code == SCN_UPDATEUI {
        update_status_bar();
    }
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
        path: Some(path),
        title: title.clone(),
        text: content,
        language: lang,
    };
    s.tabs.push(doc);
    let idx = s.tabs.len() - 1;
    insert_tab_item(idx, &title);
    SendMessageW(s.hwnd_tab, TCM_SETCURSEL, idx, 0);
    switch_tab(idx);
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

// ── Placeholder helper ──

unsafe fn show_todo(msg: &str) {
    let title = wide("Not Yet Implemented");
    let text = wide(msg);
    MessageBoxW(app().hwnd_main, text.as_ptr(), title.as_ptr(), MB_OK | 0x40);
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
