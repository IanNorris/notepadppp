use iced::widget::{button, column, container, mouse_area, row, text, Space};
use iced::{Element, Length, Theme};

use super::app::Message;
use super::theme::AppTheme;

/// Menu labels in order — used for positioning dropdowns
pub const MENU_LABELS: &[&str] = &[
    "File", "Edit", "Search", "View", "Language", "Tools", "Macro", "Settings", "Help",
];

// ── Public API ──

pub fn view_menu_bar<'a>(active_menu: &Option<String>, theme: &AppTheme) -> Element<'a, Message> {
    let t_tab_bar = theme.tab_bar_bg;
    let t_menu_hover = theme.menu_hover;
    let t_text = theme.text;
    let mut items = row![].spacing(0).padding([0, 4]);

    for label in MENU_LABELS {
        let is_active = active_menu.as_deref() == Some(*label);
        let bg = if is_active { t_menu_hover } else { t_tab_bar };
        let has_open_menu = active_menu.is_some();
        let label_owned = label.to_string();

        let btn = button(text(*label).size(13))
            .on_press(Message::MenuToggle(label.to_string()))
            .padding([4, 10])
            .style(move |_theme: &Theme, status| {
                let bg = match status {
                    button::Status::Hovered | button::Status::Pressed => t_menu_hover,
                    _ => bg,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: t_text,
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
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_tab_bar)),
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
        // Approximate width: ~7.0px per char at 13px font + 20px button padding
        offset += (label.len() as f32) * 7.0 + 20.0;
    }
    offset
}

pub fn view_dropdown<'a>(state: &super::app::NotepadIced, menu_name: &str) -> Element<'a, Message> {
    let expanded = &state.expanded_submenus;
    let t = &state.theme;
    let items: Vec<Element<'a, Message>> = match menu_name {
        "File" => build_file_menu(expanded, t),
        "Edit" => build_edit_menu(state, expanded, t),
        "Search" => build_search_menu(expanded, t),
        "View" => build_view_menu(state, expanded, t),
        "Language" => build_language_menu(state, t),
        "Tools" => build_tools_menu(state, expanded, t),
        "Macro" => build_macro_menu(t),
        "Settings" => build_settings_menu(t),
        "Help" => build_help_menu(t),
        _ => vec![],
    };

    if items.is_empty() {
        return column![].into();
    }

    let t_menu_bg = t.menu_bg;
    let t_separator = t.separator;
    let menu_column = column(items).spacing(0).padding(4).width(Length::Fixed(280.0));

    container(menu_column)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(t_menu_bg)),
            border: iced::Border {
                color: t_separator,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}

// ── Menu builders ──

fn build_file_menu<'a>(expanded: &std::collections::HashSet<String>, t: &AppTheme) -> Vec<Element<'a, Message>> {
    let mut items = vec![
        menu_item("New", "Ctrl+N", Message::NewTab, t),
        menu_item("Open...", "Ctrl+O", Message::OpenFile, t),
        menu_item("Save", "Ctrl+S", Message::Save, t),
        menu_item("Save As...", "Ctrl+Shift+S", Message::SaveAs, t),
        menu_item("Save All", "", Message::SaveAll, t),
        separator(t),
        menu_item("Close", "Ctrl+W", Message::CloseTab(usize::MAX), t),
        menu_item("Close All", "", Message::CloseAll, t),
        separator(t),
        menu_item("Save Session...", "", Message::SaveSession, t),
        menu_item("Load Session...", "", Message::LoadSession, t),
        menu_item("Auto-restore Session", "", Message::ToggleAutoRestore, t),
        separator(t),
    ];
    push_submenu(&mut items, "Export", expanded, vec![
        menu_item("As HTML...", "", Message::ExportHtml, t),
        menu_item("As RTF...", "", Message::ExportRtf, t),
    ], t);
    items.push(separator(t));
    items.push(menu_item("Exit", "", Message::Exit, t));
    items
}

