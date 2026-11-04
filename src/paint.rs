//! Painting that egui's shapes don't cover: gradients, star-shaped fills and
//! glows.
//!
//! Hand-built meshes get no anti-aliasing from egui, so every fill here adds
//! the same one-physical-pixel feathered edge egui's tessellator puts around
//! its own shapes. Edges stay smooth at any display scale.
//!
//! ```
//! use spaceui::{egui, paint};
//!
//! # let ctx = egui::Context::default();
//! # ctx.run_ui(Default::default(), |ui| {
//! let rect = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(120.0, 32.0));
//! // A pill: the radius is half the height.
//! paint::gradient(ui.painter(), rect, egui::Color32::LIGHT_BLUE, egui::Color32::DARK_BLUE, 16.0);
//! # }).drop_without_applying_deltas();
//! ```
use egui::{Color32, CornerRadius, Mesh, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2, pos2, vec2};

/// Linear blend of two (premultiplied) colours; `t` is clamped to `0..=1`.
/// NaN selects the first colour.
#[must_use]
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
    let lerp = |from: u8, to: u8| (f32::from(from) + (f32::from(to) - f32::from(from)) * t).round() as u8;
    Color32::from_rgba_premultiplied(lerp(a.r(), b.r()), lerp(a.g(), b.g()), lerp(a.b(), b.b()), lerp(a.a(), b.a()))
}

/// Points around a rounded rectangle, clockwise from the top-left corner,
/// with enough segments per corner that arcs stay round at `pixels_per_point`:
/// a chord strays at most about 0.1 px from the true arc.
/// Empty, non-finite or overflowing geometry produces no points. Negative
/// radii are clamped to zero; display density must be finite and positive.
#[must_use]
pub fn rounded_rect_points(rect: Rect, radius: f32, pixels_per_point: f32) -> Vec<Pos2> {
    if !rect.is_finite()
        || !rect.size().is_finite()
        || !rect.is_positive()
        || !radius.is_finite()
        || !pixels_per_point.is_finite()
        || pixels_per_point <= 0.0
    {
        return Vec::new();
    }
    let radius = radius.min(rect.width() / 2.0).min(rect.height() / 2.0).max(0.0);
    let corners = [
        (pos2(rect.left() + radius, rect.top() + radius), 180.0_f32),
        (pos2(rect.right() - radius, rect.top() + radius), 270.0),
        (pos2(rect.right() - radius, rect.bottom() - radius), 0.0),
        (pos2(rect.left() + radius, rect.bottom() - radius), 90.0),
    ];
    let pixels = radius * pixels_per_point;
    let segments = if pixels < 0.5 {
        0
    } else {
        (std::f32::consts::FRAC_PI_2 / (0.8 / pixels).sqrt()).ceil().clamp(2.0, 48.0) as usize
    };
    let mut points = Vec::with_capacity(4 * (segments + 1));
    for (center, start) in corners {
        for i in 0..=segments {
            let angle = (start + 90.0 * i as f32 / segments.max(1) as f32).to_radians();
            points.push(center + vec2(angle.cos(), angle.sin()) * radius);
        }
    }
    points.dedup_by(|a, b| (*a - *b).length_sq() < 1e-4);
    points
}

