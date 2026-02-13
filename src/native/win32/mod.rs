//! Native Win32 + Scintilla backend for Notepad+++.

#![allow(unsafe_op_in_unsafe_fn)]

pub mod scintilla;

use std::path::PathBuf;

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::*;
use windows_sys::Win32::UI::Controls::Dialogs::*;
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

// ── Constants ──
const IDM_FILE_NEW: u16 = 101;
const IDM_FILE_OPEN: u16 = 102;
const IDM_FILE_SAVE: u16 = 103;
const IDM_FILE_SAVE_AS: u16 = 104;
const IDM_FILE_CLOSE: u16 = 105;
const IDM_FILE_EXIT: u16 = 106;
const IDM_EDIT_UNDO: u16 = 201;
const IDM_EDIT_REDO: u16 = 202;
const IDM_EDIT_CUT: u16 = 203;
const IDM_EDIT_COPY: u16 = 204;
const IDM_EDIT_PASTE: u16 = 205;
const IDM_EDIT_SELECT_ALL: u16 = 206;
const IDM_VIEW_WORDWRAP: u16 = 301;
const IDM_VIEW_LINENUMBERS: u16 = 302;
const IDM_HELP_ABOUT: u16 = 401;

const ID_TAB_CONTROL: i32 = 1000;
const ID_STATUS_BAR: i32 = 1001;
const ID_SCINTILLA: i32 = 1002;

// WM_NOTIFY codes for tab control
const TCN_FIRST: i32 = -550;
const TCN_SELCHANGE: i32 = TCN_FIRST - 1;

/// A single open document/tab.
struct TabDocument {
    path: Option<PathBuf>,
    title: String,
    text: Vec<u8>,
}

impl TabDocument {
    fn new_untitled(n: usize) -> Self {
        Self {
            path: None,
            title: format!("Untitled-{n}"),
            text: Vec::new(),
        }
    }
}

// ── App State (global, since Win32 WndProc is a C callback) ──
struct AppState {
    hwnd_main: HWND,
    hwnd_tab: HWND,
    hwnd_scintilla: HWND,
    hwnd_status: HWND,
    tabs: Vec<TabDocument>,
    active_tab: usize,
    untitled_counter: usize,
    word_wrap: bool,
    line_numbers: bool,
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

        // Init app state
        let state = Box::new(AppState {
            hwnd_main: hwnd,
            hwnd_tab: std::ptr::null_mut(),
            hwnd_scintilla: std::ptr::null_mut(),
            hwnd_status: std::ptr::null_mut(),
            tabs: Vec::new(),
            active_tab: 0,
            untitled_counter: 0,
            word_wrap: false,
            line_numbers: true,
        });
        APP = Box::into_raw(state);

        // Create child controls
        create_controls(hwnd, hinstance);

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
            // Switch to first opened file (index 1, since 0 is the untitled)
            SendMessageW((*APP).hwnd_tab, TCM_SETCURSEL, 1, 0);
            switch_tab(1);
            // Remove the untitled tab
            (*APP).tabs.remove(0);
            SendMessageW((*APP).hwnd_tab, TCM_DELETEITEM, 0, 0);
            (*APP).active_tab = 0;
            SendMessageW((*APP).hwnd_tab, TCM_SETCURSEL, 0, 0);
        }

        ShowWindow(hwnd, SW_SHOWMAXIMIZED);
        UpdateWindow(hwnd);

        // Message loop
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Cleanup
        let _ = Box::from_raw(APP);
        APP = std::ptr::null_mut();
    }
}

