use crate::{Theme, theme::ControlMetrics};
use egui::{Align2, Frame, Id, Response, RichText, Sense, Stroke, StrokeKind, TextEdit, Ui, vec2};

/// A single-line field widget with caller-owned text, configurable width and
/// stable identity. Existing field helpers use this implementation.
#[derive(Debug)]
#[must_use = "add the text field to a Ui"]
pub struct TextField<'a> {
    text: &'a mut String,
    hint: &'a str,
    salt: Option<egui::IdSalt>,
    width: Option<f32>,
}

impl<'a> TextField<'a> {
    /// A field that fills the available width and uses its position as identity.
    pub fn new(text: &'a mut String) -> Self {
        Self { text, hint: "", salt: None, width: None }
    }

    /// Show a hint while empty. Supply an accessible name via the response.
    pub fn hint_text(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
    }

    /// Keep focus stable when sibling widgets are inserted or reordered.
    /// The salt is scoped to the UI, just like [`text_field_id`].
    pub fn id_salt(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.salt = Some(egui::IdSalt::new(salt));
        self
    }

    /// Override the width in points. Finite values are clamped to `1..=4096`;
    /// non-finite values use the available width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl egui::Widget for TextField<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let id =
            self.salt.map_or_else(|| ui.next_auto_id().with("spaceui-field"), |salt| ui.scope_id().with_salt(salt));
        show(ui, id, self.text, self.hint, self.width)
    }
}

/// A single-line text field that fills the available width. Returns the text
/// edit's response (`.changed()`, `.lost_focus()`, ...). Identity follows its
/// position in the widget sequence, rather than its screen coordinates.
/// Use [`text_field_id`] when surrounding widgets are inserted or reordered.
pub fn text_field(ui: &mut Ui, text: &mut String, hint: &str) -> Response {
    ui.add(TextField::new(text).hint_text(hint))
}

/// A text field with a stable, caller-supplied identity within the current UI
/// scope. Use a record or field key to preserve focus through layout changes.
/// The salt must be unique within that scope; use [`Ui::push_id`] for repeated
/// forms with the same field keys. See the [module example](super) for naming
/// a field for assistive technology.
pub fn text_field_id(
    ui: &mut Ui,
    id_salt: impl std::hash::Hash + std::fmt::Debug,
    text: &mut String,
    hint: &str,
) -> Response {
    ui.add(TextField::new(text).hint_text(hint).id_salt(id_salt))
}

fn show(ui: &mut Ui, id: Id, text: &mut String, hint: &str, width: Option<f32>) -> Response {
    let theme = Theme::of(ui.ctx());
    let p = &theme.palette;
    let metrics = ControlMetrics::of(ui.ctx()).field;
    let height = metrics.min_size.y.max(super::text_size(ui, 14.0) + 2.0 * metrics.padding.y);
    let width = width.filter(|width| width.is_finite()).map_or(ui.available_width(), |width| width.clamp(1.0, 4096.0));
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let focused = ui.memory(|m| m.has_focus(id));
    ui.painter().rect(
        rect,
        theme.metrics.control_radius + 2.0,
        p.well,
        Stroke::new(if focused { 2.0 } else { 1.0 }, if focused { p.accent_hi } else { p.border }),
        StrokeKind::Inside,
    );
    let inner = rect.shrink2(vec2(metrics.padding.x.min(rect.width() / 2.0), 0.0));
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
