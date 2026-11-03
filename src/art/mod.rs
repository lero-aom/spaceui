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
use std::collections::HashMap;

/// An SVG document and a name that is unique across the app (the name keys
/// the texture cache, so two different documents must not share one).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Svg {
    /// Unique key for the texture cache, such as `"myapp/logo"`.
    pub name: &'static str,
    /// The SVG document itself, usually from `include_str!`.
    pub source: &'static str,
}

impl Svg {
    /// Art named `name` (unique across the app) from SVG text.
    #[must_use]
    pub const fn new(name: &'static str, source: &'static str) -> Self {
        Self { name, source }
    }

    /// Paints into `rect`; `tint` multiplies the art (white keeps its colours).
    /// Invalid SVG paints nothing.
    pub fn paint(self, painter: &Painter, rect: Rect, tint: Color32) {
        // Whole pixels: art sampled between pixels looks soft.
        let rect = rect.round_to_pixels(painter.pixels_per_point());
        if let Some(texture) = texture(painter.ctx(), self, rect.width().max(rect.height())) {
            painter.image(texture.id(), rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), tint);
        }
    }

    /// Rasterizes to a square image `px` pixels wide (premultiplied alpha),
    /// or `None` if the SVG doesn't parse.
    #[must_use]
    pub fn rasterize(self, px: u32) -> Option<ColorImage> {
        let tree = usvg::Tree::from_str(self.source, &usvg::Options::default()).ok()?;
        let mut pixmap = tiny_skia::Pixmap::new(px, px)?;
        let scale = px as f32 / tree.size().width().max(tree.size().height());
        resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
        Some(ColorImage::from_rgba_premultiplied([px as usize, px as usize], pixmap.data()))
    }

    /// The art as a window and taskbar icon (straight alpha), for
    /// `ViewportBuilder::with_icon`.
    #[must_use]
    pub fn window_icon(self, px: u32) -> Option<IconData> {
        let image = self.rasterize(px)?;
        Some(IconData {
            width: px,
            height: px,
            rgba: image.pixels.iter().flat_map(Color32::to_srgba_unmultiplied).collect(),
        })
    }
}

#[derive(Clone, Default)]
struct Cache(HashMap<(&'static str, u32), Option<TextureHandle>>);

/// The texture for `svg` drawn `points` wide, rendered once per pixel size.
fn texture(ctx: &Context, svg: Svg, points: f32) -> Option<TextureHandle> {
    let px = ((points * ctx.pixels_per_point()).round() as u32).clamp(4, 2048);
    let id = Id::unique("spaceui-art-cache");
    if let Some(found) = ctx.data_mut(|d| d.get_temp_mut_or_default::<Cache>(id).0.get(&(svg.name, px)).cloned()) {
        return found;
    }
    let handle =
        svg.rasterize(px).map(|image| ctx.load_texture(format!("{}@{px}", svg.name), image, TextureOptions::LINEAR));
    ctx.data_mut(|d| d.get_temp_mut_or_default::<Cache>(id).0.insert((svg.name, px), handle.clone()));
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
}
