use super::Tone;
use crate::{Theme, paint, theme::semibold};
use egui::{Color32, Frame, Painter, Rect, RichText, Shape, Stroke, StrokeKind, Ui};

/// Paints the plate a card or column sits on: a gently graded panel, a
/// hairline border and a faint top sheen. [`Shell`](crate::Shell) paints
/// these for its main card and side column.
pub fn plate(painter: &Painter, rect: Rect) {
    let theme = Theme::of(painter.ctx());
    for shape in plate_shapes(rect, theme.metrics.card_radius, &theme, painter.pixels_per_point()) {
        painter.add(shape);
    }
}

fn plate_shapes(rect: Rect, radius: f32, theme: &Theme, pixels_per_point: f32) -> [Shape; 3] {
    let p = &theme.palette;
    [
        Shape::mesh(paint::gradient_mesh(rect, p.panel_hi, p.panel, radius, pixels_per_point)),
        Shape::rect_stroke(rect, radius, Stroke::new(1.0, p.border_soft), StrokeKind::Inside),
        Shape::hline(
            (rect.left() + radius)..=(rect.right() - radius),
            rect.top() + 1.0,
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
        ui.label(RichText::new(number).color(p.accent).size(11.0).strong());
        ui.label(RichText::new(title).font(semibold(15.0)).color(p.text));
    });
}

/// Secondary explanatory text.
pub fn caption(ui: &mut Ui, text: impl Into<String>) {
    let p = Theme::of(ui.ctx()).palette;
    ui.label(RichText::new(text.into()).color(p.muted).size(12.0));
}

/// A card with a caption, a large value in `tone` and a detail line.
pub fn metric(ui: &mut Ui, label: &str, value: &str, detail: &str, tone: Tone) {
    let p = Theme::of(ui.ctx()).palette;
    card(ui, |ui| {
        caption(ui, label);
        ui.label(RichText::new(value).size(25.0).color(tone.color(&p)));
        ui.label(RichText::new(detail).size(10.5).color(p.muted));
    });
}
