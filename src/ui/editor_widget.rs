use std::collections::BTreeSet;
use egui::{self, Color32, FontId, Rect, RichText, ScrollArea, TextEdit, TextFormat, Ui, Vec2};
use egui::text::LayoutJob;

use crate::editor::syntax::SyntaxHighlighter;
use crate::search::SearchMatch;

/// Build a LayoutJob with syntax-highlighted spans for the given text.
fn highlight_text(
    text: &str,
    extension: &str,
    highlighter: &SyntaxHighlighter,
    font: FontId,
    default_color: Color32,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;

    if extension.is_empty() {
        // Plain text — single span
        job.append(
            text,
            0.0,
            TextFormat {
                font_id: font,
                color: default_color,
                ..Default::default()
            },
        );
        return job;
    }

    for line in text.split_inclusive('\n') {
        let spans = highlighter.highlight_line(line, extension);
        for (style, segment) in spans {
            let fg = style.foreground;
            let color = Color32::from_rgb(fg.r, fg.g, fg.b);
            job.append(
                &segment,
                0.0,
                TextFormat {
                    font_id: font.clone(),
                    color,
                    ..Default::default()
                },
            );
        }
    }
    // Handle text that doesn't end with newline — split_inclusive covers this fine.
    job
}

/// Renders the editor widget with optional line numbers, word wrap, search highlights, and syntax highlighting.
pub fn editor_widget(
    ui: &mut Ui,
    text: &mut String,
    font_size: f32,
    show_line_numbers: bool,
    word_wrap: bool,
    search_matches: &[SearchMatch],
    current_match_index: Option<usize>,
    highlighter: &SyntaxHighlighter,
    file_extension: &str,
    bookmarked_lines: &[usize],
    matching_bracket_pos: Option<usize>,
) {
    let font = FontId::monospace(font_size);
    let available = ui.available_size();
    let default_color = ui.visuals().text_color();

    let ext = file_extension.to_string();
    let hl = highlighter;
    let f = font.clone();
    let dc = default_color;

    let mut layouter = |ui: &egui::Ui, s: &str, _wrap_width: f32| {
        let job = highlight_text(s, &ext, hl, f.clone(), dc);
        ui.fonts(|fonts| fonts.layout_job(job))
    };

    if show_line_numbers {
        let line_count = text.lines().count().max(1);
        let gutter_width = gutter_width_for(line_count, font_size) + if bookmarked_lines.is_empty() { 0.0 } else { font_size };
        let bookmark_set: BTreeSet<usize> = bookmarked_lines.iter().copied().collect();

        ui.horizontal_top(|ui| {
            // Gutter
            ScrollArea::vertical()
                .id_salt("gutter_scroll")
                .max_width(gutter_width)
                .max_height(available.y)
                .auto_shrink([true, false])
                .show(ui, |ui| {
                    let line_count = text.lines().count().max(1);
                    let extra = if text.ends_with('\n') { 1 } else { 0 };
                    let total_lines = line_count + extra;
                    let mut gutter_text = String::with_capacity(total_lines * 7);
                    for i in 1..=total_lines {
                        let marker = if bookmark_set.contains(&(i - 1)) { "● " } else { "  " };
                        gutter_text.push_str(&format!("{}{:>width$}\n", marker, i, width = digit_count(total_lines)));
                    }
                    ui.label(
                        RichText::new(gutter_text.trim_end())
                            .font(font.clone())
                            .color(ui.visuals().weak_text_color()),
                    );
                });

            // Editor area
            ScrollArea::both()
                .id_salt("editor_scroll")
                .max_height(available.y)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let mut editor = TextEdit::multiline(text)
                        .font(font.clone())
                        .desired_width(f32::INFINITY)
                        .code_editor()
                        .lock_focus(true)
                        .layouter(&mut layouter);
                    if !word_wrap {
                        editor = editor.desired_rows(1);
                    }
                    let response = ui.add_sized(
                        Vec2::new(ui.available_width(), ui.available_height()),
                        editor,
                    );
                    paint_search_highlights(ui, &response, text, font_size, search_matches, current_match_index);
                    if let Some(match_pos) = matching_bracket_pos {
                        paint_bracket_highlight(ui, &response, text, font_size, match_pos);
                    }
                });
        });
    } else {
        ScrollArea::both()
            .id_salt("editor_scroll_no_gutter")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut editor = TextEdit::multiline(text)
                    .font(font.clone())
                    .desired_width(f32::INFINITY)
                    .code_editor()
                    .lock_focus(true)
                    .layouter(&mut layouter);
                if !word_wrap {
                    editor = editor.desired_rows(1);
                }
                let response = ui.add_sized(
                    Vec2::new(ui.available_width(), ui.available_height()),
                    editor,
                );
                paint_search_highlights(ui, &response, text, font_size, search_matches, current_match_index);
                if let Some(match_pos) = matching_bracket_pos {
                    paint_bracket_highlight(ui, &response, text, font_size, match_pos);
                }
            });
    }
}

