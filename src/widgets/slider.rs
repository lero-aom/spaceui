use crate::{Theme, paint, theme::Palette};
use egui::accesskit::{Action, ActionData};
use egui::{
    Color32, CursorIcon, EventFilter, Key, Modifiers, Painter, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    WidgetInfo, pos2, vec2,
};

/// A slider over `count` notches (effort levels, say): an accent gel fills up
/// to a round cap. Click or drag to pick a notch; `index` is clamped to the
/// notches and updated in place. Arrow keys step by one; Home/End select the
/// first/last notch. A changed value, including initial clamping, marks the
/// response changed. Zero notches shows an empty track and resets the index
/// to zero; one notch is read-only. Non-finite widths use 160 points, and
/// widths below 16 points use 16. Name the control with `.labelled_by(...)`
/// or [`notches_labeled`]. Ranges above 512 notches paint representative
/// marks while retaining the full value range.
pub fn notches(ui: &mut Ui, width: f32, count: usize, index: &mut usize) -> Response {
    notches_labeled(ui, width, count, index, "")
}

/// A notched slider with an accessible name; the label is not painted.
/// Input normalization, change reporting and keyboard behavior match [`notches`].
pub fn notches_labeled(ui: &mut Ui, width: f32, count: usize, index: &mut usize, label: &str) -> Response {
    let p = Theme::of(ui.ctx()).palette;
    let width = if width.is_finite() { width.max(16.0) } else { 160.0 };
    let editable = ui.is_enabled() && count > 1;
    let sense = if editable { Sense::click_and_drag() } else { Sense::hover() };
    let (rect, mut response) = ui.allocate_exact_size(vec2(width, 20.0), sense);
    let (x0, x1) = (rect.left() + 7.0, rect.right() - 7.0);
    let steps = count.saturating_sub(1).max(1) as f32;
    let x_of = |k: usize| x0 + (x1 - x0) * k as f32 / steps;
    let old = *index;
    *index = (*index).min(count.saturating_sub(1));
    if editable
        && (response.clicked() || response.dragged())
        && let Some(pointer) = response.interact_pointer_pos()
    {
        *index = (((pointer.x - x0) / (x1 - x0)).clamp(0.0, 1.0) * steps).round() as usize;
        response.request_focus();
    }
    if editable {
        adjust(ui, &response, count - 1, index);
    }
    *index = (*index).min(count.saturating_sub(1));
    if old != *index {
        response.mark_changed();
    }
    response.widget_info(|| WidgetInfo::slider(editable, *index as f64, label));
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_min_numeric_value(0.0);
        node.set_max_numeric_value(count.saturating_sub(1) as f64);
        node.set_numeric_value_step(1.0);
        if editable {
            node.add_action(Action::SetValue);
            if *index > 0 {
                node.add_action(Action::Decrement);
            }
            if *index < count - 1 {
                node.add_action(Action::Increment);
            }
        }
    });
    let active = editable && (response.hovered() || response.dragged());
    if active {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    let painter = ui.painter_at(rect.expand(4.0));
    let y = rect.center().y;
    let slot = Rect::from_min_max(pos2(x0 - 4.0, y - 3.0), pos2(x1 + 4.0, y + 3.0));
    well(&painter, slot, &p);
    if count == 0 {
        return response;
    }
    gel(&painter, Rect::from_min_max(slot.min, pos2(x_of(*index) + 3.0, slot.bottom())), 3.0, &p);
    // Bound paint work even if an application supplies a very large range.
    let marks = count.min(512);
    for mark in 0..marks {
        let k = (mark as f64 / marks.saturating_sub(1).max(1) as f64 * count.saturating_sub(1) as f64).round() as usize;
        let color = if k <= *index { Color32::from_black_alpha(110) } else { p.muted.gamma_multiply(0.7) };
        painter.circle_filled(pos2(x_of(k), y), 1.5, color);
    }
    let cap = Rect::from_center_size(pos2(x_of(*index), y), vec2(14.0, 14.0));
    if active {
        painter.circle_stroke(cap.center(), 8.5, Stroke::new(3.0, p.accent.gamma_multiply(0.3)));
    }
    gel(&painter, cap, 7.0, &p);
    super::focus_ring(ui, &response, 4.0);
    response
}

fn adjust(ui: &Ui, response: &Response, max: usize, index: &mut usize) {
    if response.has_focus() {
        ui.memory_mut(|m| {
            m.set_focus_lock_filter(
                response.id,
                EventFilter { horizontal_arrows: true, vertical_arrows: true, ..Default::default() },
            );
        });
        ui.input_mut(|input| {
            let increment =
                input.consume_key(Modifiers::NONE, Key::ArrowRight) | input.consume_key(Modifiers::NONE, Key::ArrowUp);
            let decrement =
                input.consume_key(Modifiers::NONE, Key::ArrowLeft) | input.consume_key(Modifiers::NONE, Key::ArrowDown);
            if increment {
                *index = index.saturating_add(1).min(max);
            }
            if decrement {
                *index = index.saturating_sub(1);
            }
            if input.consume_key(Modifiers::NONE, Key::Home) {
                *index = 0;
            }
            if input.consume_key(Modifiers::NONE, Key::End) {
                *index = max;
            }
        });
    }
    ui.input(|input| {
        let increment = input.num_accesskit_action_requests(response.id, Action::Increment);
        let decrement = input.num_accesskit_action_requests(response.id, Action::Decrement);
        *index = index.saturating_add(increment).saturating_sub(decrement).min(max);
        for request in input.accesskit_action_requests(response.id, Action::SetValue) {
            if let Some(ActionData::NumericValue(value)) = request.data
                && value.is_finite()
            {
                *index = value.round().clamp(0.0, max as f64) as usize;
            }
        }
    });
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
