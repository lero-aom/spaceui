use super::ChatLabels;
use crate::{
    Icon, Theme, paint,
    theme::{Palette, normalize, normalize_vec},
};
use egui::{
    Align, Color32, CursorIcon, Event, FontId, Frame, Id, ImeEvent, Key, KeyboardShortcut, Layout, Modifiers, Rect,
    Response, RichText, Role, ScrollArea, Sense, Spinner, Stroke, StrokeKind, TextEdit, Ui, UiBuilder, Vec2,
    WidgetInfo, pos2, vec2,
};

/// Sizing and growth limits for a chat composer, in points.
///
/// Defaults preserve the original six-line composer. Install context-wide
/// defaults with [`Self::install`] or override one composer with
/// [`Composer::metrics`]. Use [`Composer::desired_height`] when measuring a
/// configured builder so measurement and painting use the same options.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComposerMetrics {
    /// Growth limit in `line_height` units, clamped to `1..=256`.
    /// Four extra points allow for glyph extents; longer drafts scroll.
    pub max_lines: usize,
    /// Text size, clamped to `1..=256` points.
    pub font_size: f32,
    /// Line-height estimate for measuring and limiting the editing area,
    /// at least `font_size`. Font metrics determine actual text line spacing.
    pub line_height: f32,
    /// Horizontal and vertical padding around the editing area.
    pub padding: Vec2,
    /// Accessory row height, at least `button_size + 6` points.
    pub row_height: f32,
    /// Diameter of the send/stop button, at least one point.
    pub button_size: f32,
    /// Outer corner radius, capped by the supplied rectangle when painting.
    pub corner_radius: f32,
}

impl Default for ComposerMetrics {
    fn default() -> Self {
        Self {
            max_lines: 6,
            font_size: 14.0,
            line_height: 18.0,
            padding: vec2(14.0, 12.0),
            row_height: 38.0,
            button_size: 32.0,
            corner_radius: 16.0,
        }
    }
}

impl ComposerMetrics {
    fn id() -> Id {
        Id::unique("spaceui-composer-metrics")
    }

    fn normalized(self) -> Self {
        let defaults = Self::default();
        let font_size = normalize(self.font_size, defaults.font_size).clamp(1.0, 256.0);
        let button_size = normalize(self.button_size, defaults.button_size).max(1.0);
        Self {
            max_lines: self.max_lines.clamp(1, 256),
            font_size,
            line_height: normalize(self.line_height, defaults.line_height).max(font_size),
            padding: normalize_vec(self.padding, defaults.padding),
            row_height: normalize(self.row_height, defaults.row_height).max(button_size + 6.0),
            button_size,
            corner_radius: normalize(self.corner_radius, defaults.corner_radius),
        }
    }

    /// Install defaults for this context, preserving them across theme changes.
    /// Non-finite values use the corresponding default; finite values are
    /// clamped to `0..=4096` before applying the documented field-specific bounds.
    /// A changed configuration requests a repaint.
    pub fn install(self, ctx: &egui::Context) {
        let normalized = self.normalized();
        if Self::of(ctx) != normalized {
            ctx.data_mut(|data| data.insert_temp(Self::id(), normalized));
            ctx.request_repaint();
        }
    }

    /// Installed defaults, or [`Self::default`] before installation.
    #[must_use]
    pub fn of(ctx: &egui::Context) -> Self {
        ctx.data(|data| data.get_temp(Self::id()).unwrap_or_default())
    }
}

/// The message box at the foot of a chat column: a draft that grows to a
/// configurable line limit (six by default), accessories on the left of its bottom row (model and effort
/// dropdowns, say) and a round send button on the right.
///
/// Enter sends and Shift+Enter starts a new line. While [`busy`](Self::busy),
/// the button stops the running reply (or shows a spinner, if it can't be
/// stopped). Measure first with [`Composer::measure`] to lay out the column.
/// Held-key repeats do not send. Keyboard submission requires the draft to
/// retain focus through input handling; IME preedit and commit frames do not
/// submit. Disabled composers emit neither action. The app owns the draft and
/// should take it or set `busy` when accepting a send request.
#[derive(Debug)]
#[must_use = "a composer does nothing until `show` is called"]
pub struct Composer<'a> {
    id: Id,
    draft: &'a mut String,
    hint: &'a str,
    busy: bool,
    stoppable: bool,
    focus: bool,
    labels: ChatLabels<'a>,
    metrics: Option<ComposerMetrics>,
}

