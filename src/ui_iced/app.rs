use iced::keyboard;
use iced::widget::{button, column, container, row, text, text_editor, Space};
use iced::{Element, Length, Subscription, Task, Theme};

use crate::editor::document::{Encoding, LineEnding};
use crate::editor::tab_manager::TabManager;

use super::theme::AppColors;

/// Per-tab state that pairs an iced text_editor::Content with our Document index.
struct TabContent {
    content: text_editor::Content,
}

impl TabContent {
    fn new() -> Self {
        Self {
            content: text_editor::Content::new(),
        }
    }

    fn with_text(s: &str) -> Self {
        Self {
            content: text_editor::Content::with_text(s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    EditorAction(text_editor::Action),
    NewTab,
    OpenFile,
    FileOpened(Result<(String, std::path::PathBuf), String>),
    Save,
    SaveAs,
    FileSaved(Result<std::path::PathBuf, String>),
    CloseTab(usize),
    SelectTab(usize),
    Undo,
    Redo,
}

pub struct NotepadIced {
    tab_manager: TabManager,
    tab_contents: Vec<TabContent>,
}

impl Default for NotepadIced {
    fn default() -> Self {
        Self {
            tab_manager: TabManager::new(),
            tab_contents: vec![TabContent::new()],
        }
    }
}

pub fn title(state: &NotepadIced) -> String {
    let doc = state.tab_manager.active_document();
    let tab_title = state
        .tab_manager
        .get_tab_title(state.tab_manager.active_index());
    let modified = if doc.is_modified() { " •" } else { "" };
    format!("{}{} — Notepad+++", tab_title, modified)
}

pub fn update(state: &mut NotepadIced, message: Message) -> Task<Message> {
    match message {
        Message::EditorAction(action) => {
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
                }
            }
            Task::none()
        }
        Message::NewTab => {
            state.tab_manager.new_tab();
            state.tab_contents.push(TabContent::new());
            Task::none()
        }
        Message::OpenFile => Task::perform(
            async {
                let handle = rfd::AsyncFileDialog::new()
                    .set_title("Open File")
                    .pick_file()
                    .await;
                match handle {
                    Some(h) => {
                        let path = h.path().to_path_buf();
                        match std::fs::read_to_string(&path) {
                            Ok(content) => Ok((content, path)),
                            Err(e) => Err(e.to_string()),
                        }
                    }
                    None => Err("Cancelled".to_string()),
                }
            },
            Message::FileOpened,
        ),
        Message::FileOpened(result) => {
            if let Ok((content, path)) = result {
                match state.tab_manager.open_file(path) {
                    Ok(idx) => {
                        let doc = state.tab_manager.get_document(idx).unwrap();
                        let buf_text = doc.buffer.text();
                        state.tab_contents.push(TabContent::with_text(&buf_text));
                    }
                    Err(_) => {
                        state.tab_manager.new_tab();
                        state.tab_contents.push(TabContent::with_text(&content));
                    }
                }
            }
            Task::none()
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
                let idx = state.tab_manager.active_index();
                if let Err(e) = state.tab_manager.save_tab_as(idx, path) {
                    log::error!("Save As failed: {}", e);
                }
            }
            Task::none()
        }
        Message::CloseTab(idx) => {
            if idx < state.tab_contents.len() {
                state.tab_contents.remove(idx);
                state.tab_manager.close_tab(idx);
                while state.tab_contents.len() < state.tab_manager.tab_count() {
                    state.tab_contents.push(TabContent::new());
                }
            }
            Task::none()
        }
        Message::SelectTab(idx) => {
            if idx < state.tab_manager.tab_count() {
                state.tab_manager.set_active(idx);
            }
            Task::none()
        }
        Message::Undo => {
            let doc = state.tab_manager.active_document_mut();
            doc.buffer.undo();
            let buf_text = doc.buffer.text();
            let idx = state.tab_manager.active_index();
            state.tab_contents[idx] = TabContent::with_text(&buf_text);
            Task::none()
        }
        Message::Redo => {
            let doc = state.tab_manager.active_document_mut();
            doc.buffer.redo();
            let buf_text = doc.buffer.text();
            let idx = state.tab_manager.active_index();
            state.tab_contents[idx] = TabContent::with_text(&buf_text);
            Task::none()
        }
    }
}

