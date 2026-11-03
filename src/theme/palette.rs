use egui::Color32;

/// Semantic colour roles. Widgets ask for a role ("the accent", "a field"),
/// never for a hue, so swapping the palette restyles everything.
///
/// Use [`Palette::CARBON_COPPER`] or [`Palette::DEEP_SPACE`], or derive your
/// own with struct update syntax:
///
/// ```
/// use spaceui::{Palette, egui::Color32};
///
/// let teal = Palette { accent: Color32::from_rgb(40, 190, 170), ..Palette::CARBON_COPPER };
/// assert_eq!(teal.panel, Palette::CARBON_COPPER.panel);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    /// Top of the window backdrop.
    pub bg_top: Color32,
    /// Bottom of the window backdrop.
    pub bg_bottom: Color32,
    /// Card surfaces, at their foot.
    pub panel: Color32,
    /// Card surfaces, at their top.
    pub panel_hi: Color32,
    /// Inputs and inactive controls.
    pub field: Color32,
    /// A field or inactive control under the pointer.
    pub field_hover: Color32,
    /// Recessed wells: text-edit backgrounds, code, chart plots.
    pub well: Color32,
    /// Hairlines on controls.
    pub border: Color32,
    /// Hairlines on cards and dividers.
    pub border_soft: Color32,
    /// Body text.
    pub text: Color32,
    /// Secondary text and idle icons.
    pub text_2: Color32,
    /// Captions, hints and disabled ink.
    pub muted: Color32,
    /// Selection and the primary action.
    pub accent: Color32,
    /// The lit end of accent gradients; links.
    pub accent_hi: Color32,
    /// The shaded end of accent gradients; selection fill.
    pub accent_lo: Color32,
    /// Ink on an accent fill.
    pub on_accent: Color32,
    /// Gains, health, "connected".
    pub positive: Color32,
    /// Losses, errors, destructive actions.
    pub negative: Color32,
    /// Caution, preview or degraded states.
    pub warning: Color32,
    /// Neutral information.
    pub info: Color32,
    /// The owner's own chat bubbles.
    pub bubble: Color32,
}

impl Palette {
    /// Near-black carbon with brushed-copper accents.
    pub const CARBON_COPPER: Palette = Palette {
        bg_top: Color32::from_rgb(19, 19, 22),
        bg_bottom: Color32::from_rgb(12, 12, 14),
        panel: Color32::from_rgb(22, 22, 25),
        panel_hi: Color32::from_rgb(29, 29, 33),
        field: Color32::from_rgb(34, 34, 39),
        field_hover: Color32::from_rgb(45, 45, 51),
        well: Color32::from_rgb(15, 15, 18),
        border: Color32::from_rgb(60, 59, 66),
        border_soft: Color32::from_rgb(39, 39, 44),
        text: Color32::from_rgb(238, 235, 230),
        text_2: Color32::from_rgb(186, 181, 174),
        muted: Color32::from_rgb(134, 129, 122),
        accent: Color32::from_rgb(234, 124, 52),
        accent_hi: Color32::from_rgb(255, 166, 98),
        accent_lo: Color32::from_rgb(178, 80, 24),
        on_accent: Color32::WHITE,
        positive: Color32::from_rgb(64, 208, 142),
        negative: Color32::from_rgb(242, 100, 100),
        warning: Color32::from_rgb(240, 186, 76),
        info: Color32::from_rgb(116, 170, 232),
        bubble: Color32::from_rgb(52, 40, 32),
    };

    /// Deep-space indigo with violet accents.
    pub const DEEP_SPACE: Palette = Palette {
        bg_top: Color32::from_rgb(15, 12, 38),
        bg_bottom: Color32::from_rgb(8, 7, 22),
        panel: Color32::from_rgb(17, 15, 40),
        panel_hi: Color32::from_rgb(24, 21, 54),
        field: Color32::from_rgb(27, 24, 59),
        field_hover: Color32::from_rgb(34, 30, 72),
        well: Color32::from_rgb(10, 9, 28),
        border: Color32::from_rgb(44, 39, 86),
        border_soft: Color32::from_rgb(32, 28, 66),
        text: Color32::from_rgb(236, 234, 251),
        text_2: Color32::from_rgb(182, 177, 218),
        muted: Color32::from_rgb(122, 117, 164),
        accent: Color32::from_rgb(141, 125, 247),
        accent_hi: Color32::from_rgb(190, 176, 255),
        accent_lo: Color32::from_rgb(98, 79, 216),
        on_accent: Color32::WHITE,
        positive: Color32::from_rgb(52, 211, 153),
        negative: Color32::from_rgb(248, 113, 113),
        warning: Color32::from_rgb(251, 191, 36),
        info: Color32::from_rgb(199, 186, 236),
        bubble: Color32::from_rgb(43, 40, 92),
    };
}

impl Default for Palette {
    fn default() -> Self {
        Self::CARBON_COPPER
    }
}