/// What the owner did in the composer this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[must_use]
pub struct ComposerResponse {
    /// Send the draft, which is non-empty; the app takes it from the string.
    pub send: bool,
    /// Stop the running reply.
    pub stop: bool,
}

impl<'a> Composer<'a> {
    /// A composer editing `draft`; `id_salt` keeps each conversation's draft
    /// and focus apart (a conversation id works).
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug, draft: &'a mut String) -> Self {
        Self {
            id: Id::unique(("spaceui-composer", id_salt)),
            draft,
            hint: "Ask anything",
            busy: false,
            stoppable: true,
            focus: false,
            labels: ChatLabels::default(),
            metrics: None,
        }
    }

    /// Placeholder text shown while the draft is empty.
    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
    }

    /// Accessible button names and help text, for localization. Translate the
    /// draft placeholder separately with [`Self::hint`].
    pub fn labels(mut self, labels: ChatLabels<'a>) -> Self {
        self.labels = labels;
        self
    }

    /// A reply is running: sending waits, and the button stops the reply.
    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    /// `false` shows a spinner instead of a Stop button while busy.
    pub fn stoppable(mut self, stoppable: bool) -> Self {
        self.stoppable = stoppable;
        self
    }

    /// Moves keyboard focus to the draft this frame.
    pub fn focus(mut self, focus: bool) -> Self {
        self.focus = focus;
        self
    }

    /// Override sizing and growth limits for this composer. Normalization
    /// follows [`ComposerMetrics::install`]. Measure with [`Self::desired_height`].
    pub fn metrics(mut self, metrics: ComposerMetrics) -> Self {
        self.metrics = Some(metrics.normalized());
        self
    }

    /// Measure using the installed [`ComposerMetrics`] and the given text.
    /// Non-finite widths use one point; narrow widths still wrap safely.
    #[must_use]
    pub fn measure(ui: &Ui, width: f32, draft: &str, hint: &str) -> f32 {
        measure(ui, width, draft, hint, ComposerMetrics::of(ui.ctx()))
    }

    /// Desired height with this builder's hint and sizing options.
    ///
    /// ```
    /// # use spaceui::{egui, chat::{Composer, ComposerMetrics, ChatLayout}};
    /// # let ctx = egui::Context::default();
    /// # let mut draft = String::new();
    /// # ctx.run_ui(Default::default(), |ui| {
    /// let composer = Composer::new("draft", &mut draft)
    ///     .metrics(ComposerMetrics { max_lines: 10, ..Default::default() });
    /// let column = ui.max_rect();
    /// let layout = ChatLayout::new(column, composer.desired_height(ui, column.width()));
    /// let response = composer.show(ui, layout.composer, |_| {});
    /// # let _ = response;
    /// # }).drop_without_applying_deltas();
    /// ```
    #[must_use]
    pub fn desired_height(&self, ui: &Ui, width: f32) -> f32 {
        measure(ui, width, self.draft, self.hint, self.metrics.unwrap_or_else(|| ComposerMetrics::of(ui.ctx())))
    }

    /// Draws into `rect` (from [`ChatLayout`](super::ChatLayout)); `accessories`
    /// fill the left of the bottom row, laid out left to right.
    pub fn show(self, ui: &mut Ui, rect: Rect, accessories: impl FnOnce(&mut Ui)) -> ComposerResponse {
        let metrics = self.metrics.unwrap_or_else(|| ComposerMetrics::of(ui.ctx()));
        if !rect.is_finite()
            || !rect.size().is_finite()
            || !rect.is_positive()
            || rect.width() < (metrics.padding.x * 2.0).max(1.0)
            || rect.height() < metrics.button_size + 2.0
        {
            return ComposerResponse::default();
        }
        let p = Theme::of(ui.ctx()).palette;
        let id = ui.make_persistent_id(self.id);
        let input_id = id.with("input");
        let focused = ui.memory(|m| m.has_focus(input_id));
        let border = if focused { p.accent.gamma_multiply(0.7) } else { p.border };
        ui.painter().rect(
            rect,
            metrics.corner_radius.min(rect.size().min_elem() / 2.0),
            p.well,
            Stroke::new(1.0, border),
            StrokeKind::Inside,
        );
        // A click anywhere on the composer focuses the draft; its buttons win.
        let mut focus = self.focus || ui.interact(rect, id.with("plate"), Sense::click()).clicked();

        let overhead = metrics.padding.y * 2.0 + metrics.row_height + 2.0;
        let text_height =
            text_height(ui, rect.width(), self.draft, self.hint, metrics).min((rect.height() - overhead).max(0.0));
        let text_rect =
            Rect::from_min_size(rect.min + metrics.padding, vec2(rect.width() - metrics.padding.x * 2.0, text_height));
        let edit = edit_draft(ui, id, text_rect, self.draft, self.hint, metrics, &p);
        let ime_blocks_send = composition_blocks_send(ui, input_id, focused && edit.has_focus());
        let enter = ui.is_enabled() && focused && edit.has_focus() && consume_send_key(ui) && !ime_blocks_send;
        let has_text = !self.draft.trim().is_empty();

        let row = Rect::from_min_max(
            pos2(
                rect.left() + 10.0_f32.min(rect.width() / 4.0),
                (rect.bottom() - metrics.row_height - 2.0).max(rect.top()),
            ),
            pos2(rect.right() - 8.0_f32.min(rect.width() / 4.0), rect.bottom() - 6.0_f32.min(rect.height() / 4.0)),
        );
        let mut out = ComposerResponse::default();
        let accessories_rect =
            Rect::from_min_max(row.min, pos2((row.right() - metrics.button_size - 6.0).max(row.left()), row.bottom()));
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(accessories_rect)
                .id_salt(id.with("accessories"))
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.set_clip_rect(accessories_rect.intersect(ui.clip_rect()));
                ui.spacing_mut().item_spacing.x = 4.0;
                accessories(ui);
            },
        );
        let button_rect = Rect::from_center_size(
            pos2(row.right() - metrics.button_size / 2.0, row.center().y),
            Vec2::splat(metrics.button_size),
        );
        ui.scope_builder(UiBuilder::new().max_rect(button_rect).id_salt(id.with("button")), |ui| {
            ui.set_clip_rect(rect.intersect(ui.clip_rect()));
            if self.busy && !self.stoppable {
                busy_indicator(ui, &p, self.labels.busy, metrics.button_size);
                return;
            }
            let response = ui
                .add_enabled_ui(self.busy || (has_text && !ime_blocks_send), |ui| {
                    send_button(ui, &p, self.busy, &self.labels, metrics.button_size)
                })
                .inner;
            if response.clicked() {
                if self.busy {
                    out.stop = true;
                } else if has_text {
                    out.send = true;
                }
            }
        });
        if enter && has_text && !self.busy {
            out.send = true;
        }
        focus |= out.send;
        if focus && ui.is_enabled() {
            edit.request_focus();
        }
        out
    }
}

