use iced::widget::{
    button, column, container, mouse_area, pick_list, row, scrollable, text, text_input, Space,
};
use iced::{Element, Font, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::AppTheme;
use crate::tools::disasm::{compute_branch_arrows, render_arrow_column, DisasmArch};

const VISIBLE_LINES: usize = 60;

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

    let goto_addr_input: Element<'a, Message> =
        text_input("Go to address...", &state.disasm_goto_addr)
            .on_input(Message::DisasmGotoAddressInput)
            .on_submit(Message::DisasmGotoAddress(state.disasm_goto_addr.clone()))
            .size(12)
            .width(140)
            .into();

    let goto_sym_input: Element<'a, Message> =
        text_input("Go to symbol...", &state.disasm_goto_sym)
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
    let sym_count = state
        .disasm_state
        .as_ref()
        .map(|d| d.symbols().len())
        .unwrap_or(0);

    let sym_browser_btn = button(text("Symbols").size(11))
        .on_press(Message::DisasmToggleSymBrowser)
        .padding([3, 8]);

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
            Space::with_width(6),
            sym_browser_btn,
            Space::with_width(Length::Fill),
            text(format!("{} bytes  {} syms", byte_count, sym_count))
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

    // Column header — dynamic based on toggle state
    let mut header_row = row![].align_y(iced::Alignment::Center);
    if state.disasm_show_address {
        header_row = header_row.push(
            container(text("Address").size(11).color(text_dim).font(Font::MONOSPACE)).width(160),
        );
    }
    if state.disasm_show_bytes {
        header_row = header_row.push(
            container(text("Bytes").size(11).color(text_dim).font(Font::MONOSPACE)).width(180),
        );
    }
    header_row = header_row.push(
        container(text("Mnemonic").size(11).color(text_dim).font(Font::MONOSPACE)).width(80),
    );
    header_row = header_row.push(
        container(text("Operands").size(11).color(text_dim).font(Font::MONOSPACE))
            .width(Length::Fill),
    );
    if state.disasm_show_raw_comment {
        header_row = header_row.push(
            container(text("").size(11).color(text_dim).font(Font::MONOSPACE)).width(280),
        );
    }

    let col_header = container(header_row)
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
    let mut rows = column![].spacing(0).width(Length::Fill);

    if let Some(ref disasm) = state.disasm_state {
        let lines = disasm.disassemble_range(state.disasm_offset, VISIBLE_LINES);

        if lines.is_empty() {
            rows = rows.push(
                container(
                    text("No instructions at this offset")
                        .size(13)
                        .color(text_dim),
                )
                .padding([10, 10]),
            );
        } else {
            // Compute branch arrows for the visible lines
            let arrows = if state.disasm_show_arrows {
                compute_branch_arrows(&lines)
            } else {
                Vec::new()
            };
            let num_lanes = arrows.iter().map(|a| a.lane + 1).max().unwrap_or(0);
            let num_lanes = num_lanes.min(8);
            let arrow_col_width = if state.disasm_show_arrows && num_lanes > 0 {
                (num_lanes + 1) * 8 + 4
            } else {
                0
            };

            for (line_idx, line) in lines.iter().enumerate() {
                // Symbol label — render as a row with same column structure
                if let Some(ref sym) = line.symbol {
                    let mut sym_row = row![].align_y(iced::Alignment::Center);
                    if state.disasm_show_address {
                        sym_row = sym_row.push(container(Space::with_width(0)).width(160));
                    }
                    if state.disasm_show_bytes {
                        sym_row = sym_row.push(container(Space::with_width(0)).width(180));
                    }
                    // Arrow column continuation for symbol rows
                    if state.disasm_show_arrows && num_lanes > 0 {
                        let arrow_str = render_arrow_column(line_idx, &arrows, num_lanes);
                        let arrow_color = if arrow_str.trim().is_empty() {
                            text_dim
                        } else {
                            accent
                        };
                        sym_row = sym_row.push(
                            container(
                                text(arrow_str)
                                    .size(12)
                                    .color(arrow_color)
                                    .font(Font::MONOSPACE),
                            )
                            .width(arrow_col_width as u16),
                        );
                    }
                    // Truncate long symbol names to avoid wrapping
                    let sym_display = if sym.len() > 80 {
                        format!("<{}...>:", &sym[..77])
                    } else {
                        format!("<{}>:", sym)
                    };
                    sym_row = sym_row.push(
                        text(sym_display)
                            .size(12)
                            .color(accent)
                            .font(Font::MONOSPACE),
                    );
                    rows = rows.push(container(sym_row).padding([2, 10]));
                }

                let mut insn_row = row![].align_y(iced::Alignment::Center);

                if state.disasm_show_address {
                    let addr_str = format!("0x{:016X}", line.address);
                    let addr_copy = addr_str.clone();
                    insn_row = insn_row.push(
                        container(
                            button(
                                text(addr_str)
                                    .size(12)
                                    .color(text_dim)
                                    .font(Font::MONOSPACE),
                            )
                            .on_press(Message::DisasmCopyText(addr_copy))
                            .padding(0)
                            .style(move |_theme: &Theme, _status| button::Style {
                                background: None,
                                text_color: text_dim,
                                ..Default::default()
                            }),
                        )
                        .width(160),
                    );
                }

                if state.disasm_show_bytes {
                    let bytes_str = line
                        .bytes
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let bytes_padded = format!("{:<20}", bytes_str);
                    insn_row = insn_row.push(
                        container(
                            text(bytes_padded)
                                .size(12)
                                .color(text_dim)
                                .font(Font::MONOSPACE),
                        )
                        .width(180),
                    );
                }

                // Arrow column
                if state.disasm_show_arrows && num_lanes > 0 {
                    let arrow_str = render_arrow_column(line_idx, &arrows, num_lanes);
                    let arrow_color = if arrow_str.trim().is_empty() {
                        text_dim
                    } else {
                        accent
                    };
                    insn_row = insn_row.push(
                        container(
                            text(arrow_str)
                                .size(12)
                                .color(arrow_color)
                                .font(Font::MONOSPACE),
                        )
                        .width(arrow_col_width as u16),
                    );
                }

                let mnemonic_padded = format!("{:<10}", line.mnemonic);
                // Build full line text for copy
                let full_line = {
                    let mut s = String::new();
                    s.push_str(&format!("{:016X}  ", line.address));
                    s.push_str(&format!("{:<10} {}", line.mnemonic, line.operands));
                    if let Some(ref c) = line.comment {
                        s.push_str(&format!("  ; {}", c));
                    }
                    s
                };
                insn_row = insn_row.push(
                    container(
                        button(
                            text(mnemonic_padded)
                                .size(12)
                                .color(text_color)
                                .font(Font::MONOSPACE),
                        )
                        .on_press(Message::DisasmCopyText(full_line))
                        .padding(0)
                        .style(move |_theme: &Theme, _status| button::Style {
                            background: None,
                            text_color: text_color,
                            ..Default::default()
                        }),
                    )
                    .width(80),
                );

                // Operands: show resolved or raw depending on toggle
                let operands_display = if state.disasm_show_resolved {
                    line.operands.clone()
                } else {
                    // Show raw (the comment has the raw form when resolved)
                    line.comment.clone().unwrap_or_else(|| line.operands.clone())
                };

                // Make operands clickable if this instruction has a branch/call target
                let has_nav_target = line.branch_target.is_some() && line.comment.is_some();
                if has_nav_target {
                    let target = line.branch_target.unwrap();
                    insn_row = insn_row.push(
                        container(
                            button(
                                text(operands_display)
                                    .size(12)
                                    .color(accent)
                                    .font(Font::MONOSPACE),
                            )
                            .on_press(Message::DisasmNavigateToAddress(target))
                            .padding(0)
                            .style(move |_theme: &Theme, _status| button::Style {
                                background: None,
                                text_color: accent,
                                ..Default::default()
                            }),
                        )
                        .width(Length::Fill),
                    );
                } else {
                    insn_row = insn_row.push(
                        container(
                            text(operands_display)
                                .size(12)
                                .color(text_color)
                                .font(Font::MONOSPACE),
                        )
                        .width(Length::Fill),
                    );
                }

                // Comment column: show raw when resolved, or nothing
                if state.disasm_show_raw_comment {
                    let comment_text = if state.disasm_show_resolved {
                        line.comment
                            .as_ref()
                            .map(|c| format!("; {}", c))
                            .unwrap_or_default()
                    } else {
                        // When showing raw operands, show resolved as comment
                        if line.comment.is_some() {
                            format!("; {}", line.operands)
                        } else {
                            String::new()
                        }
                    };
                    insn_row = insn_row.push(
                        container(
                            text(comment_text)
                                .size(12)
                                .color(text_dim)
                                .font(Font::MONOSPACE),
                        )
                        .width(280),
                    );
                }

                rows = rows.push(container(insn_row).padding([0, 10]));
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

    // Status bar at bottom
    let status_bar = container(
        row![
            text(format!("Offset: 0x{:X}", state.disasm_offset))
                .size(11)
                .color(text_dim),
            Space::with_width(Length::Fill),
            text(if state.disasm_edit_mode {
                "EDIT MODE"
            } else {
                "Scroll with mouse wheel · Right-click for options"
            })
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

    // Use mouse_area to capture right-click
    let body_content = mouse_area(rows)
        .on_right_press(Message::DisasmContextMenu);

    let body = container(body_content)
        .height(Length::Fill)
        .width(Length::Fill)
        .clip(true);

    let panel = column![toolbar, col_header, body, status_bar];

    let base_panel: Element<'a, Message> = container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(panel_bg)),
            ..Default::default()
        })
        .into();

    // Overlay: context menu or symbol browser
    if state.disasm_context_menu {
        let check = |v: bool| if v { "✓ " } else { "  " };

        let ctx_menu = container(
            column![
                button(
                    text(format!("{}Show Address", check(state.disasm_show_address)))
                        .size(12)
                        .font(Font::MONOSPACE),
                )
                .on_press(Message::DisasmToggleAddress)
                .width(Length::Fill)
                .padding([4, 12]),
                button(
                    text(format!("{}Show Bytes", check(state.disasm_show_bytes)))
                        .size(12)
                        .font(Font::MONOSPACE),
                )
                .on_press(Message::DisasmToggleBytes)
                .width(Length::Fill)
                .padding([4, 12]),
                button(
                    text(format!(
                        "{}Symbol Resolution",
                        check(state.disasm_show_resolved)
                    ))
                    .size(12)
                    .font(Font::MONOSPACE),
                )
                .on_press(Message::DisasmToggleResolved)
                .width(Length::Fill)
                .padding([4, 12]),
                button(
                    text(format!(
                        "{}Show Raw Comment",
                        check(state.disasm_show_raw_comment)
                    ))
                    .size(12)
                    .font(Font::MONOSPACE),
                )
                .on_press(Message::DisasmToggleRawComment)
                .width(Length::Fill)
                .padding([4, 12]),
                button(
                    text(format!(
                        "{}Branch Arrows",
                        check(state.disasm_show_arrows)
                    ))
                    .size(12)
                    .font(Font::MONOSPACE),
                )
                .on_press(Message::DisasmToggleArrows)
                .width(Length::Fill)
                .padding([4, 12]),
                container(Space::with_height(1))
                    .width(Length::Fill)
                    .style(move |_: &Theme| container::Style {
                        background: Some(iced::Background::Color(separator)),
                        ..Default::default()
                    }),
                button(text("Copy All Visible").size(12))
                    .on_press(Message::DisasmCopySelection)
                    .width(Length::Fill)
                    .padding([4, 12]),
                button(
                    text(format!(
                        "{}Edit Bytes",
                        check(state.disasm_edit_mode)
                    ))
                    .size(12)
                    .font(Font::MONOSPACE),
                )
                .on_press(Message::DisasmToggleEditMode)
                .width(Length::Fill)
                .padding([4, 12]),
            ]
            .spacing(1)
            .width(200),
        )
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(header_bg)),
            border: iced::Border {
                color: separator,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .padding(4);

        use iced::widget::{opaque, stack};
        // Overlay the context menu
        stack![
            base_panel,
            mouse_area(
                container(
                    mouse_area(opaque(ctx_menu))
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding([100, 300])
            )
            .on_press(Message::DisasmCloseContextMenu)
        ]
        .into()
    } else if state.disasm_sym_browser {
        view_symbol_browser(state, theme, base_panel)
    } else {
        base_panel
    }
}

fn view_symbol_browser<'a>(
    state: &'a NotepadIced,
    theme: &AppTheme,
    base: Element<'a, Message>,
) -> Element<'a, Message> {
    let header_bg = theme.tab_bar_bg;
    let text_color = theme.text;
    let text_dim = theme.text_dim;
    let separator = theme.border;
    let accent = theme.accent;

    let filter_input: Element<'a, Message> = text_input("Filter symbols...", &state.disasm_sym_filter)
        .on_input(Message::DisasmSymFilterInput)
        .size(13)
        .width(Length::Fill)
        .into();

    let mut sym_list = column![].spacing(0);
    let mut count = 0;
    let max_results = 1000;

    if let Some(ref disasm) = state.disasm_state {
        let filter = state.disasm_sym_filter.to_lowercase();
        for sym in disasm.symbols() {
            if !filter.is_empty() && !sym.name.to_lowercase().contains(&filter) {
                continue;
            }
            if count >= max_results {
                sym_list = sym_list.push(
                    container(
                        text(format!("... {} more (refine filter)", disasm.symbols().len() - count))
                            .size(11)
                            .color(text_dim),
                    )
                    .padding([4, 8]),
                );
                break;
            }
            let addr = sym.address;
            let sym_row = button(
                row![
                    container(
                        text(format!("0x{:016X}", addr))
                            .size(11)
                            .color(text_dim)
                            .font(Font::MONOSPACE),
                    )
                    .width(160),
                    text(&sym.name)
                        .size(11)
                        .color(text_color)
                        .font(Font::MONOSPACE),
                ]
                .align_y(iced::Alignment::Center),
            )
            .on_press(Message::DisasmGotoSymFromBrowser(addr))
            .width(Length::Fill)
            .padding([2, 8]);
            sym_list = sym_list.push(sym_row);
            count += 1;
        }
    }

    if count == 0 && !state.disasm_sym_filter.is_empty() {
        sym_list = sym_list.push(
            container(
                text("No matching symbols")
                    .size(12)
                    .color(text_dim),
            )
            .padding([10, 8]),
        );
    }

    let browser = container(
        column![
            container(
                row![
                    text("Symbol Browser").size(13).color(accent),
                    Space::with_width(Length::Fill),
                    text(format!("{} symbols", state.disasm_state.as_ref().map(|d| d.symbols().len()).unwrap_or(0)))
                        .size(11)
                        .color(text_dim),
                    Space::with_width(8),
                    button(text("✕").size(12))
                        .on_press(Message::DisasmToggleSymBrowser)
                        .padding([2, 6]),
                ]
                .align_y(iced::Alignment::Center)
                .padding([8, 12]),
            )
            .width(Length::Fill)
            .style(move |_: &Theme| container::Style {
                background: Some(iced::Background::Color(header_bg)),
                ..Default::default()
            }),
            container(filter_input).padding([6, 12]),
            scrollable(sym_list)
                .height(400)
                .width(Length::Fill),
        ]
        .width(500),
    )
    .style(move |_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(header_bg)),
        border: iced::Border {
            color: separator,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    });

    use iced::widget::{opaque, stack};
    stack![
        base,
        iced::widget::mouse_area(
            container(
                iced::widget::mouse_area(opaque(browser))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .padding([60, 0])
        )
        .on_press(Message::DisasmToggleSymBrowser)
    ]
    .into()
}
