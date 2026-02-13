; Notepad+++ NSIS Installer Script
; Requires NSIS 3.x — https://nsis.sourceforge.io/

!include "MUI2.nsh"
!include "FileFunc.nsh"

; --- General ---
Name "Notepad+++"
OutFile "notepadppp-setup.exe"
InstallDir "$PROGRAMFILES64\Notepad+++"
InstallDirRegKey HKCU "Software\NotepadPPP" "InstallDir"
RequestExecutionLevel user
Unicode True

; --- Version Info ---
VIProductVersion "0.1.0.0"
VIAddVersionKey "ProductName" "Notepad+++"
VIAddVersionKey "FileDescription" "A fast, native text editor for programmers"
VIAddVersionKey "FileVersion" "0.1.0"
VIAddVersionKey "LegalCopyright" "MIT License"

; --- MUI Settings ---
!define MUI_ABORTWARNING
!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall.ico"

; --- Pages ---
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

; Uninstaller pages
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

; --- Language ---
!insertmacro MUI_LANGUAGE "English"

; --- Sections ---

Section "Notepad+++ (required)" SecMain
    SectionIn RO

    SetOutPath $INSTDIR
    File "notepadppp.exe"
    File "README.md"
    File "FEATURES.md"

    ; Save install directory
    WriteRegStr HKCU "Software\NotepadPPP" "InstallDir" "$INSTDIR"

    ; Create uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"

    ; Add/Remove Programs entry
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotepadPPP" \
        "DisplayName" "Notepad+++"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotepadPPP" \
        "UninstallString" '"$INSTDIR\uninstall.exe"'
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotepadPPP" \
        "InstallLocation" "$INSTDIR"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotepadPPP" \
        "DisplayIcon" "$INSTDIR\notepadppp.exe"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotepadPPP" \
        "Publisher" "Notepad+++ Contributors"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotepadPPP" \
        "DisplayVersion" "0.1.0"
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotepadPPP" \
        "NoModify" 1
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotepadPPP" \
        "NoRepair" 1

    ; Get installed size
    ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
    IntFmt $0 "0x%08X" $0
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotepadPPP" \
        "EstimatedSize" "$0"
SectionEnd

Section "Desktop Shortcut" SecDesktop
    CreateShortCut "$DESKTOP\Notepad+++.lnk" "$INSTDIR\notepadppp.exe"
SectionEnd

Section "Start Menu Shortcut" SecStartMenu
    CreateDirectory "$SMPROGRAMS\Notepad+++"
    CreateShortCut "$SMPROGRAMS\Notepad+++\Notepad+++.lnk" "$INSTDIR\notepadppp.exe"
    CreateShortCut "$SMPROGRAMS\Notepad+++\Uninstall.lnk" "$INSTDIR\uninstall.exe"
SectionEnd

