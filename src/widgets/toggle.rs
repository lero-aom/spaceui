use crate::{Theme, paint};
use egui::{Color32, CursorIcon, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, WidgetInfo, lerp, pos2, vec2};

/// An on/off widget with caller-owned state, usable with [`Ui::add_enabled`].
/// The accessible label is not painted; pair it with visible text when needed.
#[derive(Debug)]
#[must_use = "add the switch to a Ui"]
pub struct Switch<'a> {
    on: &'a mut bool,
    label: &'a str,
}

impl<'a> Switch<'a> {
    /// Bind a switch to application state.
    pub fn new(on: &'a mut bool) -> Self {
        Self { on, label: "" }
    }

    /// Set an accessible name without adding visible text.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }
}

impl egui::Widget for Switch<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        show(ui, self.on, self.label)
    }
}

/// An on/off switch with an eased knob. Flips `on` when activated and marks
/// the response changed. Name it with [`Response::labelled_by`] or use
/// [`switch_labeled`] when no visible label accompanies it.
pub fn switch(ui: &mut Ui, on: &mut bool) -> Response {
    ui.add(Switch::new(on))
}

/// A switch with an accessible name; the label is not painted.
pub fn switch_labeled(ui: &mut Ui, on: &mut bool, label: &str) -> Response {
    ui.add(Switch::new(on).label(label))
}

fn show(ui: &mut Ui, on: &mut bool, label: &str) -> Response {
    let p = Theme::of(ui.ctx()).palette;
    let (rect, mut response) = ui.allocate_exact_size(vec2(44.0, 24.0), Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| WidgetInfo::selected(Role::Switch, ui.is_enabled(), *on, label));
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
    super::focus_ring(ui, &response, 12.0);
    if ui.is_enabled() { response.on_hover_cursor(CursorIcon::PointingHand) } else { response }
}
