use crate::{Theme, paint, theme::semibold};
use egui::{Align2, CursorIcon, FontId, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

/// Page tabs with an accent underline, as on a chart toolbar. Updates
/// `selected` and returns true when the owner picked another tab.
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
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (i, label) in labels.iter().enumerate() {
            let active = *selected == i;
            let font = if active { semibold(14.0) } else { FontId::proportional(14.0) };
            let width = ui.fonts_mut(|f| f.layout_no_wrap((*label).into(), font.clone(), p.text)).size().x + 24.0;
            let (rect, response) = ui.allocate_exact_size(vec2(width, 34.0), Sense::click());
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
            if response.clicked() && !active {
                *selected = i;
                changed = true;
            }
            response.on_hover_cursor(CursorIcon::PointingHand);
        }
    });
    changed
}

/// A compact segmented control ("M5 | H1 | D1"). Updates `selected` and
/// returns true on a change.
pub fn segmented(ui: &mut Ui, selected: &mut usize, labels: &[&str]) -> bool {
    let theme = Theme::of(ui.ctx());
    let p = &theme.palette;
    let font = FontId::proportional(13.0);
    let widths: Vec<f32> = labels
        .iter()
        .map(|label| ui.fonts_mut(|f| f.layout_no_wrap((*label).into(), font.clone(), p.text)).size().x + 22.0)
        .collect();
    let (outer, _) = ui.allocate_exact_size(vec2(widths.iter().sum::<f32>() + 6.0, 30.0), Sense::hover());
    let radius = theme.metrics.control_radius;
    ui.painter().rect(outer, radius, p.well, Stroke::new(1.0, p.border_soft), StrokeKind::Inside);
    let mut x = outer.left() + 3.0;
    let mut changed = false;
    for (i, (label, width)) in labels.iter().zip(&widths).enumerate() {
        let rect = Rect::from_min_size(pos2(x, outer.top() + 3.0), vec2(*width, outer.height() - 6.0));
        let response =
            ui.interact(rect, ui.scope_id().with(("spaceui-segment", outer.min.x as i32, i)), Sense::click());
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
        if response.clicked() && !active {
            *selected = i;
            changed = true;
        }
        response.on_hover_cursor(CursorIcon::PointingHand);
        x += width;
    }
    changed
}