unsafe fn create_menu_bar() -> HMENU {
    let menu_bar = CreateMenu();

    // File
    let file_menu = CreatePopupMenu();
    append_menu(file_menu, IDM_FILE_NEW, "&New\tCtrl+N");
    append_menu(file_menu, IDM_FILE_OPEN, "&Open...\tCtrl+O");
    append_menu(file_menu, IDM_FILE_SAVE, "&Save\tCtrl+S");
    append_menu(file_menu, IDM_FILE_SAVE_AS, "Save &As...\tCtrl+Shift+S");
    AppendMenuW(file_menu, MF_SEPARATOR, 0, std::ptr::null());
    append_menu(file_menu, IDM_FILE_CLOSE, "&Close\tCtrl+W");
    append_menu(file_menu, IDM_FILE_EXIT, "E&xit\tAlt+F4");
    let file_label = wide("&File");
    AppendMenuW(menu_bar, MF_POPUP, file_menu as usize, file_label.as_ptr());

    // Edit
    let edit_menu = CreatePopupMenu();
    append_menu(edit_menu, IDM_EDIT_UNDO, "&Undo\tCtrl+Z");
    append_menu(edit_menu, IDM_EDIT_REDO, "&Redo\tCtrl+Y");
    AppendMenuW(edit_menu, MF_SEPARATOR, 0, std::ptr::null());
    append_menu(edit_menu, IDM_EDIT_CUT, "Cu&t\tCtrl+X");
    append_menu(edit_menu, IDM_EDIT_COPY, "&Copy\tCtrl+C");
    append_menu(edit_menu, IDM_EDIT_PASTE, "&Paste\tCtrl+V");
    AppendMenuW(edit_menu, MF_SEPARATOR, 0, std::ptr::null());
    append_menu(edit_menu, IDM_EDIT_SELECT_ALL, "Select &All\tCtrl+A");
    let edit_label = wide("&Edit");
    AppendMenuW(menu_bar, MF_POPUP, edit_menu as usize, edit_label.as_ptr());

    // View
    let view_menu = CreatePopupMenu();
    append_menu(view_menu, IDM_VIEW_WORDWRAP, "&Word Wrap");
    append_menu(view_menu, IDM_VIEW_LINENUMBERS, "&Line Numbers");
    let view_label = wide("&View");
    AppendMenuW(menu_bar, MF_POPUP, view_menu as usize, view_label.as_ptr());

    // Help
    let help_menu = CreatePopupMenu();
    append_menu(help_menu, IDM_HELP_ABOUT, "&About");
    let help_label = wide("&Help");
    AppendMenuW(menu_bar, MF_POPUP, help_menu as usize, help_label.as_ptr());

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

    // Scintilla editor
    let sci_class = wide("Scintilla");
    s.hwnd_scintilla = CreateWindowExW(
        0,
        sci_class.as_ptr(),
        std::ptr::null(),
        WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_HSCROLL | WS_CLIPCHILDREN,
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

    // Set status bar parts
    let parts: [i32; 4] = [200, 350, 450, -1];
    SendMessageW(
        s.hwnd_status,
        SB_SETPARTS,
        parts.len(),
        parts.as_ptr() as LPARAM,
    );

    update_status_bar();
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
    match id {
        IDM_FILE_NEW => cmd_new_tab(),
        IDM_FILE_OPEN => cmd_open_file(),
        IDM_FILE_SAVE => cmd_save_file(false),
        IDM_FILE_SAVE_AS => cmd_save_file(true),
        IDM_FILE_CLOSE => cmd_close_tab(),
        IDM_FILE_EXIT => {
            DestroyWindow(app().hwnd_main);
        }
        IDM_EDIT_UNDO => {
            sci_send(app().hwnd_scintilla, SCI_UNDO, 0, 0);
        }
        IDM_EDIT_REDO => {
            sci_send(app().hwnd_scintilla, SCI_REDO, 0, 0);
        }
        IDM_EDIT_CUT => {
            sci_send(app().hwnd_scintilla, SCI_CUT, 0, 0);
        }
        IDM_EDIT_COPY => {
            sci_send(app().hwnd_scintilla, SCI_COPY, 0, 0);
        }
        IDM_EDIT_PASTE => {
            sci_send(app().hwnd_scintilla, SCI_PASTE, 0, 0);
        }
        IDM_EDIT_SELECT_ALL => {
            sci_send(app().hwnd_scintilla, SCI_SELECTALL, 0, 0);
        }
        IDM_VIEW_WORDWRAP => cmd_toggle_word_wrap(),
        IDM_VIEW_LINENUMBERS => cmd_toggle_line_numbers(),
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
    let doc = TabDocument {
        path: Some(path),
        title: title.clone(),
        text: content,
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

    set_sb_text(s.hwnd_status, 1, "UTF-8");

    let eol_mode = sci_send(s.hwnd_scintilla, SCI_GETEOLMODE, 0, 0);
    let eol_str = match eol_mode as i32 {
        SC_EOL_CRLF => "CRLF",
        SC_EOL_LF => "LF",
        _ => "CR",
    };
    set_sb_text(s.hwnd_status, 2, eol_str);

    // Language from current tab extension
    let lang = if !s.tabs.is_empty() {
        let ext = s.tabs[s.active_tab]
            .path
            .as_ref()
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .unwrap_or("txt");
        ext.to_uppercase()
    } else {
        "TXT".to_string()
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
