//! A compact Markdown renderer for chat replies: headings, bullet and
//! numbered lists, quotes, rules, fenced code, tables (as aligned rows) and
//! inline **bold**, *italic*, `code` and `[links](url)`. Clicked web links
//! open in the browser.
//!
//! ```
//! # use spaceui::{Theme, egui, chat::markdown};
//! # let ctx = egui::Context::default();
//! # Theme::carbon_copper().install(&ctx);
//! # ctx.run_ui(Default::default(), |ui| {
//! markdown::render(ui, "### Plan\n- Read **SPY** first\n- Then [the wiki](https://example.com)");
//! # }).drop_without_applying_deltas();
//! ```
use crate::{
    Theme,
    theme::{Palette, semibold},
};
use egui::{
    Align2, Color32, CursorIcon, FontId, Frame, Label, OpenUrl, RichText, Sense, Stroke, Ui,
    text::{LayoutJob, TextFormat},
    vec2,
};
use std::ops::Range;

const BODY: f32 = 13.5;
const HEADINGS: [f32; 3] = [17.5, 16.0, 14.5];
const CODE: f32 = 12.0;

/// Render `md` into the `Ui`, top to bottom.
pub fn render(ui: &mut Ui, md: &str) {
    let p = Theme::of(ui.ctx()).palette;
    let mut code: Option<String> = None;
    for raw in md.lines() {
        let line = raw.trim_end();
        if line.trim_start().starts_with("```") {
            match code.take() {
                Some(block) => code_block(ui, &block, &p),
                None => code = Some(String::new()),
            }
            continue;
        }
        if let Some(block) = code.as_mut() {
            block.push_str(raw);
            block.push('\n');
            continue;
        }
        let trimmed = line.trim_start();
        let indent = (line.len() - trimmed.len()) as f32 * 4.0;
        if trimmed.is_empty() {
            ui.add_space(4.0);
        } else if let Some((level, text)) = heading(trimmed) {
            ui.add_space(4.0);
            paragraph(ui, text, &semibold(HEADINGS[(level - 1).min(2)]), p.text, &p);
        } else if is_rule(trimmed) {
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 9.0), Sense::hover());
            ui.painter().hline(rect.x_range(), rect.center().y, Stroke::new(1.0, p.border));
        } else if let Some(text) = bullet(trimmed) {
            marker_line(ui, indent, "•", text, &p);
        } else if let Some((num, text)) = numbered(trimmed) {
            marker_line(ui, indent, &format!("{num}."), text, &p);
        } else if let Some(text) = trimmed.strip_prefix('>') {
            ui.horizontal_top(|ui| {
                let (bar, _) = ui.allocate_exact_size(vec2(3.0, BODY * 1.3), Sense::hover());
                ui.painter().rect_filled(bar, 2.0, p.accent.gamma_multiply(0.7));
                paragraph(ui, text.trim(), &FontId::proportional(BODY), p.text_2, &p);
            });
        } else if trimmed.starts_with('|') {
            if !trimmed.chars().all(|ch| matches!(ch, '|' | '-' | ':' | ' ')) {
                let cells: Vec<&str> = trimmed.trim_matches('|').split('|').map(str::trim).collect();
                paragraph(ui, &cells.join("  ·  "), &FontId::proportional(BODY - 0.5), p.text, &p);
            }
        } else {
            paragraph(ui, trimmed, &FontId::proportional(BODY), p.text, &p);
        }
    }
    if let Some(block) = code {
        code_block(ui, &block, &p);
    }
}

fn heading(line: &str) -> Option<(usize, &str)> {
    let level = line.chars().take_while(|ch| *ch == '#').count();
    (1..=6).contains(&level).then(|| line[level..].strip_prefix(' ')).flatten().map(|t| (level, t))
}

fn is_rule(line: &str) -> bool {
    line.len() >= 3 && (line.chars().all(|ch| ch == '-') || line.chars().all(|ch| ch == '*'))
}

fn bullet(line: &str) -> Option<&str> {
    ["- ", "* ", "• ", "+ "].iter().find_map(|p| line.strip_prefix(p))
}

