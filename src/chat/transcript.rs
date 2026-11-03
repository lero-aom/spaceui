//! The pieces a transcript is made of.
//!
//! Call them in order inside a [`ScrollArea`](egui::ScrollArea); with
//! `stick_to_bottom(true)` a streaming reply stays in view. A typical reply is
//! a [`byline`], then [`thinking`], any tool [`Step`]s and the [`reply`] text.
use super::markdown;
use crate::{Icon, Svg, Theme, paint, text::truncated, theme::semibold};
use egui::{
    Align, Align2, Color32, CursorIcon, FontId, Frame, Id, Label, Margin, Rect, RichText, Sense, Stroke, StrokeKind,
    Ui, Vec2, pos2, text::LayoutJob, vec2,
};

/// The owner's message, right-aligned in a bubble with inline Markdown;
/// `hover` (a time, say) shows beside it on hover.
pub fn user_bubble(ui: &mut Ui, text: &str, hover: &str) {
    let p = Theme::of(ui.ctx()).palette;
    ui.add_space(10.0);
    let max_width = ui.available_width() * 0.84;
    let mut job = markdown::inline_job(text, &FontId::proportional(14.0), p.text, &p);
    job.wrap.max_width = max_width - 28.0;
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    let size = galley.size() + vec2(28.0, 18.0);
    let (row, response) = ui.allocate_exact_size(vec2(ui.available_width(), size.y), Sense::hover());
    let bubble = Rect::from_min_size(pos2(row.right() - size.x, row.top()), size);
    ui.painter().rect(bubble, 16.0, p.bubble, Stroke::new(1.0, p.accent.gamma_multiply(0.25)), StrokeKind::Inside);
    ui.painter().galley(bubble.min + vec2(14.0, 9.0), galley, p.text);
    if response.hovered() && !hover.is_empty() {
        let at = pos2(bubble.left() - 8.0, bubble.center().y);
        ui.painter().text(at, Align2::RIGHT_CENTER, hover, FontId::proportional(11.5), p.muted);
    }
    ui.add_space(2.0);
}

/// Who answered: a small logo and a muted label ("Claude · Opus · high").
pub fn byline(ui: &mut Ui, logo: Option<Svg>, label: &str) {
    let p = Theme::of(ui.ctx()).palette;
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if let Some(logo) = logo {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::hover());
            logo.paint(ui.painter(), rect, Color32::WHITE);
        }
        ui.label(RichText::new(label).size(11.0).color(p.muted));
    });
}

/// A reply rendered as Markdown, with a blinking caret while it streams.
pub fn reply(ui: &mut Ui, text: &str, streaming: bool) {
    let p = Theme::of(ui.ctx()).palette;
    Frame::new().inner_margin(Margin::symmetric(4, 2)).show(ui, |ui| {
        markdown::render(ui, text);
        if streaming {
            let time = ui.input(|i| i.time) as f32;
            let (rect, _) = ui.allocate_exact_size(vec2(10.0, 14.0), Sense::hover());
            let ink = p.accent_hi.gamma_multiply(0.4 + 0.6 * (time * 4.0).sin().abs());
            ui.painter().rect_filled(rect.shrink2(vec2(2.0, 1.0)), 1.0, ink);
            ui.ctx().request_repaint();
        }
    });
    ui.add_space(4.0);
}

