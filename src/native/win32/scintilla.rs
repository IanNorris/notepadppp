//! Scintilla FFI constants and helpers for Win32.

#![allow(unsafe_op_in_unsafe_fn)]

use windows_sys::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::SendMessageW;

// --- Scintilla message constants ---
pub const SCI_SETTEXT: u32 = 2181;
pub const SCI_GETTEXT: u32 = 2182;
pub const SCI_GETLENGTH: u32 = 2006;
pub const SCI_GETCURRENTPOS: u32 = 2008;
pub const SCI_LINEFROMPOSITION: u32 = 2166;
pub const SCI_GETCOLUMN: u32 = 2129;
pub const SCI_GOTOPOS: u32 = 2025;
pub const SCI_SETUNDOCOLLECTION: u32 = 2012;
pub const SCI_EMPTYUNDOBUFFER: u32 = 2175;
pub const SCI_SETSAVEPOINT: u32 = 2014;
pub const SCI_UNDO: u32 = 2176;
pub const SCI_REDO: u32 = 2011;
pub const SCI_CUT: u32 = 2177;
pub const SCI_COPY: u32 = 2178;
pub const SCI_PASTE: u32 = 2179;
pub const SCI_SELECTALL: u32 = 2013;
pub const SCI_CLEARALL: u32 = 2004;
pub const SCI_GETLINECOUNT: u32 = 2154;
pub const SCI_GETLINE: u32 = 2153;

// Style
pub const SCI_STYLECLEARALL: u32 = 2050;
pub const SCI_STYLESETFORE: u32 = 2051;
pub const SCI_STYLESETBACK: u32 = 2052;
pub const SCI_STYLESETSIZE: u32 = 2055;
pub const SCI_STYLESETFONT: u32 = 2056;
pub const STYLE_DEFAULT: usize = 32;
pub const STYLE_LINENUMBER: usize = 33;

// Margin
pub const SCI_SETMARGINTYPEN: u32 = 2240;
pub const SCI_SETMARGINWIDTHN: u32 = 2242;
pub const SCI_SETMARGINMASKN: u32 = 2244;
pub const SCI_SETMARGINSENSITIVEN: u32 = 2246;
pub const SC_MARGIN_NUMBER: i32 = 1;
pub const SC_MARGIN_SYMBOL: i32 = 0;

// Markers / Folding
pub const SCI_MARKERDEFINE: u32 = 2040;
pub const SCI_MARKERSETFORE: u32 = 2041;
pub const SCI_MARKERSETBACK: u32 = 2042;
pub const SC_MARKNUM_FOLDEROPEN: usize = 31;
pub const SC_MARKNUM_FOLDER: usize = 30;
pub const SC_MARKNUM_FOLDERSUB: usize = 29;
pub const SC_MARKNUM_FOLDERTAIL: usize = 28;
pub const SC_MARKNUM_FOLDEREND: usize = 25;
pub const SC_MARKNUM_FOLDEROPENMID: usize = 26;
pub const SC_MARKNUM_FOLDERMIDTAIL: usize = 27;
pub const SC_MARK_BOXPLUS: i32 = 12;
pub const SC_MARK_BOXPLUSCONNECTED: i32 = 13;
pub const SC_MARK_BOXMINUS: i32 = 14;
pub const SC_MARK_BOXMINUSCONNECTED: i32 = 15;
pub const SC_MARK_LCORNER: i32 = 10;
pub const SC_MARK_TCORNER: i32 = 11;
pub const SC_MARK_VLINE: i32 = 9;

pub const SC_MASK_FOLDERS: u32 = 0xFE00_0000;
pub const SCI_SETFOLDFLAGS: u32 = 2233;
pub const SCI_SETPROPERTY: u32 = 4004;

// Caret line
pub const SCI_SETCARETLINEVISIBLE: u32 = 2096;
pub const SCI_SETCARETLINEBACK: u32 = 2098;
pub const SCI_SETCARETFORE: u32 = 2069;

// Wrap
pub const SCI_SETWRAPMODE: u32 = 2268;
pub const SC_WRAP_NONE: i32 = 0;
pub const SC_WRAP_WORD: i32 = 1;

// Tab
pub const SCI_SETTABWIDTH: u32 = 2036;

