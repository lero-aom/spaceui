//! `CommonMark` rendering for chat replies and user bubbles, powered by
//! [`egui_commonmark`]. Parsing, layout, code highlighting and link interaction
//! are delegated to its viewer; this module only applies SpaceUI's theme.
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
use egui::{Context, FontId, Id, TextStyle, Ui};
use std::sync::{Arc, Mutex};

type SharedCache = Arc<Mutex<CommonMarkCache>>;

fn cache_id() -> Id {
    Id::unique("spaceui-commonmark-cache")
}

/// Render `CommonMark` using a cache shared by the viewers in this egui context.
pub fn render(ui: &mut Ui, markdown: &str) {
    let cache = ui.ctx().data_mut(|data| data.get_temp_mut_or_default::<SharedCache>(cache_id()).clone());
    // Release the context's data lock before the viewer accesses fonts and loaders.
    let mut cache = cache.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    render_with_cache(ui, &mut cache, markdown);
}

/// Render `CommonMark` with application-owned cache state and SpaceUI's theme.
///
/// Link destinations follow `egui_commonmark`/egui navigation behavior. Use
/// [`CommonMarkCache::add_link_hook`] or handle egui's output commands when the
/// application needs its own navigation policy. HTML is displayed as text.
pub fn render_with_cache(ui: &mut Ui, cache: &mut CommonMarkCache, markdown: &str) {
    let palette = Theme::of(ui.ctx()).palette;
    ui.scope(|ui| {
        let style = ui.style_mut();
        style.text_styles.insert(TextStyle::Body, FontId::proportional(13.5));
        style.text_styles.insert(TextStyle::Monospace, FontId::monospace(12.0));
        style.visuals.override_text_color = Some(palette.text);
        style.visuals.hyperlink_color = palette.accent_hi;
        style.visuals.code_bg_color = palette.field;
        style.visuals.extreme_bg_color = palette.well;
        CommonMarkViewer::new().show(ui, cache, markdown);
    });
}

/// Drop the cache used by [`render`] in this context.
/// Application-owned caches and egui image-loader caches are managed separately.
pub fn clear_cache(ctx: &Context) {
    ctx.data_mut(|data| data.remove::<SharedCache>(cache_id()));
}
