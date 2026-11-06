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
//! const LOGO: Svg<'static> = Svg::new(
//!     "myapp/logo",
//!     r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 8 8"><circle cx="4" cy="4" r="4" fill="#EA7C34"/></svg>"##,
//! );
//!
//! # let ctx = egui::Context::default();
//! # ctx.run_ui(Default::default(), |ui| {
//! let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::Vec2::splat(32.0));
//! LOGO.paint(ui.painter(), rect, egui::Color32::WHITE);
//! # }).drop_without_applying_deltas();
//! let window_icon = LOGO.try_window_icon(256)?;
//! assert_eq!(window_icon.width, 256);
//! # Ok::<(), spaceui::art::SvgError>(())
//! ```
//!
//! [resvg]: https://github.com/linebender/resvg
mod icons;
#[cfg(not(target_arch = "wasm32"))]
mod loader;
#[cfg(not(target_arch = "wasm32"))]
pub use loader::SvgLoader;

pub use icons::Icon;

use egui::{
    Color32, ColorImage, Context, IconData, Id, Painter, Rect, TextureHandle, TextureOptions, Vec2, emath::GuiRounding,
    pos2,
};
use resvg::{tiny_skia, usvg};
use std::{collections::HashMap, error::Error, fmt, sync::Arc};

/// Largest supported raster dimension, in pixels (16 MiB of RGBA at this size).
pub const MAX_RASTER_SIZE: u32 = 2048;
const MAX_CACHE_BYTES: usize = 32 * 1024 * 1024;
const MAX_CACHE_ENTRIES: usize = 256;
const MAX_CACHE_DOCUMENTS: usize = 128;
const MAX_CACHE_SOURCE_BYTES: usize = 4 * 1024 * 1024;

/// Why an SVG could not be rendered.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SvgError {
    /// The requested dimension is zero or exceeds [`MAX_RASTER_SIZE`].
    InvalidSize(u32),
    /// The SVG parser rejected the document; includes its diagnostic.
    InvalidDocument(String),
    /// The raster backing buffer could not be constructed. This does not
    /// represent every failure of the process's global allocator.
    AllocationFailed,
    /// The background loader rejects source text larger than 4 MiB.
    SourceTooLarge(usize),
    /// The background loader's pending-work or pending-source budget is full.
    /// Retry on a later frame; completion requests a repaint.
    QueueFull,
    /// The background worker stopped. Construct a new loader to retry.
    WorkerStopped,
}

impl fmt::Display for SvgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSize(px) => write!(f, "SVG size {px} is outside 1..={MAX_RASTER_SIZE}"),
            Self::InvalidDocument(error) => write!(f, "invalid SVG: {error}"),
            Self::AllocationFailed => f.write_str("could not allocate the SVG raster"),
            Self::SourceTooLarge(bytes) => write!(f, "SVG source ({bytes} bytes) exceeds the 4 MiB background limit"),
            Self::QueueFull => f.write_str("SVG background work queue is full"),
            Self::WorkerStopped => f.write_str("SVG background worker stopped"),
        }
    }
}

impl Error for SvgError {}

/// Current per-context retention. Texture and source bytes are exact; parsed
/// tree allocations, renderer overhead and externally held textures are excluded.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CacheStats {
    /// Cached successful and failed raster requests (at most 256).
    pub entries: usize,
    /// RGBA texture bytes retained by the cache (at most 32 MiB).
    pub bytes: usize,
    /// Cached parsed documents, including failures (at most 128).
    pub documents: usize,
    /// UTF-8 source bytes retained by the document cache (at most 4 MiB).
    /// This is not a measurement of parsed-tree memory.
    pub source_bytes: usize,
}

/// Inspect SVG texture and document retention in this context.
#[must_use]
pub fn cache_stats(ctx: &Context) -> CacheStats {
    ctx.data_mut(|d| {
        let cache = d.get_temp_mut_or_default::<Cache>(cache_id());
        CacheStats {
            entries: cache.entries.len(),
            bytes: cache.bytes,
            documents: cache.documents.len(),
            source_bytes: cache.source_bytes,
        }
    })
}

/// Release cached SVG textures and parsed documents.
pub fn clear_cache(ctx: &Context) {
    ctx.data_mut(|d| d.remove::<Cache>(cache_id()));
}

