//! Vector art: the built-in [`Icon`] set and app-supplied [`Svg`]s such as
//! logos.
//!
//! Art is rasterized with [resvg] at the exact pixel size it is drawn at,
//! cached per size and placed on whole pixels, so it stays crisp at any
//! display scale. Icons are white so painting can tint them; logos keep their
//! colours when painted with a white tint.
//!
//! ```
//! use spaceui::{Svg, egui};
//!
//! const LOGO: Svg = Svg::new(
//!     "myapp/logo",
//!     r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 8 8"><circle cx="4" cy="4" r="4" fill="#EA7C34"/></svg>"##,
//! );
//!
//! # let ctx = egui::Context::default();
//! # ctx.run_ui(Default::default(), |ui| {
//! let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::Vec2::splat(32.0));
//! LOGO.paint(ui.painter(), rect, egui::Color32::WHITE);
//! # }).drop_without_applying_deltas();
//! let window_icon = LOGO.window_icon(256).expect("valid SVG");
//! assert_eq!(window_icon.width, 256);
//! ```
//!
//! [resvg]: https://github.com/linebender/resvg
mod icons;

pub use icons::Icon;

use egui::{
    Color32, ColorImage, Context, IconData, Id, Painter, Rect, TextureHandle, TextureOptions, emath::GuiRounding, pos2,
};
use resvg::{tiny_skia, usvg};
use std::{collections::HashMap, error::Error, fmt};

/// Largest supported raster dimension, in pixels (16 MiB of RGBA at this size).
pub const MAX_RASTER_SIZE: u32 = 2048;
const MAX_CACHE_BYTES: usize = 32 * 1024 * 1024;
const MAX_CACHE_ENTRIES: usize = 256;

/// Why an SVG could not be rendered.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SvgError {
    /// The requested dimension is zero or exceeds [`MAX_RASTER_SIZE`].
    InvalidSize(u32),
    /// The SVG parser rejected the document; includes its diagnostic.
    InvalidDocument(String),
    /// The raster image could not be allocated.
    AllocationFailed,
}

impl fmt::Display for SvgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSize(px) => write!(f, "SVG size {px} is outside 1..={MAX_RASTER_SIZE}"),
            Self::InvalidDocument(error) => write!(f, "invalid SVG: {error}"),
            Self::AllocationFailed => f.write_str("could not allocate the SVG raster"),
        }
    }
}

impl Error for SvgError {}

/// Current per-context texture retention. Excludes renderer/driver overhead
/// and textures the application has retained separately.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CacheStats {
    /// Cached successful and failed raster requests (at most 256).
    pub entries: usize,
    /// RGBA texture bytes retained by the cache (at most 32 MiB).
    pub bytes: usize,
}

/// Inspect SVG texture retention in this context.
#[must_use]
pub fn cache_stats(ctx: &Context) -> CacheStats {
    ctx.data_mut(|d| {
        let cache = d.get_temp_mut_or_default::<Cache>(cache_id());
        CacheStats { entries: cache.entries.len(), bytes: cache.bytes }
    })
}

/// Release cached SVG textures, for example after closing an asset browser.
pub fn clear_cache(ctx: &Context) {
    ctx.data_mut(|d| d.remove::<Cache>(cache_id()));
}

/// A static SVG document and a diagnostic name. The cache also uses source
/// identity, so different documents with the same name remain independent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Svg {
    /// Diagnostic texture name, such as `"myapp/logo"`.
    pub name: &'static str,
    /// The SVG document itself, usually from `include_str!`.
    pub source: &'static str,
}

impl Svg {
    /// Art named `name` from SVG text.
    #[must_use]
    pub const fn new(name: &'static str, source: &'static str) -> Self {
        Self { name, source }
    }

    /// Paints into `rect`; `tint` multiplies the art (white keeps its colours).
    /// Invalid SVG paints nothing. Use [`Self::try_paint`] for diagnostics.
    pub fn paint(self, painter: &Painter, rect: Rect, tint: Color32) {
        let _ = self.try_paint(painter, rect, tint);
    }

