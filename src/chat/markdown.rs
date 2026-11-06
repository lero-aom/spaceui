//! `CommonMark` rendering for chat replies and user bubbles, powered by
//! [`egui_commonmark`]. Parsing, layout, code highlighting and link interaction
//! are delegated to its viewer. Plain, fragmented paragraphs are coalesced
//! after validation by the same `CommonMark` parser. [`Markdown`] also offers an
//! explicit rendering budget for applications that accept a plain-source fallback.
//!
//! [`render`] keeps a shared cache in the egui context. Use [`render_with_cache`]
//! with an application-owned [`CommonMarkCache`] to configure link hooks or
//! manage its lifetime yourself. Image support uses the context's image loaders;
//! SpaceUI does not enable file or network loaders.
//!
//! ```
//! # use spaceui::{Theme, egui, chat::markdown};
//! # let ctx = egui::Context::default();
//! # Theme::carbon_copper().install(&ctx);
//! # ctx.run_ui(Default::default(), |ui| {
//! markdown::render(ui, "### Plan\n- Observe **M31** first\n- Then [the field guide](https://example.com)");
//! # }).drop_without_applying_deltas();
//! ```

pub use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::Theme;
use egui::{Context, FontId, Id, Response, TextStyle, Ui, Widget};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

type SharedCache = Arc<Mutex<CommonMarkCache>>;

fn cache_id() -> Id {
    Id::unique("spaceui-commonmark-cache")
}

/// A themed Markdown widget. Existing [`render`] helpers use this implementation.
///
/// By default all Markdown is rendered without a content limit. A single plain
/// paragraph with many text fragments is merged into one label. Formatting,
/// links, code, multiple paragraphs and line breaks stay with the upstream viewer.
///
/// ```
/// # use spaceui::{egui, chat::markdown::Markdown};
/// # let ctx = egui::Context::default();
/// # ctx.run_ui(Default::default(), |ui| {
/// let shown = Markdown::new("**Hello**").max_events(4096).show(ui);
/// if shown.fallback { /* Offer a full-document view if desired. */ }
/// # }).drop_without_applying_deltas();
/// ```
#[derive(Clone, Copy, Debug)]
#[must_use = "show the Markdown or add it to a Ui"]
pub struct Markdown<'a> {
    source: &'a str,
    max_events: Option<usize>,
}

/// The interaction response and whether an explicitly requested budget was exceeded.
#[derive(Debug)]
pub struct MarkdownResponse {
    /// The viewer or plain label's response.
    pub response: Response,
    /// True when the unmodified source was displayed as plain text because the
    /// event budget was exceeded. Nothing is truncated, but markup and links
    /// are not interpreted in this mode.
    pub fallback: bool,
}

impl<'a> Markdown<'a> {
    /// Render a borrowed `CommonMark` document without a rendering budget.
    pub fn new(source: &'a str) -> Self {
        Self { source, max_events: None }
    }

    /// Limit the number of parser events passed to the rich renderer. When the
    /// limit is exceeded, show the complete original source as one plain label.
    /// Zero permits only documents with no events. This bounds rich widget work,
    /// not parser complexity, source size, text layout or elapsed frame time.
    /// Applications must separately limit incoming document sizes.
    pub fn max_events(mut self, limit: usize) -> Self {
        self.max_events = Some(limit);
        self
    }

    /// Show using the shared context cache. Inspect [`MarkdownResponse::fallback`]
    /// to distinguish rich rendering from a budget fallback.
    pub fn show(self, ui: &mut Ui) -> MarkdownResponse {
        let cache = ui.ctx().data_mut(|data| data.get_temp_mut_or_default::<SharedCache>(cache_id()).clone());
        let mut cache = cache.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        self.show_with_cache(ui, &mut cache)
    }

    /// Show using an application-owned cache, including its link hooks.
    pub fn show_with_cache(self, ui: &mut Ui, cache: &mut CommonMarkCache) -> MarkdownResponse {
        let plan = cached_plan(ui.ctx(), self.source, self.max_events);
        let palette = Theme::of(ui.ctx()).palette;
        ui.scope(|ui| {
            let style = ui.style_mut();
            style.text_styles.insert(TextStyle::Body, FontId::proportional(13.5));
            style.text_styles.insert(TextStyle::Monospace, FontId::monospace(12.0));
            style.visuals.override_text_color = Some(palette.text);
            style.visuals.hyperlink_color = palette.accent_hi;
            style.visuals.code_bg_color = palette.field;
            style.visuals.extreme_bg_color = palette.well;
            match plan {
                Plan::Rich => MarkdownResponse {
                    response: CommonMarkViewer::new().show(ui, cache, self.source).response,
                    fallback: false,
                },
                Plan::Paragraph(text) => {
                    MarkdownResponse { response: ui.add(egui::Label::new(text.as_ref()).wrap()), fallback: false }
                }
                Plan::Fallback => {
                    MarkdownResponse { response: ui.add(egui::Label::new(self.source).wrap()), fallback: true }
                }
            }
        })
        .inner
    }
}

