use super::Tone;
use crate::{Theme, paint, theme::semibold};
use egui::{Color32, Frame, Painter, Rect, RichText, Shape, Stroke, StrokeKind, Ui};

/// Paints the plate a card or column sits on: a gently graded panel, a
/// hairline border and a faint top sheen. The shell paints
/// these for its main card and side column.
/// Empty, non-finite or overflowing rectangles and non-finite theme radii
/// paint nothing. Finite radii are clamped to fit the rectangle.
pub fn plate(painter: &Painter, rect: Rect) {
    let theme = Theme::of(painter.ctx());
    for shape in plate_shapes(rect, theme.metrics.card_radius, &theme, painter.pixels_per_point()) {
        painter.add(shape);
    }
}

fn plate_shapes(rect: Rect, radius: f32, theme: &Theme, pixels_per_point: f32) -> [Shape; 3] {
    if !rect.is_finite() || !rect.size().is_finite() || !rect.is_positive() || !radius.is_finite() {
        return std::array::from_fn(|_| Shape::Noop);
    }
    let radius = radius.max(0.0).min(rect.width() * 0.5).min(rect.height() * 0.5);
    let p = &theme.palette;
    [
        Shape::mesh(paint::gradient_mesh(rect, p.panel_hi, p.panel, radius, pixels_per_point)),
        Shape::rect_stroke(rect, radius, Stroke::new(1.0, p.border_soft), StrokeKind::Inside),
        Shape::hline(
            (rect.left() + radius)..=(rect.right() - radius),
            rect.top() + 1.0_f32.min(rect.height() * 0.5),
            Stroke::new(1.0, Color32::from_white_alpha(10)),
        ),
    ]
}

/// Content on a card sized to the content, with the theme's inner radius.
/// Returns what `add` returns.
///
/// ```
/// # use spaceui::{Theme, egui, widgets};
/// # let ctx = egui::Context::default();
/// # Theme::carbon_copper().install(&ctx);
/// # ctx.run_ui(Default::default(), |ui| {
/// widgets::card(ui, |ui| {
///     widgets::section(ui, "01", "Interactive Brokers");
///     widgets::caption(ui, "Data access only.");
/// });
/// # }).drop_without_applying_deltas();
/// ```
pub fn card<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    let theme = Theme::of(ui.ctx());
    // Reserve the background's place so it paints under the content.
    let background = ui.painter().add(Shape::Noop);
    let response = Frame::new().inner_margin(14).show(ui, add);
    let shapes = plate_shapes(response.response.rect, theme.metrics.inner_radius(), &theme, ui.pixels_per_point());
    ui.painter().set(background, Shape::Vec(shapes.into()));
    response.inner
}

/// A numbered section heading ("01  Interactive Brokers").
pub fn section(ui: &mut Ui, number: &str, title: &str) {
    let p = Theme::of(ui.ctx()).palette;
    ui.horizontal(|ui| {
        ui.label(RichText::new(number).color(p.accent).size(super::text_size(ui, 11.0)).strong());
        ui.label(RichText::new(title).font(semibold(super::text_size(ui, 15.0))).color(p.text));
    });
}

/// Secondary explanatory text.
pub fn caption(ui: &mut Ui, text: impl Into<String>) {
    let p = Theme::of(ui.ctx()).palette;
    ui.label(RichText::new(text.into()).color(p.muted).size(super::text_size(ui, 12.0)));
}

/// A card with a caption, a large value in `tone` and a detail line.
pub fn metric(ui: &mut Ui, label: &str, value: &str, detail: &str, tone: Tone) {
    let p = Theme::of(ui.ctx()).palette;
    card(ui, |ui| {
        caption(ui, label);
        ui.label(RichText::new(value).size(super::text_size(ui, 25.0)).color(tone.color(&p)));
        ui.label(RichText::new(detail).size(super::text_size(ui, 10.5)).color(p.muted));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    #[test]
    fn invalid_plate_geometry_emits_no_shapes() {
        let theme = Theme::default();
        for rect in [
            Rect::NOTHING,
            Rect::EVERYTHING,
            Rect::from_min_size(pos2(10.0, 10.0), vec2(0.0, 20.0)),
            Rect::from_min_max(pos2(20.0, 20.0), pos2(10.0, 10.0)),
            Rect::from_min_max(pos2(f32::NAN, 0.0), pos2(20.0, 20.0)),
            Rect::from_min_max(pos2(-f32::MAX, 0.0), pos2(f32::MAX, 20.0)),
        ] {
            assert!(plate_shapes(rect, 8.0, &theme, 1.0).iter().all(|shape| matches!(shape, Shape::Noop)));
        }
        let rect = Rect::from_min_size(pos2(10.0, 10.0), vec2(100.0, 40.0));
        for radius in [f32::NAN, f32::NEG_INFINITY, f32::INFINITY] {
            assert!(plate_shapes(rect, radius, &theme, 1.0).iter().all(|shape| matches!(shape, Shape::Noop)));
        }
    }

    #[test]
    fn narrow_plate_sheen_stays_ordered_and_inside_the_rectangle() {
        let rect = Rect::from_min_size(pos2(10.0, 10.0), vec2(3.0, 0.5));
        for radius in [-10.0, 16.0, f32::MAX] {
            let shapes = plate_shapes(rect, radius, &Theme::default(), 1.0);
            let Shape::LineSegment { points, .. } = &shapes[2] else {
                panic!("a valid plate has a sheen line");
            };
            assert!(points[0].x <= points[1].x, "the sheen must have ordered endpoints");
            assert!(points.iter().all(|point| rect.contains(*point)), "the sheen must fit the plate");
        }
    }
}
