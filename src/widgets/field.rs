use crate::Theme;
use egui::{Align2, Frame, Response, RichText, Sense, Stroke, StrokeKind, TextEdit, Ui, vec2};

/// A single-line text field: a recessed well whose border turns accent while
/// focused, showing `hint` while empty. Fills the available width and returns
/// the text edit's response (`.changed()`, `.lost_focus()`, ...).
pub fn text_field(ui: &mut Ui, text: &mut String, hint: &str) -> Response {
    let theme = Theme::of(ui.ctx());
    let p = &theme.palette;
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::hover());
    let id = ui.scope_id().with(("spaceui-field", rect.min.x as i32, rect.min.y as i32));
    let focused = ui.memory(|m| m.has_focus(id));
    ui.painter().rect(
        rect,
        theme.metrics.control_radius + 2.0,
        p.well,
        Stroke::new(1.0, if focused { p.accent } else { p.border }),
        StrokeKind::Inside,
    );
    let inner = rect.shrink2(vec2(12.0, 0.0));
    ui.put(
        inner,
        TextEdit::singleline(text)
            .id(id)
            .frame(Frame::NONE)
            .text_color(p.text)
            .hint_text(RichText::new(hint).color(p.muted))
            .align(Align2::LEFT_CENTER)
            .desired_width(inner.width()),
    )
}
