use iced::widget::{column, container, scrollable, text, row, Space};
use iced::{Element, Font, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::AppColors;

const PANEL_BG: iced::Color = iced::Color::from_rgb(0.14, 0.14, 0.17);
const CODE_BG: iced::Color = iced::Color::from_rgb(0.10, 0.10, 0.13);
const HEADING_COLOR: iced::Color = iced::Color::from_rgb(0.80, 0.85, 0.95);

pub fn view_markdown_preview<'a>(state: &'a NotepadIced) -> Element<'a, Message> {
    let content_text = state
        .tab_contents
        .get(state.tab_manager.active_index())
        .map(|tc| tc.content.text())
        .unwrap_or_default();

    let header = container(
        text("Markdown Preview").size(13).color(AppColors::TEXT),
    )
    .padding([8, 10])
    .width(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(0.12, 0.12, 0.15))),
        ..Default::default()
    });

    let rendered = render_markdown_content(&content_text);

    let body = scrollable(
        container(rendered).padding([8, 12]).width(Length::Fill),
    )
    .height(Length::Fill);

    let panel = column![header, body];

    container(panel)
        .width(Length::FillPortion(1))
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

fn render_markdown_content<'a>(md_text: &str) -> Element<'a, Message> {
    let mut elements: Vec<Element<'a, Message>> = Vec::new();
    let mut in_code_block = false;
    let mut code_block_lines: Vec<String> = Vec::new();

    for line in md_text.lines() {
        if line.starts_with("```") {
            if in_code_block {
                // End code block
                let code_text = code_block_lines.join("\n");
                elements.push(render_code_block(&code_text));
                code_block_lines.clear();
                in_code_block = false;
            } else {
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            code_block_lines.push(line.to_string());
            continue;
        }

        if line.trim().is_empty() {
            elements.push(Space::with_height(8).into());
            continue;
        }

        if line.starts_with("### ") {
            elements.push(render_heading(&line[4..], 16.0));
        } else if line.starts_with("## ") {
            elements.push(render_heading(&line[3..], 19.0));
        } else if line.starts_with("# ") {
            elements.push(render_heading(&line[2..], 22.0));
        } else if line.starts_with("- ") || line.starts_with("* ") {
            elements.push(render_bullet(&line[2..]));
        } else if line.starts_with("> ") {
            elements.push(render_blockquote(&line[2..]));
        } else {
            elements.push(render_inline_text(line));
        }
    }

    // Unclosed code block
    if in_code_block && !code_block_lines.is_empty() {
        let code_text = code_block_lines.join("\n");
        elements.push(render_code_block(&code_text));
    }

    column(elements).spacing(2).width(Length::Fill).into()
}

fn render_heading<'a>(content: &str, size: f32) -> Element<'a, Message> {
    container(
        text(content.to_string())
            .size(size)
            .color(HEADING_COLOR)
            .font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::DEFAULT
            }),
    )
    .padding([4, 0])
    .into()
}

fn render_bullet<'a>(content: &str) -> Element<'a, Message> {
    row![
        Space::with_width(12),
        text("•").size(13).color(AppColors::TEXT_DIM),
        Space::with_width(6),
        render_inline_text(content),
    ]
    .align_y(iced::Alignment::Start)
    .into()
}

fn render_blockquote<'a>(content: &str) -> Element<'a, Message> {
    container(
        row![
            container(Space::with_width(3))
                .height(Length::Shrink)
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(AppColors::ACCENT)),
                    ..Default::default()
                }),
            Space::with_width(8),
            text(content.to_string()).size(13).color(AppColors::TEXT_DIM),
        ],
    )
    .padding([2, 0])
    .into()
}

fn render_code_block<'a>(code: &str) -> Element<'a, Message> {
    container(
        text(code.to_string())
            .size(12)
            .color(AppColors::TEXT)
            .font(Font::MONOSPACE),
    )
    .padding([6, 10])
    .width(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(CODE_BG)),
        border: iced::Border {
            color: iced::Color::from_rgb(0.25, 0.25, 0.30),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn render_inline_text<'a>(line: &str) -> Element<'a, Message> {
    // Simple inline parsing: **bold**, *italic*, `code`
    // For simplicity, render as segments in a row
    let mut elements: Vec<Element<'a, Message>> = Vec::new();
    let mut chars = line.chars().peekable();
    let mut current = String::new();

    while let Some(c) = chars.next() {
        if c == '`' {
            // Flush current
            if !current.is_empty() {
                elements.push(text(std::mem::take(&mut current)).size(13).color(AppColors::TEXT).into());
            }
            // Collect code span
            let mut code = String::new();
            while let Some(&nc) = chars.peek() {
                if nc == '`' {
                    chars.next();
                    break;
                }
                code.push(nc);
                chars.next();
            }
            elements.push(
                container(
                    text(code).size(12).color(AppColors::TEXT).font(Font::MONOSPACE),
                )
                .padding([1, 4])
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(CODE_BG)),
                    border: iced::Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into(),
            );
        } else if c == '*' {
            if chars.peek() == Some(&'*') {
                // Bold
                chars.next();
                if !current.is_empty() {
                    elements.push(text(std::mem::take(&mut current)).size(13).color(AppColors::TEXT).into());
                }
                let mut bold_text = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc == '*' {
                        chars.next();
                        if chars.peek() == Some(&'*') {
                            chars.next();
                        }
                        break;
                    }
                    bold_text.push(nc);
                    chars.next();
                }
                elements.push(
                    text(bold_text)
                        .size(13)
                        .color(AppColors::TEXT)
                        .font(Font {
                            weight: iced::font::Weight::Bold,
                            ..Font::DEFAULT
                        })
                        .into(),
                );
            } else {
                // Italic — iced doesn't have italic style easily, render as-is with dim color
                if !current.is_empty() {
                    elements.push(text(std::mem::take(&mut current)).size(13).color(AppColors::TEXT).into());
                }
                let mut italic_text = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc == '*' {
                        chars.next();
                        break;
                    }
                    italic_text.push(nc);
                    chars.next();
                }
                elements.push(
                    text(italic_text)
                        .size(13)
                        .color(iced::Color::from_rgb(0.75, 0.80, 0.85))
                        .into(),
                );
            }
        } else {
            current.push(c);
        }
    }

    if !current.is_empty() {
        elements.push(text(current).size(13).color(AppColors::TEXT).into());
    }

    if elements.is_empty() {
        Space::with_height(0).into()
    } else if elements.len() == 1 {
        elements.remove(0)
    } else {
        row(elements)
            .align_y(iced::Alignment::Center)
            .into()
    }
}
