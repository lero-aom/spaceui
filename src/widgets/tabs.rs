use crate::{Theme, paint, theme::semibold};
use egui::{
    Align2, CursorIcon, EventFilter, Id, Key, Modifiers, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui,
    WidgetInfo, pos2, vec2,
};

/// Page tabs with an accent underline, as on a chart toolbar. Updates
/// `selected` and returns true when it changes, including clamping an invalid
/// index. Arrow keys move between choices, Home/End select the first/last,
/// and Tab moves focus. Empty labels reset `selected` to zero.
///
/// ```
/// # use spaceui::{Theme, egui, widgets};
/// # let ctx = egui::Context::default();
/// # Theme::carbon_copper().install(&ctx);
/// # ctx.run_ui(Default::default(), |ui| {
/// let mut tab = 0;
/// if widgets::tabs(ui, &mut tab, &["Agent", "Manual", "Room"]) {
///     // the view changed
/// }
/// # }).drop_without_applying_deltas();
/// ```
pub fn tabs(ui: &mut Ui, selected: &mut usize, labels: &[&str]) -> bool {
    let p = Theme::of(ui.ctx()).palette;
    let old = *selected;
    *selected = (*selected).min(labels.len().saturating_sub(1));
    let group = ui.next_auto_id().with("spaceui-tabs");
    ui.skip_ahead_auto_ids(1);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (i, label) in labels.iter().enumerate() {
            let active = *selected == i;
            let font = if active { semibold(super::text_size(ui, 14.0)) } else { super::text_font(ui, 14.0) };
            let width = ui.fonts_mut(|f| f.layout_no_wrap((*label).into(), font.clone(), p.text)).size().x + 24.0;
            let height = 34.0_f32.max(font.size + 20.0);
            let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
            let mut response = ui.interact(rect, group.with(i), Sense::click());
            select(ui, &mut response, i, selected, labels.len(), group);
            response.widget_info(|| WidgetInfo::labeled(Role::Tab, ui.is_enabled(), *label));
            ui.ctx().accesskit_node_builder(response.id, |node| node.set_selected(*selected == i));
            let painter = ui.painter();
            if response.hovered() && !active {
                painter.rect_filled(rect.shrink2(vec2(0.0, 3.0)), 7.0, p.field);
            }
            let color = if active {
                p.accent_hi
            } else if response.hovered() {
                p.text
            } else {
                p.text_2
            };
            painter.text(rect.center() - vec2(0.0, 1.0), Align2::CENTER_CENTER, *label, font, color);
            if active {
                let bar = Rect::from_min_max(
                    pos2(rect.left() + 8.0, rect.bottom() - 2.5),
                    pos2(rect.right() - 8.0, rect.bottom()),
                );
                paint::gradient_h(painter, bar, p.accent_hi, p.accent, 1.25);
            }
            super::focus_ring(ui, &response, 7.0);
            response.on_hover_cursor(CursorIcon::PointingHand);
        }
    });
    old != *selected
}

/// A compact segmented control ("M5 | H1 | D1"). Updates `selected` and
/// returns true on a change. Keyboard navigation and index clamping match [`tabs`].
pub fn segmented(ui: &mut Ui, selected: &mut usize, labels: &[&str]) -> bool {
    let theme = Theme::of(ui.ctx());
    let p = &theme.palette;
    let font = super::text_font(ui, 13.0);
    let widths: Vec<f32> = labels
        .iter()
        .map(|label| ui.fonts_mut(|f| f.layout_no_wrap((*label).into(), font.clone(), p.text)).size().x + 22.0)
        .collect();
    let height = 30.0_f32.max(font.size + 17.0);
    let (outer, outer_response) =
        ui.allocate_exact_size(vec2(widths.iter().sum::<f32>() + 6.0, height), Sense::hover());
    let radius = theme.metrics.control_radius;
    ui.painter().rect(outer, radius, p.well, Stroke::new(1.0, p.border_soft), StrokeKind::Inside);
    let mut x = outer.left() + 3.0;
    let old = *selected;
    *selected = (*selected).min(labels.len().saturating_sub(1));
    for (i, (label, width)) in labels.iter().zip(&widths).enumerate() {
        let rect = Rect::from_min_size(pos2(x, outer.top() + 3.0), vec2(*width, outer.height() - 6.0));
        let mut response = ui.interact(rect, outer_response.id.with(i), Sense::click());
        select(ui, &mut response, i, selected, labels.len(), outer_response.id);
        response.widget_info(|| WidgetInfo::selected(Role::RadioButton, ui.is_enabled(), *selected == i, *label));
        let active = *selected == i;
        if active {
            paint::gradient(ui.painter(), rect, p.accent, p.accent_lo, radius - 2.0);
        } else if response.hovered() {
            ui.painter().rect_filled(rect, radius - 2.0, p.field);
        }
        let color = if active {
            p.on_accent
        } else if response.hovered() {
            p.text
        } else {
            p.text_2
        };
        ui.painter().text(rect.center(), Align2::CENTER_CENTER, *label, font.clone(), color);
        super::focus_ring(ui, &response, (radius - 2.0).max(0.0));
        response.on_hover_cursor(CursorIcon::PointingHand);
        x += width;
    }
    old != *selected
}

fn select(ui: &Ui, response: &mut Response, index: usize, selected: &mut usize, count: usize, group: Id) {
    if !ui.is_enabled() {
        return;
    }
    let old = *selected;
    if response.clicked() {
        *selected = index;
    }
    if response.has_focus() {
        ui.memory_mut(|m| {
            m.set_focus_lock_filter(response.id, EventFilter { horizontal_arrows: true, ..Default::default() });
        });
        let next = ui.input_mut(|input| {
            if input.consume_key(Modifiers::NONE, Key::ArrowRight) {
                Some((index + 1) % count)
            } else if input.consume_key(Modifiers::NONE, Key::ArrowLeft) {
                Some(if index == 0 { count - 1 } else { index - 1 })
            } else if input.consume_key(Modifiers::NONE, Key::Home) {
                Some(0)
            } else if input.consume_key(Modifiers::NONE, Key::End) {
                Some(count - 1)
            } else {
                None
            }
        });
        if let Some(next) = next {
            *selected = next;
            ui.memory_mut(|m| m.request_focus(group.with(next)));
        }
    }
    if old != *selected {
        response.mark_changed();
    }
}
