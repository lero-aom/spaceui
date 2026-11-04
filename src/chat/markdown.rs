//! A compact Markdown renderer for chat replies: headings, bullet and
//! numbered lists, quotes, rules, fenced code, table-like rows separated by
//! dots, and inline **bold**, *italic*, `code` and `[links](url)`.
//! Web links support pointer and keyboard activation.
//!
//! This is a deliberately small subset, not a `CommonMark` parser. It does not
//! implement HTML, images, reference links, nested lists, syntax highlighting,
//! or aligned table columns. Inline links accept simple destinations without
//! nested parentheses. Backslash escapes are recognized; unmatched inline
//! delimiters remain visible. Raw HTML is never executed. Only HTTP(S) links open.
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
    Align2, Color32, CursorIcon, FontId, Frame, Key, Label, Modifiers, OpenUrl, Rect, RichText, Role, Sense, Stroke,
    Ui, WidgetInfo,
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
    job.wrap.max_width = ui.available_width().max(1.0);
    if links.is_empty() {
        ui.add(Label::new(job).wrap().selectable(true));
        return;
    }
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    // Pass the same galley to the label so hit rectangles match wrapped glyphs.
    let response = ui.add(Label::new(galley.clone()).selectable(true));
    let mut rectangles = vec![Vec::new(); links.len()];
    let mut row_start = 0;
    let mut first_link = 0;
    for row in &galley.rows {
        let row_end = row_start + row.glyphs.len();
        while first_link < links.len() && links[first_link].characters.end <= row_start {
            first_link += 1;
        }
        for (index, link) in links.iter().enumerate().skip(first_link) {
            let range = &link.characters;
            if range.start >= row_end {
                break;
            }
            let start = range.start.max(row_start) - row_start;
            let end = range.end.min(row_end) - row_start;
            if start < end {
                let left = row.glyphs[start].pos.x;
                let right = row.glyphs[end - 1].max_x();
                rectangles[index].push(Rect::from_min_max(
                    response.rect.min + row.pos.to_vec2() + vec2(left, 0.0),
                    response.rect.min + row.pos.to_vec2() + vec2(right, row.size.y),
                ));
            }
        }
        row_start = row_end + usize::from(row.ends_with_newline);
    }
    for (index, (link, parts)) in links.iter().zip(rectangles).enumerate() {
        let Some(bounds) = parts.iter().copied().reduce(Rect::union) else {
            continue;
        };
        let id = response.id.with(("link", index));
        let focus = ui.interact(bounds, id, Sense::focusable_noninteractive());
        focus.widget_info(|| WidgetInfo::labeled(Role::Link, ui.is_enabled(), link.label));
        ui.ctx().accesskit_node_builder(id, |node| {
            node.add_action(egui::accesskit::Action::Click);
        });
        let mut clicked = focus.has_focus()
            && ui.is_enabled()
            && ui.input_mut(|input| {
                input.consume_key(Modifiers::NONE, Key::Enter) || input.consume_key(Modifiers::NONE, Key::Space)
            });
        clicked |=
            ui.is_enabled() && ui.input(|input| input.has_accesskit_action_request(id, egui::accesskit::Action::Click));
        for (part, rect) in parts.into_iter().enumerate() {
            let hit = ui.interact(rect, id.with(part), Sense::click() - Sense::FOCUSABLE);
            if hit.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            }
            if hit.hovered() || focus.has_focus() {
                ui.painter().hline(rect.x_range(), rect.bottom(), Stroke::new(2.0, p.accent_hi));
            }
            clicked |= hit.on_hover_text(link.target).clicked();
        }
        if clicked {
            ui.ctx().open_url(OpenUrl::new_tab(link.target));
        }
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
fn inline_job_links<'a>(text: &'a str, font: &FontId, color: Color32, p: &Palette) -> (LayoutJob, Vec<Link<'a>>) {
    let mut out = Inline {
        job: LayoutJob::default(),
        links: Vec::new(),
        plain: TextFormat { font_id: font.clone(), color, ..Default::default() },
        bold_font: semibold(font.size),
        bold: false,
        italic: false,
        buffer: String::new(),
        characters: 0,
    };
    let markers = Markers::new(text);
    let mut bold_end = None;
    let mut italic_end = None;
    let mut i = 0;
    while i < text.len() {
        let rest = &text[i..];
        let ch = rest.chars().next().expect("i is a character boundary inside the text");
        if ch == '\\'
            && let Some(next) = rest[1..].chars().next()
            && next.is_ascii_punctuation()
        {
            out.buffer.push(next);
            i += 1 + next.len_utf8();
            continue;
        }
        if bold_end == Some(i) {
            out.flush();
            out.bold = false;
            bold_end = None;
            i += 2;
        } else if italic_end == Some(i) {
            out.flush();
            out.italic = false;
            italic_end = None;
            i += 1;
        } else if !out.bold
            && (rest.starts_with("**") || rest.starts_with("__"))
            && let Some(end) = markers.emphasis_end(text, i, 2)
        {
            out.flush();
            out.bold = true;
            bold_end = Some(end);
            i += 2;
        } else if !out.italic
            && (ch == '*' || ch == '_')
            && !rest.starts_with("**")
            && !rest.starts_with("__")
            && let Some(end) = markers.emphasis_end(text, i, 1)
        {
            out.flush();
            out.italic = true;
            italic_end = Some(end);
            i += 1;
        } else if ch == '`'
            && let Some(end) = markers.next(4, i)
        {
            out.flush();
            let code = &text[i + 1..end];
            let format = TextFormat {
                font_id: FontId::monospace(font.size - 1.5),
                color: p.accent_hi,
                background: p.field,
                ..Default::default()
            };
            out.job.append(code, 2.0, format);
            out.characters += code.chars().count();
            i = end + 1;
        } else if ch == '['
            && let Some((label, target, used)) = markdown_link(text, i, &markers)
        {
            out.link(label, target, p);
            i += used;
        } else if let Some(url) = bare_url(text, i) {
            i += url.len();
            out.link(url, url, p);
        } else {
            out.buffer.push(ch);
            i += ch.len_utf8();
        }
    }
    out.flush();
    (out.job, out.links)
}

