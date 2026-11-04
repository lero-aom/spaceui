use super::ChatLabels;
use crate::{Icon, Theme, paint, theme::Palette};
use egui::{
    Align, Color32, CursorIcon, Event, FontId, Frame, Id, Key, KeyboardShortcut, Layout, Modifiers, Rect, Response,
    RichText, Role, ScrollArea, Sense, Spinner, Stroke, StrokeKind, TextEdit, Ui, UiBuilder, Vec2, WidgetInfo, pos2,
    vec2,
};

const FONT_SIZE: f32 = 14.0;
const LINE_HEIGHT: f32 = 18.0;
const MAX_LINES: f32 = 6.0;
/// The accessory row under the text.
const ROW_HEIGHT: f32 = 38.0;

/// The message box at the foot of a chat column: a draft that grows to six
/// lines, accessories on the left of its bottom row (model and effort
/// dropdowns, say) and a round send button on the right.
///
/// Enter sends and Shift+Enter starts a new line. While [`busy`](Self::busy),
/// the button stops the running reply (or shows a spinner, if it can't be
/// stopped). Measure first with [`Composer::measure`] to lay out the column.
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

    /// The height a composer `width` points wide wants for `draft` (or, when
    /// empty, the whole `hint`): up to six lines plus the accessory row.
    #[must_use]
    pub fn measure(ui: &Ui, width: f32, draft: &str, hint: &str) -> f32 {
        14.0 + text_height(ui, width, draft, hint) + 10.0 + ROW_HEIGHT
    }

    /// Draws into `rect` (from [`ChatLayout`](super::ChatLayout)); `accessories`
    /// fill the left of the bottom row, laid out left to right.
    pub fn show(self, ui: &mut Ui, rect: Rect, accessories: impl FnOnce(&mut Ui)) -> ComposerResponse {
        if rect.width() < 28.0 || rect.height() < 34.0 {
            return ComposerResponse::default();
        }
        let p = Theme::of(ui.ctx()).palette;
        let id = ui.make_persistent_id(self.id);
        let input_id = id.with("input");
        let focused = ui.memory(|m| m.has_focus(input_id));
        let border = if focused { p.accent.gamma_multiply(0.7) } else { p.border };
        ui.painter().rect(rect, 16.0, p.well, Stroke::new(1.0, border), StrokeKind::Inside);
        // A click anywhere on the composer focuses the draft; its buttons win.
        let mut focus = self.focus || ui.interact(rect, id.with("plate"), Sense::click()).clicked();

        let text_height = text_height(ui, rect.width(), self.draft, self.hint).min((rect.height() - 64.0).max(0.0));
        let text_rect = Rect::from_min_size(rect.min + vec2(14.0, 12.0), vec2(rect.width() - 28.0, text_height));
        // An empty draft shows the whole hint, which may wrap.
        let rows = if self.draft.is_empty() { (text_height / LINE_HEIGHT).round().max(1.0) as usize } else { 1 };
        let mut child = ui.new_child(UiBuilder::new().max_rect(text_rect).id_salt(id.with("text")));
        child.set_clip_rect(text_rect.intersect(ui.clip_rect()));
        let edit = ScrollArea::vertical()
            .id_salt(id.with("scroll"))
            .max_height(text_rect.height())
            .show(&mut child, |ui| {
                ui.add(
                    TextEdit::multiline(self.draft)
                        .id(input_id)
                        .frame(Frame::NONE)
                        .font(FontId::proportional(FONT_SIZE))
                        .text_color(p.text)
                        .hint_text(RichText::new(self.hint).color(p.muted))
                        .desired_rows(rows)
                        .desired_width(text_rect.width())
                        .return_key(Some(KeyboardShortcut::new(Modifiers::SHIFT, Key::Enter))),
                )
            })
            .inner;
        let enter = ui.is_enabled()
            && (edit.has_focus() || edit.lost_focus())
            && ui.input(|i| {
                // Modifiers can change again before the frame is processed.
                // Use the Enter event's state so releasing Shift cannot send
                // a draft after inserting a newline in the same frame.
                i.events.iter().any(|event| {
                    matches!(event, Event::Key { key: Key::Enter, pressed: true, modifiers, .. } if !modifiers.shift)
                })
            });
        let has_text = !self.draft.trim().is_empty();

        let row = Rect::from_min_max(
            pos2(rect.left() + 10.0_f32.min(rect.width() / 4.0), (rect.bottom() - 40.0).max(rect.top())),
            pos2(rect.right() - 8.0_f32.min(rect.width() / 4.0), rect.bottom() - 6.0_f32.min(rect.height() / 4.0)),
        );
        let mut out = ComposerResponse::default();
        let accessories_rect = Rect::from_min_max(row.min, pos2((row.right() - 38.0).max(row.left()), row.bottom()));
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
        let button_rect = Rect::from_center_size(pos2(row.right() - 16.0, row.center().y), Vec2::splat(32.0));
        ui.scope_builder(UiBuilder::new().max_rect(button_rect).id_salt(id.with("button")), |ui| {
            ui.set_clip_rect(rect.intersect(ui.clip_rect()));
            if self.busy && !self.stoppable {
                let (spot, response) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::hover());
                response.widget_info(|| WidgetInfo::labeled(Role::Label, ui.is_enabled(), self.labels.busy));
                if ui.style().animation_time > 0.0 {
                    let spinner = Spinner::new().size(18.0).color(p.accent_hi);
                    ui.put(Rect::from_center_size(spot.center(), Vec2::splat(18.0)), spinner);
                } else {
                    Icon::Hourglass.paint(ui.painter(), spot.shrink(7.0), p.accent_hi);
                }
                return;
            }
            let response =
                ui.add_enabled_ui(self.busy || has_text, |ui| send_button(ui, &p, self.busy, &self.labels)).inner;
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

/// The round send (or stop) button: an accent gradient when it can act.
fn send_button(ui: &mut Ui, p: &Palette, busy: bool, labels: &ChatLabels<'_>) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::click());
    response.widget_info(|| {
        WidgetInfo::labeled(Role::Button, ui.is_enabled(), if busy { labels.stop } else { labels.send })
    });
    let painter = ui.painter();
    let live = ui.is_enabled();
    if live {
        if response.hovered() || response.has_focus() {
            paint::glow(painter, rect, 16.0, p.accent, 6.0);
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        let top = if response.hovered() { p.accent_hi } else { paint::mix(p.accent, p.accent_hi, 0.35) };
        paint::gradient(painter, rect, top, p.accent_lo, 16.0);
    } else {
        painter.circle_filled(rect.center(), 16.0, p.field_hover);
    }
    let (icon, size) = if busy { (Icon::Stop, 14.0) } else { (Icon::Send, 18.0) };
    let ink = if live { p.on_accent } else { p.muted };
    icon.paint(painter, Rect::from_center_size(rect.center(), Vec2::splat(size)), ink);
    response.on_hover_text(if busy { labels.stop } else { labels.send_help })
}

fn text_height(ui: &Ui, width: f32, draft: &str, hint: &str) -> f32 {
    let measured = if draft.is_empty() { hint } else { draft };
    ui.fonts_mut(|f| {
        f.layout(measured.to_owned(), FontId::proportional(FONT_SIZE), Color32::WHITE, (width - 28.0).max(1.0))
    })
    .size()
    .y
    .clamp(LINE_HEIGHT, LINE_HEIGHT * MAX_LINES + 4.0)
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
            assert!(many <= 14.0 + LINE_HEIGHT * MAX_LINES + 4.0 + 10.0 + ROW_HEIGHT);
        });
        drop(ctx);
    }
}