impl Widget for Markdown<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

/// Render `CommonMark` using a cache shared by the viewers in this egui context.
pub fn render(ui: &mut Ui, markdown: &str) {
    Markdown::new(markdown).show(ui);
}

/// Render `CommonMark` with application-owned cache state and SpaceUI's theme.
///
/// Link destinations follow `egui_commonmark`/egui navigation behavior. Use
/// [`CommonMarkCache::add_link_hook`] or handle egui's output commands when the
/// application needs its own navigation policy. HTML is displayed as text.
pub fn render_with_cache(ui: &mut Ui, cache: &mut CommonMarkCache, markdown: &str) {
    Markdown::new(markdown).show_with_cache(ui, cache);
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Plan {
    Rich,
    Paragraph(Arc<str>),
    Fallback,
}

const MAX_PLAN_BYTES: usize = 1024 * 1024;
const MAX_PLAN_ENTRIES: usize = 32;

#[derive(Clone)]
struct PlanEntry {
    source: String,
    limit: Option<usize>,
    plan: Plan,
}

impl PlanEntry {
    fn bytes(&self) -> usize {
        self.source.len()
            + match &self.plan {
                Plan::Paragraph(text) => text.len(),
                _ => 0,
            }
    }
}

#[derive(Clone, Default)]
struct PlanCache {
    entries: VecDeque<PlanEntry>,
    bytes: usize,
}

fn plan_cache_id() -> Id {
    Id::unique("spaceui-markdown-plans")
}

fn cached_plan(ctx: &Context, source: &str, limit: Option<usize>) -> Plan {
    // Ordinary unbudgeted messages need neither a second parse nor retained source.
    if limit.is_none() && !fragmented_candidate(source) {
        return Plan::Rich;
    }
    if source.len() > MAX_PLAN_BYTES / 2 {
        return plan(source, limit);
    }
    if let Some(found) = ctx.data_mut(|data| {
        let cache = data.get_temp_mut_or_default::<PlanCache>(plan_cache_id());
        let index = cache.entries.iter().position(|entry| entry.limit == limit && entry.source == source)?;
        let entry = cache.entries.remove(index)?;
        let plan = entry.plan.clone();
        cache.entries.push_back(entry);
        Some(plan)
    }) {
        return found;
    }
    // Parsing happens outside egui's context data lock.
    let plan = plan(source, limit);
    ctx.data_mut(|data| {
        let cache = data.get_temp_mut_or_default::<PlanCache>(plan_cache_id());
        let entry = PlanEntry { source: source.to_owned(), limit, plan: plan.clone() };
        let bytes = entry.bytes();
        if bytes > MAX_PLAN_BYTES {
            return;
        }
        while cache.entries.len() >= MAX_PLAN_ENTRIES || cache.bytes + bytes > MAX_PLAN_BYTES {
            let Some(oldest) = cache.entries.pop_front() else {
                break;
            };
            cache.bytes -= oldest.bytes();
        }
        cache.bytes += bytes;
        cache.entries.push_back(entry);
    });
    plan
}

fn fragmented_candidate(source: &str) -> bool {
    source.len() >= 1024 && source.bytes().filter(|byte| b"[]\\&*_~".contains(byte)).take(128).count() == 128
}

fn plan(source: &str, limit: Option<usize>) -> Plan {
    // Only investigate inputs likely to produce many adjacent Text events.
    // This is a cost heuristic, never a Markdown grammar or correctness test.
    let candidate = fragmented_candidate(source);
    if !candidate && limit.is_none() {
        return Plan::Rich;
    }
    // Match egui_commonmark_backend 0.26's parser_options. Keep extensions here
    // in step with dependency upgrades; no math/heading-ID options are enabled
    // by SpaceUI's default viewer.
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_DEFINITION_LIST;
    let mut paragraph = candidate;
    let mut ended = false;
    let mut text = String::new();
    for (index, event) in Parser::new_ext(source, options).enumerate() {
        if limit.is_some_and(|limit| index >= limit) {
            return Plan::Fallback;
        }
        match event {
            Event::Start(Tag::Paragraph) if index == 0 => {}
            Event::Text(fragment) if paragraph && !ended => text.push_str(&fragment),
            Event::End(TagEnd::Paragraph) if paragraph && !ended => ended = true,
            _ => paragraph = false,
        }
        if !paragraph && limit.is_none() {
            return Plan::Rich;
        }
    }
    if paragraph && ended { Plan::Paragraph(text.into()) } else { Plan::Rich }
}

/// Drop the shared viewer cache and analysis cache in this context.
/// Analysis retains at most 32 entries and 1 MiB of source and decoded text;
/// larger inputs bypass retention. This does not include egui's font/layout caches.
/// Application-owned caches and egui image-loader caches are managed separately.
pub fn clear_cache(ctx: &Context) {
    ctx.data_mut(|data| {
        data.remove::<SharedCache>(cache_id());
        data.remove::<PlanCache>(plan_cache_id());
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fragmented_paragraph_decodes_entities_and_preserves_unicode() {
        let source = "[é &amp; ".repeat(256);
        assert_eq!(plan(&source, None), Plan::Paragraph("[é & ".repeat(256).trim_end().into()));
        assert_eq!(plan(&"[".repeat(16_384), None), Plan::Paragraph("[".repeat(16_384).into()));
        let escaped = r"\[\*".repeat(512);
        assert_eq!(plan(&escaped, None), Plan::Paragraph("[*".repeat(512).into()));
    }

    #[test]
    fn formatting_and_block_boundaries_never_take_the_plain_fast_path() {
        let brackets = "[".repeat(2048);
        for source in [
            format!("{brackets} **bold**"),
            format!("{brackets}\nnext"),
            format!("{brackets}  \nnext"),
            format!("{brackets}\n\nnext"),
            format!("{brackets} [link](https://example.org)"),
            format!("# {brackets}"),
            format!("```\n{brackets}\n```"),
            format!("{brackets} [reference]\n\n[reference]: https://example.org"),
            format!("{brackets} <b>HTML</b>"),
        ] {
            assert_eq!(plan(&source, None), Plan::Rich, "{source}");
        }
    }

    #[test]
    fn event_budget_is_explicit_and_counts_boundaries() {
        assert_eq!(plan("hello", Some(3)), Plan::Rich);
        assert_eq!(plan("hello", Some(2)), Plan::Fallback);
        assert_eq!(plan("", Some(0)), Plan::Rich);
        assert_eq!(plan("hello", Some(0)), Plan::Fallback);
        assert_eq!(plan(&"[".repeat(16_384), Some(1024)), Plan::Fallback);
    }

    #[test]
    fn analysis_cache_is_bounded_tracks_source_and_budget_and_can_be_cleared() {
        let ctx = Context::default();
        let source = "[".repeat(2048);
        assert!(matches!(cached_plan(&ctx, &source, None), Plan::Paragraph(_)));
        assert_eq!(cached_plan(&ctx, &format!("{source} **bold**"), None), Plan::Rich);
        assert_eq!(cached_plan(&ctx, &source, Some(5)), Plan::Fallback);
        for i in 0..40 {
            cached_plan(&ctx, &format!("{} {i}", "[".repeat(32_768)), None);
        }
        let before = ctx.data_mut(|data| {
            let cache = data.get_temp::<PlanCache>(plan_cache_id()).unwrap();
            assert!(cache.entries.len() <= MAX_PLAN_ENTRIES);
            assert!(cache.bytes <= MAX_PLAN_BYTES);
            assert_eq!(cache.bytes, cache.entries.iter().map(PlanEntry::bytes).sum::<usize>());
            cache.bytes
        });
        cached_plan(&ctx, &"[".repeat(MAX_PLAN_BYTES), Some(1));
        assert_eq!(ctx.data_mut(|data| data.get_temp::<PlanCache>(plan_cache_id()).unwrap().bytes), before);
        // Binary control characters must not undermine byte accounting.
        let controls = format!("{}{}", "[".repeat(128), "\0".repeat(MAX_PLAN_BYTES / 2 - 256));
        assert!(matches!(cached_plan(&ctx, &controls, None), Plan::Paragraph(_)));
        assert!(ctx.data_mut(|data| data.get_temp::<PlanCache>(plan_cache_id()).unwrap().bytes) <= MAX_PLAN_BYTES);
        clear_cache(&ctx);
        assert!(ctx.data_mut(|data| data.get_temp::<PlanCache>(plan_cache_id())).is_none());
    }
}