/// A borrowed SVG document and diagnostic name, from static assets or runtime
/// strings. Painting caches by source content, so callers can edit or drop their
/// strings after the call without stale textures or leaked string allocations.
///
/// ```
/// use spaceui::Svg;
/// let name = String::from("downloaded/logo");
/// let source = String::from(r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10"/>"#);
/// let image = Svg::new(&name, &source).try_rasterize(32).unwrap();
/// assert_eq!(image.size, [32, 32]);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Svg<'a> {
    /// Diagnostic texture name, such as `"myapp/logo"`.
    pub name: &'a str,
    /// The SVG document itself, usually from `include_str!`.
    pub source: &'a str,
}

impl<'a> Svg<'a> {
    /// Art named `name` from SVG text.
    #[must_use]
    pub const fn new(name: &'a str, source: &'a str) -> Self {
        Self { name, source }
    }

    /// Fits the art inside `rect`, centered and preserving its aspect ratio.
    /// `tint` multiplies the art (white keeps its colours).
    /// Invalid SVG paints nothing. Use [`Self::try_paint`] for diagnostics.
    pub fn paint(self, painter: &Painter, rect: Rect, tint: Color32) {
        let _ = self.try_paint(painter, rect, tint);
    }

    /// Paints SVG art, reporting invalid documents or raster-construction errors.
    /// Empty, non-finite or overflowing rectangles are ignored. Parsed documents
    /// are reused across raster sizes. Parsing and rasterization on a cache miss
    /// remain synchronous; cache bounds do not guarantee a frame-time limit.
    /// On native targets, use [`crate::art::SvgLoader`] to move parsing and
    /// rasterization off the UI thread.
    ///
    /// # Errors
    /// Returns [`SvgError`] when the document cannot be rasterized.
    pub fn try_paint(self, painter: &Painter, rect: Rect, tint: Color32) -> Result<(), SvgError> {
        if !rect.is_finite() || !rect.size().is_finite() || !rect.is_positive() {
            return Ok(());
        }
        // Whole pixels: art sampled between pixels looks soft.
        let rect = rect.round_to_pixels(painter.pixels_per_point());
        if !rect.is_finite() || !rect.size().is_finite() || !rect.is_positive() {
            return Ok(());
        }
        let texture = texture(painter.ctx(), self, rect.size())?;
        painter.image(texture.id(), rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), tint);
        Ok(())
    }

    /// Rasterizes to a square image `px` pixels wide (premultiplied alpha),
    /// or `None` for an invalid document, size, or rejected raster buffer.
    /// Use [`Self::try_rasterize`] for diagnostics.
    #[must_use]
    pub fn rasterize(self, px: u32) -> Option<ColorImage> {
        self.try_rasterize(px).ok()
    }

    /// Rasterize a square image with a bounded allocation. The art is centered
    /// with its aspect ratio preserved; unused pixels remain transparent.
    ///
    /// # Errors
    /// Rejects dimensions outside `1..=MAX_RASTER_SIZE`, invalid SVG, and
    /// rejected raster buffers. These bounds limit image dimensions; they do
    /// not make process-wide memory exhaustion recoverable. Source parsing runs
    /// synchronously; applications handling untrusted documents should also
    /// limit source length/complexity.
    pub fn try_rasterize(self, px: u32) -> Result<ColorImage, SvgError> {
        self.rasterize_size([px, px])
    }

    fn rasterize_size(self, size: [u32; 2]) -> Result<ColorImage, SvgError> {
        let [width, height] = size;
        for dimension in size {
            if !(1..=MAX_RASTER_SIZE).contains(&dimension) {
                return Err(SvgError::InvalidSize(dimension));
            }
        }
        rasterize_tree(&parse(self.source)?, [width, height])
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

fn parse(source: &str) -> Result<usvg::Tree, SvgError> {
    usvg::Tree::from_str(source, &usvg::Options::default())
        .map_err(|error| SvgError::InvalidDocument(error.to_string()))
}

fn rasterize_tree(tree: &usvg::Tree, [width, height]: [u32; 2]) -> Result<ColorImage, SvgError> {
    let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or(SvgError::AllocationFailed)?;
    let scale = (width as f32 / tree.size().width()).min(height as f32 / tree.size().height());
    let x = (width as f32 - tree.size().width() * scale) * 0.5;
    let y = (height as f32 - tree.size().height() * scale) * 0.5;
    let transform = tiny_skia::Transform::from_row(scale, 0.0, 0.0, scale, x, y);
    resvg::render(tree, transform, &mut pixmap.as_mut());
    Ok(ColorImage::from_rgba_premultiplied([width as usize, height as usize], pixmap.data()))
}

type CacheKey = (Arc<str>, [u32; 2]);

#[derive(Clone)]
struct Document {
    source: Arc<str>,
    tree: Result<Arc<usvg::Tree>, SvgError>,
    touched: u64,
}

#[derive(Clone)]
struct Entry {
    texture: Result<TextureHandle, SvgError>,
    bytes: usize,
    touched: u64,
}

#[derive(Clone, Default)]
struct Cache {
    documents: HashMap<Arc<str>, Document>,
    entries: HashMap<CacheKey, Entry>,
    bytes: usize,
    source_bytes: usize,
    clock: u64,
}

impl Cache {
    fn evict_document(&mut self, source: &str) {
        if let Some(document) = self.documents.remove(source) {
            self.source_bytes -= document.source.len();
            // Raster keys also hold source text; evict them with their document
            // so the source-byte budget covers every retained copy.
            self.entries.retain(|(key, _), entry| {
                if key.as_ref() == source {
                    self.bytes -= entry.bytes;
                    false
                } else {
                    true
                }
            });
        }
    }
}

fn document(ctx: &Context, source: &str) -> Document {
    if let Some(document) =
        ctx.data_mut(|data| data.get_temp_mut_or_default::<Cache>(cache_id()).documents.get(source).cloned())
    {
        return document;
    }
    // Never parse while holding the context's data lock.
    let parsed = Document { source: Arc::from(source), tree: parse(source).map(Arc::new), touched: 0 };
    ctx.data_mut(|data| {
        let cache = data.get_temp_mut_or_default::<Cache>(cache_id());
        if let Some(existing) = cache.documents.get(source) {
            return existing.clone();
        }
        while cache.documents.len() >= MAX_CACHE_DOCUMENTS || cache.source_bytes + source.len() > MAX_CACHE_SOURCE_BYTES
        {
            let Some(oldest) = cache
                .documents
                .values()
                .min_by_key(|document| document.touched)
                .map(|document| document.source.clone())
            else {
                break;
            };
            cache.evict_document(&oldest);
        }
        let document = Document { touched: cache.clock, ..parsed };
        cache.source_bytes += source.len();
        cache.documents.insert(document.source.clone(), document.clone());
        document
    })
}

fn cache_id() -> Id {
    Id::unique("spaceui-art-cache")
}

/// The texture for `svg`, rendered once per pair of pixel dimensions.
fn texture(ctx: &Context, svg: Svg<'_>, points: Vec2) -> Result<TextureHandle, SvgError> {
    // Bound both dimensions with a shared scale, including enormous finite rectangles.
    let scale = f64::from(ctx.pixels_per_point()).min(f64::from(MAX_RASTER_SIZE) / f64::from(points.max_elem()));
    let size =
        [points.x, points.y].map(|dimension| ((f64::from(dimension) * scale).round() as u32).clamp(1, MAX_RASTER_SIZE));
    let load = |image| ctx.load_texture(format!("{}@{}x{}", svg.name, size[0], size[1]), image, TextureOptions::LINEAR);
    // Oversized sources still render, but neither their trees nor textures are retained.
    if svg.source.len() > MAX_CACHE_SOURCE_BYTES {
        return svg.rasterize_size(size).map(load);
    }
    if let Some(found) = ctx.data_mut(|d| {
        let cache = d.get_temp_mut_or_default::<Cache>(cache_id());
        cache.clock = cache.clock.saturating_add(1);
        let document = cache.documents.get_mut(svg.source)?;
        document.touched = cache.clock;
        let key = (document.source.clone(), size);
        cache.entries.get_mut(&key).map(|entry| {
            entry.touched = cache.clock;
            entry.texture.clone()
        })
    }) {
        return found;
    }
    let document = document(ctx, svg.source);
    let handle = document.tree.as_ref().map_err(Clone::clone).and_then(|tree| rasterize_tree(tree, size)).map(load);
    let bytes = if handle.is_ok() { size[0] as usize * size[1] as usize * 4 } else { 0 };
    ctx.data_mut(|d| {
        let cache = d.get_temp_mut_or_default::<Cache>(cache_id());
        let Some(current) = cache.documents.get(svg.source) else {
            return;
        };
        let key = (current.source.clone(), size);
        if let Some(previous) = cache.entries.remove(&key) {
            cache.bytes -= previous.bytes;
        }
        while cache.entries.len() >= MAX_CACHE_ENTRIES || cache.bytes + bytes > MAX_CACHE_BYTES {
            let Some(oldest) = cache.entries.iter().min_by_key(|(_, entry)| entry.touched).map(|(key, _)| key.clone())
            else {
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

    const DOT: Svg<'static> = Svg::new(
        "test/dot",
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 4 4"><circle cx="2" cy="2" r="2" fill="#FF8000" fill-opacity="0.5"/></svg>"##,
    );

    const WIDE: Svg<'static> = Svg::new(
        "test/wide",
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 10"><path d="M0 0H20V10H0Z" fill="white"/></svg>"#,
    );
    const TALL: Svg<'static> = Svg::new(
        "test/tall",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="20"><path d="M0 0H10V20H0Z" fill="white"/></svg>"#,
    );

    #[test]
    fn rectangular_art_fills_matching_destinations_and_is_centered_in_square_exports() {
        for (svg, size, origin, extent) in [
            (WIDE, [200, 100], [0, 0], [200, 100]),
            (TALL, [100, 200], [0, 0], [100, 200]),
            (WIDE, [100, 100], [0, 25], [100, 50]),
            (TALL, [100, 100], [25, 0], [50, 100]),
            (WIDE, [100, 200], [0, 75], [100, 50]),
        ] {
            let image = if size[0] == size[1] {
                svg.try_rasterize(size[0]).unwrap()
            } else {
                svg.rasterize_size(size).unwrap()
            };
            assert_eq!(image.size, size.map(|n| n as usize));
            for y in 0..size[1] {
                for x in 0..size[0] {
                    let inside = (origin[0]..origin[0] + extent[0]).contains(&x)
                        && (origin[1]..origin[1] + extent[1]).contains(&y);
                    assert_eq!(
                        image[(x as usize, y as usize)].a(),
                        if inside { 255 } else { 0 },
                        "{size:?} at {x},{y}"
                    );
                }
            }
        }
    }

    #[test]
    fn painting_uploads_rectangular_textures_at_display_density() {
        for density in [1.0, 1.5, 2.0] {
            for (svg, size) in [(WIDE, Vec2::new(200.0, 100.0)), (TALL, Vec2::new(100.0, 200.0))] {
                let ctx = Context::default();
                ctx.set_pixels_per_point(density);
                let rect = Rect::from_min_size(pos2(12.0, 20.0), size);
                let output = ctx
                    .run_ui(egui::RawInput::default(), |ui| svg.try_paint(ui.painter(), rect, Color32::WHITE).unwrap());
                let mesh = output
                    .shapes
                    .iter()
                    .find_map(|shape| match &shape.shape {
                        egui::Shape::Mesh(mesh) => Some(mesh),
                        _ => None,
                    })
                    .expect("painted image");
                assert_eq!(mesh.calc_bounds(), rect);
                let delta = &output.textures_delta.set.get(&mesh.texture_id).expect("texture upload")[0];
                let egui::ImageData::Color(image) = &delta.image;
                assert_eq!(image.size, [(size.x * density) as usize, (size.y * density) as usize]);
                assert!(image.pixels.iter().all(|pixel| pixel.a() == 255), "art must fill the matching rectangle");
                output.drop_without_applying_deltas();
            }
        }
    }

    #[test]
    fn rectangular_cache_keys_and_memory_account_for_both_dimensions() {
        let ctx = Context::default();
        let wide = texture(&ctx, WIDE, Vec2::new(200.0, 100.0)).unwrap();
        let tall = texture(&ctx, WIDE, Vec2::new(100.0, 200.0)).unwrap();
        assert_ne!(wide.id(), tall.id());
        assert_eq!(wide.id(), texture(&ctx, WIDE, Vec2::new(200.0, 100.0)).unwrap().id());
        assert_eq!(
            cache_stats(&ctx),
            CacheStats { entries: 2, bytes: 2 * 200 * 100 * 4, documents: 1, source_bytes: WIDE.source.len() }
        );
        let capped = texture(&ctx, WIDE, Vec2::new(8000.0, 4000.0)).unwrap();
        assert_eq!(capped.size(), [2048, 1024]);
    }

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
        let a = texture(&ctx, DOT, Vec2::splat(20.0)).expect("texture");
        let b = texture(&ctx, DOT, Vec2::splat(20.0)).expect("texture");
        let c = texture(&ctx, DOT, Vec2::splat(40.0)).expect("texture");
        assert_eq!(a.id(), b.id());
        assert_ne!(a.id(), c.id());
    }

    #[test]
    fn parsed_tree_survives_raster_size_changes_and_texture_eviction() {
        let ctx = Context::default();
        let first = texture(&ctx, WIDE, Vec2::splat(1.0)).unwrap();
        let tree = document(&ctx, WIDE.source).tree.unwrap();
        for px in 2..=MAX_CACHE_ENTRIES + 2 {
            let _ = texture(&ctx, WIDE, Vec2::new(px as f32, 1.0)).unwrap();
        }
        let replacement = texture(&ctx, WIDE, Vec2::splat(1.0)).unwrap();
        assert_ne!(first.id(), replacement.id(), "old raster must have been evicted");
        assert!(Arc::ptr_eq(&tree, &document(&ctx, WIDE.source).tree.unwrap()));
        assert_eq!(cache_stats(&ctx).documents, 1);
    }

    #[test]
    fn runtime_sources_are_owned_by_the_cache_and_keyed_by_content() {
        let ctx = Context::default();
        let mut source = WIDE.source.to_owned();
        let name = String::from("runtime");
        let original = texture(&ctx, Svg::new(&name, &source), Vec2::splat(16.0)).unwrap();
        let white = source.find("white").unwrap();
        source.replace_range(white..white + 5, "black");
        let edited = texture(&ctx, Svg::new(&name, &source), Vec2::splat(16.0)).unwrap();
        assert_ne!(original.id(), edited.id());
        drop(source);
        drop(name);
        let copied_source = WIDE.source.to_owned();
        let reused = texture(&ctx, Svg::new("another name", &copied_source), Vec2::splat(16.0)).unwrap();
        assert_eq!(original.id(), reused.id(), "equal content shares a cache entry");
        assert_eq!(cache_stats(&ctx).documents, 2);
        assert_eq!(cache_stats(&ctx).source_bytes, 2 * WIDE.source.len());
        clear_cache(&ctx);
        assert_eq!(cache_stats(&ctx), CacheStats::default());
        assert_eq!(original.size(), [16, 16], "external handles remain valid");
    }

    #[test]
    fn document_lru_evicts_its_textures_and_keeps_recently_used_sources() {
        let ctx = Context::default();
        let sources: Vec<_> = (0..=MAX_CACHE_DOCUMENTS).map(|i| format!("{}<!--{i}-->", WIDE.source)).collect();
        for source in &sources[..MAX_CACHE_DOCUMENTS] {
            let _ = texture(&ctx, Svg::new("runtime", source), Vec2::splat(4.0)).unwrap();
        }
        let recent = texture(&ctx, Svg::new("runtime", &sources[0]), Vec2::splat(4.0)).unwrap();
        let _ = texture(&ctx, Svg::new("runtime", &sources[MAX_CACHE_DOCUMENTS]), Vec2::splat(4.0)).unwrap();
        ctx.data_mut(|data| {
            let cache = data.get_temp_mut_or_default::<Cache>(cache_id());
            assert!(!cache.documents.contains_key(sources[1].as_str()));
            assert!(cache.entries.keys().all(|(source, _)| source.as_ref() != sources[1]));
        });
        assert_eq!(cache_stats(&ctx).documents, MAX_CACHE_DOCUMENTS);
        assert_eq!(cache_stats(&ctx).entries, MAX_CACHE_DOCUMENTS);
        assert_eq!(cache_stats(&ctx).bytes, MAX_CACHE_DOCUMENTS * 4 * 4 * 4);
        assert_eq!(recent.id(), texture(&ctx, Svg::new("runtime", &sources[0]), Vec2::splat(4.0)).unwrap().id());
    }

    #[test]
    fn source_budget_evicts_documents_and_oversized_sources_bypass_retention() {
        let ctx = Context::default();
        let source = format!("{}<!--{}-->", WIDE.source, " ".repeat(MAX_CACHE_SOURCE_BYTES / 2));
        let _ = texture(&ctx, Svg::new("large", &source), Vec2::splat(4.0)).unwrap();
        let second = format!("{source}\n");
        let _ = texture(&ctx, Svg::new("large", &second), Vec2::splat(4.0)).unwrap();
        let retained = cache_stats(&ctx);
        assert_eq!(retained, CacheStats { entries: 1, bytes: 64, documents: 1, source_bytes: second.len() });
        let oversized = format!("{source}<!--{}-->", " ".repeat(MAX_CACHE_SOURCE_BYTES / 2));
        for _ in 0..2 {
            assert_eq!(texture(&ctx, Svg::new("oversized", &oversized), Vec2::splat(4.0)).unwrap().size(), [4, 4]);
            assert_eq!(cache_stats(&ctx), retained);
        }
    }

    #[test]
    fn invalid_document_retention_is_bounded_across_distinct_sources() {
        let ctx = Context::default();
        for i in 0..MAX_CACHE_DOCUMENTS + 10 {
            assert!(texture(&ctx, Svg::new("bad", &format!("<invalid-{i}")), Vec2::splat(4.0)).is_err());
            let stats = cache_stats(&ctx);
            assert!(stats.documents <= MAX_CACHE_DOCUMENTS && stats.source_bytes <= MAX_CACHE_SOURCE_BYTES);
            assert_eq!(stats.entries, stats.documents);
            assert_eq!(stats.bytes, 0);
        }
    }

    #[test]
    fn diagnostics_distinguish_invalid_dimensions_and_documents() {
        assert!(matches!(DOT.try_rasterize(0), Err(SvgError::InvalidSize(0))));
        assert!(matches!(DOT.try_rasterize(MAX_RASTER_SIZE + 1), Err(SvgError::InvalidSize(_))));
        assert!(matches!(Svg::new("bad", "<svg").try_rasterize(16), Err(SvgError::InvalidDocument(_))));
    }

    #[test]
    fn invalid_paint_rectangles_do_not_allocate_textures() {
        let ctx = Context::default();
        let painter = ctx.layer_painter(egui::LayerId::background());
        for rect in [
            Rect::from_min_max(pos2(-f32::MAX, -f32::MAX), pos2(f32::MAX, f32::MAX)),
            Rect::from_min_max(pos2(1.0, 1.0), pos2(0.0, 0.0)),
            Rect::from_min_size(pos2(0.0, 0.0), egui::Vec2::splat(0.01)),
        ] {
            DOT.try_paint(&painter, rect, Color32::WHITE).expect("ignored rectangle");
            assert_eq!(cache_stats(&ctx), CacheStats::default());
        }
    }

    #[test]
    fn different_sources_with_the_same_name_do_not_share_a_texture() {
        let other = Svg::new(DOT.name, r#"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"/>"#);
        let ctx = Context::default();
        assert_ne!(
            texture(&ctx, DOT, Vec2::splat(16.0)).unwrap().id(),
            texture(&ctx, other, Vec2::splat(16.0)).unwrap().id()
        );
    }

    #[test]
    fn failed_requests_are_bounded_and_cache_can_be_released() {
        let ctx = Context::default();
        for px in 4..400 {
            assert!(texture(&ctx, Svg::new("bad", "<svg"), Vec2::splat(px as f32)).is_err());
        }
        assert_eq!(
            cache_stats(&ctx),
            CacheStats { entries: MAX_CACHE_ENTRIES, bytes: 0, documents: 1, source_bytes: 4 }
        );
        clear_cache(&ctx);
        assert_eq!(cache_stats(&ctx), CacheStats::default());
    }

    #[test]
    fn successful_requests_stay_within_the_memory_budget() {
        let ctx = Context::default();
        for px in 1500..1505 {
            let _ = texture(&ctx, DOT, Vec2::splat(px as f32)).unwrap();
            assert!(cache_stats(&ctx).bytes <= MAX_CACHE_BYTES);
        }
        assert!(cache_stats(&ctx).entries < 5, "old sizes must be evicted");
    }
}
