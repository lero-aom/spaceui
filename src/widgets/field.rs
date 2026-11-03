use crate::Theme;
use egui::{Align2, Frame, Id, Response, RichText, Sense, Stroke, StrokeKind, TextEdit, Ui, vec2};

/// A single-line text field that fills the available width. Returns the text
/// edit's response (`.changed()`, `.lost_focus()`, ...). Identity follows its
/// position in the widget sequence, rather than its screen coordinates.
/// Use [`text_field_id`] when surrounding widgets are inserted or reordered.
pub fn text_field(ui: &mut Ui, text: &mut String, hint: &str) -> Response {
    let id = ui.next_auto_id().with("spaceui-field");
    show(ui, id, text, hint)
}

/// A text field with a stable, caller-supplied identity within the current UI
/// scope. Use a record or field key to preserve focus through layout changes.
pub fn text_field_id(
    ui: &mut Ui,
    id_salt: impl std::hash::Hash + std::fmt::Debug,
    text: &mut String,
    hint: &str,
) -> Response {
    let id = ui.scope_id().with(id_salt);
    show(ui, id, text, hint)
}

fn show(ui: &mut Ui, id: Id, text: &mut String, hint: &str) -> Response {
    let theme = Theme::of(ui.ctx());
    let p = &theme.palette;
    let height = 34.0_f32.max(super::text_size(ui, 14.0) + 20.0);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    let focused = ui.memory(|m| m.has_focus(id));
    ui.painter().rect(
        rect,
        theme.metrics.control_radius + 2.0,
        p.well,
        Stroke::new(if focused { 2.0 } else { 1.0 }, if focused { p.accent_hi } else { p.border }),
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