fn build_edit_menu<'a>(state: &super::app::NotepadIced, expanded: &std::collections::HashSet<String>, t: &AppTheme) -> Vec<Element<'a, Message>> {
    let enc = state.tab_manager.active_document().encoding;
    let le = state.tab_manager.active_document().line_ending;

    use crate::editor::document::{Encoding, LineEnding};

    let mut items = vec![
        menu_item("Undo", "Ctrl+Z", Message::Undo, t),
        menu_item("Redo", "Ctrl+Y", Message::Redo, t),
        separator(t),
        menu_item_disabled("Cut", "Ctrl+X", t),
        menu_item_disabled("Copy", "Ctrl+C", t),
        menu_item_disabled("Paste", "Ctrl+V", t),
        menu_item_disabled("Delete", "Del", t),
        separator(t),
        menu_item("Select All", "Ctrl+A", Message::SelectAll, t),
        separator(t),
        menu_item("Toggle Comment", "Ctrl+/", Message::ToggleComment, t),
        separator(t),
    ];

    push_submenu(&mut items, "Line Operations", expanded, vec![
        menu_item("Duplicate Line", "", Message::DuplicateLine, t),
        menu_item("Delete Line", "", Message::DeleteLine, t),
        menu_item("Move Line Up", "", Message::MoveLineUp, t),
        menu_item("Move Line Down", "", Message::MoveLineDown, t),
        separator(t),
        menu_item("Sort Ascending", "", Message::SortAsc, t),
        menu_item("Sort Descending", "", Message::SortDesc, t),
        menu_item("Sort (Case Insensitive)", "", Message::SortCaseInsensitive, t),
        menu_item("Sort (Numeric)", "", Message::SortNumeric, t),
        separator(t),
        menu_item("Remove Empty Lines", "", Message::RemoveEmpty, t),
        menu_item("Remove Duplicate Lines", "", Message::RemoveDuplicates, t),
        menu_item("Trim Trailing Whitespace", "", Message::TrimTrailing, t),
        menu_item("Trim Leading", "", Message::TrimLeading, t),
        menu_item("Trim Both", "", Message::TrimBoth, t),
        separator(t),
        menu_item("Join Lines", "", Message::JoinLines, t),
        menu_item("Split Line", "", Message::SplitLine, t),
        menu_item("Insert Blank Above", "", Message::InsertAbove, t),
        menu_item("Insert Blank Below", "", Message::InsertBelow, t),
        menu_item("Reverse Line Order", "", Message::ReverseLines, t),
    ], t);

    push_submenu(&mut items, "Case Conversion", expanded, vec![
        menu_item("UPPERCASE", "", Message::UpperCase, t),
        menu_item("lowercase", "", Message::LowerCase, t),
        menu_item("Title Case", "", Message::TitleCase, t),
        menu_item("Sentence case", "", Message::SentenceCase, t),
        menu_item("iNVERSE Case", "", Message::InverseCase, t),
    ], t);

    items.push(separator(t));

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
            t,
        ));
    }
    push_submenu(&mut items, "Encoding", expanded, enc_children, t);

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
            t,
        ));
    }
    push_submenu(&mut items, "Line Endings", expanded, le_children, t);

    items.push(separator(t));
    let ro = state.tab_manager.active_document().read_only;
    items.push(check_item("Read Only", ro, Message::ToggleReadOnly, t));

    items
}

fn build_search_menu<'a>(expanded: &std::collections::HashSet<String>, t: &AppTheme) -> Vec<Element<'a, Message>> {
    let mut items = vec![
        menu_item("Find...", "Ctrl+F", Message::ShowFind, t),
        menu_item("Replace...", "Ctrl+H", Message::ShowReplace, t),
        menu_item("Find in Files...", "", Message::ShowFindInFiles, t),
        menu_item("Select All Occurrences", "", Message::SelectAllOccurrences, t),
        separator(t),
        menu_item("Mark All", "", Message::MarkAll, t),
        menu_item("Clear All Marks", "", Message::ClearAllMarks, t),
        menu_item("Bookmark Matching Lines", "", Message::BookmarkMatchingLines, t),
        separator(t),
        menu_item("Go to Line...", "Ctrl+G", Message::GotoLine, t),
        separator(t),
    ];
    push_submenu(&mut items, "Bookmarks", expanded, vec![
        menu_item("Toggle Bookmark", "Ctrl+F2", Message::ToggleBookmark, t),
        menu_item("Next Bookmark", "F2", Message::NextBookmark, t),
        menu_item("Previous Bookmark", "Shift+F2", Message::PrevBookmark, t),
        separator(t),
        menu_item("Clear All Bookmarks", "", Message::ClearBookmarks, t),
        menu_item("Copy Bookmarked Lines", "", Message::CopyBookmarkedLines, t),
        menu_item("Cut Bookmarked Lines", "", Message::CutBookmarkedLines, t),
        menu_item("Remove Bookmarked Lines", "", Message::RemoveBookmarkedLines, t),
        menu_item("Remove Unbookmarked Lines", "", Message::RemoveUnbookmarkedLines, t),
    ], t);
    items
}