pub fn view(state: &NotepadIced) -> Element<'_, Message> {
    let menu_bar = view_menu_bar();
    let tab_bar = view_tab_bar(state);
    let editor = view_editor(state);
    let status_bar = view_status_bar(state);

    let content = column![menu_bar, tab_bar, editor, status_bar];

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn subscription(_state: &NotepadIced) -> Subscription<Message> {
    keyboard::on_key_press(|key, modifiers| {
        if !modifiers.command() {
            return None;
        }
        match key.as_ref() {
            keyboard::Key::Character("n") if !modifiers.shift() => Some(Message::NewTab),
            keyboard::Key::Character("o") => Some(Message::OpenFile),
            keyboard::Key::Character("s") if modifiers.shift() => Some(Message::SaveAs),
            keyboard::Key::Character("s") => Some(Message::Save),
            keyboard::Key::Character("w") => Some(Message::CloseTab(usize::MAX)), // sentinel
            keyboard::Key::Character("z") if !modifiers.shift() => Some(Message::Undo),
            keyboard::Key::Character("y") => Some(Message::Redo),
            _ => None,
        }
    })
}

pub fn theme(_state: &NotepadIced) -> Theme {
    Theme::Dark
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

fn view_menu_bar<'a>() -> Element<'a, Message> {
    let file_buttons = row![
        menu_button("New", Message::NewTab),
        menu_button("Open", Message::OpenFile),
        menu_button("Save", Message::Save),
        menu_button("Save As", Message::SaveAs),
    ]
    .spacing(4);

    let edit_buttons = row![
        menu_button("Undo", Message::Undo),
        menu_button("Redo", Message::Redo),
    ]
    .spacing(4);

    container(
        row![file_buttons, Space::with_width(20), edit_buttons]
            .spacing(4)
            .padding(4),
    )
    .width(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(AppColors::TAB_BAR_BG)),
        ..Default::default()
    })
    .into()
}

fn view_tab_bar<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let count = state.tab_manager.tab_count();
    let active = state.tab_manager.active_index();

    let mut tabs = row![].spacing(2).padding(4);

    for i in 0..count {
        let tab_title = state.tab_manager.get_tab_title(i);
        let is_active = i == active;

        let bg_color = if is_active {
            AppColors::TAB_ACTIVE_BG
        } else {
            AppColors::TAB_INACTIVE_BG
        };

        let label = text(tab_title).size(13);
        let close = button(text("×").size(13))
            .on_press(Message::CloseTab(i))
            .padding(2)
            .style(|_theme: &Theme, _status| button::Style {
                background: None,
                text_color: AppColors::TEXT_DIM,
                ..Default::default()
            });

        let tab = button(row![label, close].spacing(6).padding([4, 8]))
            .on_press(Message::SelectTab(i))
            .style(move |_theme: &Theme, _status| button::Style {
                background: Some(iced::Background::Color(bg_color)),
                text_color: AppColors::TEXT,
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        tabs = tabs.push(tab);
    }

    container(tabs)
        .width(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(AppColors::TAB_BAR_BG)),
            ..Default::default()
        })
        .into()
}

fn view_editor<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let active = state.tab_manager.active_index();

    if let Some(tc) = state.tab_contents.get(active) {
        let editor = text_editor(&tc.content)
            .on_action(Message::EditorAction)
            .height(Length::Fill);

        container(editor)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(AppColors::BACKGROUND)),
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

fn view_status_bar<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let active = state.tab_manager.active_index();
    let (line, col, encoding_str, line_ending_str, language) =
        if let Some(tc) = state.tab_contents.get(active) {
            let (cursor_line, cursor_col) = tc.content.cursor_position();
            let doc = state.tab_manager.active_document();
            let enc = match doc.encoding {
                Encoding::UTF8 => "UTF-8",
                Encoding::UTF8BOM => "UTF-8 BOM",
                Encoding::UTF16LE => "UTF-16 LE",
                Encoding::UTF16BE => "UTF-16 BE",
                Encoding::ASCII => "ASCII",
            };
            let le = match doc.line_ending {
                LineEnding::LF => "LF",
                LineEnding::CRLF => "CRLF",
                LineEnding::CR => "CR",
            };
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

    let status_text = text(format!(
        "  Ln {}, Col {}    {}    {}    {}",
        line, col, encoding_str, line_ending_str, language
    ))
    .size(12);

    container(status_text)
        .width(Length::Fill)
        .padding([2, 8])
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(AppColors::STATUS_BAR_BG)),
            ..Default::default()
        })
        .into()
}

fn menu_button(label: &str, msg: Message) -> Element<'_, Message> {
    button(text(label).size(13))
        .on_press(msg)
        .padding([4, 10])
        .style(|_theme: &Theme, _status| button::Style {
            background: None,
            text_color: AppColors::TEXT,
            ..Default::default()
        })
        .into()
}
