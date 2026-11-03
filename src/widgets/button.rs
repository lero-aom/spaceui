use crate::{Icon, Theme, paint, theme::semibold};
use egui::{Color32, CursorIcon, FontId, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, vec2};

/// The main action: an accent gradient with a lit top edge and a glow on hover.
/// Disabled, it drops the accent so it never looks pressable.
pub fn primary(ui: &mut Ui, label: &str) -> Response {
    push_button(ui, label, true)
}

/// Any other action: a field-coloured button with a hairline border.
pub fn secondary(ui: &mut Ui, label: &str) -> Response {
    push_button(ui, label, false)
}

fn push_button(ui: &mut Ui, label: &str, accent: bool) -> Response {
    let theme = Theme::of(ui.ctx());
    let (p, radius) = (&theme.palette, theme.metrics.control_radius);
    let enabled = ui.is_enabled();
    let accent = accent && enabled;
    let font = if accent { semibold(13.0) } else { FontId::proportional(13.0) };
    let galley = ui.fonts_mut(|f| f.layout_no_wrap(label.into(), font, p.text));
    let (rect, response) = ui.allocate_exact_size(vec2((galley.size().x + 28.0).max(64.0), 30.0), Sense::click());
    let hot = enabled && (response.hovered() || response.is_pointer_button_down_on());
    let painter = ui.painter();
    if accent {
        let top = if hot { p.accent_hi } else { paint::mix(p.accent, p.accent_hi, 0.35) };
        let bottom = if hot { p.accent } else { p.accent_lo };
        if hot {
            paint::glow(painter, rect, radius, p.accent, 6.0);
        }
        paint::gradient(painter, rect, top, bottom, radius);
        painter.hline(
            (rect.left() + radius)..=(rect.right() - radius),
            rect.top() + 1.0,
            Stroke::new(1.0, Color32::from_white_alpha(70)),
        );
    } else {
        let border = if hot { p.accent.gamma_multiply(0.6) } else { p.border };
        painter.rect(
            rect,
            radius,
            if hot { p.field_hover } else { p.field },
            Stroke::new(1.0, border),
            StrokeKind::Inside,
        );
    }
    let ink = match (accent, enabled) {
        (_, false) => p.muted,
        (true, true) => p.on_accent,
        (false, true) => p.text,
    };
    painter.galley(rect.center() - galley.size() / 2.0, galley, ink);
    if enabled { response.on_hover_cursor(CursorIcon::PointingHand) } else { response }
}

/// A square icon button `size` points wide with a soft hover plate; the icon
/// fills 55% of it. Add `.on_hover_text(...)` to name it.
pub fn icon_button(ui: &mut Ui, icon: Icon, size: f32) -> Response {
    let theme = Theme::of(ui.ctx());
    let p = &theme.palette;
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let hot = response.hovered() && ui.is_enabled();
    if hot {
        ui.painter().rect_filled(rect, theme.metrics.control_radius, p.field_hover);
    }
    let color = if !ui.is_enabled() {
        p.muted.gamma_multiply(0.5)
    } else if hot {
        p.text
    } else {
        p.text_2
    };
    icon.paint(ui.painter(), Rect::from_center_size(rect.center(), Vec2::splat(size * 0.55)), color);
    if hot { response.on_hover_cursor(CursorIcon::PointingHand) } else { response }
}
