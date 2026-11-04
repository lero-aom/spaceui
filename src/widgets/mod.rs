//! Controls in the shared style.
//!
//! Each widget reads the installed [`Theme`](crate::Theme), allocates its own
//! space in the [`Ui`](egui::Ui) and, where it is interactive, returns egui's
//! [`Response`](egui::Response), so the usual `.clicked()`, `.on_hover_text()`
//! and popup helpers work.
//! Install a [`Theme`](crate::Theme::install) before the first UI frame to
//! configure the fonts and egui styles used by these controls.
//!
//! Controls publish accessible roles and values and show a keyboard focus
//! outline. Buttons activate with Enter/Space; tabs and segmented controls
//! use Left/Right and Home/End; notched sliders also accept Up/Down.
//! Give an unlabelled switch or slider a visible label with
//! [`Response::labelled_by`](egui::Response::labelled_by), or use
//! [`switch_labeled`] / [`notches_labeled`] to supply an accessible name.
//! `icon_button_labeled` supplies a translated or action-specific name for
//! an icon button. Text sizes follow egui's body font; increasing it also
//! expands text-bearing controls. For whole-interface scaling, use
//! [`Context::set_zoom_factor`](egui::Context::set_zoom_factor).
//!
//! ```
//! use spaceui::{Theme, egui, widgets, widgets::Tone};
//! # #[cfg(feature = "svg")]
//! # use spaceui::Icon;
//!
//! # let ctx = egui::Context::default();
//! # Theme::carbon_copper().install(&ctx);
//! # ctx.run_ui(Default::default(), |ui| {
//! ui.horizontal(|ui| {
//!     if widgets::primary(ui, "Connect").clicked() {
//!         // ...
//!     }
//!     widgets::secondary(ui, "Cancel");
//!     # #[cfg(feature = "svg")]
//!     widgets::badge(ui, Icon::Shield, "Read-only", Tone::Positive);
//!     widgets::chip(ui, "DEMO", Tone::Warning);
//! });
//! # }).drop_without_applying_deltas();
//! ```
//!
//! Name input controls through a visible label or an accessible name:
//!
//! ```
//! # use spaceui::{Theme, egui, widgets};
//! # let ctx = egui::Context::default();
//! # Theme::default().install(&ctx);
//! # let (mut notifications, mut effort, mut query) = (true, 2, String::new());
//! # ctx.run_ui(Default::default(), |ui| {
//! ui.horizontal(|ui| {
//!     let label = ui.label("Notifications");
//!     widgets::switch(ui, &mut notifications).labelled_by(label.id);
//! });
//! widgets::notches_labeled(ui, 160.0, 5, &mut effort, "Effort");
//! widgets::text_field_id(ui, "search", &mut query, "Search…")
//!     .accessible_name("Target search");
//! # }).drop_without_applying_deltas();
//! ```
//!
//! | Widget | Use |
//! |---|---|
//! | [`primary`], [`secondary`], `icon_button` | Actions |
//! | [`pill`], `badge`, [`chip`], [`dropdown`] | Status and menus |
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

#[cfg(feature = "svg")]
pub use button::{icon_button, icon_button_labeled};
pub use button::{primary, secondary};
pub use card::{caption, card, metric, plate, section};
pub use field::{text_field, text_field_id};
#[cfg(feature = "svg")]
pub use pill::badge;
pub use pill::{chip, dropdown, pill};
pub use slider::{notches, notches_labeled};
pub use tabs::{segmented, tabs};
pub use toggle::{switch, switch_labeled};

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

// Respect the host application's body font and text scaling. The stock theme
// uses a 14-point body, which preserves the original visual proportions.
fn text_font(ui: &egui::Ui, size: f32) -> egui::FontId {
    let mut font = egui::TextStyle::Body.resolve(ui.style());
    font.size *= size / 14.0;
    font
}

fn text_size(ui: &egui::Ui, size: f32) -> f32 {
    text_font(ui, size).size
}

fn focus_ring(ui: &egui::Ui, response: &egui::Response, radius: f32) {
    if response.has_focus() && ui.is_enabled() {
        ui.painter().rect_stroke(
            response.rect.expand(2.0),
            radius + 2.0,
            egui::Stroke::new(2.0, crate::Theme::of(ui.ctx()).palette.accent_hi),
            egui::StrokeKind::Outside,
        );
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
