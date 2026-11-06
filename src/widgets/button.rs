#[cfg(feature = "svg")]
use crate::Icon;
use crate::{
    Theme, paint,
    theme::{ControlMetrics, normalize, normalize_vec, semibold},
};
#[cfg(feature = "svg")]
use egui::Rect;
use egui::{Color32, CursorIcon, Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2, Widget, WidgetInfo};

/// A configurable SpaceUI push button, usable with [`Ui::add`] and [`Ui::add_enabled`].
/// The function helpers [`primary`] and [`secondary`] use the same implementation.
///
/// ```
/// # use spaceui::{egui, widgets::Button};
/// # let ctx = egui::Context::default();
/// # ctx.run_ui(Default::default(), |ui| {
/// let response = ui.add(Button::primary("Connect").min_size(egui::vec2(120.0, 40.0)));
/// if response.clicked() { /* connect */ }
/// # }).drop_without_applying_deltas();
/// ```
#[derive(Clone, Copy, Debug)]
#[must_use = "add the button to a Ui"]
pub struct Button<'a> {
    label: &'a str,
    accent: bool,
    min_size: Option<Vec2>,
    padding: Option<Vec2>,
    radius: Option<f32>,
    fill: Option<Color32>,
    text_color: Option<Color32>,
}

impl<'a> Button<'a> {
    /// A secondary action with a field-coloured background and hairline border.
    pub fn new(label: &'a str) -> Self {
        Self { label, accent: false, min_size: None, padding: None, radius: None, fill: None, text_color: None }
    }

    /// The main action, with an accent gradient and hover glow.
    pub fn primary(label: &'a str) -> Self {
        Self { accent: true, ..Self::new(label) }
    }

    /// A secondary action; equivalent to [`Self::new`].
    pub fn secondary(label: &'a str) -> Self {
        Self::new(label)
    }

    /// Override the minimum size; text and padding may require more space.
    /// Finite components are clamped to `0..=4096`; non-finite components use
    /// the installed [`ControlMetrics`] value. Padding follows the same rule.
    pub fn min_size(mut self, size: Vec2) -> Self {
        self.min_size = Some(size);
        self
    }

    /// Override horizontal and vertical padding on each side of the text.
    /// Normalization follows [`Self::min_size`].
    pub fn padding(mut self, padding: Vec2) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Override the corner radius, clamped to the button's half-size.
    /// Negative values become zero; non-finite values use the theme's radius.
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }

    /// Use a solid fill while enabled, in place of the usual background.
    /// Pair this with [`Self::text_color`] when a different foreground is needed.
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Override the text colour while enabled. Disabled buttons use the palette.
    pub fn text_color(mut self, color: Color32) -> Self {
        self.text_color = Some(color);
        self
    }
}

/// The main action: an accent gradient with a lit top edge and a glow on hover.
/// Disabled, it drops the accent so it never looks pressable.
pub fn primary(ui: &mut Ui, label: &str) -> Response {
    ui.add(Button::primary(label))
}

/// Any other action: a field-coloured button with a hairline border.
pub fn secondary(ui: &mut Ui, label: &str) -> Response {
    ui.add(Button::secondary(label))
}

impl Widget for Button<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        push_button(ui, self)
    }
}

fn push_button(ui: &mut Ui, button: Button<'_>) -> Response {
    let theme = Theme::of(ui.ctx());
    let p = &theme.palette;
    let metrics = ControlMetrics::of(ui.ctx()).button;
    let padding = normalize_vec(button.padding.unwrap_or(metrics.padding), metrics.padding);
    let min_size = normalize_vec(button.min_size.unwrap_or(metrics.min_size), metrics.min_size);
    let enabled = ui.is_enabled();
    let accent = button.accent && enabled;
    let label = button.label;
    let font = if accent { semibold(super::text_size(ui, 13.0)) } else { super::text_font(ui, 13.0) };
    let galley = ui.fonts_mut(|f| f.layout_no_wrap(label.into(), font, p.text));
    let size = (galley.size() + padding * 2.0).max(min_size);
    let default_radius = normalize(theme.metrics.control_radius, 8.0);
    let radius = normalize(button.radius.unwrap_or(default_radius), default_radius).min(size.min_elem() / 2.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, label));
    let hot = enabled && (response.hovered() || response.is_pointer_button_down_on());
    let painter = ui.painter();
    if let Some(fill) = button.fill.filter(|_| enabled) {
        painter.rect(rect, radius, fill, Stroke::new(1.0, p.border), StrokeKind::Inside);
    } else if accent {
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
    let ink = if enabled { button.text_color.unwrap_or(ink) } else { ink };
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
