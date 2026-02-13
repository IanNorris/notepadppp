use crate::editor::syntax::SyntaxHighlighter;

/// Export text as a complete HTML document with syntax highlighting.
pub fn export_html(text: &str, extension: &str, highlighter: &SyntaxHighlighter) -> String {
    let mut body = String::new();
    for line in text.lines() {
        let spans = highlighter.highlight_line(&format!("{}\n", line), extension);
        for (style, segment) in spans {
            let fg = style.foreground;
            let escaped = html_escape(&segment);
            body.push_str(&format!(
                "<span style=\"color:#{:02x}{:02x}{:02x}\">{}</span>",
                fg.r, fg.g, fg.b, escaped
            ));
        }
        body.push_str("<br>\n");
    }

    format!(
        "<!DOCTYPE html>\n\
         <html>\n\
         <head>\n\
         <meta charset=\"UTF-8\">\n\
         <title>Exported from Notepad+++</title>\n\
         <style>\n\
         body {{ font-family: monospace; white-space: pre-wrap; background: #2b303b; color: #c0c5ce; padding: 1em; }}\n\
         </style>\n\
         </head>\n\
         <body>\n\
         {}\
         </body>\n\
         </html>",
        body
    )
}

/// Export text as RTF with basic formatting.
pub fn export_rtf(text: &str) -> String {
    let mut rtf = String::from("{\\rtf1\\ansi\\deff0{\\fonttbl{\\f0 Courier New;}}\\f0\\fs24\n");
    for line in text.lines() {
        rtf.push_str(&rtf_escape(line));
        rtf.push_str("\\par\n");
    }
    rtf.push('}');
    rtf
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn rtf_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            c if c.is_ascii() => out.push(c),
            c => {
                // RTF unicode escape
                out.push_str(&format!("\\u{}?", c as i32));
            }
        }
    }
    out
}
