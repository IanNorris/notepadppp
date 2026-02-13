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
pub const SCI_STYLESETBOLD: u32 = 2053;
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
pub const SCI_GETTABWIDTH: u32 = 2121;
pub const SCI_SETUSETABS: u32 = 2124;
pub const SCI_GETUSETABS: u32 = 2125;

// Style get
pub const SCI_STYLEGETSIZE: u32 = 2485;

// Margin get
pub const SCI_GETMARGINWIDTHN: u32 = 2243;

// Lexer
pub const SCI_SETILEXER: u32 = 4033;
pub const SCI_COLOURISE: u32 = 4003;

// EOL
pub const SCI_SETEOLMODE: u32 = 2031;
pub const SCI_GETEOLMODE: u32 = 2030;
pub const SC_EOL_CRLF: i32 = 0;
pub const SC_EOL_CR: i32 = 1;
pub const SC_EOL_LF: i32 = 2;

// Zoom
pub const SCI_ZOOMIN: u32 = 2333;
pub const SCI_ZOOMOUT: u32 = 2334;
pub const SCI_SETZOOM: u32 = 2373;

// Folding actions
pub const SCI_TOGGLEFOLD: u32 = 2231;
pub const SCI_FOLDALL: u32 = 2662;
pub const SC_FOLDACTION_TOGGLE: i32 = 2;
pub const SC_FOLDACTION_CONTRACT: i32 = 0;
pub const SC_FOLDACTION_EXPAND: i32 = 1;

// Bookmarks
pub const SCI_MARKERADD: u32 = 2043;
pub const SCI_MARKERDELETE: u32 = 2044;
pub const SCI_MARKERDELETEALL: u32 = 2045;
pub const SCI_MARKERGET: u32 = 2046;
pub const SCI_MARKERNEXT: u32 = 2047;
pub const SCI_MARKERPREVIOUS: u32 = 2048;
pub const SC_MARK_CIRCLE: i32 = 0;
pub const SC_MARK_BACKGROUND: i32 = 22;

// Whitespace visibility
pub const SCI_SETVIEWWS: u32 = 2021;
pub const SCI_GETVIEWWS: u32 = 2020;
pub const SCWS_INVISIBLE: i32 = 0;
pub const SCWS_VISIBLEALWAYS: i32 = 1;

// Line operations
pub const SCI_GOTOLINE: u32 = 2024;
pub const SCI_LINEDUP: u32 = 2404;
pub const SCI_LINEDELETE: u32 = 2338;
pub const SCI_LINETRANSPOSE: u32 = 2339;
pub const SCI_UPPERCASE: u32 = 2341;
pub const SCI_LOWERCASE: u32 = 2340;
pub const SCI_LINESCROLLDOWN: u32 = 2342;
pub const SCI_LINESCROLLUP: u32 = 2343;

// Selection
pub const SCI_GETSELTEXT: u32 = 2161;
pub const SCI_REPLACESEL: u32 = 2170;
pub const SCI_DELETERANGE: u32 = 2645;
pub const SCI_CLEAR: u32 = 2180;

// Search
pub const SCI_SEARCHNEXT: u32 = 2367;
pub const SCI_SEARCHPREV: u32 = 2368;
pub const SCI_SETTARGETSTART: u32 = 2190;
pub const SCI_SETTARGETEND: u32 = 2191;
pub const SCI_GETTARGETSTART: u32 = 2191;
pub const SCI_GETTARGETEND: u32 = 2193;
pub const SCI_SEARCHINTARGET: u32 = 2197;
pub const SCI_SETSEARCHFLAGS: u32 = 2198;
pub const SCI_REPLACETARGET: u32 = 2194;
pub const SCI_GETSELECTIONSTART: u32 = 2143;
pub const SCI_GETSELECTIONEND: u32 = 2145;
pub const SCI_SETSEL: u32 = 2160;
pub const SCI_SCROLLCARET: u32 = 2169;
pub const SCI_TARGETWHOLEDOCUMENT: u32 = 2690;

// Search flags
pub const SCFIND_MATCHCASE: i32 = 4;
pub const SCFIND_WHOLEWORD: i32 = 2;
pub const SCFIND_REGEXP: i32 = 0x00200000;