/// "Thinking…" with a pulsing sparkle while `active`, then "Thought for Ns".
/// When there is reasoning `text`, a click toggles it; the app keeps `open`
/// and flips it when this returns true.
pub fn thinking(ui: &mut Ui, text: &str, active: bool, secs: f32, open: bool) -> bool {
    let p = Theme::of(ui.ctx()).palette;
    ui.add_space(2.0);
    let (row, response) = ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::click());
    let time = ui.input(|i| i.time) as f32;
    let star = pos2(row.left() + 14.0, row.center().y);
    if active {
        paint::radial_glow(ui.painter(), star, 16.0, p.accent, (70.0 * (0.55 + 0.45 * (time * 2.6).sin())) as u8);
        ui.ctx().request_repaint();
    }
    paint::sparkle(ui.painter(), star, if active { 8.0 + 0.8 * (time * 2.6).sin() } else { 8.0 }, p.accent_hi);
    let label = if active { "Thinking…".to_owned() } else { format!("Thought for {}s", secs.round().max(1.0)) };
    let label_rect = ui.painter().text(
        pos2(row.left() + 30.0, row.center().y),
        Align2::LEFT_CENTER,
        &label,
        FontId::proportional(13.5),
        if active { p.text } else { p.text_2 },
    );
    if active {
        // A soft shimmer sweeping across the label.
        let x = label_rect.left() + (time * 90.0) % (label_rect.width() + 60.0) - 30.0;
        let sweep = Rect::from_min_max(pos2(x - 18.0, label_rect.top()), pos2(x + 18.0, label_rect.bottom()));
        let sweep = sweep.intersect(label_rect);
        if sweep.is_positive() {
            ui.painter().rect_filled(sweep, 0.0, Color32::from_white_alpha(16));
        }
    }
    let has_text = !text.trim().is_empty();
    if has_text {
        let chevron = if open { Icon::ChevronDown } else { Icon::ChevronRight };
        let at = pos2(label_rect.right() + 12.0, row.center().y);
        chevron.paint(ui.painter(), Rect::from_center_size(at, Vec2::splat(13.0)), p.muted);
        if open {
            Frame::new().inner_margin(Margin { left: 30, right: 6, top: 2, bottom: 4 }).show(ui, |ui| {
                ui.add(Label::new(RichText::new(text.trim()).size(12.5).italics().color(p.muted)).wrap());
            });
        }
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
    }
    response.clicked() && has_text
}

/// The state of a tool call on the timeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StepStatus {
    /// Still running: a pulsing dot.
    Running,
    /// Finished: its time shows.
    Done,
    /// Failed: painted in the negative colour.
    Failed,
}

/// One tool call on a reply's timeline.
///
/// ```
/// # use spaceui::{Icon, Theme, egui, chat::transcript::{Step, StepStatus}};
/// # let ctx = egui::Context::default();
/// # Theme::carbon_copper().install(&ctx);
/// # ctx.run_ui(Default::default(), |ui| {
/// let steps = [
///     Step { icon: Icon::Globe, title: "Opened page", detail: "example.com", time: "09:41", status: StepStatus::Done },
///     Step { icon: Icon::Search, title: "Searching", detail: "SPY trend", time: "", status: StepStatus::Running },
/// ];
/// for (i, step) in steps.iter().enumerate() {
///     step.show(ui, i > 0, i + 1 < steps.len());
/// }
/// # }).drop_without_applying_deltas();
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step<'a> {
    /// What kind of tool ran.
    pub icon: Icon,
    /// What it did ("Read `get_desk`").
    pub title: &'a str,
    /// Its arguments or target, in monospace; may be empty.
    pub detail: &'a str,
    /// When it finished; shows once it is no longer running.
    pub time: &'a str,
    /// Running, done or failed.
    pub status: StepStatus,
}

impl Step<'_> {
    /// Draws the step; `prev` and `next` join it to the steps around it.
    pub fn show(&self, ui: &mut Ui, prev: bool, next: bool) {
        let p = Theme::of(ui.ctx()).palette;
        let height = if self.detail.is_empty() { 30.0 } else { 46.0 };
        let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
        let (line_x, dot_y) = (row.left() + 14.0, row.top() + 14.0);
        let line = Stroke::new(1.5, p.border);
        if prev {
            ui.painter().vline(line_x, row.top()..=dot_y, line);
        }
        if next {
            ui.painter().vline(line_x, dot_y..=row.bottom(), line);
        }
        let dot = match self.status {
            StepStatus::Running => p.accent_hi,
            StepStatus::Done => p.accent.gamma_multiply(0.8),
            StepStatus::Failed => p.negative,
        };
        ui.painter().circle_filled(pos2(line_x, dot_y), 3.5, dot);
        let ink = if self.status == StepStatus::Failed { p.negative } else { p.text_2 };
        let icon_rect = Rect::from_center_size(pos2(row.left() + 38.0, dot_y), Vec2::splat(15.0));
        self.icon.paint(ui.painter(), icon_rect, ink);
        let text_x = row.left() + 54.0;
        ui.painter().text(pos2(text_x, dot_y), Align2::LEFT_CENTER, self.title, FontId::proportional(13.0), p.text);
        if !self.detail.is_empty() {
            let galley = truncated(ui, self.detail, FontId::monospace(11.0), p.muted, row.right() - text_x - 50.0);
            ui.painter().galley(pos2(text_x, dot_y + 18.0 - galley.size().y / 2.0), galley, p.muted);
        }
        let right = pos2(row.right() - 12.0, dot_y);
        if self.status == StepStatus::Running {
            let time = ui.input(|i| i.time) as f32;
            ui.painter().circle_filled(right, 8.0, p.accent.gamma_multiply(0.2 + 0.2 * (time * 4.0).sin().abs()));
            ui.painter().circle_filled(right, 4.0, p.accent_hi);
            ui.ctx().request_repaint();
        } else if !self.time.is_empty() {
            let at = right + vec2(6.0, 0.0);
            ui.painter().text(at, Align2::RIGHT_CENTER, self.time, FontId::proportional(11.5), p.muted);
        }
    }
}

