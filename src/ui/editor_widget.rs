use egui::{self, FontId, RichText, ScrollArea, TextEdit, Ui, Vec2};

/// Renders the editor widget with optional line numbers and word wrap.
/// Operates on a `&mut String` that is synced with the active document's buffer.
pub fn editor_widget(
    ui: &mut Ui,
    text: &mut String,
    font_size: f32,
    show_line_numbers: bool,
    word_wrap: bool,
) {
    let font = FontId::monospace(font_size);
    let available = ui.available_size();

    if show_line_numbers {
        // Layout: line numbers gutter + editor
        let line_count = text.lines().count().max(1);
        let gutter_width = gutter_width_for(line_count, font_size);

        ui.horizontal_top(|ui| {
            // Gutter
            ScrollArea::vertical()
                .id_salt("gutter_scroll")
                .max_width(gutter_width)
                .max_height(available.y)
                .auto_shrink([true, false])
                .show(ui, |ui| {
                    let line_count = text.lines().count().max(1);
                    // Account for trailing newline
                    let extra = if text.ends_with('\n') { 1 } else { 0 };
                    let total_lines = line_count + extra;
                    let mut gutter_text = String::with_capacity(total_lines * 5);
                    for i in 1..=total_lines {
                        gutter_text.push_str(&format!("{:>width$}\n", i, width = digit_count(total_lines)));
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
                        .lock_focus(true);
                    if !word_wrap {
                        editor = editor.desired_rows(1);
                    }
                    ui.add_sized(
                        Vec2::new(ui.available_width(), ui.available_height()),
                        editor,
                    );
                });
        });
    } else {
        // No gutter, just the editor
        ScrollArea::both()
            .id_salt("editor_scroll_no_gutter")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut editor = TextEdit::multiline(text)
                    .font(font)
                    .desired_width(f32::INFINITY)
                    .code_editor()
                    .lock_focus(true);
                if !word_wrap {
                    editor = editor.desired_rows(1);
                }
                ui.add_sized(
                    Vec2::new(ui.available_width(), ui.available_height()),
                    editor,
                );
            });
    }
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