/// A filled polygon (convex, or visible from its centroid) with an
/// anti-aliased edge `feather` points wide (use `1.0 / pixels_per_point`).
/// `color_at` colours each vertex; colours linear in position give exact
/// gradients. Invalid/degenerate geometry or a negative/non-finite feather
/// produces an empty mesh. Repeated adjacent vertices are ignored.
#[must_use]
pub fn polygon_mesh(points: &[Pos2], feather: f32, color_at: impl Fn(Pos2) -> Color32) -> Mesh {
    let mut mesh = Mesh::default();
    let n = points.len();
    if n < 3
        || n > (u32::MAX as usize - 1) / 2
        || !feather.is_finite()
        || feather < 0.0
        || points.iter().any(|p| !p.is_finite())
    {
        return mesh;
    }
    if (0..n).any(|i| (points[i] - points[(i + 1) % n]).length_sq() < 1e-12) {
        let mut clean = points.to_vec();
        clean.dedup_by(|a, b| (*a - *b).length_sq() < 1e-12);
        while clean.len() > 1 && (clean[0] - clean[clean.len() - 1]).length_sq() < 1e-12 {
            clean.pop();
        }
        return polygon_mesh(&clean, feather, color_at);
    }
    // Translate before multiplying: absolute coordinates can cancel away the
    // area of small polygons far from the origin, even at ordinary UI sizes.
    let origin = points[0];
    let area: f32 = (0..n)
        .map(|i| {
            let a = points[i] - origin;
            let b = points[(i + 1) % n] - origin;
            a.x * b.y - b.x * a.y
        })
        .sum();
    if !area.is_finite() || area.abs() < 1e-12 {
        return mesh;
    }
    let outward = |from: Pos2, to: Pos2| {
        let along = (to - from).normalized();
        if area > 0.0 { vec2(along.y, -along.x) } else { vec2(-along.y, along.x) }
    };
    let center = (points.iter().fold(Vec2::ZERO, |sum, p| sum + p.to_vec2()) / n as f32).to_pos2();
    if !center.is_finite() {
        return mesh;
    }
    mesh.colored_vertex(center, color_at(center));
    for i in 0..n {
        let (prev, point, next) = (points[(i + n - 1) % n], points[i], points[(i + 1) % n]);
        let (before, after) = (outward(prev, point), outward(point, next));
        let bisector = (before + after).normalized();
        // Mitred so the fringe keeps its width; capped so sharp tips don't spike.
        let miter = bisector / bisector.dot(before).max(0.35);
        mesh.colored_vertex(point - miter * (feather * 0.5), color_at(point));
        mesh.colored_vertex(point + miter * (feather * 0.5), Color32::TRANSPARENT);
    }
    for i in 0..n as u32 {
        let j = (i + 1) % n as u32;
        let (inner_i, outer_i, inner_j, outer_j) = (1 + 2 * i, 2 + 2 * i, 1 + 2 * j, 2 + 2 * j);
        mesh.add_triangle(0, inner_i, inner_j);
        mesh.add_triangle(inner_i, outer_i, outer_j);
        mesh.add_triangle(inner_i, outer_j, inner_j);
    }
    if mesh.vertices.iter().any(|vertex| !vertex.pos.is_finite()) { Mesh::default() } else { mesh }
}

/// Fills a polygon in one colour with an anti-aliased edge.
pub fn fill_polygon(painter: &Painter, points: &[Pos2], color: Color32) {
    painter.add(polygon_mesh(points, 1.0 / painter.pixels_per_point(), |_| color));
}

/// A vertically graded rounded rectangle, for shapes built before painting
/// (such as a background placed behind content with `Painter::set`).
#[must_use]
pub fn gradient_mesh(rect: Rect, top: Color32, bottom: Color32, radius: f32, pixels_per_point: f32) -> Mesh {
    let points = rounded_rect_points(rect, radius, pixels_per_point);
    polygon_mesh(&points, 1.0 / pixels_per_point, |p| mix(top, bottom, (p.y - rect.top()) / rect.height().max(1e-3)))
}

/// Paints a vertically graded rounded rectangle. A radius of half the height
/// makes a pill; of half the side of a square, a circle.
pub fn gradient(painter: &Painter, rect: Rect, top: Color32, bottom: Color32, radius: f32) {
    painter.add(gradient_mesh(rect, top, bottom, radius, painter.pixels_per_point()));
}

/// Paints a horizontally graded rounded rectangle.
pub fn gradient_h(painter: &Painter, rect: Rect, left: Color32, right: Color32, radius: f32) {
    let pixels_per_point = painter.pixels_per_point();
    let points = rounded_rect_points(rect, radius, pixels_per_point);
    painter.add(polygon_mesh(&points, 1.0 / pixels_per_point, |p| {
        mix(left, right, (p.x - rect.left()) / rect.width().max(1e-3))
    }));
}

/// A soft disc fading from `color` at `alpha` in the centre to nothing.
/// `alpha` replaces the colour's opacity while preserving its unmultiplied RGB.
/// Non-finite/overflowing geometry or a non-positive radius paints nothing.
pub fn radial_glow(painter: &Painter, center: Pos2, radius: f32, color: Color32, alpha: u8) {
    const SEGMENTS: u32 = 48;
    if !center.is_finite() || !radius.is_finite() || radius <= 0.0 {
        return;
    }
    let [red, green, blue, _] = color.to_srgba_unmultiplied();
    let inner = Color32::from_rgba_unmultiplied(red, green, blue, alpha);
    let mut mesh = Mesh::default();
    mesh.colored_vertex(center, inner);
    for i in 0..SEGMENTS {
        let angle = i as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
        mesh.colored_vertex(center + vec2(angle.cos(), angle.sin()) * radius, Color32::TRANSPARENT);
    }
    for i in 0..SEGMENTS {
        mesh.add_triangle(0, 1 + i, 1 + (i + 1) % SEGMENTS);
    }
    if mesh.vertices.iter().all(|vertex| vertex.pos.is_finite()) {
        painter.add(mesh);
    }
}

