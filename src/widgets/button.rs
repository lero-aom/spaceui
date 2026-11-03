#[cfg(feature = "svg")]
use crate::Icon;
use crate::{Theme, paint, theme::semibold};
use egui::{Color32, CursorIcon, Response, Role, Sense, Stroke, StrokeKind, Ui, WidgetInfo, vec2};
#[cfg(feature = "svg")]
use egui::{Rect, Vec2};

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
    let font = if accent { semibold(super::text_size(ui, 13.0)) } else { super::text_font(ui, 13.0) };
    let galley = ui.fonts_mut(|f| f.layout_no_wrap(label.into(), font, p.text));
    let size = vec2((galley.size().x + 28.0).max(64.0), 30.0_f32.max(galley.size().y + 12.0));
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, label));
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
    painter.galley_with_override_text_color(rect.center() - galley.size() / 2.0, galley, ink);
    super::focus_ring(ui, &response, radius);
    if enabled { response.on_hover_cursor(CursorIcon::PointingHand) } else { response }
}

/// A square icon button. Its accessible name defaults to the icon's name;
/// use [`icon_button_labeled`] for an action-specific or translated name.
/// `size` is in points, clamped to at least one; non-finite sizes use 28.
#[cfg(feature = "svg")]
pub fn icon_button(ui: &mut Ui, icon: Icon, size: f32) -> Response {
    icon_button_labeled(ui, icon, size, icon.name())
}

/// An icon button with an accessible action name and matching tooltip.
/// Size normalization matches [`icon_button`].
#[cfg(feature = "svg")]
pub fn icon_button_labeled(ui: &mut Ui, icon: Icon, size: f32, label: &str) -> Response {
    let theme = Theme::of(ui.ctx());
    let p = &theme.palette;
    let size = if size.is_finite() { size.max(1.0) } else { 28.0 };
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), label));
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
    super::focus_ring(ui, &response, theme.metrics.control_radius);
    let response = response.on_hover_text(label);
    if ui.is_enabled() { response.on_hover_cursor(CursorIcon::PointingHand) } else { response }
}
