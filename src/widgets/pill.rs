use super::Tone;
use crate::{Icon, Theme, text::truncated, theme::semibold};
use egui::{CursorIcon, FontId, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};

/// A small tinted status tag ("READ-ONLY", "PAPER"), sized to its text.
pub fn chip(ui: &mut Ui, text: &str, tone: Tone) {
    let color = tone.color(&Theme::of(ui.ctx()).palette);
    let galley = ui.fonts_mut(|f| f.layout_no_wrap(text.into(), semibold(10.5), color));
    let (rect, _) = ui.allocate_exact_size(galley.size() + vec2(16.0, 7.0), Sense::hover());
    ui.painter().rect(
        rect,
        6.0,
        color.gamma_multiply(0.12),
        Stroke::new(1.0, color.gamma_multiply(0.4)),
        StrokeKind::Inside,
    );
    ui.painter().galley(rect.center() - galley.size() / 2.0, galley, color);
}

/// A rounded, clickable status pill with an optional dot ("● Live data").
pub fn pill(ui: &mut Ui, dot: Option<Tone>, text: &str) -> Response {
    let theme = Theme::of(ui.ctx());
    let p = &theme.palette;
    let galley = ui.fonts_mut(|f| f.layout_no_wrap(text.into(), FontId::proportional(12.5), p.text_2));
    let lead = if dot.is_some() { 24.0 } else { 12.0 };
    let (rect, response) = ui.allocate_exact_size(vec2(galley.size().x + lead + 12.0, 28.0), Sense::click());
    let hot = response.hovered();
    ui.painter().rect(
        rect,
        theme.metrics.pill_radius,
        if hot { p.field_hover } else { p.field },
        Stroke::new(1.0, if hot { p.border } else { p.border_soft }),
        StrokeKind::Inside,
    );
    if let Some(dot) = dot {
        let (center, color) = (pos2(rect.left() + 13.0, rect.center().y), dot.color(p));
        ui.painter().circle_filled(center, 6.0, color.gamma_multiply(0.18));
        ui.painter().circle_filled(center, 3.5, color);
    }
    ui.painter().galley(pos2(rect.left() + lead, rect.center().y - galley.size().y / 2.0), galley, p.text_2);
    response.on_hover_cursor(CursorIcon::PointingHand)
}

/// A tinted pill led by an icon, for standing facts ("Read-only").
pub fn badge(ui: &mut Ui, icon: Icon, text: &str, tone: Tone) -> Response {
    let theme = Theme::of(ui.ctx());
    let color = tone.color(&theme.palette);
    let galley = ui.fonts_mut(|f| f.layout_no_wrap(text.into(), semibold(11.5), color));
    let (rect, response) = ui.allocate_exact_size(vec2(galley.size().x + 36.0, 28.0), Sense::hover());
    ui.painter().rect(
        rect,
        theme.metrics.pill_radius,
        color.gamma_multiply(0.1),
        Stroke::new(1.0, color.gamma_multiply(0.35)),
        StrokeKind::Inside,
    );
    let icon_rect = Rect::from_center_size(pos2(rect.left() + 15.0, rect.center().y), Vec2::splat(15.0));
    icon.paint(ui.painter(), icon_rect, color);
    ui.painter().galley(pos2(rect.left() + 26.0, rect.center().y - galley.size().y / 2.0), galley, color);
    response
}

/// A borderless dropdown trigger with a chevron, as under a chat composer
/// ("● Claude · Opus ⌄"); the text is cut with an ellipsis at `max_width`.
/// Pair it with [`egui::Popup::menu`]:
///
/// ```
/// # use spaceui::{Theme, egui, widgets, widgets::Tone};
/// # let ctx = egui::Context::default();
/// # Theme::carbon_copper().install(&ctx);
/// # ctx.run_ui(Default::default(), |ui| {
/// let model = widgets::dropdown(ui, Some(Tone::Positive), "Claude · Opus", 160.0);
/// egui::Popup::menu(&model).show(|ui| {
///     let _ = ui.selectable_label(true, "Opus");
/// });
/// # }).drop_without_applying_deltas();
/// ```
pub fn dropdown(ui: &mut Ui, dot: Option<Tone>, text: &str, max_width: f32) -> Response {
    let theme = Theme::of(ui.ctx());
    let p = &theme.palette;
    let lead = if dot.is_some() { 20.0 } else { 10.0 };
    let galley = truncated(ui, text, FontId::proportional(12.5), p.text_2, max_width);
    let (rect, response) = ui.allocate_exact_size(vec2(lead + galley.size().x + 26.0, 28.0), Sense::click());
    let painter = ui.painter();
    if response.hovered() || response.is_pointer_button_down_on() {
        painter.rect_filled(rect, theme.metrics.control_radius, p.field_hover);
    }
    if let Some(dot) = dot {
        painter.circle_filled(pos2(rect.left() + 11.0, rect.center().y), 3.5, dot.color(p));
    }
    painter.galley(pos2(rect.left() + lead, rect.center().y - galley.size().y / 2.0), galley, p.text_2);
    let chevron = Rect::from_center_size(pos2(rect.right() - 12.0, rect.center().y), Vec2::splat(12.0));
    Icon::ChevronDown.paint(painter, chevron, p.muted);
    response.on_hover_cursor(CursorIcon::PointingHand)
}