fn build_view_menu<'a>(state: &super::app::NotepadIced, expanded: &std::collections::HashSet<String>, t: &AppTheme) -> Vec<Element<'a, Message>> {
    let mut items = vec![
        check_item("Toolbar", state.show_toolbar, Message::ToggleToolbar, t),
        check_item("Word Wrap", state.word_wrap, Message::ToggleWordWrap, t),
        check_item("Line Numbers", state.show_line_numbers, Message::ToggleLineNumbers, t),
        check_item("Show Whitespace", state.show_whitespace, Message::ToggleWhitespace, t),
        check_item("Status Bar", state.show_status_bar, Message::ToggleStatusBar, t),
        check_item("Minimap", state.show_minimap, Message::ToggleMinimap, t),
        check_item("Function List", state.show_function_list, Message::ToggleFunctionList, t),
        check_item("Search Results", state.show_search_results_panel, Message::ToggleSearchResultsPanel, t),
        check_item("Show Indent Guides", state.show_indent_guides, Message::ToggleIndentGuides, t),
        check_item("Show Line Endings", state.show_line_endings, Message::ToggleLineEndings, t),
        separator(t),
        menu_item("Split Horizontal", "", Message::SplitHorizontal, t),
        menu_item("Split Vertical", "", Message::SplitVertical, t),
        menu_item("Remove Split", "", Message::RemoveSplit, t),
        separator(t),
        menu_item("Zoom In", "Ctrl+=", Message::ZoomIn, t),
        menu_item("Zoom Out", "Ctrl+-", Message::ZoomOut, t),
        menu_item("Reset Zoom", "Ctrl+0", Message::ZoomReset, t),
        separator(t),
        check_item("Full Screen", state.is_fullscreen, Message::ToggleFullScreen, t),
        separator(t),
    ];
    push_submenu(&mut items, "Folding", expanded, vec![
        menu_item("Toggle Fold", "Ctrl+Shift+[", Message::ToggleFold, t),
        menu_item("Fold All", "", Message::FoldAll, t),
        menu_item("Unfold All", "", Message::UnfoldAll, t),
        separator(t),
        menu_item("Fold Level 1", "", Message::FoldLevel(0), t),
        menu_item("Fold Level 2", "", Message::FoldLevel(4), t),
        menu_item("Fold Level 3", "", Message::FoldLevel(8), t),
    ], t);

    // Theme submenu
    let current_theme = &state.theme.name;
    let theme_names = ["Dark", "Light", "White", "High Contrast", "Solarized Dark", "Solarized Light"];
    let mut theme_children = Vec::new();
    for name in &theme_names {
        let check = if current_theme == *name { " * " } else { "   " };
        theme_children.push(menu_item(
            &format!("{}{}", check, name),
            "",
            Message::SetTheme(name.to_string()),
            t,
        ));
    }
    items.push(separator(t));
    push_submenu(&mut items, "Theme", expanded, theme_children, t);

    items
}

fn build_language_menu<'a>(state: &super::app::NotepadIced, t: &AppTheme) -> Vec<Element<'a, Message>> {
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
            t,
        ));
    }
    items
}

fn build_tools_menu<'a>(state: &super::app::NotepadIced, expanded: &std::collections::HashSet<String>, t: &AppTheme) -> Vec<Element<'a, Message>> {
    let mut items = Vec::new();
    push_submenu(&mut items, "JSON", expanded, vec![
        menu_item("Format JSON", "Ctrl+Shift+J", Message::JsonFormat, t),
        menu_item("Compact JSON", "", Message::JsonCompact, t),
        menu_item("Validate JSON", "", Message::JsonValidate, t),
        menu_item("Sort JSON Keys", "", Message::JsonSortKeys, t),
    ], t);
    items.push(separator(t));
    items.push(check_item("Markdown Preview", state.show_markdown_preview, Message::ToggleMarkdownPreview, t));
    items.push(check_item("CSV Viewer", state.show_csv_viewer, Message::ToggleCsvViewer, t));
    items.push(separator(t));
    push_submenu(&mut items, "MIME Tools", expanded, vec![
        menu_item("Base64 Encode", "", Message::MimeBase64Encode, t),
        menu_item("Base64 Decode", "", Message::MimeBase64Decode, t),
        separator(t),
        menu_item("URL Encode", "", Message::MimeUrlEncode, t),
        menu_item("URL Decode", "", Message::MimeUrlDecode, t),
        separator(t),
        menu_item("HTML Entity Encode", "", Message::MimeHtmlEncode, t),
        menu_item("HTML Entity Decode", "", Message::MimeHtmlDecode, t),
        separator(t),
        menu_item("Hex Encode", "", Message::MimeHexEncode, t),
        menu_item("Hex Decode", "", Message::MimeHexDecode, t),
    ], t);
    items.push(separator(t));
    items.push(menu_item("Compare Files...", "", Message::CompareFiles, t));
    items.push(check_item("Hex Viewer", state.show_hex_viewer, Message::ToggleHexViewer, t));
    items
}