/// Link text is copied into the layout job, but its metadata can borrow the input.
struct Link<'a> {
    characters: Range<usize>,
    label: &'a str,
    target: &'a str,
}

/// The inline renderer's state while it walks the text.
struct Inline<'a> {
    job: LayoutJob,
    links: Vec<Link<'a>>,
    plain: TextFormat,
    bold_font: FontId,
    bold: bool,
    italic: bool,
    buffer: String,
    characters: usize,
}

impl<'a> Inline<'a> {
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
            self.characters += self.buffer.chars().count();
            self.buffer.clear();
        }
    }

    fn link(&mut self, label: &'a str, target: &'a str, p: &Palette) {
        self.flush();
        let mut format = self.format();
        format.color = p.accent_hi;
        format.underline = Stroke::new(1.0, p.accent_hi.gamma_multiply(0.5));
        let start = self.characters;
        self.job.append(label, 0.0, format);
        self.characters += label.chars().count();
        self.links.push(Link { characters: start..self.characters, label, target });
    }
}

/// Pre-index delimiters once. Looking up an unmatched opening bracket or
/// backtick must not repeatedly scan the remaining (possibly streaming) reply.
struct Markers([Vec<usize>; 7]);

impl Markers {
    fn new(text: &str) -> Self {
        let mut result = Self(std::array::from_fn(|_| Vec::new()));
        let mut i = 0;
        while i < text.len() {
            let rest = &text[i..];
            let ch = rest.chars().next().expect("valid character boundary");
            if ch == '\\'
                && let Some(next) = rest[1..].chars().next()
                && next.is_ascii_punctuation()
            {
                i += 1 + next.len_utf8();
                continue;
            }
            let (kind, length) = match ch {
                '*' if rest.starts_with("**") => (Some(2), 2),
                '_' if rest.starts_with("__") => (Some(3), 2),
                '*' => (Some(0), 1),
                '_' => (Some(1), 1),
                '`' => (Some(4), 1),
                ']' => (Some(5), 1),
                ')' => (Some(6), 1),
                _ => (None, ch.len_utf8()),
            };
            if let Some(kind) = kind {
                result.0[kind].push(i);
            }
            i += length;
        }
        result.exclude_token_emphasis(text);
        result
    }

