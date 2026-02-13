use std::collections::BTreeSet;
use egui::{self, Color32, FontId, Layout, Rect, RichText, ScrollArea, TextEdit, TextFormat, Ui, Vec2};
use egui::text::LayoutJob;

use crate::editor::column_select::ColumnSelection;
use crate::editor::syntax::SyntaxHighlighter;
use crate::search::SearchMatch;

/// Action triggered from the editor's right-click context menu.
#[derive(Debug, Clone, PartialEq)]
pub enum ContextMenuAction {
    None,
    ToggleComment,
    Uppercase,
    Lowercase,
}

/// Output returned from `editor_widget()` including cursor information.
pub struct EditorWidgetOutput {
    pub ctx_action: ContextMenuAction,
    pub cursor_line: usize,
    pub cursor_col: usize,
}

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
    extra_cursors: &[usize],
    extra_selections: &[(usize, usize)],
    column_selection: &ColumnSelection,
    layout_cache: &mut Option<(u64, LayoutJob)>,
) -> EditorWidgetOutput {
    let font = FontId::monospace(font_size);
    let available = ui.available_size();
    let default_color = ui.visuals().text_color();
    let mut ctx_action = ContextMenuAction::None;
    let mut cursor_line: usize = 0;
    let mut cursor_col: usize = 0;

    let ext = file_extension.to_string();
    let hl = highlighter;
    let f = font.clone();
    let dc = default_color;

    // Compute a content hash for layout caching
    let content_hash = {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.len().hash(&mut hasher);
        ext.hash(&mut hasher);
        // Hash a sample of the text for fast change detection
        let sample_end = 1000.min(text.len());
        text.get(..sample_end).hash(&mut hasher);
        if text.len() > 1000 {
            let tail_start = text.len().saturating_sub(1000);
            text.get(tail_start..).hash(&mut hasher);
        }
        hasher.finish()
    };

    // Regenerate layout cache if content changed
    if layout_cache.as_ref().map_or(true, |(h, _)| *h != content_hash) {
        let job = highlight_text(&text, &ext, hl, f.clone(), dc);
        *layout_cache = Some((content_hash, job));
    }

    let cached_job = layout_cache.as_ref().map(|(_, job)| job.clone());
    let text_len = text.len();
    let mut layouter = |ui: &egui::Ui, s: &str, _wrap_width: f32| {
        if let Some(ref job) = cached_job {
            if s.len() == text_len {
                return ui.fonts(|fonts| fonts.layout_job(job.clone()));
            }
        }
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
                    let te_output = ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), ui.available_height()),
                        Layout::centered_and_justified(ui.layout().main_dir()),
                        |ui| editor.show(ui),
                    ).inner;
                    let response = te_output.response;
                    if let Some(cursor_range) = te_output.cursor_range {
                        let pcursor = cursor_range.primary.pcursor;
                        cursor_line = pcursor.paragraph;
                        cursor_col = pcursor.offset;
                    }
                    paint_search_highlights(ui, &response, text, font_size, search_matches, current_match_index);
                    if let Some(match_pos) = matching_bracket_pos {
                        paint_bracket_highlight(ui, &response, text, font_size, match_pos);
                    }
                    paint_extra_cursors(ui, &response, text, font_size, extra_cursors);
                    paint_extra_selections(ui, &response, text, font_size, extra_selections);
                    paint_column_selection(ui, &response, font_size, column_selection);
                    ctx_action = show_editor_context_menu(&response);
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
                let te_output = ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), ui.available_height()),
                    Layout::centered_and_justified(ui.layout().main_dir()),
                    |ui| editor.show(ui),
                ).inner;
                let response = te_output.response;
                if let Some(cursor_range) = te_output.cursor_range {
                    let pcursor = cursor_range.primary.pcursor;
                    cursor_line = pcursor.paragraph;
                    cursor_col = pcursor.offset;
                }
                paint_search_highlights(ui, &response, text, font_size, search_matches, current_match_index);
                if let Some(match_pos) = matching_bracket_pos {
                    paint_bracket_highlight(ui, &response, text, font_size, match_pos);
                }
                paint_extra_cursors(ui, &response, text, font_size, extra_cursors);
                paint_extra_selections(ui, &response, text, font_size, extra_selections);
                paint_column_selection(ui, &response, font_size, column_selection);
                ctx_action = show_editor_context_menu(&response);
            });
    }

    EditorWidgetOutput {
        ctx_action,
        cursor_line,
        cursor_col,
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

/// Paint colored cursor lines at each extra cursor byte position.
fn paint_extra_cursors(
    ui: &Ui,
    response: &egui::Response,
    text: &str,
    font_size: f32,
    cursor_offsets: &[usize],
) {
    if cursor_offsets.is_empty() {
        return;
    }

    let painter = ui.painter();
    let text_rect = response.rect;
    let char_width = font_size * 0.6;
    let line_height = font_size * 1.4;
    let text_origin = egui::pos2(text_rect.left() + 4.0, text_rect.top() + 2.0);
    let cursor_color = Color32::from_rgba_premultiplied(255, 200, 50, 200);

    for &byte_pos in cursor_offsets {
        let clamped = byte_pos.min(text.len());
        let line = text[..clamped].matches('\n').count();
        let col = col_in_line(text, clamped);
        let y = text_origin.y + (line as f32) * line_height;
        let x = text_origin.x + (col as f32) * char_width;

        let cursor_rect = Rect::from_min_size(egui::pos2(x - 1.0, y), egui::vec2(2.0, line_height));
        if cursor_rect.intersects(text_rect) {
            painter.rect_filled(cursor_rect, 0.0, cursor_color);
        }
    }
}

/// Paint colored rectangles over extra cursor selections.
fn paint_extra_selections(
    ui: &Ui,
    response: &egui::Response,
    text: &str,
    font_size: f32,
    selections: &[(usize, usize)],
) {
    if selections.is_empty() {
        return;
    }

    let painter = ui.painter();
    let text_rect = response.rect;
    let char_width = font_size * 0.6;
    let line_height = font_size * 1.4;
    let text_origin = egui::pos2(text_rect.left() + 4.0, text_rect.top() + 2.0);
    let sel_color = Color32::from_rgba_premultiplied(100, 160, 255, 60);

    for &(start, end) in selections {
        let clamped_start = start.min(text.len());
        let clamped_end = end.min(text.len());
        let start_line = text[..clamped_start].matches('\n').count();
        let end_line = text[..clamped_end].matches('\n').count();

        if start_line == end_line {
            let col_start = col_in_line(text, clamped_start);
            let col_end = col_in_line(text, clamped_end);
            let y = text_origin.y + (start_line as f32) * line_height;
            let x = text_origin.x + (col_start as f32) * char_width;
            let w = ((col_end - col_start) as f32) * char_width;
            let rect = Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, line_height));
            if rect.intersects(text_rect) {
                painter.rect_filled(rect, 1.0, sel_color);
            }
        } else {
            for line in start_line..=end_line {
                let (col_start, col_end) = if line == start_line {
                    let cs = col_in_line(text, clamped_start);
                    let line_end_byte = text[clamped_start..].find('\n').map(|p| clamped_start + p).unwrap_or(text.len());
                    let ce = col_in_line(text, line_end_byte);
                    (cs, ce)
                } else if line == end_line {
                    (0, col_in_line(text, clamped_end))
                } else {
                    (0, 80)
                };
                let y = text_origin.y + (line as f32) * line_height;
                let x = text_origin.x + (col_start as f32) * char_width;
                let w = ((col_end - col_start).max(1) as f32) * char_width;
                let rect = Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, line_height));
                if rect.intersects(text_rect) {
                    painter.rect_filled(rect, 1.0, sel_color);
                }
            }
        }
    }
}