// Multiple selection
pub const SCI_SETMULTIPLESELECTION: u32 = 2563;
pub const SCI_SETADDITIONALSELECTIONTYPING: u32 = 2564;
pub const SCI_ADDSELECTION: u32 = 2573;
pub const SCI_SETMAINSELECTION: u32 = 2574;

// Indicators
pub const SCI_INDICSETSTYLE: u32 = 2080;
pub const SCI_INDICSETFORE: u32 = 2082;
pub const SCI_INDICSETALPHA: u32 = 2523;
pub const SCI_SETINDICATORCURRENT: u32 = 2500;
pub const SCI_INDICATORFILLRANGE: u32 = 2504;
pub const SCI_INDICATORCLEARRANGE: u32 = 2505;
pub const INDIC_ROUNDBOX: i32 = 7;

// Line position
pub const SCI_POSITIONFROMLINE: u32 = 2167;
pub const SCI_GETLINEENDPOSITION: u32 = 2136;
pub const SCI_LINELENGTH: u32 = 2350;
pub const SCI_BEGINUNDOACTION: u32 = 2078;
pub const SCI_ENDUNDOACTION: u32 = 2079;
pub const SCI_INSERTTEXT: u32 = 2003;
pub const SCI_SETTARGETRANGE: u32 = 2686;

// Lexer language
pub const SCI_SETLEXERLANGUAGE: u32 = 4006;

// Codepage
pub const SCI_SETCODEPAGE: u32 = 2037;
pub const SC_CP_UTF8: i32 = 65001;

// Technology
pub const SCI_SETTECHNOLOGY: u32 = 2630;
pub const SC_TECHNOLOGY_DIRECTWRITE: i32 = 1;

// Notifications
pub const SCN_UPDATEUI: u32 = 2007;
pub const SCN_MACRORECORD: u32 = 2009;

// Document pointer (split view)
pub const SCI_GETDOCPOINTER: u32 = 2357;
pub const SCI_SETDOCPOINTER: u32 = 2358;
pub const SCI_ADDREFDOCUMENT: u32 = 2376;

// Macro recording
pub const SCI_STARTRECORD: u32 = 3001;
pub const SCI_STOPRECORD: u32 = 3002;

// Read-only
pub const SCI_SETREADONLY: u32 = 2171;

// Keywords
pub const SCI_SETKEYWORDS: u32 = 4005;

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

    // Syntax highlighting colors (common across most lexers)
    // Style 1: Comments
    sci_send(hwnd, SCI_STYLESETFORE, 1, rgb(106, 153, 85) as isize);
    // Style 2: Comments (line)
    sci_send(hwnd, SCI_STYLESETFORE, 2, rgb(106, 153, 85) as isize);
    // Style 3: Doc comments
    sci_send(hwnd, SCI_STYLESETFORE, 3, rgb(106, 153, 85) as isize);
    // Style 4: Numbers
    sci_send(hwnd, SCI_STYLESETFORE, 4, rgb(181, 206, 168) as isize);
    // Style 5: Keywords
    sci_send(hwnd, SCI_STYLESETFORE, 5, rgb(86, 156, 214) as isize);
    sci_send(hwnd, SCI_STYLESETBOLD, 5, 1);
    // Style 6: Strings
    sci_send(hwnd, SCI_STYLESETFORE, 6, rgb(206, 145, 120) as isize);
    // Style 7: Characters
    sci_send(hwnd, SCI_STYLESETFORE, 7, rgb(206, 145, 120) as isize);
    // Style 8: UUIDs / special
    sci_send(hwnd, SCI_STYLESETFORE, 8, rgb(220, 220, 170) as isize);
    // Style 9: Preprocessor
    sci_send(hwnd, SCI_STYLESETFORE, 9, rgb(155, 155, 155) as isize);
    // Style 10: Operators
    sci_send(hwnd, SCI_STYLESETFORE, 10, rgb(212, 212, 212) as isize);
    // Style 11: Identifiers
    sci_send(hwnd, SCI_STYLESETFORE, 11, rgb(156, 220, 254) as isize);
    // Style 12: Unclosed strings
    sci_send(hwnd, SCI_STYLESETFORE, 12, rgb(206, 145, 120) as isize);
    // Style 13-15: Additional keywords/types
    sci_send(hwnd, SCI_STYLESETFORE, 13, rgb(78, 201, 176) as isize);
    sci_send(hwnd, SCI_STYLESETFORE, 14, rgb(220, 220, 170) as isize);
    sci_send(hwnd, SCI_STYLESETFORE, 15, rgb(197, 134, 192) as isize);
    // Style 16: Secondary keywords (types, built-ins)
    sci_send(hwnd, SCI_STYLESETFORE, 16, rgb(78, 201, 176) as isize);
    // Style 17-19: More keyword groups
    sci_send(hwnd, SCI_STYLESETFORE, 17, rgb(220, 220, 170) as isize);
    sci_send(hwnd, SCI_STYLESETFORE, 18, rgb(197, 134, 192) as isize);
    sci_send(hwnd, SCI_STYLESETFORE, 19, rgb(86, 156, 214) as isize);

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

