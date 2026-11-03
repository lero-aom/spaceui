use super::RailItem;
use crate::{Theme, paint};
use egui::{Color32, CursorIcon, Id, Rect, Sense, Stroke, StrokeKind, Ui, Vec2, pos2};

/// Icon tiles down the left edge; the selected page glows in the accent.
pub(super) fn show(ui: &Ui, rect: Rect, items: &[RailItem], page: &mut usize, enabled: bool) {
    let theme = Theme::of(ui.ctx());
    let (palette, metrics) = (&theme.palette, theme.metrics);
    let x = rect.center().x;
    let mut y = rect.top() + 4.0;
    let mut index = 0;
    for item in items {
        let RailItem::Page { icon, label } = *item else {
            ui.painter().hline((x - 16.0)..=(x + 16.0), y + 4.0, Stroke::new(1.0, palette.border));
            y += 12.0;
            continue;
        };
        let tile = Rect::from_min_size(pos2(x - metrics.tile / 2.0, y), Vec2::splat(metrics.tile));
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let response = ui.interact(tile, Id::unique(("spaceui-rail", index)), sense).on_hover_text(label);
        let active = *page == index;
        let hot = enabled && response.hovered();
        let painter = ui.painter();
        let radius = metrics.tile_radius;
        if active {
            paint::glow(painter, tile, radius, palette.accent, 9.0);
            let top = paint::mix(palette.accent, palette.accent_hi, 0.4);
            paint::gradient(painter, tile, top, palette.accent_lo, radius);
            painter.rect_stroke(tile, radius, Stroke::new(1.0, Color32::from_white_alpha(46)), StrokeKind::Inside);
        } else if hot {
            painter.rect_filled(tile, radius, palette.field_hover);
        }
        let ink = match (active, hot) {
            (true, _) => palette.on_accent,
            (false, true) => palette.text,
            (false, false) => palette.text_2,
        };
        icon.paint(painter, Rect::from_center_size(tile.center(), Vec2::splat(metrics.tile * 0.5)), ink);
        if hot {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        if response.clicked() {
            *page = index;
        }
        index += 1;
        y += metrics.tile + 10.0;
    }
}