fn build_macro_menu<'a>(t: &AppTheme) -> Vec<Element<'a, Message>> {
    vec![
        menu_item("Start/Stop Recording", "Ctrl+Shift+R", Message::ToggleMacroRecording, t),
        menu_item("Play Last Macro", "Ctrl+Shift+P", Message::PlayLastMacro, t),
        menu_item("Play Multiple Times...", "", Message::PlayMacroMultiple, t),
    ]
}

fn build_settings_menu<'a>(t: &AppTheme) -> Vec<Element<'a, Message>> {
    vec![
        menu_item("Preferences...", "", Message::ShowPreferences, t),
        menu_item("Keyboard Shortcuts...", "", Message::ShowKeybindings, t),
    ]
}

fn build_help_menu<'a>(t: &AppTheme) -> Vec<Element<'a, Message>> {
    vec![
        menu_item("About", "", Message::ShowAbout, t),
    ]
}

// ── Widget helpers ──

fn menu_item<'a>(label: &str, shortcut: &str, msg: Message, t: &AppTheme) -> Element<'a, Message> {
    let t_text = t.text;
    let t_text_dim = t.text_dim;
    let t_menu_hover = t.menu_hover;
    let label_text = text(label.to_string()).size(13);

    let content: Element<'a, Message> = if shortcut.is_empty() {
        container(label_text).width(Length::Fill).into()
    } else {
        row![
            container(label_text).width(Length::Fill),
            text(shortcut.to_string()).size(11).color(t_text_dim),
        ]
        .spacing(8)
        .into()
    };

    button(content)
        .on_press(msg)
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
                border: iced::Border {
                    radius: 2.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

fn menu_item_disabled<'a>(label: &str, shortcut: &str, t: &AppTheme) -> Element<'a, Message> {
    let t_text_dim = t.text_dim;
    let content: Element<'a, Message> = row![
        container(text(label.to_string()).size(13).color(t_text_dim)).width(Length::Fill),
        text(shortcut.to_string()).size(11).color(t_text_dim),
    ]
    .spacing(8)
    .into();

    container(content)
        .width(Length::Fill)
        .padding([3, 8])
        .into()
}

fn check_item<'a>(label: &str, checked: bool, msg: Message, t: &AppTheme) -> Element<'a, Message> {
    let prefix = if checked { " * " } else { "   " };
    menu_item(&format!("{}{}", prefix, label), "", msg, t)
}

fn submenu_toggle<'a>(label: &str, is_expanded: bool, t: &AppTheme) -> Element<'a, Message> {
    let t_text_dim = t.text_dim;
    let t_menu_hover = t.menu_hover;
    let arrow = if is_expanded { "v" } else { ">" };
    button(
        row![
            text(format!("{} {}", arrow, label)).size(13).color(t_text_dim),
        ]
        .spacing(4),
    )
    .on_press(Message::SubMenuToggle(label.to_string()))
    .width(Length::Fill)
    .padding([4, 8])
    .style(move |_theme: &Theme, status| {
        let bg = match status {
            button::Status::Hovered | button::Status::Pressed => Some(iced::Background::Color(t_menu_hover)),
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
    })
    .into()
}

fn push_submenu<'a>(
    items: &mut Vec<Element<'a, Message>>,
    label: &str,
    expanded: &std::collections::HashSet<String>,
    children: Vec<Element<'a, Message>>,
    t: &AppTheme,
) {
    let is_expanded = expanded.contains(label);
    items.push(submenu_toggle(label, is_expanded, t));
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

fn separator<'a>(t: &AppTheme) -> Element<'a, Message> {
    let t_separator = t.separator;
    container(Space::with_height(1))
        .width(Length::Fill)
        .padding([3, 4])
        .style(move |_theme: &Theme| container::Style {
            border: iced::Border {
                color: t_separator,
                width: 0.0,
                radius: 0.0.into(),
            },
            background: Some(iced::Background::Color(t_separator)),
            ..Default::default()
        })
        .into()
}