    /// Code and link contents are emitted as one token. A delimiter inside
    /// them cannot close surrounding emphasis: the renderer jumps over it.
    fn exclude_token_emphasis(&mut self, text: &str) {
        if self.0[..4].iter().all(Vec::is_empty) {
            return;
        }
        let mut tokens = Vec::new();
        let mut i = 0;
        while i < text.len() {
            let rest = &text[i..];
            let ch = rest.chars().next().expect("valid character boundary");
            if ch == '\\'
                && let Some(next) = rest[1..].chars().next()
                && next.is_ascii_punctuation()
            {
                i += 1 + next.len_utf8();
                continue;
            }
            let end = match ch {
                '`' => self.next(4, i).map(|end| end + 1),
                '[' => markdown_link(text, i, self).map(|(_, _, used)| i + used),
                _ => bare_url(text, i).map(|url| i + url.len()),
            };
            if let Some(end) = end {
                tokens.push(i..end);
                i = end;
            } else {
                i += ch.len_utf8();
            }
        }
        // Both lists are sorted. Each token is visited at most once per
        // delimiter kind, avoiding a suffix scan for every emphasis opener.
        for positions in &mut self.0[..4] {
            let mut token = 0;
            positions.retain(|position| {
                while token < tokens.len() && tokens[token].end <= *position {
                    token += 1;
                }
                token == tokens.len() || *position < tokens[token].start
            });
        }
    }

    fn next(&self, kind: usize, after: usize) -> Option<usize> {
        let positions = &self.0[kind];
        positions.get(positions.partition_point(|position| *position <= after)).copied()
    }

    fn emphasis_end(&self, text: &str, start: usize, length: usize) -> Option<usize> {
        let underscore = text.as_bytes()[start] == b'_';
        if underscore && text[..start].chars().next_back().is_some_and(char::is_alphanumeric) {
            return None;
        }
        let kind = usize::from(underscore) + if length == 2 { 2 } else { 0 };
        let end = self.next(kind, start)?;
        // An unmatched opener can advance into an indexed double delimiter.
        // Such overlapping delimiters are literal text, not a valid body.
        let body = text.get(start + length..end)?;
        (!body.is_empty() && !body.starts_with(char::is_whitespace) && !body.ends_with(char::is_whitespace))
            .then_some(end)
    }
}

/// A simple inline link, returning borrowed label/destination and bytes used.
fn markdown_link<'a>(text: &'a str, start: usize, markers: &Markers) -> Option<(&'a str, &'a str, usize)> {
    let close = markers.next(5, start)?;
    if text.as_bytes().get(close + 1) != Some(&b'(') {
        return None;
    }
    let label = &text[start + 1..close];
    if label.is_empty() || label.contains('[') {
        return None;
    }
    let paren = markers.next(6, close)?;
    let target = text[close + 2..paren].split_whitespace().next()?.trim_matches(['<', '>']);
    if target.contains('(') || target.chars().any(char::is_control) || !web_url(target) {
        return None;
    }
    Some((label, target, paren + 1 - start))
}

fn web_url(text: &str) -> bool {
    text.get(..8).is_some_and(|prefix| prefix.eq_ignore_ascii_case("https://"))
        || text.get(..7).is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"))
}

