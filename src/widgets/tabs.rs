use crate::{
    Theme, paint,
    theme::{ControlMetrics, semibold},
};
use egui::{
    Align2, CursorIcon, EventFilter, Id, Key, Modifiers, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui,
    WidgetInfo, pos2, vec2,
};

/// A choice group's aggregate response and the responses of its individual choices.
/// Selection stays in the caller-owned index.
#[derive(Clone, Debug)]
#[must_use]
pub struct ChoiceResponse {
    /// Union of the group's layout and choice responses, useful for bounds,
    /// clicks and hover help. `changed()` reports a final selection change,
    /// including index clamping even when disabled or empty.
    /// Focus belongs to individual [`Self::choices`], not the aggregate ID.
    /// As with [`Response::union`], do not call `interact` on this response.
    pub response: Response,
    /// Responses in label order, retaining each choice's focus and identity.
    /// A choice is marked changed when its selected state changes.
    pub choices: Vec<Response>,
}

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
    tabs_response(ui, selected, labels).response.changed()
}

/// Page tabs with aggregate and per-choice responses. Interaction and index
/// normalization match [`tabs`]; sizing follows [`ControlMetrics::tab`].
///
/// ```
/// # use spaceui::{egui, widgets};
/// # let ctx = egui::Context::default();
/// # ctx.run_ui(Default::default(), |ui| {
/// let mut selected = 0;
/// let tabs = widgets::tabs_response(ui, &mut selected, &["Overview", "History"]);
/// if tabs.response.changed() { /* update the view */ }
/// tabs.response.on_hover_text("Choose a view");
/// # }).drop_without_applying_deltas();
/// ```
pub fn tabs_response(ui: &mut Ui, selected: &mut usize, labels: &[&str]) -> ChoiceResponse {
    let p = Theme::of(ui.ctx()).palette;
    let metrics = ControlMetrics::of(ui.ctx()).tab;
    let old = *selected;
    *selected = (*selected).min(labels.len().saturating_sub(1));
    let group = ui.next_auto_id().with("spaceui-tabs");
    ui.skip_ahead_auto_ids(1);
    let output = ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let mut choices = Vec::with_capacity(labels.len());
        for (i, label) in labels.iter().enumerate() {
            let active = *selected == i;
            let font = if active { semibold(super::text_size(ui, 14.0)) } else { super::text_font(ui, 14.0) };
            let width = (ui.fonts_mut(|f| f.layout_no_wrap((*label).into(), font.clone(), p.text)).size().x
                + 2.0 * metrics.padding.x)
                .max(metrics.min_size.x);
            let height = metrics.min_size.y.max(font.size + 2.0 * metrics.padding.y);
            let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
            let response = ui.interact(rect, group.with(i), Sense::click());
            select(ui, &response, i, selected, labels.len(), group);
            choices.push(response);
        }
        // Resolve every interaction before publishing the group's selection.
        // A later click must clear an earlier choice in this same frame.
        for (i, (label, response)) in labels.iter().copied().zip(&mut choices).enumerate() {
            let rect = response.rect;
            let active = *selected == i;
            let font = if active { semibold(super::text_size(ui, 14.0)) } else { super::text_font(ui, 14.0) };
            response.widget_info(|| WidgetInfo::labeled(Role::Tab, ui.is_enabled(), label));
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
            painter.text(rect.center() - vec2(0.0, 1.0), Align2::CENTER_CENTER, label, font, color);
            if active {
                let bar = Rect::from_min_max(
                    pos2(rect.left() + 8.0, rect.bottom() - 2.5),
                    pos2(rect.right() - 8.0, rect.bottom()),
                );
                paint::gradient_h(painter, bar, p.accent_hi, p.accent, 1.25);
            }
            super::focus_ring(ui, response, 7.0);
            if old != *selected && (old == i || *selected == i) {
                response.mark_changed();
            }
            if response.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            }
        }
        choices
    });
    group_response(output.response, output.inner, old != *selected)
}

/// A compact segmented control ("M5 | H1 | D1"). Updates `selected` and
/// returns true on a change. Keyboard navigation and index clamping match [`tabs`].
pub fn segmented(ui: &mut Ui, selected: &mut usize, labels: &[&str]) -> bool {
    segmented_response(ui, selected, labels).response.changed()
}

/// A segmented control with aggregate and per-choice responses.
/// Interaction and index normalization match [`segmented`]; sizing follows
/// [`ControlMetrics::segmented`]. Each choice retains its radio-button role.
pub fn segmented_response(ui: &mut Ui, selected: &mut usize, labels: &[&str]) -> ChoiceResponse {
    let metrics = ControlMetrics::of(ui.ctx()).segmented;
    let theme = Theme::of(ui.ctx());
    let p = &theme.palette;
    let font = super::text_font(ui, 13.0);
    let widths: Vec<f32> = labels
        .iter()
        .map(|label| {
            (ui.fonts_mut(|f| f.layout_no_wrap((*label).into(), font.clone(), p.text)).size().x
                + 2.0 * metrics.padding.x)
                .max(metrics.min_size.x)
        })
        .collect();
    let height = metrics.min_size.y.max(font.size + 2.0 * metrics.padding.y).max(6.0);
    let (outer, outer_response) =
        ui.allocate_exact_size(vec2(widths.iter().sum::<f32>() + 6.0, height), Sense::hover());
    let radius = theme.metrics.control_radius;
    ui.painter().rect(outer, radius, p.well, Stroke::new(1.0, p.border_soft), StrokeKind::Inside);
    let mut x = outer.left() + 3.0;
    let old = *selected;
    *selected = (*selected).min(labels.len().saturating_sub(1));
    let mut choices = Vec::with_capacity(labels.len());
    for (i, width) in widths.iter().enumerate() {
        let rect = Rect::from_min_size(pos2(x, outer.top() + 3.0), vec2(*width, outer.height() - 6.0));
        let response = ui.interact(rect, outer_response.id.with(i), Sense::click());
        select(ui, &response, i, selected, labels.len(), outer_response.id);
        choices.push(response);
        x += width;
    }
    for (i, (label, response)) in labels.iter().copied().zip(&mut choices).enumerate() {
        let rect = response.rect;
        response.widget_info(|| WidgetInfo::selected(Role::RadioButton, ui.is_enabled(), *selected == i, label));
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
        ui.painter().text(rect.center(), Align2::CENTER_CENTER, label, font.clone(), color);
        super::focus_ring(ui, response, (radius - 2.0).max(0.0));
        if old != *selected && (old == i || *selected == i) {
            response.mark_changed();
        }
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
    }
    group_response(outer_response, choices, old != *selected)
}

fn group_response(mut response: Response, choices: Vec<Response>, changed: bool) -> ChoiceResponse {
    for choice in &choices {
        response |= choice.clone();
    }
    if changed {
        response.mark_changed();
    }
    ChoiceResponse { response, choices }
}

fn select(ui: &Ui, response: &Response, index: usize, selected: &mut usize, count: usize, group: Id) {
    if !ui.is_enabled() {
        return;
    }
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
}