/// A framed note in the transcript; `error` paints it in the negative colour.
pub fn notice(ui: &mut Ui, text: &str, error: bool) {
    let p = Theme::of(ui.ctx()).palette;
    ui.add_space(6.0);
    let color = if error { p.negative } else { p.text_2 };
    Frame::new()
        .fill(if error { color.gamma_multiply(0.07) } else { p.panel_hi })
        .stroke(Stroke::new(1.0, color.gamma_multiply(0.35)))
        .corner_radius(10)
        .inner_margin(Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
                Icon::Alert.paint(ui.painter(), rect, color);
                ui.add(Label::new(RichText::new(text).size(13.0).color(color)).wrap());
            });
        });
}

/// A centred welcome for an empty conversation in `rect`: the logo, a
/// headline, a hint and clickable starters. Returns the index of a starter
/// the owner clicked.
pub fn empty_state(
    ui: &mut Ui,
    rect: Rect,
    logo: Option<Svg>,
    headline: &str,
    hint: &str,
    starters: &[&str],
) -> Option<usize> {
    let p = Theme::of(ui.ctx()).palette;
    let block = 110.0 + starters.len() as f32 * 46.0;
    let center = pos2(rect.center().x, rect.center().y - block / 2.0 + 30.0);
    if let Some(logo) = logo {
        let logo_rect = Rect::from_center_size(center - vec2(0.0, 40.0), Vec2::splat(78.0));
        paint::radial_glow(ui.painter(), logo_rect.center(), 70.0, p.accent, 34);
        logo.paint(ui.painter(), logo_rect, Color32::WHITE);
    }
    ui.painter().text(pos2(center.x, center.y + 18.0), Align2::CENTER_CENTER, headline, semibold(18.0), p.text);
    let mut job = LayoutJob::simple(hint.to_owned(), FontId::proportional(12.5), p.muted, rect.width() - 56.0);
    job.halign = Align::Center;
    let hint = ui.fonts_mut(|f| f.layout_job(job));
    let mut y = center.y + 48.0 + hint.size().y;
    ui.painter().galley(pos2(center.x, center.y + 34.0), hint, p.muted);
    let mut picked = None;
    for (i, text) in starters.iter().enumerate() {
        let starter = Rect::from_min_size(pos2(rect.left() + 18.0, y), vec2(rect.width() - 36.0, 38.0));
        let response = ui.interact(starter, Id::unique(("spaceui-starter", i)), Sense::click());
        let hot = response.hovered() && ui.is_enabled();
        let border = if hot { p.accent.gamma_multiply(0.5) } else { p.border_soft };
        ui.painter().rect(
            starter,
            10.0,
            if hot { p.field_hover } else { p.field },
            Stroke::new(1.0, border),
            StrokeKind::Inside,
        );
        let spark = Rect::from_center_size(pos2(starter.left() + 18.0, starter.center().y), Vec2::splat(14.0));
        Icon::Sparkle.paint(ui.painter(), spark, p.accent_hi);
        let galley = truncated(ui, text, FontId::proportional(13.0), p.text_2, starter.width() - 46.0);
        ui.painter().galley(pos2(starter.left() + 34.0, starter.center().y - galley.size().y / 2.0), galley, p.text_2);
        if hot {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        if response.clicked() {
            picked = Some(i);
        }
        y += 46.0;
    }
    picked
}