fn edit_draft(
    ui: &mut Ui,
    id: Id,
    text_rect: Rect,
    draft: &mut String,
    hint: &str,
    metrics: ComposerMetrics,
    palette: &Palette,
) -> Response {
    let input_id = id.with("input");
    let text_height = text_rect.height();
    // An empty draft shows the whole hint, which may wrap.
    let rows = if draft.is_empty() { (text_height / metrics.line_height).round().max(1.0) as usize } else { 1 };
    let mut child = ui.new_child(UiBuilder::new().max_rect(text_rect).id_salt(id.with("text")));
    child.set_clip_rect(text_rect.intersect(ui.clip_rect()));
    ScrollArea::vertical()
        .id_salt(id.with("scroll"))
        .max_height(text_rect.height())
        .show(&mut child, |ui| {
            ui.add(
                TextEdit::multiline(draft)
                    .id(input_id)
                    .frame(Frame::NONE)
                    .font(FontId::proportional(metrics.font_size))
                    .text_color(palette.text)
                    .hint_text(RichText::new(hint).color(palette.muted))
                    .desired_rows(rows)
                    .desired_width(text_rect.width())
                    .return_key(Some(KeyboardShortcut::new(Modifiers::SHIFT, Key::Enter))),
            )
        })
        .inner
}

/// Consume Enter presses, including repeats, but only report a fresh press.
fn consume_send_key(ui: &Ui) -> bool {
    ui.input_mut(|input| {
        let mut pressed = false;
        input.events.retain(|event| {
            // Use the event's modifiers: Shift may be released later in this frame.
            if let Event::Key { key: Key::Enter, pressed: true, repeat, modifiers, .. } = event
                && !modifiers.shift
            {
                pressed |= !repeat;
                return false;
            }
            true
        });
        pressed
    })
}