    /// Paints SVG art, reporting invalid documents or raster allocation errors.
    /// Empty or non-finite rectangles are ignored.
    ///
    /// # Errors
    /// Returns [`SvgError`] when the document cannot be rasterized.
    pub fn try_paint(self, painter: &Painter, rect: Rect, tint: Color32) -> Result<(), SvgError> {
        if !rect.is_finite() || !rect.is_positive() {
            return Ok(());
        }
        // Whole pixels: art sampled between pixels looks soft.
        let rect = rect.round_to_pixels(painter.pixels_per_point());
        let texture = texture(painter.ctx(), self, rect.width().max(rect.height()))?;
        painter.image(texture.id(), rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), tint);
        Ok(())
    }

    /// Rasterizes to a square image `px` pixels wide (premultiplied alpha),
    /// or `None` for an invalid document, size, or allocation failure.
    /// Use [`Self::try_rasterize`] for diagnostics.
    #[must_use]
    pub fn rasterize(self, px: u32) -> Option<ColorImage> {
        self.try_rasterize(px).ok()
    }

    /// Rasterize a square image with a bounded allocation.
    ///
    /// # Errors
    /// Rejects dimensions outside `1..=MAX_RASTER_SIZE`, invalid SVG, and
    /// failed image allocation. Source parsing runs synchronously; applications
    /// handling untrusted documents should also limit source length/complexity.
    pub fn try_rasterize(self, px: u32) -> Result<ColorImage, SvgError> {
        if !(1..=MAX_RASTER_SIZE).contains(&px) {
            return Err(SvgError::InvalidSize(px));
        }
        let tree = usvg::Tree::from_str(self.source, &usvg::Options::default())
            .map_err(|error| SvgError::InvalidDocument(error.to_string()))?;
        let mut pixmap = tiny_skia::Pixmap::new(px, px).ok_or(SvgError::AllocationFailed)?;
        let scale = px as f32 / tree.size().width().max(tree.size().height());
        resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
        Ok(ColorImage::from_rgba_premultiplied([px as usize, px as usize], pixmap.data()))
    }

    /// The art as a window and taskbar icon (straight alpha), for
    /// `ViewportBuilder::with_icon`.
    #[must_use]
    pub fn window_icon(self, px: u32) -> Option<IconData> {
        self.try_window_icon(px).ok()
    }

    /// Rasterize a straight-alpha window icon, preserving failure diagnostics.
    ///
    /// # Errors
    /// Returns the errors described by [`Self::try_rasterize`].
    pub fn try_window_icon(self, px: u32) -> Result<IconData, SvgError> {
        let image = self.try_rasterize(px)?;
        Ok(IconData {
            width: px,
            height: px,
            rgba: image.pixels.iter().flat_map(Color32::to_srgba_unmultiplied).collect(),
        })
    }
}

