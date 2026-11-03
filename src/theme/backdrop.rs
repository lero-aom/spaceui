use super::{Backdrop, Palette, Theme};
use crate::paint;
use egui::{Color32, ColorImage, Context, Id, LayerId, Painter, Rect, TextureHandle, TextureOptions, pos2, vec2};

/// Paints the installed theme's backdrop on the background layer, under
/// every panel. [`Shell::show`](crate::Shell::show) calls it for you.
pub fn paint_backdrop(ctx: &Context) {
    let theme = Theme::of(ctx);
    let p = &theme.palette;
    let painter = ctx.layer_painter(LayerId::background());
    let rect = ctx.content_rect();
    paint::gradient(&painter, rect, p.bg_top, p.bg_bottom, 0.0);
    match theme.backdrop {
        Backdrop::Plain => {}
        Backdrop::Carbon { weave, bloom } => {
            const TILE: f32 = 32.0;
            let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(rect.width() / TILE, rect.height() / TILE));
            painter.image(weave_texture(ctx).id(), rect, uv, Color32::from_white_alpha(weave));
            paint::radial_glow(&painter, rect.left_top() + vec2(120.0, 40.0), 520.0, p.accent, bloom);
        }
        Backdrop::Starfield => starfield(&painter, rect, p),
    }
}

/// A 2x2 twill of carbon tows, eight pixels each, as a repeating texture.
fn weave_texture(ctx: &Context) -> TextureHandle {
    const TOW: usize = 8;
    let id = Id::unique("spaceui-carbon-weave");
    if let Some(texture) = ctx.data(|d| d.get_temp::<TextureHandle>(id)) {
        return texture;
    }
    let size = TOW * 4;
    let mut pixels = Vec::with_capacity(size * size);
    for y in 0..size {
        for x in 0..size {
            // Twill: each tow is offset one step per row of tows.
            let (column, row) = (x / TOW, y / TOW);
            let horizontal = (column + 4 - row % 4) % 4 < 2;
            let along = if horizontal { y % TOW } else { x % TOW } as f32 + 0.5;
            let across = (along / TOW as f32 * std::f32::consts::PI).sin();
            let shade = 0.25 + 0.75 * across * across;
            let value = if horizontal { shade } else { shade * 0.55 };
            pixels.push(Color32::from_white_alpha((value * 255.0) as u8));
        }
    }
    let texture =
        ctx.load_texture("spaceui-carbon-weave", ColorImage::new([size, size], pixels), TextureOptions::LINEAR_REPEAT);
    ctx.data_mut(|d| d.insert_temp(id, texture.clone()));
    texture
}

/// Two nebula glows in the accent's shade and a deterministic starfield, so
/// the stars hold still.
fn starfield(painter: &Painter, rect: Rect, p: &Palette) {
    let nebulae = [(0.18, 0.12, 420.0, 0.7), (0.82, 0.92, 520.0, 0.8)];
    for (fx, fy, radius, depth) in nebulae {
        let color = paint::mix(p.accent_lo, p.bg_top, depth);
        let center = pos2(rect.left() + rect.width() * fx, rect.top() + rect.height() * fy);
        paint::radial_glow(painter, center, radius, color, 26);
    }
    let mut seed: u32 = 0x9E37_79B9;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        seed as f32 / u32::MAX as f32
    };
    let count = ((rect.area() / 9000.0) as usize).clamp(60, 320);
    for _ in 0..count {
        let at = pos2(rect.left() + next() * rect.width(), rect.top() + next() * rect.height());
        let size = 0.35 + next().powf(3.0) * 1.25;
        let alpha = 0.25 + next() * 0.55;
        painter.circle_filled(at, size, paint::mix(Color32::WHITE, p.accent_hi, next()).gamma_multiply(alpha));
    }
}