/// Track only composition ownership; egui still handles editing and cursor state.
fn composition_blocks_send(ui: &Ui, input_id: Id, focused: bool) -> bool {
    let state_id = input_id.with("composition");
    let mut composing = ui.data_mut(|data| data.get_temp::<bool>(state_id).unwrap_or(false));
    if !focused || !ui.is_enabled() {
        ui.data_mut(|data| data.remove::<bool>(state_id));
        return false;
    }
    let mut blocked = composing;
    ui.input(|input| {
        for event in &input.events {
            match event {
                Event::Ime(ImeEvent::Preedit { text, .. }) => {
                    composing = !text.is_empty();
                    blocked |= composing;
                }
                Event::Ime(ImeEvent::Commit(text)) => {
                    blocked |= !text.is_empty();
                    composing = false;
                }
                _ => {}
            }
        }
    });
    ui.data_mut(|data| data.insert_temp(state_id, composing));
    blocked
}

fn busy_indicator(ui: &mut Ui, palette: &Palette, label: &str, diameter: f32) {
    let (spot, response) = ui.allocate_exact_size(Vec2::splat(diameter), Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(Role::Label, ui.is_enabled(), label));
    let size = diameter * (18.0 / 32.0);
    let icon_rect = Rect::from_center_size(spot.center(), Vec2::splat(size));
    if ui.style().animation_time > 0.0 {
        ui.put(icon_rect, Spinner::new().size(size).color(palette.accent_hi));
    } else {
        Icon::Hourglass.paint(ui.painter(), icon_rect, palette.accent_hi);
    }
}

/// The round send (or stop) button: an accent gradient when it can act.
fn send_button(ui: &mut Ui, p: &Palette, busy: bool, labels: &ChatLabels<'_>, diameter: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(diameter), Sense::click());
    response.widget_info(|| {
        WidgetInfo::labeled(Role::Button, ui.is_enabled(), if busy { labels.stop } else { labels.send })
    });
    let painter = ui.painter();
    let live = ui.is_enabled();
    if live {
        if response.hovered() || response.has_focus() {
            paint::glow(painter, rect, diameter / 2.0, p.accent, 6.0);
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        let top = if response.hovered() { p.accent_hi } else { paint::mix(p.accent, p.accent_hi, 0.35) };
        paint::gradient(painter, rect, top, p.accent_lo, diameter / 2.0);
    } else {
        painter.circle_filled(rect.center(), diameter / 2.0, p.field_hover);
    }
    let (icon, size) = if busy { (Icon::Stop, 14.0) } else { (Icon::Send, 18.0) };
    let size = size * diameter / 32.0;
    let ink = if live { p.on_accent } else { p.muted };
    icon.paint(painter, Rect::from_center_size(rect.center(), Vec2::splat(size)), ink);
    response.on_hover_text(if busy { labels.stop } else { labels.send_help })
}

fn measure(ui: &Ui, width: f32, draft: &str, hint: &str, metrics: ComposerMetrics) -> f32 {
    metrics.padding.y * 2.0 + text_height(ui, width, draft, hint, metrics) + metrics.row_height
}

fn text_height(ui: &Ui, width: f32, draft: &str, hint: &str, metrics: ComposerMetrics) -> f32 {
    let width = if width.is_finite() { (width - metrics.padding.x * 2.0).max(1.0) } else { 1.0 };
    let measured = if draft.is_empty() { hint } else { draft };
    ui.fonts_mut(|f| f.layout(measured.to_owned(), FontId::proportional(metrics.font_size), Color32::WHITE, width))
        .size()
        .y
        .clamp(metrics.line_height, metrics.line_height * metrics.max_lines as f32 + 4.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_composer_grows_with_the_draft_up_to_six_lines() {
        let ctx = crate::test_util::run_ui(|ui| {
            let one = Composer::measure(ui, 360.0, "short", "hint");
            let three = Composer::measure(ui, 360.0, "a\nb\nc", "hint");
            let many = Composer::measure(ui, 360.0, &"line\n".repeat(30), "hint");
            assert!(one < three && three < many);
            assert!(many <= 14.0 + 18.0 * 6.0 + 4.0 + 10.0 + 38.0);
        });
        drop(ctx);
    }
}
