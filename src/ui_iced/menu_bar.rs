use iced::widget::{button, column, container, mouse_area, row, text, Space};
use iced::{Element, Length, Theme};

use super::app::Message;
use super::theme::AppColors;

/// Color constants for menu rendering
const MENU_BG: iced::Color = iced::Color::from_rgb(0.18, 0.18, 0.22);
const MENU_HOVER: iced::Color = iced::Color::from_rgb(0.25, 0.25, 0.30);
const SEPARATOR_COLOR: iced::Color = iced::Color::from_rgb(0.30, 0.30, 0.35);
const SUBMENU_HEADER: iced::Color = iced::Color::from_rgb(0.55, 0.55, 0.60);

/// Menu labels in order — used for positioning dropdowns
pub const MENU_LABELS: &[&str] = &[
    "File", "Edit", "Search", "View", "Language", "Tools", "Macro", "Settings", "Help",
];

// ── Public API ──

pub fn view_menu_bar<'a>(active_menu: &Option<String>) -> Element<'a, Message> {
    let mut items = row![].spacing(0).padding([0, 4]);

    for label in MENU_LABELS {
        let is_active = active_menu.as_deref() == Some(*label);
        let bg = if is_active { MENU_HOVER } else { AppColors::TAB_BAR_BG };
        let has_open_menu = active_menu.is_some();
        let label_owned = label.to_string();

        let btn = button(text(*label).size(13))
            .on_press(Message::MenuToggle(label.to_string()))
            .padding([4, 10])
            .style(move |_theme: &Theme, status| {
                let bg = match status {
                    button::Status::Hovered | button::Status::Pressed => MENU_HOVER,
                    _ => bg,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: AppColors::TEXT,
                    border: iced::Border {
                        radius: 2.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            });

        // When a menu is already open, hovering another menu button should switch to it
        let btn_element: Element<'a, Message> = if has_open_menu && !is_active {
            mouse_area(btn)
                .on_enter(Message::MenuSwitch(label_owned))
                .into()
        } else {
            btn.into()
        };

        items = items.push(btn_element);
    }

    container(items)
        .width(Length::Fill)
        .clip(true)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(AppColors::TAB_BAR_BG)),
            ..Default::default()
        })
        .into()
}

/// Calculate the horizontal offset for a dropdown menu based on the menu name
pub fn menu_x_offset(menu_name: &str) -> f32 {
    let mut offset = 4.0; // initial padding
    for label in MENU_LABELS {
        if *label == menu_name {
            return offset;
        }
        // Approximate width: ~8px per char + 20px padding
        offset += (label.len() as f32) * 8.0 + 20.0;
    }
    offset
}

pub fn view_dropdown<'a>(state: &super::app::NotepadIced, menu_name: &str) -> Element<'a, Message> {
    let expanded = &state.expanded_submenus;
    let items: Vec<Element<'a, Message>> = match menu_name {
        "File" => build_file_menu(expanded),
        "Edit" => build_edit_menu(state, expanded),
        "Search" => build_search_menu(expanded),
        "View" => build_view_menu(state, expanded),
        "Language" => build_language_menu(state),
        "Tools" => build_tools_menu(state, expanded),
        "Macro" => build_macro_menu(),
        "Settings" => build_settings_menu(),
        "Help" => build_help_menu(),
        _ => vec![],
    };

    if items.is_empty() {
        return column![].into();
    }

    let menu_column = column(items).spacing(0).padding(4).width(Length::Fixed(280.0));

    container(menu_column)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(MENU_BG)),
            border: iced::Border {
                color: SEPARATOR_COLOR,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}

// ── Menu builders ──

fn build_file_menu<'a>(expanded: &std::collections::HashSet<String>) -> Vec<Element<'a, Message>> {
    let mut items = vec![
        menu_item("New", "Ctrl+N", Message::NewTab),
        menu_item("Open...", "Ctrl+O", Message::OpenFile),
        menu_item("Save", "Ctrl+S", Message::Save),
        menu_item("Save As...", "Ctrl+Shift+S", Message::SaveAs),
        menu_item("Save All", "", Message::SaveAll),
        separator(),
        menu_item("Close", "Ctrl+W", Message::CloseTab(usize::MAX)),
        menu_item("Close All", "", Message::CloseAll),
        separator(),
        menu_item("Save Session...", "", Message::SaveSession),
        menu_item("Load Session...", "", Message::LoadSession),
        menu_item("Auto-restore Session", "", Message::ToggleAutoRestore),
        separator(),
    ];
    push_submenu(&mut items, "Export", expanded, vec![
        menu_item("As HTML...", "", Message::ExportHtml),
        menu_item("As RTF...", "", Message::ExportRtf),
    ]);
    items.push(separator());
    items.push(menu_item("Exit", "", Message::Exit));
    items
}