Section "Shell Integration" SecShell
    ; ProgID for Notepad+++
    WriteRegStr HKCU "Software\Classes\NotepadPPP.File" "" "Notepad+++ File"
    WriteRegStr HKCU "Software\Classes\NotepadPPP.File\DefaultIcon" "" "$INSTDIR\notepadppp.exe,0"
    WriteRegStr HKCU "Software\Classes\NotepadPPP.File\shell\open\command" "" '"$INSTDIR\notepadppp.exe" "%1"'

    ; "Open with Notepad+++" context menu for all files
    WriteRegStr HKCU "Software\Classes\*\shell\NotepadPPP.OpenFile" "" "Open with Notepad+++"
    WriteRegStr HKCU "Software\Classes\*\shell\NotepadPPP.OpenFile" "Icon" "$INSTDIR\notepadppp.exe,0"
    WriteRegStr HKCU "Software\Classes\*\shell\NotepadPPP.OpenFile\command" "" '"$INSTDIR\notepadppp.exe" "%1"'

    ; "Open folder in Notepad+++" context menu for directories
    WriteRegStr HKCU "Software\Classes\Directory\Background\shell\NotepadPPP.OpenFolder" "" "Open folder in Notepad+++"
    WriteRegStr HKCU "Software\Classes\Directory\Background\shell\NotepadPPP.OpenFolder" "Icon" "$INSTDIR\notepadppp.exe,0"
    WriteRegStr HKCU "Software\Classes\Directory\Background\shell\NotepadPPP.OpenFolder\command" "" '"$INSTDIR\notepadppp.exe" "%V"'
    WriteRegStr HKCU "Software\Classes\Directory\shell\NotepadPPP.OpenFolder" "" "Open folder in Notepad+++"
    WriteRegStr HKCU "Software\Classes\Directory\shell\NotepadPPP.OpenFolder" "Icon" "$INSTDIR\notepadppp.exe,0"
    WriteRegStr HKCU "Software\Classes\Directory\shell\NotepadPPP.OpenFolder\command" "" '"$INSTDIR\notepadppp.exe" "%1"'

    ; File extension associations (common text/code file types)
    !macro AssocExt ext
        WriteRegStr HKCU "Software\Classes\.${ext}\OpenWithProgids" "NotepadPPP.File" ""
    !macroend

    ; Plain text
    !insertmacro AssocExt "txt"
    !insertmacro AssocExt "log"
    !insertmacro AssocExt "cfg"
    !insertmacro AssocExt "conf"
    !insertmacro AssocExt "ini"
    ; Web
    !insertmacro AssocExt "html"
    !insertmacro AssocExt "htm"
    !insertmacro AssocExt "css"
    !insertmacro AssocExt "js"
    !insertmacro AssocExt "ts"
    !insertmacro AssocExt "jsx"
    !insertmacro AssocExt "tsx"
    !insertmacro AssocExt "json"
    !insertmacro AssocExt "xml"
    !insertmacro AssocExt "yaml"
    !insertmacro AssocExt "yml"
    !insertmacro AssocExt "toml"
    ; Programming
    !insertmacro AssocExt "rs"
    !insertmacro AssocExt "py"
    !insertmacro AssocExt "rb"
    !insertmacro AssocExt "go"
    !insertmacro AssocExt "java"
    !insertmacro AssocExt "kt"
    !insertmacro AssocExt "c"
    !insertmacro AssocExt "cpp"
    !insertmacro AssocExt "h"
    !insertmacro AssocExt "hpp"
    !insertmacro AssocExt "cs"
    !insertmacro AssocExt "swift"
    !insertmacro AssocExt "zig"
    !insertmacro AssocExt "lua"
    !insertmacro AssocExt "pl"
    !insertmacro AssocExt "php"
    !insertmacro AssocExt "r"
    !insertmacro AssocExt "scala"
    ; Shell/scripting
    !insertmacro AssocExt "sh"
    !insertmacro AssocExt "bash"
    !insertmacro AssocExt "bat"
    !insertmacro AssocExt "cmd"
    !insertmacro AssocExt "ps1"
    ; Data
    !insertmacro AssocExt "csv"
    !insertmacro AssocExt "tsv"
    !insertmacro AssocExt "sql"
    ; Markdown/docs
    !insertmacro AssocExt "md"
    !insertmacro AssocExt "markdown"
    !insertmacro AssocExt "rst"
    !insertmacro AssocExt "tex"

    ; Notify Explorer of changes
    System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0x0000, p 0, p 0)'
SectionEnd

Section "Add to PATH" SecPath
    ; Add install directory to user PATH
    ReadRegStr $0 HKCU "Environment" "Path"
    StrCmp $0 "" 0 +2
        StrCpy $0 ""
    WriteRegExpandStr HKCU "Environment" "Path" "$0;$INSTDIR"
    SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
SectionEnd

; --- Section Descriptions ---
!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
    !insertmacro MUI_DESCRIPTION_TEXT ${SecMain} "Core Notepad+++ editor (required)"
    !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} "Create a desktop shortcut"
    !insertmacro MUI_DESCRIPTION_TEXT ${SecStartMenu} "Create Start Menu shortcuts"
    !insertmacro MUI_DESCRIPTION_TEXT ${SecShell} "Add 'Open with Notepad+++' to context menus and register file associations"
    !insertmacro MUI_DESCRIPTION_TEXT ${SecPath} "Add Notepad+++ to your PATH for command-line usage"
!insertmacro MUI_FUNCTION_DESCRIPTION_END

; --- Uninstaller ---
Section "Uninstall"
    ; Remove files
    Delete "$INSTDIR\notepadppp.exe"
    Delete "$INSTDIR\README.md"
    Delete "$INSTDIR\FEATURES.md"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"

    ; Remove shortcuts
    Delete "$DESKTOP\Notepad+++.lnk"
    Delete "$SMPROGRAMS\Notepad+++\Notepad+++.lnk"
    Delete "$SMPROGRAMS\Notepad+++\Uninstall.lnk"
    RMDir "$SMPROGRAMS\Notepad+++"

    ; Remove registry keys
    DeleteRegKey HKCU "Software\NotepadPPP"
    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NotepadPPP"

    ; Remove shell integration
    DeleteRegKey HKCU "Software\Classes\NotepadPPP.File"
    DeleteRegKey HKCU "Software\Classes\*\shell\NotepadPPP.OpenFile"
    DeleteRegKey HKCU "Software\Classes\Directory\Background\shell\NotepadPPP.OpenFolder"
    DeleteRegKey HKCU "Software\Classes\Directory\shell\NotepadPPP.OpenFolder"

    ; Remove from PATH
    ReadRegStr $0 HKCU "Environment" "Path"
    ${WordReplace} $0 ";$INSTDIR" "" "+" $1
    WriteRegExpandStr HKCU "Environment" "Path" "$1"
    SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000

    ; Notify Explorer
    System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0x0000, p 0, p 0)'
SectionEnd
