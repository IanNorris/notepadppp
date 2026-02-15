use iced::widget::{
    button, column, container, pick_list, row, scrollable, text, text_input, Space,
};
use iced::{Element, Font, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::AppTheme;
use crate::tools::disasm::DisasmArch;

const VISIBLE_LINES: usize = 50;

pub fn view_disasm_panel<'a>(state: &'a NotepadIced, theme: &AppTheme) -> Element<'a, Message> {
    let panel_bg = theme.background;
    let header_bg = theme.tab_bar_bg;
    let text_color = theme.text;
    let text_dim = theme.text_dim;
    let accent = theme.accent;
    let separator = theme.border;

    // Toolbar row
    let arch_label = match state.disasm_arch {
        DisasmArch::X86_32 => "x86-32",
        DisasmArch::X86_64 => "x86-64",
        DisasmArch::ARM => "ARM",
        DisasmArch::AArch64 => "AArch64",
    };
    let arch_options = vec!["x86-32", "x86-64", "ARM", "AArch64"];

    let arch_picker: Element<'a, Message> = pick_list(arch_options, Some(arch_label), |s| {
        let arch = match s {
            "x86-32" => DisasmArch::X86_32,
            "x86-64" => DisasmArch::X86_64,
            "ARM" => DisasmArch::ARM,
            "AArch64" => DisasmArch::AArch64,
            _ => DisasmArch::X86_64,
        };
        Message::DisasmSetArch(arch)
    })
    .text_size(12)
    .into();

    let goto_addr_input: Element<'a, Message> = text_input("Go to address...", &state.disasm_goto_addr)
        .on_input(Message::DisasmGotoAddressInput)
        .on_submit(Message::DisasmGotoAddress(state.disasm_goto_addr.clone()))
        .size(12)
        .width(140)
        .into();

    let goto_sym_input: Element<'a, Message> = text_input("Go to symbol...", &state.disasm_goto_sym)
        .on_input(Message::DisasmGotoSymbolInput)
        .on_submit(Message::DisasmGotoSymbol(state.disasm_goto_sym.clone()))
        .size(12)
        .width(140)
        .into();

    let base_addr_input: Element<'a, Message> =
        text_input("Base address", &state.disasm_base_addr_input)
            .on_input(Message::DisasmBaseAddressInput)
            .on_submit(Message::DisasmSetBaseAddress(
                state.disasm_base_addr_input.clone(),
            ))
            .size(12)
            .width(120)
            .into();

    let byte_count = state
        .disasm_state
        .as_ref()
        .map(|d| d.bytes().len())
        .unwrap_or(0);

    let toolbar = container(
        row![
            text("Disassembler").size(13).color(text_color),
            Space::with_width(10),
            arch_picker,
            Space::with_width(6),
            text("Base:").size(11).color(text_dim),
            base_addr_input,
            Space::with_width(6),
            goto_addr_input,
            Space::with_width(6),
            goto_sym_input,
            Space::with_width(Length::Fill),
            text(format!("{} bytes", byte_count))
                .size(12)
                .color(text_dim),
        ]
        .align_y(iced::Alignment::Center)
        .padding([6, 10]),
    )
    .width(Length::Fill)
    .style(move |_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(header_bg)),
        ..Default::default()
    });

    // Column header
    let col_header = container(
        row![
            container(text("Address").size(11).color(text_dim).font(Font::MONOSPACE)).width(160),
            container(text("Bytes").size(11).color(text_dim).font(Font::MONOSPACE)).width(180),
            container(text("Mnemonic").size(11).color(text_dim).font(Font::MONOSPACE)).width(80),
            container(text("Operands").size(11).color(text_dim).font(Font::MONOSPACE)).width(300),
            container(text("Comment").size(11).color(text_dim).font(Font::MONOSPACE)).width(Length::Fill),
        ]
        .align_y(iced::Alignment::Center),
    )
    .padding([4, 10])
    .width(Length::Fill)
    .style(move |_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(panel_bg)),
        border: iced::Border {
            color: separator,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    });

    // Disassembly rows
    let mut rows = column![].spacing(0);

    if let Some(ref disasm) = state.disasm_state {
        let lines = disasm.disassemble_range(state.disasm_offset, VISIBLE_LINES);

        if lines.is_empty() {
            rows = rows.push(
                container(text("No instructions at this offset").size(13).color(text_dim))
                    .padding([10, 10]),
            );
        } else {
            for line in &lines {
                // Symbol label
                if let Some(ref sym) = line.symbol {
                    rows = rows.push(
                        container(
                            text(format!("<{}>:", sym))
                                .size(12)
                                .color(accent)
                                .font(Font::MONOSPACE),
                        )
                        .padding([2, 10]),
                    );
                }

                let addr_str = format!("0x{:016X}", line.address);
                let bytes_str = line
                    .bytes
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                // Pad bytes column to 20 chars
                let bytes_padded = format!("{:<20}", bytes_str);

                let mnemonic_padded = format!("{:<10}", line.mnemonic);

                let addr_color = if line.is_branch_target {
                    accent
                } else {
                    text_dim
                };

                let operands_str = line.operands.clone();
                let comment_str = line.comment.clone().unwrap_or_default();

                let insn_row = row![
                    container(text(addr_str)
                        .size(12)
                        .color(addr_color)
                        .font(Font::MONOSPACE)).width(160),
                    container(text(bytes_padded)
                        .size(12)
                        .color(text_dim)
                        .font(Font::MONOSPACE)).width(180),
                    container(text(mnemonic_padded)
                        .size(12)
                        .color(text_color)
                        .font(Font::MONOSPACE)).width(80),
                    container(text(operands_str)
                        .size(12)
                        .color(text_color)
                        .font(Font::MONOSPACE)).width(300),
                    container(text(if comment_str.is_empty() { String::new() } else { format!("; {}", comment_str) })
                        .size(12)
                        .color(text_dim)
                        .font(Font::MONOSPACE)).width(Length::Fill),
                ]
                .align_y(iced::Alignment::Center);

                rows = rows.push(container(insn_row).padding([1, 10]));
            }
        }
    } else {
        rows = rows.push(
            container(
                text("No binary loaded. Open a binary file and toggle the Disassembler view.")
                    .size(13)
                    .color(text_dim),
            )
            .padding([10, 10]),
        );
    }

    // Navigation buttons
    let nav_row = container(
        row![
            button(text("↑ Page Up").size(11))
                .on_press(Message::DisasmScroll(-50))
                .padding([4, 8]),
            Space::with_width(4),
            button(text("↓ Page Down").size(11))
                .on_press(Message::DisasmScroll(50))
                .padding([4, 8]),
            Space::with_width(4),
            button(text("⇤ Top").size(11))
                .on_press(Message::DisasmScroll(i32::MIN))
                .padding([4, 8]),
            Space::with_width(Length::Fill),
            text(format!("Offset: 0x{:X}", state.disasm_offset))
                .size(11)
                .color(text_dim),
        ]
        .align_y(iced::Alignment::Center)
        .padding([4, 10]),
    )
    .width(Length::Fill)
    .style(move |_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(header_bg)),
        border: iced::Border {
            color: separator,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    });

    let body = scrollable(rows.width(Length::Fill))
        .height(Length::Fill)
        .width(Length::Fill)
        .direction(iced::widget::scrollable::Direction::Vertical(
            iced::widget::scrollable::Scrollbar::new()
                .width(10)
                .scroller_width(8),
        ));
    let panel = column![toolbar, col_header, body, nav_row];

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(panel_bg)),
            ..Default::default()
        })
        .into()
}