fn build_edit_menu<'a>(state: &super::app::NotepadIced, expanded: &std::collections::HashSet<String>) -> Vec<Element<'a, Message>> {
    let enc = state.tab_manager.active_document().encoding;
    let le = state.tab_manager.active_document().line_ending;

    use crate::editor::document::{Encoding, LineEnding};

    let mut items = vec![
        menu_item("Undo", "Ctrl+Z", Message::Undo),
        menu_item("Redo", "Ctrl+Y", Message::Redo),
        separator(),
        menu_item_disabled("Cut", "Ctrl+X"),
        menu_item_disabled("Copy", "Ctrl+C"),
        menu_item_disabled("Paste", "Ctrl+V"),
        menu_item_disabled("Delete", "Del"),
        separator(),
        menu_item("Select All", "Ctrl+A", Message::SelectAll),
        separator(),
        menu_item("Toggle Comment", "Ctrl+/", Message::ToggleComment),
        separator(),
    ];

    push_submenu(&mut items, "Line Operations", expanded, vec![
        menu_item("Duplicate Line", "", Message::DuplicateLine),
        menu_item("Delete Line", "", Message::DeleteLine),
        menu_item("Move Line Up", "", Message::MoveLineUp),
        menu_item("Move Line Down", "", Message::MoveLineDown),
        separator(),
        menu_item("Sort Ascending", "", Message::SortAsc),
        menu_item("Sort Descending", "", Message::SortDesc),
        menu_item("Sort (Case Insensitive)", "", Message::SortCaseInsensitive),
        menu_item("Sort (Numeric)", "", Message::SortNumeric),
        separator(),
        menu_item("Remove Empty Lines", "", Message::RemoveEmpty),
        menu_item("Remove Duplicate Lines", "", Message::RemoveDuplicates),
        menu_item("Trim Trailing Whitespace", "", Message::TrimTrailing),
        menu_item("Trim Leading", "", Message::TrimLeading),
        menu_item("Trim Both", "", Message::TrimBoth),
        separator(),
        menu_item("Join Lines", "", Message::JoinLines),
        menu_item("Split Line", "", Message::SplitLine),
        menu_item("Insert Blank Above", "", Message::InsertAbove),
        menu_item("Insert Blank Below", "", Message::InsertBelow),
        menu_item("Reverse Line Order", "", Message::ReverseLines),
    ]);

    push_submenu(&mut items, "Case Conversion", expanded, vec![
        menu_item("UPPERCASE", "", Message::UpperCase),
        menu_item("lowercase", "", Message::LowerCase),
        menu_item("Title Case", "", Message::TitleCase),
        menu_item("Sentence case", "", Message::SentenceCase),
        menu_item("iNVERSE Case", "", Message::InverseCase),
    ]);

    items.push(separator());

    let mut enc_children = Vec::new();
    let encodings = [
        (Encoding::UTF8, "UTF-8"),
        (Encoding::UTF8BOM, "UTF-8 BOM"),
        (Encoding::UTF16LE, "UTF-16 LE"),
        (Encoding::UTF16BE, "UTF-16 BE"),
        (Encoding::ASCII, "ANSI"),
    ];
    for (e, label) in &encodings {
        let check = if enc == *e { " * " } else { "   " };
        enc_children.push(menu_item(
            &format!("{}{}", check, label),
            "",
            Message::SetEncoding(*e),
        ));
    }
    push_submenu(&mut items, "Encoding", expanded, enc_children);

    let mut le_children = Vec::new();
    let endings = [
        (LineEnding::CRLF, "Windows (CRLF)"),
        (LineEnding::LF, "Unix (LF)"),
        (LineEnding::CR, "Mac (CR)"),
    ];
    for (l, label) in &endings {
        let check = if le == *l { " * " } else { "   " };
        le_children.push(menu_item(
            &format!("{}{}", check, label),
            "",
            Message::SetLineEnding(*l),
        ));
    }
    push_submenu(&mut items, "Line Endings", expanded, le_children);

    items
}