/// Paint colored rectangles over search match positions in the editor.
fn paint_search_highlights(
    ui: &Ui,
    response: &egui::Response,
    text: &str,
    font_size: f32,
    matches: &[SearchMatch],
    current_match_index: Option<usize>,
) {
    if matches.is_empty() {
        return;
    }

    let painter = ui.painter();
    let text_rect = response.rect;
    let char_width = font_size * 0.6;
    let line_height = font_size * 1.4;
    // Approximate text area origin (accounts for padding in code_editor)
    let text_origin = egui::pos2(text_rect.left() + 4.0, text_rect.top() + 2.0);

    let highlight_color = Color32::from_rgba_premultiplied(200, 160, 40, 80);
    let current_color = Color32::from_rgba_premultiplied(255, 140, 0, 120);

    for (i, m) in matches.iter().enumerate() {
        let col = col_in_line(text, m.start);
        let y = text_origin.y + (m.line as f32) * line_height;
        let x = text_origin.x + (col as f32) * char_width;
        let match_len = m.end - m.start;
        let w = (match_len as f32) * char_width;

        let rect = Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, line_height));

        // Only paint if within the visible area
        if rect.intersects(text_rect) {
            let color = if current_match_index == Some(i) {
                current_color
            } else {
                highlight_color
            };
            painter.rect_filled(rect, 2.0, color);
        }
    }
}

/// Compute the column offset of a byte position within its line.
fn col_in_line(text: &str, byte_offset: usize) -> usize {
    let line_start = text[..byte_offset].rfind('\n').map_or(0, |p| p + 1);
    text[line_start..byte_offset].chars().count()
}

fn digit_count(n: usize) -> usize {
    if n == 0 { return 1; }
    ((n as f64).log10().floor() as usize) + 1
}

fn gutter_width_for(line_count: usize, font_size: f32) -> f32 {
    let digits = digit_count(line_count);
    // Approximate: each digit is ~0.6 * font_size wide for monospace, plus padding
    (digits as f32) * font_size * 0.6 + 16.0
}

fn paint_bracket_highlight(
    ui: &Ui,
    response: &egui::Response,
    text: &str,
    font_size: f32,
    byte_pos: usize,
) {
    let painter = ui.painter();
    let text_rect = response.rect;
    let char_width = font_size * 0.6;
    let line_height = font_size * 1.4;
    let text_origin = egui::pos2(text_rect.left() + 4.0, text_rect.top() + 2.0);

    let line = text[..byte_pos].matches('\n').count();
    let col = col_in_line(text, byte_pos);
    let y = text_origin.y + (line as f32) * line_height;
    let x = text_origin.x + (col as f32) * char_width;
    let rect = Rect::from_min_size(egui::pos2(x, y), egui::vec2(char_width, line_height));

    if rect.intersects(text_rect) {
        painter.rect_stroke(rect, 2.0, egui::Stroke::new(2.0, Color32::from_rgb(100, 180, 255)), egui::StrokeKind::Outside);
    }
}
