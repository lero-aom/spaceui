use crate::{Theme, paint, theme::Palette};
use egui::{Color32, CursorIcon, Painter, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

/// A slider over `count` notches (effort levels, say): an accent gel fills up
/// to a round cap. Click or drag to pick a notch; `index` is clamped to the
/// notches and updated in place.
pub fn notches(ui: &mut Ui, width: f32, count: usize, index: &mut usize) -> Response {
    let p = Theme::of(ui.ctx()).palette;
    let (rect, response) = ui.allocate_exact_size(vec2(width, 20.0), Sense::click_and_drag());
    let (x0, x1) = (rect.left() + 7.0, rect.right() - 7.0);
    let steps = count.saturating_sub(1).max(1) as f32;
    let x_of = |k: usize| x0 + (x1 - x0) * k as f32 / steps;
    if (response.clicked() || response.dragged())
        && let Some(pointer) = response.interact_pointer_pos()
    {
        *index = (((pointer.x - x0) / (x1 - x0)).clamp(0.0, 1.0) * steps).round() as usize;
    }
    *index = (*index).min(count.saturating_sub(1));
    let active = response.hovered() || response.dragged();
    if active {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    let painter = ui.painter_at(rect.expand(4.0));
    let y = rect.center().y;
    let slot = Rect::from_min_max(pos2(x0 - 4.0, y - 3.0), pos2(x1 + 4.0, y + 3.0));
    well(&painter, slot, &p);
    gel(&painter, Rect::from_min_max(slot.min, pos2(x_of(*index) + 3.0, slot.bottom())), 3.0, &p);
    for k in 0..count {
        let color = if k <= *index { Color32::from_black_alpha(110) } else { p.muted.gamma_multiply(0.7) };
        painter.circle_filled(pos2(x_of(k), y), 1.5, color);
    }
    let cap = Rect::from_center_size(pos2(x_of(*index), y), vec2(14.0, 14.0));
    if active {
        painter.circle_stroke(cap.center(), 8.5, Stroke::new(3.0, p.accent.gamma_multiply(0.3)));
    }
    gel(&painter, cap, 7.0, &p);
    response
}

/// An accent gel: graded body, a soft sheen on the upper half, a dark edge.
fn gel(painter: &Painter, rect: Rect, radius: f32, p: &Palette) {
    paint::gradient(painter, rect, p.accent_hi, p.accent_lo, radius);
    let sheen = Rect::from_min_max(rect.min + vec2(1.0, 1.0), pos2(rect.max.x - 1.0, rect.center().y));
    let (glint, fade) = (Color32::from_white_alpha(46), Color32::from_white_alpha(4));
    paint::gradient(painter, sheen, glint, fade, (radius - 1.0).max(0.5));
    painter.rect_stroke(rect, radius, Stroke::new(1.0, Color32::from_black_alpha(150)), StrokeKind::Inside);
}

/// A recessed slot.
fn well(painter: &Painter, rect: Rect, p: &Palette) {
    paint::gradient(painter, rect, paint::mix(p.well, Color32::BLACK, 0.5), p.well, 3.0);
    painter.rect_stroke(rect, 3.0, Stroke::new(1.0, p.border_soft), StrokeKind::Inside);
}
