use egui::{self, Color32, RichText, Ui};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

/// Render markdown text as rich egui widgets.
pub fn render_markdown(ui: &mut Ui, markdown_text: &str) {
    let options = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES;
    let parser = Parser::new_ext(markdown_text, options);

    let mut state = RenderState::default();

    for event in parser {
        match event {
            Event::Start(tag) => handle_start_tag(&mut state, &tag),
            Event::End(tag_end) => handle_end_tag(ui, &mut state, &tag_end),
            Event::Text(text) => state.push_text(&text),
            Event::Code(code) => {
                state.spans.push(Span::Code(code.to_string()));
            }
            Event::SoftBreak => state.push_text(" "),
            Event::HardBreak => {
                flush_spans(ui, &mut state);
            }
            Event::Rule => {
                flush_spans(ui, &mut state);
                ui.separator();
            }
            _ => {}
        }
    }
    flush_spans(ui, &mut state);
}

#[derive(Default)]
struct RenderState {
    spans: Vec<Span>,
    bold: bool,
    italic: bool,
    heading: Option<HeadingLevel>,
    in_code_block: bool,
    code_block_text: String,
    in_blockquote: bool,
    in_list: bool,
    list_number: Option<u64>,
    list_item_started: bool,
    link_url: Option<String>,
}

enum Span {
    Text(String),
    Code(String),
}

impl RenderState {
    fn push_text(&mut self, text: &str) {
        if self.in_code_block {
            self.code_block_text.push_str(text);
        } else {
            self.spans.push(Span::Text(text.to_string()));
        }
    }
}

fn handle_start_tag(state: &mut RenderState, tag: &Tag) {
    match tag {
        Tag::Heading { level, .. } => {
            state.heading = Some(*level);
        }
        Tag::Strong => state.bold = true,
        Tag::Emphasis => state.italic = true,
        Tag::CodeBlock(_) => {
            state.in_code_block = true;
            state.code_block_text.clear();
        }
        Tag::BlockQuote(_) => state.in_blockquote = true,
        Tag::List(start) => {
            state.in_list = true;
            state.list_number = *start;
        }
        Tag::Item => {
            state.list_item_started = true;
        }
        Tag::Link { dest_url, .. } => {
            state.link_url = Some(dest_url.to_string());
        }
        Tag::Paragraph => {}
        _ => {}
    }
}

fn handle_end_tag(ui: &mut Ui, state: &mut RenderState, tag_end: &TagEnd) {
    match tag_end {
        TagEnd::Heading(_) => {
            let level = state.heading.take();
            let text = take_span_text(state);
            let size = match level {
                Some(HeadingLevel::H1) => 28.0,
                Some(HeadingLevel::H2) => 24.0,
                Some(HeadingLevel::H3) => 20.0,
                _ => 18.0,
            };
            ui.label(RichText::new(&text).size(size).strong());
            ui.add_space(4.0);
        }
        TagEnd::Paragraph => {
            if state.in_blockquote {
                let text = take_span_text(state);
                render_blockquote(ui, &text);
            } else {
                flush_spans(ui, state);
            }
            ui.add_space(6.0);
        }
        TagEnd::Strong => state.bold = false,
        TagEnd::Emphasis => state.italic = false,
        TagEnd::CodeBlock => {
            state.in_code_block = false;
            let code = std::mem::take(&mut state.code_block_text);
            render_code_block(ui, &code);
        }
        TagEnd::BlockQuote(_) => {
            state.in_blockquote = false;
        }
        TagEnd::List(_) => {
            state.in_list = false;
            state.list_number = None;
        }
        TagEnd::Item => {
            let text = take_span_text(state);
            let prefix = if let Some(ref mut n) = state.list_number {
                let p = format!("{}. ", n);
                *n += 1;
                p
            } else {
                "• ".to_string()
            };
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(format!("{}{}", prefix, text));
            });
        }
        TagEnd::Link => {
            let url = state.link_url.take().unwrap_or_default();
            let text = take_span_text(state);
            let label = if text.is_empty() { url.clone() } else { text };
            if ui.link(&label).clicked() {
                let _ = open::that(&url);
            }
        }
        _ => {}
    }
}

fn take_span_text(state: &mut RenderState) -> String {
    let mut result = String::new();
    for span in state.spans.drain(..) {
        match span {
            Span::Text(t) => result.push_str(&t),
            Span::Code(c) => {
                result.push('`');
                result.push_str(&c);
                result.push('`');
            }
        }
    }
    result
}

fn flush_spans(ui: &mut Ui, state: &mut RenderState) {
    if state.spans.is_empty() {
        return;
    }

    ui.horizontal_wrapped(|ui| {
        for span in state.spans.drain(..) {
            match span {
                Span::Text(text) => {
                    let mut rt = RichText::new(&text);
                    if state.bold {
                        rt = rt.strong();
                    }
                    if state.italic {
                        rt = rt.italics();
                    }
                    if state.heading.is_some() {
                        rt = rt.strong().size(20.0);
                    }
                    ui.label(rt);
                }
                Span::Code(code) => {
                    ui.label(
                        RichText::new(&code)
                            .monospace()
                            .background_color(Color32::from_gray(40)),
                    );
                }
            }
        }
    });
}

fn render_code_block(ui: &mut Ui, code: &str) {
    let frame = egui::Frame::new()
        .fill(Color32::from_gray(30))
        .inner_margin(8.0)
        .corner_radius(4.0);
    frame.show(ui, |ui| {
        ui.label(RichText::new(code).monospace());
    });
    ui.add_space(4.0);
}

fn render_blockquote(ui: &mut Ui, text: &str) {
    ui.horizontal(|ui| {
        let rect = ui.available_rect_before_wrap();
        let bar_rect = egui::Rect::from_min_size(
            rect.min,
            egui::vec2(3.0, ui.spacing().interact_size.y),
        );
        ui.painter()
            .rect_filled(bar_rect, 0.0, Color32::from_gray(128));
        ui.add_space(12.0);
        ui.label(RichText::new(text).italics().color(Color32::from_gray(180)));
    });
}