fn numbered(line: &str) -> Option<(&str, &str)> {
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    if digits == 0 || digits > 3 {
        return None;
    }
    let rest = &line[digits..];
    rest.strip_prefix(". ").or_else(|| rest.strip_prefix(") ")).map(|t| (&line[..digits], t))
}

fn marker_line(ui: &mut Ui, indent: f32, marker: &str, text: &str, p: &Palette) {
    ui.horizontal_top(|ui| {
        ui.add_space(indent + 2.0);
        let w = if marker == "•" { BODY * 0.9 } else { BODY * 1.5 };
        let (rect, _) = ui.allocate_exact_size(vec2(w, BODY * 1.36), Sense::hover());
        ui.painter().text(rect.left_center(), Align2::LEFT_CENTER, marker, semibold(BODY), p.accent_hi);
        paragraph(ui, text, &FontId::proportional(BODY), p.text, p);
    });
}

/// One paragraph; a click on a web link opens it in the browser.
fn paragraph(ui: &mut Ui, text: &str, font: &FontId, color: Color32, p: &Palette) {
    let (mut job, links) = inline_job_links(text, font, color, p);
    job.wrap.max_width = ui.available_width();
    if links.is_empty() {
        ui.add(Label::new(job).wrap().selectable(true));
        return;
    }
    // The label lays the job out the same way, so this copy tells which
    // character is under the pointer.
    let galley = ui.fonts_mut(|f| f.layout_job(job.clone()));
    let response = ui.add(Label::new(job).wrap().selectable(true));
    let Some(pointer) = response.hover_pos() else {
        return;
    };
    let index = galley.cursor_from_pos(pointer - response.rect.min).index.0;
    let Some((_, target)) =
        links.iter().find(|(range, _)| range.start <= index && index < range.end.max(range.start + 1))
    else {
        return;
    };
    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    let clicked = response.clicked();
    response.on_hover_text_at_pointer(target.as_str());
    if clicked && (target.starts_with("https://") || target.starts_with("http://")) {
        ui.ctx().open_url(OpenUrl::new_tab(target));
    }
}

fn code_block(ui: &mut Ui, code: &str, p: &Palette) {
    Frame::new().fill(p.well).stroke(Stroke::new(1.0, p.border_soft)).corner_radius(8).inner_margin(10).show(
        ui,
        |ui| {
            ui.set_width(ui.available_width());
            ui.add(
                Label::new(RichText::new(code.trim_end()).font(FontId::monospace(CODE)).color(p.accent_hi))
                    .wrap()
                    .selectable(true),
            );
        },
    );
}

/// Inline Markdown (bold, italic, code, links) as one `LayoutJob`, for
/// custom layouts such as chat bubbles.
#[must_use]
pub fn inline_job(text: &str, font: &FontId, color: Color32, palette: &Palette) -> LayoutJob {
    inline_job_links(text, font, color, palette).0
}

/// Like [`inline_job`], also returning each link's character range and target.
fn inline_job_links(
    text: &str,
    font: &FontId,
    color: Color32,
    p: &Palette,
) -> (LayoutJob, Vec<(Range<usize>, String)>) {
    let mut out = Inline {
        job: LayoutJob::default(),
        links: Vec::new(),
        plain: TextFormat { font_id: font.clone(), color, ..Default::default() },
        bold_font: semibold(font.size),
        bold: false,
        italic: false,
        buffer: String::new(),
    };
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        let next = chars.get(i + 1).copied();
        if ch == '*' && next == Some('*') || ch == '_' && next == Some('_') {
            out.flush();
            out.bold = !out.bold;
            i += 2;
        } else if (ch == '*' || ch == '_')
            && (out.italic || next.is_some_and(|n| !n.is_whitespace()))
            && (ch == '*' || i == 0 || !chars[i - 1].is_alphanumeric())
        {
            out.flush();
            out.italic = !out.italic;
            i += 1;
        } else if ch == '`'
            && let Some(end) = chars[i + 1..].iter().position(|c| *c == '`')
        {
            out.flush();
            let code: String = chars[i + 1..i + 1 + end].iter().collect();
            let format = TextFormat {
                font_id: FontId::monospace(font.size - 1.5),
                color: p.accent_hi,
                background: p.field,
                ..Default::default()
            };
            out.job.append(&code, 2.0, format);
            i += end + 2;
        } else if let Some((label, target, used)) = markdown_link(&chars[i..]) {
            out.link(&label, target, p);
            i += used;
        } else if let Some(url) = bare_url(&chars, i) {
            i += url.chars().count();
            out.link(&url.clone(), url, p);
        } else {
            out.buffer.push(ch);
            i += 1;
        }
    }
    out.flush();
    (out.job, out.links)
}

