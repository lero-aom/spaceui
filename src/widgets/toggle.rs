use crate::{Theme, paint};
use egui::{Color32, CursorIcon, Rect, Response, Sense, Stroke, StrokeKind, Ui, lerp, pos2, vec2};

/// An on/off switch: an accent track when on, a carbon one when off, with an
/// eased knob. Flips `on` when clicked and marks the response changed.
pub fn switch(ui: &mut Ui, on: &mut bool) -> Response {
    let p = Theme::of(ui.ctx()).palette;
    let (rect, mut response) = ui.allocate_exact_size(vec2(44.0, 24.0), Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let t = ui.ctx().animate_bool(response.id, *on);
    let painter = ui.painter();
    painter.rect(rect, 12.0, p.field, Stroke::new(1.0, p.border), StrokeKind::Inside);
    if t > 0.0 {
        let fill = Rect::from_min_max(rect.min, pos2(lerp(rect.left() + 24.0..=rect.right(), t), rect.bottom()));
        paint::gradient(painter, fill, p.accent_hi.gamma_multiply(t), p.accent_lo.gamma_multiply(t), 12.0);
    }
    let x = lerp(rect.left() + 12.0..=rect.right() - 12.0, t);
    painter.circle_filled(pos2(x, rect.center().y + 1.0), 9.0, Color32::from_black_alpha(60));
    painter.circle_filled(pos2(x, rect.center().y), 8.5, p.text);
    response.on_hover_cursor(CursorIcon::PointingHand)
}