/// Set keywords and syntax colors for a given lexer.
pub unsafe fn sci_setup_lexer_styles(hwnd: HWND, lexer_name: &str) {
    // Set keyword list 0 (primary keywords) for common languages
    let keywords: &[u8] = match lexer_name {
        "rust" => b"as async await break const continue crate dyn else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type unsafe use where while yield abstract become box do final macro override priv typeof unsized virtual\0",
        "cpp" => b"alignas alignof and and_eq asm auto bitand bitor bool break case catch char char8_t char16_t char32_t class compl concept const consteval constexpr constinit const_cast continue co_await co_return co_yield decltype default delete do double dynamic_cast else enum explicit export extern false float for friend goto if inline int long mutable namespace new noexcept not not_eq nullptr operator or or_eq private protected public register reinterpret_cast requires return short signed sizeof static static_assert static_cast struct switch template this thread_local throw true try typedef typeid typename union unsigned using virtual void volatile wchar_t while abstract boolean byte extends final finally implements import instanceof interface native package strictfp super synchronized throws transient let var const function of in class yield async await undefined null NaN Infinity\0",
        "python" => b"False None True and as assert async await break class continue def del elif else except finally for from global if import in is lambda nonlocal not or pass raise return try while with yield\0",
        "ruby" => b"BEGIN END __ENCODING__ __END__ __FILE__ __LINE__ alias and begin break case class def defined? do else elsif end ensure false for if in module next nil not or redo rescue retry return self super then true undef unless until when while yield\0",
        "perl" => b"__DATA__ __END__ __FILE__ __LINE__ __PACKAGE__ and bless caller chomp chop cmp continue CORE defined delete die do dump each else elsif eq eval exists for foreach format ge goto grep gt hex if import join keys last le length local lt map my ne next no not oct open or our pack package pop pos print printf prototype push q qq quotemeta qw qx redo ref require return reverse scalar seek shift sort splice split sprintf study sub substr tie tied uc ucfirst undef unless unlink unshift untie until use values wantarray warn while write\0",
        "bash" => b"alias bg bind break builtin caller case cd command compgen complete continue declare dirs disown do done echo elif else enable esac eval exec exit export false fc fg fi for function getopts hash help history if in jobs kill let local logout mapfile popd printf pushd pwd read readarray readonly return select set shift shopt source suspend test then time times trap true type typeset ulimit umask unalias unset until wait while\0",
        "sql" => b"add all alter and any as asc authorization backup begin between break browse bulk by cascade case check checkpoint close clustered coalesce column commit compute constraint contains continue convert create cross current current_date current_time cursor database dbcc deallocate declare default delete deny desc disk distinct distributed double drop dump else end errlvl escape except exec execute exists exit external fetch file fillfactor for foreign from full function goto grant group having holdlock identity if in index inner insert intersect into is join key kill left like lineno load merge national nocheck nonclustered not null of off offsets on open option or order outer over percent pivot plan primary print proc procedure public raiserror read readtext reconfigure references replication restore restrict return revert revoke right rollback rowcount rowguidcol rule save schema select session_user set setuser shutdown some statistics table tablesample textsize then to top tran transaction trigger truncate union unique unpivot update updatetext use user values varying view waitfor when where while with writetext\0",
        "lua" => b"and break do else elseif end false for function goto if in local nil not or repeat return then true until while\0",
        "json" => b"false null true\0",
        _ => b"\0",
    };
    sci_send(hwnd, SCI_SETKEYWORDS, 0, keywords.as_ptr() as isize);

    // Set keyword list 1 (secondary keywords / types) for some languages
    let keywords2: &[u8] = match lexer_name {
        "rust" => b"i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize f32 f64 bool char str String Vec Option Result Box Rc Arc Cell RefCell HashMap HashSet BTreeMap BTreeSet VecDeque LinkedList Some None Ok Err println eprintln format vec todo unimplemented unreachable panic assert assert_eq assert_ne cfg test derive Debug Clone Copy Default PartialEq Eq PartialOrd Ord Hash Display From Into TryFrom TryInto AsRef AsMut Iterator IntoIterator Send Sync Sized Unpin Drop Fn FnMut FnOnce Future\0",
        "cpp" => b"int8_t int16_t int32_t int64_t uint8_t uint16_t uint32_t uint64_t size_t ptrdiff_t intptr_t uintptr_t wchar_t string vector map set list queue stack array shared_ptr unique_ptr weak_ptr optional variant tuple pair any bitset deque priority_queue unordered_map unordered_set span string_view byte FILE errno stdin stdout stderr NULL EOF SEEK_SET SEEK_CUR SEEK_END EXIT_SUCCESS EXIT_FAILURE\0",
        "python" => b"int float complex bool str bytes bytearray list tuple range dict set frozenset type object Exception BaseException ArithmeticError BufferError EOFError FileExistsError FileNotFoundError ImportError IndexError KeyError MemoryError NameError NotImplementedError OSError OverflowError PermissionError RuntimeError StopIteration SyntaxError SystemError TypeError ValueError ZeroDivisionError print input len range enumerate zip map filter sorted reversed any all min max sum abs round open isinstance issubclass hasattr getattr setattr delattr dir vars super property staticmethod classmethod\0",
        _ => b"\0",
    };
    sci_send(hwnd, SCI_SETKEYWORDS, 1, keywords2.as_ptr() as isize);
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
        "d" => Some("d"),
        "go" => Some("cpp"),
        "hs" | "lhs" => Some("haskell"),
        "jl" => Some("julia"),
        "kt" | "kts" => Some("kotlin"),
        "m" | "mm" => Some("objc"),
        "nim" => Some("nim"),
        "scala" | "sc" => Some("scala"),
        "swift" => Some("swift"),
        "vhd" | "vhdl" => Some("vhdl"),
        "mat" => Some("matlab"),
        _ => None,
    }
}