/// The inline renderer's state while it walks the text.
struct Inline {
    job: LayoutJob,
    links: Vec<(Range<usize>, String)>,
    plain: TextFormat,
    bold_font: FontId,
    bold: bool,
    italic: bool,
    buffer: String,
}

impl Inline {
    fn format(&self) -> TextFormat {
        let mut format = self.plain.clone();
        if self.bold {
            format.font_id = self.bold_font.clone();
        }
        format.italics = self.italic;
        format
    }

    /// Appends the pending plain text in the current style.
    fn flush(&mut self) {
        if !self.buffer.is_empty() {
            let format = self.format();
            self.job.append(&self.buffer, 0.0, format);
            self.buffer.clear();
        }
    }

    fn link(&mut self, label: &str, target: String, p: &Palette) {
        self.flush();
        let mut format = self.format();
        format.color = p.accent_hi;
        format.underline = Stroke::new(1.0, p.accent_hi.gamma_multiply(0.5));
        let start = self.job.text.chars().count();
        self.job.append(label, 0.0, format);
        self.links.push((start..start + label.chars().count(), target));
    }
}

/// `[label](target "optional title")` at the start of `chars`: the label,
/// the target and the characters used.
fn markdown_link(chars: &[char]) -> Option<(String, String, usize)> {
    if chars.first() != Some(&'[') {
        return None;
    }
    let close = chars.iter().position(|c| *c == ']')?;
    if chars.get(close + 1) != Some(&'(') {
        return None;
    }
    let paren = chars[close + 1..].iter().position(|c| *c == ')')?;
    let label: String = chars[1..close].iter().collect();
    let inside: String = chars[close + 2..close + 1 + paren].iter().collect();
    let target = inside.split_whitespace().next().unwrap_or_default().trim_matches(['<', '>']).to_owned();
    Some((label, target, close + paren + 2))
}

/// A bare web address starting at `chars[i]`, as GitHub links them, without
/// a sentence's closing punctuation.
fn bare_url(chars: &[char], i: usize) -> Option<String> {
    let rest: String = chars[i..].iter().take(8).collect();
    let starts = rest.starts_with("https://") || rest.starts_with("http://");
    if !starts || (i > 0 && chars[i - 1].is_alphanumeric()) {
        return None;
    }
    let len =
        chars[i..].iter().position(|c| c.is_whitespace() || matches!(c, '<' | '>' | '"')).unwrap_or(chars.len() - i);
    let mut url: String = chars[i..i + len].iter().collect();
    while url.ends_with(['.', ',', ';', ':', '!', '?', ')']) {
        url.pop();
    }
    Some(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_markup_becomes_formats_and_links() {
        let p = Palette::CARBON_COPPER;
        let (job, links) = inline_job_links(
            "**Long** NVDA, see [wiki](https://example.com/w) or https://x.com/a.",
            &FontId::proportional(13.0),
            p.text,
            &p,
        );
        assert_eq!(job.text, "Long NVDA, see wiki or https://x.com/a.");
        assert_eq!(job.sections[0].format.font_id, semibold(13.0));
        let targets: Vec<_> = links.iter().map(|(_, t)| t.as_str()).collect();
        assert_eq!(targets, ["https://example.com/w", "https://x.com/a"]);
    }

    #[test]
    fn block_markers_are_recognised() {
        assert_eq!(heading("## Plan"), Some((2, "Plan")));
        assert_eq!(heading("#hashtag"), None);
        assert_eq!(numbered("12. Item"), Some(("12", "Item")));
        assert_eq!(bullet("- item"), Some("item"));
        assert!(is_rule("---") && !is_rule("--"));
    }
}
