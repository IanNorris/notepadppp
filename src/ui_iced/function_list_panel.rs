use iced::widget::{button, column, container, scrollable, text, row, Space};
use iced::{Element, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::AppColors;
use crate::editor::function_list::{extract_symbols, SymbolKind};

const PANEL_BG: iced::Color = iced::Color::from_rgb(0.14, 0.14, 0.17);
const ITEM_HOVER: iced::Color = iced::Color::from_rgb(0.20, 0.20, 0.25);
const KIND_FN: iced::Color = iced::Color::from_rgb(0.56, 0.74, 0.96);
const KIND_TYPE: iced::Color = iced::Color::from_rgb(0.90, 0.75, 0.40);

pub fn view_function_list<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let doc = state.tab_manager.active_document();
    let content_text = state
        .tab_contents
        .get(state.tab_manager.active_index())
        .map(|tc| tc.content.text())
        .unwrap_or_default();

    let language = if doc.language.is_empty() {
        "plain"
    } else {
        &doc.language
    };

    let symbols = extract_symbols(&content_text, language);

    let header = container(
        text("Function List").size(13).color(AppColors::TEXT),
    )
    .padding([8, 10])
    .width(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(0.12, 0.12, 0.15))),
        border: iced::Border {
            color: iced::Color::from_rgb(0.25, 0.25, 0.30),
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    });

    let mut items = column![].spacing(1);

    if symbols.is_empty() {
        items = items.push(
            container(text("No symbols found").size(12).color(AppColors::TEXT_DIM))
                .padding([6, 10]),
        );
    } else {
        for sym in &symbols {
            let kind_color = match sym.kind {
                SymbolKind::Function | SymbolKind::Method => KIND_FN,
                _ => KIND_TYPE,
            };
            let kind_label = format!("{}", sym.kind);
            let line = sym.line;

            let item_row = row![
                text(kind_label).size(11).color(kind_color),
                Space::with_width(6),
                text(sym.name.clone()).size(12).color(AppColors::TEXT),
                Space::with_width(Length::Fill),
                text(format!(":{}", line + 1)).size(11).color(AppColors::TEXT_DIM),
            ]
            .align_y(iced::Alignment::Center);

            let btn = button(
                container(item_row).padding([3, 8]),
            )
            .on_press(Message::GotoSymbol(line))
            .width(Length::Fill)
            .style(|_theme: &Theme, status| {
                let bg = match status {
                    button::Status::Hovered | button::Status::Pressed => ITEM_HOVER,
                    _ => PANEL_BG,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: AppColors::TEXT,
                    border: iced::Border::default(),
                    ..Default::default()
                }
            });

            items = items.push(btn);
        }
    }

    let panel = column![header, scrollable(items).height(Length::Fill)];

    container(panel)
        .width(Length::Fixed(200.0))
        .height(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(PANEL_BG)),
            border: iced::Border {
                color: iced::Color::from_rgb(0.25, 0.25, 0.30),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}
