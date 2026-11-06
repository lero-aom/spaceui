use egui::{Context, Id, Vec2, vec2};

/// Minimum size and padding on each side of a text-bearing control, in points.
/// Text can make the control larger than its minimum.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlSize {
    /// Minimum allocated size. Zero allows the text and padding to decide.
    pub min_size: Vec2,
    /// Horizontal and vertical padding on each side of the text.
    pub padding: Vec2,
}

impl ControlSize {
    pub(crate) fn normalized(self, fallback: Self) -> Self {
        Self {
            min_size: normalize_vec(self.min_size, fallback.min_size),
            padding: normalize_vec(self.padding, fallback.padding),
        }
    }
}

/// Context-wide sizing defaults for buttons, choice groups and text fields.
///
/// Install once with [`Self::install`]. These settings are independent of the
/// palette and survive [`super::Theme::install`], so switching colours preserves
/// the application's density preference. Defaults reproduce the original sizes.
/// Button builders can override individual values. Radii still come from
/// [`super::Metrics`], and fonts follow egui's body style.
///
/// Values are normalized on installation: non-finite components use the default,
/// and finite components are clamped to `0..=4096` points.
///
/// ```
/// use spaceui::{egui, theme::{ControlMetrics, ControlSize}};
/// let ctx = egui::Context::default();
/// ControlMetrics {
///     button: ControlSize {
///         min_size: egui::vec2(80.0, 40.0),
///         padding: egui::vec2(18.0, 10.0),
///     },
///     ..ControlMetrics::default()
/// }.install(&ctx);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlMetrics {
    /// Push buttons, including the `primary` and `secondary` helpers.
    pub button: ControlSize,
    /// Each page tab; the minimum width applies to each choice.
    pub tab: ControlSize,
    /// Each segment; minimum height and vertical padding include the
    /// surrounding three-point inset on each side.
    pub segmented: ControlSize,
    /// Text fields. Width still fills the available space; `min_size.x` is unused.
    pub field: ControlSize,
}

impl Default for ControlMetrics {
    fn default() -> Self {
        Self {
            button: ControlSize { min_size: vec2(64.0, 30.0), padding: vec2(14.0, 6.0) },
            tab: ControlSize { min_size: vec2(0.0, 34.0), padding: vec2(12.0, 10.0) },
            segmented: ControlSize { min_size: vec2(0.0, 30.0), padding: vec2(11.0, 8.5) },
            field: ControlSize { min_size: vec2(0.0, 34.0), padding: vec2(12.0, 10.0) },
        }
    }
}

impl ControlMetrics {
    fn id() -> Id {
        Id::unique("spaceui-control-metrics")
    }

    /// Install normalized sizing defaults and request a repaint when they change.
    pub fn install(self, ctx: &Context) {
        let defaults = Self::default();
        let normalized = Self {
            button: self.button.normalized(defaults.button),
            tab: self.tab.normalized(defaults.tab),
            segmented: self.segmented.normalized(defaults.segmented),
            field: self.field.normalized(defaults.field),
        };
        if Self::of(ctx) != normalized {
            ctx.data_mut(|data| data.insert_temp(Self::id(), normalized));
            ctx.request_repaint();
        }
    }

    /// Installed sizing defaults, or [`Self::default`] before installation.
    #[must_use]
    pub fn of(ctx: &Context) -> Self {
        ctx.data(|data| data.get_temp(Self::id()).unwrap_or_default())
    }
}

pub(crate) fn normalize(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value.clamp(0.0, 4096.0) } else { fallback }
}

pub(crate) fn normalize_vec(value: Vec2, fallback: Vec2) -> Vec2 {
    vec2(normalize(value.x, fallback.x), normalize(value.y, fallback.y))
}