/// A bare web address without sentence punctuation. Balanced URL parentheses
/// are retained, unlike a closing parenthesis around the whole address.
fn bare_url(text: &str, i: usize) -> Option<&str> {
    let rest = &text[i..];
    if !web_url(rest) || text[..i].chars().next_back().is_some_and(char::is_alphanumeric) {
        return None;
    }
    let len = rest.find(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"')).unwrap_or(rest.len());
    let mut url = rest[..len].trim_end_matches(['.', ',', ';', ':', '!', '?']);
    let opens = url.bytes().filter(|byte| *byte == b'(').count();
    let mut closes = url.bytes().filter(|byte| *byte == b')').count();
    while closes > opens && url.ends_with(')') {
        url = &url[..url.len() - 1];
        closes -= 1;
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
        let targets: Vec<_> = links.iter().map(|link| link.target).collect();
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

    #[test]
    fn incomplete_markup_and_escaped_delimiters_remain_readable() {
        let p = Palette::CARBON_COPPER;
        for (source, expected) in [
            ("**incomplete", "**incomplete"),
            ("`unfinished", "`unfinished"),
            ("[unfinished", "[unfinished"),
            ("_italic_ and snake_case", "italic and snake_case"),
            (r"\*literal\*", "*literal*"),
            ("[bad](javascript:alert)", "[bad](javascript:alert)"),
        ] {
            assert_eq!(inline_job(source, &FontId::proportional(13.0), p.text, &p).text, expected);
        }
    }

    #[test]
    fn repeated_emphasis_delimiters_never_form_inverted_ranges() {
        let p = Palette::CARBON_COPPER;
        let font = FontId::proportional(13.0);
        // Minimized hosted-fuzzer input: an unmatched opener overlaps a later pair.
        let source = "**bold** _ital*****d bol_italic_ bol";
        let job = inline_job(source, &font, p.text, &p);
        assert_eq!(job.text, "bold ital*****d bolitalic_ bol");
        for delimiter in ['*', '_'] {
            for count in 1..=64 {
                let run = delimiter.to_string().repeat(count);
                assert_eq!(inline_job(&run, &font, p.text, &p).text, run);
                for source in [format!("{run}é{run}"), format!("{run} **bold** _italic_")] {
                    let job = inline_job(&source, &font, p.text, &p);
                    for section in &job.sections {
                        assert!(job.text.get(section.byte_range.start.0..section.byte_range.end.0).is_some());
                    }
                }
            }
        }
    }

    #[test]
    fn delimiters_in_code_and_links_do_not_leak_emphasis_into_trailing_text() {
        let p = Palette::CARBON_COPPER;
        let font = FontId::proportional(13.0);
        for (source, expected) in [
            ("**a `**` tail", "**a ** tail"),
            ("*a `*` tail", "*a * tail"),
            ("**a [**](https://example.org) tail", "**a ** tail"),
            ("**a [label](https://example.org/**) tail", "**a label tail"),
            ("**a https://example.org/** tail", "**a https://example.org/** tail"),
            ("**a `**` b** tail", "a ** b tail"),
            ("_a `__` b_ tail", "a __ b tail"),
        ] {
            let job = inline_job(source, &font, p.text, &p);
            assert_eq!(job.text, expected, "{source:?}");
            let tail = job.sections.last().expect("trailing text has a layout section");
            assert_eq!(tail.format.font_id, font, "{source:?}");
            assert!(!tail.format.italics, "{source:?}");
        }
    }

    #[test]
    fn unicode_link_ranges_and_balanced_url_parentheses_are_correct() {
        let p = Palette::CARBON_COPPER;
        let (job, links) = inline_job_links(
            "é [世界](https://example.org) (https://example.org/a_(b)).",
            &FontId::proportional(13.0),
            p.text,
            &p,
        );
        assert_eq!(links[0].characters, 2..4);
        assert_eq!(links[0].label, "世界");
        assert_eq!(links[1].target, "https://example.org/a_(b)");
        assert_eq!(job.text, "é 世界 (https://example.org/a_(b)).");
    }

    #[test]
    fn arbitrary_unicode_and_malformed_delimiters_keep_valid_ranges() {
        let p = Palette::CARBON_COPPER;
        let alphabet = ['[', ']', '(', ')', '*', '_', '`', '\\', 'é', '界', '🙂', '\n', ' ', 'a'];
        let mut seed = 7_u64;
        for length in 0..256 {
            let text: String = (0..length)
                .map(|_| {
                    seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                    alphabet[(seed as usize) % alphabet.len()]
                })
                .collect();
            let (job, links) = inline_job_links(&text, &FontId::proportional(13.0), p.text, &p);
            for section in &job.sections {
                assert!(job.text.get(section.byte_range.start.0..section.byte_range.end.0).is_some());
            }
            for link in links {
                assert!(link.characters.end <= job.text.chars().count());
            }
        }
        // A regression for repeated unmatched openers: no suffix is rescanned.
        let malformed = "[".repeat(50_000);
        assert_eq!(inline_job(&malformed, &FontId::proportional(13.0), p.text, &p).text, malformed);
    }
}