fn build_search_menu<'a>(expanded: &std::collections::HashSet<String>) -> Vec<Element<'a, Message>> {
    let mut items = vec![
        menu_item("Find...", "Ctrl+F", Message::ShowFind),
        menu_item("Replace...", "Ctrl+H", Message::ShowReplace),
        menu_item("Find in Files...", "", Message::ShowFindInFiles),
        menu_item("Select All Occurrences", "", Message::SelectAllOccurrences),
        separator(),
        menu_item("Go to Line...", "Ctrl+G", Message::GotoLine),
        separator(),
    ];
    push_submenu(&mut items, "Bookmarks", expanded, vec![
        menu_item("Toggle Bookmark", "Ctrl+F2", Message::ToggleBookmark),
        menu_item("Next Bookmark", "F2", Message::NextBookmark),
        menu_item("Previous Bookmark", "Shift+F2", Message::PrevBookmark),
        separator(),
        menu_item("Clear All Bookmarks", "", Message::ClearBookmarks),
        menu_item("Copy Bookmarked Lines", "", Message::CopyBookmarkedLines),
        menu_item("Remove Bookmarked Lines", "", Message::RemoveBookmarkedLines),
        menu_item("Remove Unbookmarked Lines", "", Message::RemoveUnbookmarkedLines),
    ]);
    items
}

fn build_view_menu<'a>(state: &super::app::NotepadIced, expanded: &std::collections::HashSet<String>) -> Vec<Element<'a, Message>> {
    let mut items = vec![
        check_item("Word Wrap", state.word_wrap, Message::ToggleWordWrap),
        check_item("Line Numbers", state.show_line_numbers, Message::ToggleLineNumbers),
        check_item("Show Whitespace", state.show_whitespace, Message::ToggleWhitespace),
        check_item("Status Bar", state.show_status_bar, Message::ToggleStatusBar),
        check_item("Minimap", state.show_minimap, Message::ToggleMinimap),
        check_item("Function List", state.show_function_list, Message::ToggleFunctionList),
        separator(),
        menu_item("Split Horizontal", "", Message::SplitHorizontal),
        menu_item("Split Vertical", "", Message::SplitVertical),
        menu_item("Remove Split", "", Message::RemoveSplit),
        separator(),
        menu_item("Zoom In", "Ctrl+=", Message::ZoomIn),
        menu_item("Zoom Out", "Ctrl+-", Message::ZoomOut),
        menu_item("Reset Zoom", "Ctrl+0", Message::ZoomReset),
        separator(),
    ];
    push_submenu(&mut items, "Folding", expanded, vec![
        menu_item("Toggle Fold", "Ctrl+Shift+[", Message::ToggleFold),
        menu_item("Fold All", "", Message::FoldAll),
        menu_item("Unfold All", "", Message::UnfoldAll),
        separator(),
        menu_item("Fold Level 1", "", Message::FoldLevel(0)),
        menu_item("Fold Level 2", "", Message::FoldLevel(4)),
        menu_item("Fold Level 3", "", Message::FoldLevel(8)),
    ]);
    items
}

fn build_language_menu<'a>(state: &super::app::NotepadIced) -> Vec<Element<'a, Message>> {
    let current_lang = &state.tab_manager.active_document().language;
    let common_langs = [
        "Plain Text", "Rust", "Python", "JavaScript", "TypeScript", "C", "C++", "C#",
        "Java", "Go", "Ruby", "PHP", "HTML", "CSS", "JSON", "XML", "YAML", "TOML",
        "Markdown", "SQL", "Shell", "Bash", "PowerShell", "Lua", "Perl", "R",
        "Swift", "Kotlin", "Scala", "Haskell", "Erlang", "Elixir", "Clojure",
        "Dart", "Objective-C", "Makefile", "Dockerfile", "Diff",
    ];

    let mut items = Vec::new();
    for lang in &common_langs {
        let check = if current_lang == *lang { " * " } else { "   " };
        items.push(menu_item(
            &format!("{}{}", check, lang),
            "",
            Message::SetLanguage(lang.to_string()),
        ));
    }
    items
}

fn build_tools_menu<'a>(state: &super::app::NotepadIced, expanded: &std::collections::HashSet<String>) -> Vec<Element<'a, Message>> {
    let mut items = Vec::new();
    push_submenu(&mut items, "JSON", expanded, vec![
        menu_item("Format JSON", "Ctrl+Shift+J", Message::JsonFormat),
        menu_item("Compact JSON", "", Message::JsonCompact),
        menu_item("Validate JSON", "", Message::JsonValidate),
        menu_item("Sort JSON Keys", "", Message::JsonSortKeys),
    ]);
    items.push(separator());
    items.push(check_item("Markdown Preview", state.show_markdown_preview, Message::ToggleMarkdownPreview));
    items.push(check_item("CSV Viewer", state.show_csv_viewer, Message::ToggleCsvViewer));
    items.push(separator());
    push_submenu(&mut items, "MIME Tools", expanded, vec![
        menu_item("Base64 Encode", "", Message::MimeBase64Encode),
        menu_item("Base64 Decode", "", Message::MimeBase64Decode),
        separator(),
        menu_item("URL Encode", "", Message::MimeUrlEncode),
        menu_item("URL Decode", "", Message::MimeUrlDecode),
        separator(),
        menu_item("HTML Entity Encode", "", Message::MimeHtmlEncode),
        menu_item("HTML Entity Decode", "", Message::MimeHtmlDecode),
        separator(),
        menu_item("Hex Encode", "", Message::MimeHexEncode),
        menu_item("Hex Decode", "", Message::MimeHexDecode),
    ]);
    items.push(separator());
    items.push(menu_item("Compare Files...", "", Message::CompareFiles));
    items.push(check_item("Hex Viewer", state.show_hex_viewer, Message::ToggleHexViewer));
    items
}