// Lexer
pub const SCI_SETILEXER: u32 = 4033;
pub const SCI_COLOURISE: u32 = 4003;

// EOL
pub const SCI_SETEOLMODE: u32 = 2031;
pub const SCI_GETEOLMODE: u32 = 2030;
pub const SC_EOL_CRLF: i32 = 0;
pub const SC_EOL_LF: i32 = 2;

// Codepage
pub const SCI_SETCODEPAGE: u32 = 2037;
pub const SC_CP_UTF8: i32 = 65001;

// Technology
pub const SCI_SETTECHNOLOGY: u32 = 2630;
pub const SC_TECHNOLOGY_DIRECTWRITE: i32 = 1;

// Notifications
pub const SCN_UPDATEUI: u32 = 2007;

// Helper: make an RGB color for Scintilla (0x00BBGGRR)
pub const fn rgb(r: u8, g: u8, b: u8) -> i32 {
    (r as i32) | ((g as i32) << 8) | ((b as i32) << 16)
}

/// Send a Scintilla message with no params.
#[inline]
pub unsafe fn sci_send(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> isize {
    SendMessageW(hwnd, msg, wparam as WPARAM, lparam as LPARAM)
}

/// Set editor text (UTF-8 byte slice).
pub unsafe fn sci_set_text(hwnd: HWND, text: &[u8]) {
    // Scintilla expects a null-terminated C string
    let mut buf = text.to_vec();
    if buf.last() != Some(&0) {
        buf.push(0);
    }
    sci_send(hwnd, SCI_SETTEXT, 0, buf.as_ptr() as isize);
}

/// Get editor text as a Vec<u8>.
pub unsafe fn sci_get_text(hwnd: HWND) -> Vec<u8> {
    let len = sci_send(hwnd, SCI_GETLENGTH, 0, 0) as usize;
    if len == 0 {
        return Vec::new();
    }
    let mut buf = vec![0u8; len + 1];
    sci_send(hwnd, SCI_GETTEXT, len + 1, buf.as_mut_ptr() as isize);
    buf.truncate(len);
    buf
}

/// Get current line number (1-based).
pub unsafe fn sci_current_line(hwnd: HWND) -> usize {
    let pos = sci_send(hwnd, SCI_GETCURRENTPOS, 0, 0);
    sci_send(hwnd, SCI_LINEFROMPOSITION, pos as usize, 0) as usize + 1
}

/// Get current column (1-based).
pub unsafe fn sci_current_col(hwnd: HWND) -> usize {
    let pos = sci_send(hwnd, SCI_GETCURRENTPOS, 0, 0);
    sci_send(hwnd, SCI_GETCOLUMN, pos as usize, 0) as usize + 1
}

/// Configure default dark theme.
pub unsafe fn sci_configure_dark(hwnd: HWND) {
    let bg = rgb(30, 30, 30);
    let fg = rgb(212, 212, 212);

    // UTF-8 codepage
    sci_send(hwnd, SCI_SETCODEPAGE, SC_CP_UTF8 as usize, 0);

    // Tab width
    sci_send(hwnd, SCI_SETTABWIDTH, 4, 0);

    // Default style
    let font = b"Consolas\0";
    sci_send(hwnd, SCI_STYLESETFONT, STYLE_DEFAULT, font.as_ptr() as isize);
    sci_send(hwnd, SCI_STYLESETSIZE, STYLE_DEFAULT, 11);
    sci_send(hwnd, SCI_STYLESETFORE, STYLE_DEFAULT, fg as isize);
    sci_send(hwnd, SCI_STYLESETBACK, STYLE_DEFAULT, bg as isize);
    sci_send(hwnd, SCI_STYLECLEARALL, 0, 0);

    // Line number margin
    sci_send(hwnd, SCI_SETMARGINTYPEN, 0, SC_MARGIN_NUMBER as isize);
    sci_send(hwnd, SCI_SETMARGINWIDTHN, 0, 48);
    sci_send(hwnd, SCI_STYLESETFORE, STYLE_LINENUMBER, rgb(130, 130, 130) as isize);
    sci_send(hwnd, SCI_STYLESETBACK, STYLE_LINENUMBER, rgb(37, 37, 38) as isize);

    // Folding margin
    sci_send(hwnd, SCI_SETMARGINTYPEN, 2, SC_MARGIN_SYMBOL as isize);
    sci_send(hwnd, SCI_SETMARGINWIDTHN, 2, 16);
    sci_send(hwnd, SCI_SETMARGINMASKN, 2, SC_MASK_FOLDERS as isize);
    sci_send(hwnd, SCI_SETMARGINSENSITIVEN, 2, 1);

    // Folder markers (box style)
    let marker_fg = rgb(60, 60, 60);
    let marker_bg = rgb(180, 180, 180);
    for &(num, mark) in &[
        (SC_MARKNUM_FOLDER, SC_MARK_BOXPLUS),
        (SC_MARKNUM_FOLDEROPEN, SC_MARK_BOXMINUS),
        (SC_MARKNUM_FOLDEREND, SC_MARK_BOXPLUSCONNECTED),
        (SC_MARKNUM_FOLDEROPENMID, SC_MARK_BOXMINUSCONNECTED),
        (SC_MARKNUM_FOLDERSUB, SC_MARK_VLINE),
        (SC_MARKNUM_FOLDERTAIL, SC_MARK_LCORNER),
        (SC_MARKNUM_FOLDERMIDTAIL, SC_MARK_TCORNER),
    ] {
        sci_send(hwnd, SCI_MARKERDEFINE, num, mark as isize);
        sci_send(hwnd, SCI_MARKERSETFORE, num, marker_bg as isize);
        sci_send(hwnd, SCI_MARKERSETBACK, num, marker_fg as isize);
    }

    // Caret line highlight
    sci_send(hwnd, SCI_SETCARETLINEVISIBLE, 1, 0);
    sci_send(hwnd, SCI_SETCARETLINEBACK, 0, rgb(40, 40, 40) as isize);
    sci_send(hwnd, SCI_SETCARETFORE, 0, rgb(220, 220, 220) as isize);

    // Fold properties
    let key = b"fold\0";
    let val = b"1\0";
    sci_send(hwnd, SCI_SETPROPERTY, key.as_ptr() as usize, val.as_ptr() as isize);
    let key2 = b"fold.compact\0";
    let val2 = b"0\0";
    sci_send(hwnd, SCI_SETPROPERTY, key2.as_ptr() as usize, val2.as_ptr() as isize);
    sci_send(hwnd, SCI_SETFOLDFLAGS, 16, 0);

    // CRLF by default (Windows)
    sci_send(hwnd, SCI_SETEOLMODE, SC_EOL_CRLF as usize, 0);
}

/// Map file extension to lexer name for CreateLexer.
pub fn lexer_for_extension(ext: &str) -> Option<&'static str> {
    match ext.to_ascii_lowercase().as_str() {
        "rs" => Some("rust"),
        "c" | "h" => Some("cpp"),
        "cc" | "cpp" | "cxx" | "hpp" | "hxx" | "hh" => Some("cpp"),
        "cs" => Some("cpp"),
        "java" => Some("cpp"),
        "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" => Some("cpp"),
        "py" | "pyw" => Some("python"),
        "rb" => Some("ruby"),
        "pl" | "pm" => Some("perl"),
        "html" | "htm" | "xhtml" => Some("hypertext"),
        "xml" | "xsl" | "xslt" | "svg" | "xaml" | "csproj" | "vcxproj" => Some("xml"),
        "css" | "scss" | "less" => Some("css"),
        "sql" => Some("sql"),
        "sh" | "bash" | "zsh" => Some("bash"),
        "bat" | "cmd" => Some("batch"),
        "ps1" | "psm1" => Some("powershell"),
        "md" | "markdown" => Some("markdown"),
        "json" => Some("json"),
        "yaml" | "yml" => Some("yaml"),
        "toml" => Some("toml"),
        "lua" => Some("lua"),
        "php" => Some("phpscript"),
        "r" => Some("r"),
        "cmake" => Some("cmake"),
        "makefile" | "mk" => Some("makefile"),
        "diff" | "patch" => Some("diff"),
        "ini" | "cfg" | "conf" | "properties" => Some("props"),
        "tex" | "latex" => Some("latex"),
        "asm" | "s" => Some("asm"),
        "pas" | "pp" | "dpr" => Some("pascal"),
        "tcl" => Some("tcl"),
        "vb" | "vbs" => Some("vb"),
        _ => None,
    }
}