/// All language entries for the Language menu: (display_name, lexer_name).
pub fn all_languages() -> &'static [(&'static str, &'static str)] {
    &[
        ("Plain Text", ""),
        ("Bash", "bash"),
        ("Batch", "batch"),
        ("C", "cpp"),
        ("C++", "cpp"),
        ("C#", "cpp"),
        ("CMake", "cmake"),
        ("CSS", "css"),
        ("D", "d"),
        ("Diff", "diff"),
        ("Go", "cpp"),
        ("Haskell", "haskell"),
        ("HTML", "hypertext"),
        ("INI/Properties", "props"),
        ("Java", "cpp"),
        ("JavaScript", "cpp"),
        ("JSON", "json"),
        ("Julia", "julia"),
        ("Kotlin", "kotlin"),
        ("LaTeX", "latex"),
        ("Lua", "lua"),
        ("Makefile", "makefile"),
        ("Markdown", "markdown"),
        ("MATLAB", "matlab"),
        ("Nim", "nim"),
        ("Objective-C", "objc"),
        ("Pascal", "pascal"),
        ("Perl", "perl"),
        ("PHP", "phpscript"),
        ("PowerShell", "powershell"),
        ("Python", "python"),
        ("R", "r"),
        ("Ruby", "ruby"),
        ("Rust", "rust"),
        ("Scala", "scala"),
        ("SQL", "sql"),
        ("Swift", "swift"),
        ("TCL", "tcl"),
        ("TOML", "toml"),
        ("TypeScript", "cpp"),
        ("VB", "vb"),
        ("VHDL", "vhdl"),
        ("XML", "xml"),
        ("YAML", "yaml"),
    ]
}