fn build_macro_menu<'a>() -> Vec<Element<'a, Message>> {
    vec![
        menu_item("Start/Stop Recording", "Ctrl+Shift+R", Message::ToggleMacroRecording),
        menu_item("Play Last Macro", "Ctrl+Shift+P", Message::PlayLastMacro),
        menu_item("Play Multiple Times...", "", Message::PlayMacroMultiple),
    ]
}

fn build_settings_menu<'a>() -> Vec<Element<'a, Message>> {
    vec![
        menu_item("Preferences...", "", Message::ShowPreferences),
        menu_item("Keyboard Shortcuts...", "", Message::ShowKeybindings),
    ]
}

fn build_help_menu<'a>() -> Vec<Element<'a, Message>> {
    vec![
        menu_item("About", "", Message::ShowAbout),
    ]
}

// ── Widget helpers ──

fn menu_item<'a>(label: &str, shortcut: &str, msg: Message) -> Element<'a, Message> {
    let label_text = text(label.to_string()).size(13);

    let content: Element<'a, Message> = if shortcut.is_empty() {
        container(label_text).width(Length::Fill).into()
    } else {
        row![
            container(label_text).width(Length::Fill),
            text(shortcut.to_string()).size(11).color(AppColors::TEXT_DIM),
        ]
        .spacing(8)
        .into()
    };

    button(content)
        .on_press(msg)
        .width(Length::Fill)
        .padding([3, 8])
        .style(|_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(MENU_HOVER)),
                _ => None,
            };
            button::Style {
                background: bg,
                text_color: AppColors::TEXT,
                border: iced::Border {
                    radius: 2.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

fn menu_item_disabled<'a>(label: &str, shortcut: &str) -> Element<'a, Message> {
    let content: Element<'a, Message> = row![
        container(text(label.to_string()).size(13).color(AppColors::TEXT_DIM)).width(Length::Fill),
        text(shortcut.to_string()).size(11).color(AppColors::TEXT_DIM),
    ]
    .spacing(8)
    .into();

    container(content)
        .width(Length::Fill)
        .padding([3, 8])
        .into()
}

fn check_item<'a>(label: &str, checked: bool, msg: Message) -> Element<'a, Message> {
    let prefix = if checked { " * " } else { "   " };
    menu_item(&format!("{}{}", prefix, label), "", msg)
}

fn submenu_toggle<'a>(label: &str, is_expanded: bool) -> Element<'a, Message> {
    let arrow = if is_expanded { "v" } else { ">" };
    button(
        row![
            text(format!("{} {}", arrow, label)).size(13).color(SUBMENU_HEADER),
        ]
        .spacing(4),
    )
    .on_press(Message::SubMenuToggle(label.to_string()))
    .width(Length::Fill)
    .padding([4, 8])
    .style(|_theme: &Theme, status| {
        let bg = match status {
            button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(MENU_HOVER)),
            _ => None,
        };
        button::Style {
            background: bg,
            text_color: SUBMENU_HEADER,
            border: iced::Border {
                radius: 2.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}

fn push_submenu<'a>(
    items: &mut Vec<Element<'a, Message>>,
    label: &str,
    expanded: &std::collections::HashSet<String>,
    children: Vec<Element<'a, Message>>,
) {
    let is_expanded = expanded.contains(label);
    items.push(submenu_toggle(label, is_expanded));
    if is_expanded {
        for child in children {
            // Indent submenu items
            items.push(
                container(child)
                    .padding(iced::Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 16.0 })
                    .width(Length::Fill)
                    .into(),
            );
        }
    }
}

fn separator<'a>() -> Element<'a, Message> {
    container(Space::with_height(1))
        .width(Length::Fill)
        .padding([3, 4])
        .style(|_theme: &Theme| container::Style {
            border: iced::Border {
                color: SEPARATOR_COLOR,
                width: 0.0,
                radius: 0.0.into(),
            },
            background: Some(iced::Background::Color(SEPARATOR_COLOR)),
            ..Default::default()
        })
        .into()
}