type CacheKey = (&'static str, usize, usize, u32);

#[derive(Clone)]
struct Entry {
    texture: Result<TextureHandle, SvgError>,
    bytes: usize,
    touched: u64,
}

#[derive(Clone, Default)]
struct Cache {
    entries: HashMap<CacheKey, Entry>,
    bytes: usize,
    clock: u64,
}

fn cache_id() -> Id {
    Id::unique("spaceui-art-cache")
}

/// The texture for `svg` drawn `points` wide, rendered once per pixel size.
fn texture(ctx: &Context, svg: Svg, points: f32) -> Result<TextureHandle, SvgError> {
    let px = ((points * ctx.pixels_per_point()).round() as u32).clamp(4, MAX_RASTER_SIZE);
    let key = (svg.name, svg.source.as_ptr() as usize, svg.source.len(), px);
    if let Some(found) = ctx.data_mut(|d| {
        let cache = d.get_temp_mut_or_default::<Cache>(cache_id());
        cache.clock = cache.clock.saturating_add(1);
        cache.entries.get_mut(&key).map(|entry| {
            entry.touched = cache.clock;
            entry.texture.clone()
        })
    }) {
        return found;
    }
    let handle = svg
        .try_rasterize(px)
        .map(|image| ctx.load_texture(format!("{}@{px}", svg.name), image, TextureOptions::LINEAR));
    let bytes = if handle.is_ok() { (px as usize).pow(2) * 4 } else { 0 };
    ctx.data_mut(|d| {
        let cache = d.get_temp_mut_or_default::<Cache>(cache_id());
        if let Some(previous) = cache.entries.remove(&key) {
            cache.bytes -= previous.bytes;
        }
        while cache.entries.len() >= MAX_CACHE_ENTRIES || cache.bytes + bytes > MAX_CACHE_BYTES {
            let Some(oldest) = cache.entries.iter().min_by_key(|(_, entry)| entry.touched).map(|(key, _)| *key) else {
                break;
            };
            if let Some(removed) = cache.entries.remove(&oldest) {
                cache.bytes -= removed.bytes;
            }
        }
        cache.bytes += bytes;
        cache.entries.insert(key, Entry { texture: handle.clone(), bytes, touched: cache.clock });
    });
    handle
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOT: Svg = Svg::new(
        "test/dot",
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 4 4"><circle cx="2" cy="2" r="2" fill="#FF8000" fill-opacity="0.5"/></svg>"##,
    );

    #[test]
    fn window_icons_use_straight_alpha() {
        let icon = DOT.window_icon(16).expect("renders");
        let centre = &icon.rgba[(8 * 16 + 8) * 4..][..4];
        assert!(centre[0] > 250 && (120..=136).contains(&centre[3]), "{centre:?}");
    }

    #[test]
    fn invalid_svg_rasterizes_to_none() {
        assert!(Svg::new("test/bad", "<svg").rasterize(16).is_none());
    }

    #[test]
    fn textures_are_cached_per_pixel_size() {
        let ctx = Context::default();
        let a = texture(&ctx, DOT, 20.0).expect("texture");
        let b = texture(&ctx, DOT, 20.0).expect("texture");
        let c = texture(&ctx, DOT, 40.0).expect("texture");
        assert_eq!(a.id(), b.id());
        assert_ne!(a.id(), c.id());
    }

    #[test]
    fn diagnostics_distinguish_invalid_dimensions_and_documents() {
        assert!(matches!(DOT.try_rasterize(0), Err(SvgError::InvalidSize(0))));
        assert!(matches!(DOT.try_rasterize(MAX_RASTER_SIZE + 1), Err(SvgError::InvalidSize(_))));
        assert!(matches!(Svg::new("bad", "<svg").try_rasterize(16), Err(SvgError::InvalidDocument(_))));
    }

    #[test]
    fn different_sources_with_the_same_name_do_not_share_a_texture() {
        let other = Svg::new(DOT.name, r#"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"/>"#);
        let ctx = Context::default();
        assert_ne!(texture(&ctx, DOT, 16.0).unwrap().id(), texture(&ctx, other, 16.0).unwrap().id());
    }

    #[test]
    fn failed_requests_are_bounded_and_cache_can_be_released() {
        let ctx = Context::default();
        for px in 4..400 {
            assert!(texture(&ctx, Svg::new("bad", "<svg"), px as f32).is_err());
        }
        assert_eq!(cache_stats(&ctx), CacheStats { entries: MAX_CACHE_ENTRIES, bytes: 0 });
        clear_cache(&ctx);
        assert_eq!(cache_stats(&ctx), CacheStats::default());
    }

    #[test]
    fn successful_requests_stay_within_the_memory_budget() {
        let ctx = Context::default();
        for px in 1500..1505 {
            let _ = texture(&ctx, DOT, px as f32).unwrap();
            assert!(cache_stats(&ctx).bytes <= MAX_CACHE_BYTES);
        }
        assert!(cache_stats(&ctx).entries < 5, "old sizes must be evicted");
    }
}