/// A soft outer glow around a rounded rectangle, from stacked strokes.
/// Empty/non-finite/overflowing geometry or negative radii/spreads paint nothing.
pub fn glow(painter: &Painter, rect: Rect, radius: f32, color: Color32, spread: f32) {
    const STEPS: u8 = 6;
    if !rect.is_finite()
        || !rect.is_positive()
        || !rect.size().is_finite()
        || !radius.is_finite()
        || radius < 0.0
        || !spread.is_finite()
        || spread < 0.0
        || !(radius + spread).is_finite()
        || !rect.expand(spread).is_finite()
        || !rect.expand(spread).size().is_finite()
    {
        return;
    }
    for step in 0..STEPS {
        let t = f32::from(step) / f32::from(STEPS);
        let grow = spread * (1.0 - t);
        painter.rect_stroke(
            rect.expand(grow),
            CornerRadius::from(radius + grow),
            Stroke::new(2.0, color.gamma_multiply(0.10 * t * t + 0.02)),
            StrokeKind::Outside,
        );
    }
}

/// A four-pointed sparkle (the "thinking" glyph) with curved flanks.
/// Non-finite geometry or a non-positive radius paints nothing.
pub fn sparkle(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
    const STEPS: usize = 12;
    if !center.is_finite() || !radius.is_finite() || radius <= 0.0 {
        return;
    }
    let tips = [vec2(0.0, -radius), vec2(radius, 0.0), vec2(0.0, radius), vec2(-radius, 0.0)];
    let pull = radius * 0.12;
    let mut points = Vec::with_capacity(4 * STEPS);
    for quarter in 0..4 {
        let (from, to) = (tips[quarter], tips[(quarter + 1) % 4]);
        let control = (from + to).normalized() * pull;
        for step in 0..STEPS {
            let t = step as f32 / STEPS as f32;
            points.push(center + from * (1.0 - t) * (1.0 - t) + control * 2.0 * t * (1.0 - t) + to * t * t);
        }
    }
    fill_polygon(painter, &points, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_fade_out_over_one_physical_pixel_at_their_edge() {
        let rect = Rect::from_min_size(pos2(10.0, 20.0), vec2(48.0, 48.0));
        let pixels_per_point = 1.25;
        let mesh = gradient_mesh(rect, Color32::WHITE, Color32::BLACK, 14.0, pixels_per_point);
        let ring = &mesh.vertices[1..];
        assert!(ring.len() / 2 > 4 * 6, "corners need enough segments to look round");
        for pair in ring.chunks(2) {
            let (inner, outer) = (&pair[0], &pair[1]);
            assert_eq!(inner.color.a(), 255);
            assert_eq!(outer.color, Color32::TRANSPARENT);
            let width = (outer.pos - inner.pos).length();
            assert!(width >= 1.0 / pixels_per_point - 1e-3 && width < 3.0 / pixels_per_point, "{width}");
            assert!(rect.expand(1.0 / pixels_per_point).contains(outer.pos));
        }
        assert_eq!(mesh.indices.len(), ring.len() / 2 * 9);
    }

    #[test]
    fn gradients_run_top_to_bottom() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(10.0, 100.0));
        let mesh = gradient_mesh(rect, Color32::WHITE, Color32::BLACK, 0.0, 1.0);
        let top = mesh.vertices.iter().find(|v| v.pos.y < 1.0 && v.color.a() == 255).expect("a top vertex");
        let bottom = mesh.vertices.iter().find(|v| v.pos.y > 99.0 && v.color.a() == 255).expect("a bottom vertex");
        assert!(top.color.r() > 250 && bottom.color.r() < 5);
    }

    #[test]
    fn degenerate_polygons_paint_nothing() {
        assert!(polygon_mesh(&[pos2(0.0, 0.0), pos2(1.0, 1.0)], 1.0, |_| Color32::WHITE).is_empty());
    }

    #[test]
    fn translating_small_polygons_preserves_their_fill_and_winding() {
        let square = [pos2(0.0, 0.0), pos2(1.0, 0.0), pos2(1.0, 1.0), pos2(0.0, 1.0)];
        let original = polygon_mesh(&square, 1.0, |_| Color32::WHITE);
        for translation in [Vec2::splat(10_000.0), Vec2::splat(-10_000.0)] {
            for clockwise in [true, false] {
                let mut points = square.map(|point| point + translation);
                if !clockwise {
                    points.reverse();
                }
                let mesh = polygon_mesh(&points, 1.0, |_| Color32::WHITE);
                assert_eq!(mesh.vertices.len(), original.vertices.len());
                assert_eq!(mesh.indices.len(), original.indices.len());
                for pair in mesh.vertices[1..].chunks_exact(2) {
                    let inner = pair[0].pos - translation;
                    let outer = pair[1].pos - translation;
                    assert!((inner - pos2(0.5, 0.5)).length() < (outer - pos2(0.5, 0.5)).length());
                }
            }
        }
    }

    #[test]
    fn radial_glow_does_not_apply_input_alpha_twice() {
        let ctx = egui::Context::default();
        let color = Color32::from_rgba_unmultiplied(255, 0, 0, 128);
        let painter = ctx.layer_painter(egui::LayerId::background());
        ctx.begin_pass(egui::RawInput::default());
        radial_glow(&painter, pos2(50.0, 50.0), 10.0, color, 128);
        let output = ctx.end_pass();
        let center = output.shapes.iter().find_map(|clipped| match &clipped.shape {
            egui::Shape::Mesh(mesh) => Some(mesh.vertices[0].color),
            _ => None,
        });
        assert_eq!(center, Some(color));
        output.drop_without_applying_deltas();
    }

    #[test]
    fn geometry_invariants_hold_across_sizes_radii_and_densities() {
        for width in [0.0, 0.01, 1.0, 32.0, 800.0] {
            for height in [0.0, 0.01, 7.0, 100.0] {
                for radius in [-2.0, 0.0, 0.1, 8.0, 1000.0] {
                    for dpi in [0.75, 1.0, 1.25, 2.0, 4.0] {
                        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(width, height));
                        let mesh = gradient_mesh(rect, Color32::WHITE, Color32::BLACK, radius, dpi);
                        assert!(mesh.vertices.iter().all(|vertex| vertex.pos.is_finite()));
                        assert!(mesh.indices.iter().all(|index| (*index as usize) < mesh.vertices.len()));
                        assert_eq!(mesh.indices.len() % 3, 0);
                    }
                }
            }
        }
    }

    #[test]
    fn invalid_and_repeated_geometry_is_safe() {
        let square = [pos2(0.0, 0.0), pos2(1.0, 0.0), pos2(1.0, 0.0), pos2(1.0, 1.0), pos2(0.0, 1.0), pos2(0.0, 0.0)];
        let mesh = polygon_mesh(&square, 1.0, |_| Color32::WHITE);
        assert!(!mesh.is_empty());
        assert!(mesh.vertices.iter().all(|v| v.pos.is_finite()));
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -1.0] {
            assert!(
                gradient_mesh(
                    Rect::from_min_size(Pos2::ZERO, Vec2::splat(10.0)),
                    Color32::WHITE,
                    Color32::BLACK,
                    2.0,
                    invalid
                )
                .is_empty()
            );
        }
    }

    #[test]
    fn a_long_degenerate_closing_tail_is_cleaned_without_recursive_growth() {
        let mut points = vec![Pos2::ZERO, pos2(0.0, 10.0)];
        points.extend((0..10_000).map(|i| pos2(if i % 2 == 0 { 0.9e-6 } else { -0.9e-6 }, 0.0)));
        assert!(polygon_mesh(&points, 1.0, |_| Color32::WHITE).is_empty());
    }

    #[test]
    fn finite_coordinates_with_overflowing_extents_are_rejected() {
        let rect = Rect::from_min_max(pos2(-f32::MAX, -f32::MAX), pos2(f32::MAX, f32::MAX));
        assert_eq!(rounded_rect_points(rect, f32::MAX, 2.0), [] as [Pos2; 0]);
        assert!(gradient_mesh(rect, Color32::WHITE, Color32::BLACK, f32::MAX, 2.0).is_empty());
    }

    #[test]
    fn decorative_painters_ignore_invalid_and_overflowing_geometry() {
        let ctx = egui::Context::default();
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let painter = ui.painter();
            let rect = Rect::from_min_size(Pos2::ZERO, Vec2::splat(10.0));
            for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0] {
                radial_glow(painter, Pos2::ZERO, invalid, Color32::WHITE, 128);
                glow(painter, rect, invalid, Color32::WHITE, 2.0);
                glow(painter, rect, 2.0, Color32::WHITE, invalid);
                sparkle(painter, Pos2::ZERO, invalid, Color32::WHITE);
            }
            radial_glow(painter, pos2(f32::MAX, f32::MAX), f32::MAX, Color32::WHITE, 128);
            glow(painter, rect, 2.0, Color32::WHITE, f32::MAX);
        });
        assert!(output.shapes.iter().all(|clipped| matches!(clipped.shape, egui::Shape::Noop)));
        output.drop_without_applying_deltas();
    }
}
