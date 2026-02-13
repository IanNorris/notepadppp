fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let is_mingw = target_env == "gnu";

    // Scintilla sources
    let sci_src: Vec<std::path::PathBuf> = glob::glob("vendor/scintilla/src/*.cxx")
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    let sci_win32: Vec<std::path::PathBuf> = glob::glob("vendor/scintilla/win32/*.cxx")
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|p| {
            p.file_name()
                .map(|f| f != "ScintillaDLL.cxx")
                .unwrap_or(true)
        })
        .collect();

    let mut sci_build = cc::Build::new();
    sci_build.cpp(true);

    if is_mingw {
        sci_build
            .flag("-std=c++17")
            .flag("-DUNICODE")
            .flag("-D_UNICODE")
            .flag("-DSCI_LEXER")
            .flag("-DNO_CXX11_REGEX");
    } else {
        sci_build
            .flag("/std:c++17")
            .flag("/DUNICODE")
            .flag("/D_UNICODE")
            .flag("/DSCI_LEXER")
            .flag("/DNO_CXX11_REGEX");
    }

    sci_build
        .include("vendor/scintilla/include")
        .include("vendor/scintilla/src")
        .include("vendor/scintilla/win32")
        .include("vendor/lexilla/include")
        .include("vendor/lexilla/lexlib");

    for f in &sci_src {
        sci_build.file(f);
    }
    for f in &sci_win32 {
        sci_build.file(f);
    }

    sci_build.warnings(false);
    sci_build.compile("scintilla");

    // Lexilla sources
    let lex_lib: Vec<std::path::PathBuf> = glob::glob("vendor/lexilla/lexlib/*.cxx")
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    let lex_lexers: Vec<std::path::PathBuf> = glob::glob("vendor/lexilla/lexers/*.cxx")
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    let mut lex_build = cc::Build::new();
    lex_build.cpp(true);

    if is_mingw {
        lex_build
            .flag("-std=c++17")
            .flag("-DUNICODE")
            .flag("-D_UNICODE")
            .flag("-DSCI_LEXER")
            .flag("-DNO_CXX11_REGEX");
    } else {
        lex_build
            .flag("/std:c++17")
            .flag("/DUNICODE")
            .flag("/D_UNICODE")
            .flag("/DSCI_LEXER")
            .flag("/DNO_CXX11_REGEX");
    }

    lex_build
        .include("vendor/scintilla/include")
        .include("vendor/scintilla/src")
        .include("vendor/lexilla/include")
        .include("vendor/lexilla/lexlib")
        .include("vendor/lexilla/src");

    for f in &lex_lib {
        lex_build.file(f);
    }
    for f in &lex_lexers {
        lex_build.file(f);
    }
    lex_build.file("vendor/lexilla/src/Lexilla.cxx");

    lex_build.warnings(false);
    lex_build.compile("lexilla");

    // Link system libraries
    println!("cargo:rustc-link-lib=static=scintilla");
    println!("cargo:rustc-link-lib=static=lexilla");
    for lib in &[
        "user32", "gdi32", "imm32", "ole32", "oleaut32", "msimg32", "comctl32", "uuid", "d2d1",
        "dwrite", "dwmapi", "uxtheme",
    ] {
        println!("cargo:rustc-link-lib=dylib={lib}");
    }

    if is_mingw {
        // Find the mingw lib directory for static linking
        let output = std::process::Command::new("x86_64-w64-mingw32-g++")
            .arg("-print-file-name=libstdc++.a")
            .output();
        if let Ok(out) = output {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if let Some(dir) = std::path::Path::new(&path).parent() {
                println!("cargo:rustc-link-search=native={}", dir.display());
            }
        }
        println!("cargo:rustc-link-lib=static=stdc++");
    }

    // Compile resource file (manifest for visual styles + DPI awareness)
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let rc_obj = std::path::Path::new(&out_dir).join("notepadppp.res.o");
    let windres = if is_mingw {
        "x86_64-w64-mingw32-windres"
    } else {
        "windres"
    };
    let rc_status = std::process::Command::new(windres)
        .arg("notepadppp.rc")
        .arg("-o")
        .arg(&rc_obj)
        .status();
    if let Ok(status) = rc_status {
        if status.success() {
            println!("cargo:rustc-link-arg={}", rc_obj.display());
        }
    }
}