/// Paint a semi-transparent rectangle overlay for column (rectangular) selection.
fn paint_column_selection(
    ui: &Ui,
    response: &egui::Response,
    font_size: f32,
    col_sel: &ColumnSelection,
) {
    if !col_sel.active {
        return;
    }

    let painter = ui.painter();
    let text_rect = response.rect;
    let char_width = font_size * 0.6;
    let line_height = font_size * 1.4;
    let text_origin = egui::pos2(text_rect.left() + 4.0, text_rect.top() + 2.0);
    let sel_color = Color32::from_rgba_premultiplied(80, 140, 220, 80);

    let (min_row, max_row) = col_sel.rows();
    let (min_col, max_col) = col_sel.cols();

    for row in min_row..=max_row {
        let y = text_origin.y + (row as f32) * line_height;
        let x = text_origin.x + (min_col as f32) * char_width;
        let w = ((max_col - min_col).max(1) as f32) * char_width;
        let rect = Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, line_height));
        if rect.intersects(text_rect) {
            painter.rect_filled(rect, 1.0, sel_color);
        }
    }
}

/// Show the right-click context menu on the editor and return an action if selected.
fn show_editor_context_menu(response: &egui::Response) -> ContextMenuAction {
    let mut action = ContextMenuAction::None;
    response.context_menu(|ui| {
        if ui.button("Cut               Ctrl+X").clicked() {
            ui.ctx().input_mut(|i| {
                i.events.push(egui::Event::Cut);
            });
            ui.close_menu();
        }
        if ui.button("Copy              Ctrl+C").clicked() {
            ui.ctx().input_mut(|i| {
                i.events.push(egui::Event::Copy);
            });
            ui.close_menu();
        }
        if ui.button("Paste             Ctrl+V").clicked() {
            ui.ctx().input_mut(|i| {
                i.events.push(egui::Event::Paste(String::new()));
            });
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Select All        Ctrl+A").clicked() {
            ui.ctx().input_mut(|i| {
                i.events.push(egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::CTRL,
                });
            });
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Toggle Comment    Ctrl+/").clicked() {
            action = ContextMenuAction::ToggleComment;
            ui.close_menu();
        }
        if ui.button("UPPERCASE      Ctrl+Shift+U").clicked() {
            action = ContextMenuAction::Uppercase;
            ui.close_menu();
        }
        if ui.button("lowercase         Ctrl+U").clicked() {
            action = ContextMenuAction::Lowercase;
            ui.close_menu();
        }
    });
    action
}
