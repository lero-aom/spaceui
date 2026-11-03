//! Controls in the shared style.
//!
//! Each widget reads the installed [`Theme`](crate::Theme), allocates its own
//! space in the [`Ui`](egui::Ui) and, where it is interactive, returns egui's
//! [`Response`](egui::Response), so the usual `.clicked()`, `.on_hover_text()`
//! and popup helpers work.
//!
//! ```
//! use spaceui::{Icon, Theme, egui, widgets, widgets::Tone};
//!
//! # let ctx = egui::Context::default();
//! # Theme::carbon_copper().install(&ctx);
//! # ctx.run_ui(Default::default(), |ui| {
//! ui.horizontal(|ui| {
//!     if widgets::primary(ui, "Connect").clicked() {
//!         // ...
//!     }
//!     widgets::secondary(ui, "Cancel");
//!     widgets::badge(ui, Icon::Shield, "Read-only", Tone::Positive);
//!     widgets::chip(ui, "PAPER", Tone::Warning);
//! });
//! # }).drop_without_applying_deltas();
//! ```
//!
//! | Widget | Use |
//! |---|---|
//! | [`primary`], [`secondary`], [`icon_button`] | Actions |
//! | [`pill`], [`badge`], [`chip`], [`dropdown`] | Status and menus |
//! | [`tabs`], [`segmented`] | Choosing a view |
//! | [`card`], [`plate`], [`section`], [`caption`], [`metric`] | Layout and text |
//! | [`switch`], [`text_field`], [`notches`] | Input |
mod button;
mod card;
mod field;
mod pill;
mod slider;
mod tabs;
mod toggle;

pub use button::{icon_button, primary, secondary};
pub use card::{caption, card, metric, plate, section};
pub use field::text_field;
pub use pill::{badge, chip, dropdown, pill};
pub use slider::notches;
pub use tabs::{segmented, tabs};
pub use toggle::switch;

use crate::theme::Palette;
use egui::Color32;

/// A semantic colour for status widgets: a palette role, or a colour of
/// your own with [`Tone::Custom`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Tone {
    /// Body text colour.
    Neutral,
    /// The accent (selection, emphasis).
    Accent,
    /// Gains, health, "connected".
    Positive,
    /// Losses and errors.
    Negative,
    /// Caution and preview states.
    Warning,
    /// Neutral information.
    Info,
    /// De-emphasised.
    Muted,
    /// A colour of your own, used as given.
    Custom(Color32),
}

impl Tone {
    /// The tone's colour in `palette`.
    #[must_use]
    pub fn color(self, palette: &Palette) -> Color32 {
        match self {
            Tone::Neutral => palette.text,
            Tone::Accent => palette.accent_hi,
            Tone::Positive => palette.positive,
            Tone::Negative => palette.negative,
            Tone::Warning => palette.warning,
            Tone::Info => palette.info,
            Tone::Muted => palette.muted,
            Tone::Custom(color) => color,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tones_follow_the_palette_unless_custom() {
        let palette = Palette::DEEP_SPACE;
        assert_eq!(Tone::Accent.color(&palette), palette.accent_hi);
        assert_eq!(Tone::Negative.color(&palette), palette.negative);
        let teal = Color32::from_rgb(40, 190, 170);
        assert_eq!(Tone::Custom(teal).color(&palette), teal);
    }
}
