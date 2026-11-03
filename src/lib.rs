//! A design system for [egui] desktop apps.
//!
//! SpaceUI gives an egui app a complete, consistent look and window: dark,
//! card-based layouts with an accent colour, an icon rail, a custom title bar
//! and a chat column, built from small parts you can also use on their own.
//!
//! - **One theme, everywhere.** A [`Theme`] (a semantic [`Palette`],
//!   [`Metrics`], [`Fonts`] and a [`Backdrop`]) is installed once per context;
//!   every widget reads it back with [`Theme::of`]. Nothing hard-codes a colour.
//! - **Smooth at any scale.** Gradients and fills get a one-pixel feathered
//!   edge like egui's own shapes ([`paint`]); icons and logos are SVG,
//!   rasterized at the exact pixel size they're drawn at (`art`).
//! - **Shell, not framework.** `Shell` paints the window (a title bar, an
//!   optional rail, columns on either side) and returns the `Regions` to
//!   fill; your app keeps its own state, pages and backends.
//!
//! # Quick start
//!
//! ```
//! use spaceui::{Theme, egui, widgets};
//! let ctx = egui::Context::default();
//! Theme::carbon_copper().install(&ctx);
//! ctx.run_ui(Default::default(), |ui| {
//!     if widgets::primary(ui, "Connect").clicked() {
//!         // Start the application's action.
//!     }
//! }).drop_without_applying_deltas();
//! ```
//!
//! The repository's `minimal`, `custom_theme` and `gallery` examples show
//! complete eframe applications, a custom palette, and every component.
//!
//! # Features
//!
//! Defaults include `svg`, `shell`, `chat`, `native`, and `screenshot`.
//! Disable defaults for themes, painting, and basic widgets alone. `shell`
//! and `chat` enable `svg`; `screenshot` enables the SVG/PNG dependency.
//! `native` enables native window handles. Accessibility metadata is always
//! available through egui. Optional modules are absent when disabled.
//!
//! # Organisation
//!
//! The modules build on each other, from colours to whole windows:
//!
//! | Module | Holds |
//! |---|---|
//! | [`theme`] | [`Theme`], [`Palette`], [`Metrics`], [`Fonts`] and [`Backdrop`] |
//! | [`paint`] | Anti-aliased gradients, polygon fills and glows |
//! | `art` | The built-in `Icon` set and app `Svg`s such as logos |
//! | [`widgets`] | Buttons, pills, tabs, cards, switches, fields and sliders |
//! | `shell` | The window: title bar, optional icon rail, cards and columns on either side |
//! | `chat` | A chat column: tabs, transcript, composer and Markdown |
//! | [`window`] | Native window setup and screenshot runs |
//!
//! [`prelude`] imports the names most apps use.
//!
//! # Themes
//!
//! Two themes ship: [`Theme::carbon_copper`] (near-black carbon with copper
//! accents) and [`Theme::deep_space`] (indigo with violet accents and a
//! starfield). A theme is plain data, so a new one is a struct literal; see
//! [`Theme`]. Installing another theme restyles the app on the next frame.
//!
//! # Platform support
//!
//! CI targets Windows, Linux and macOS desktop builds. Headless tests do not
//! certify native window-manager behavior. Other egui targets, including the
//! web, are not currently part of SpaceUI's support promise. On Windows the
//! default fonts come from Segoe UI; other systems use bundled fallback fonts.
//! The `native` feature provides optional Windows 11 rounded corners.
//!
//! # Minimum supported Rust version
//!
//! 1.95, the minimum of egui 0.37. Raising it is a minor-version change.
//!
//! [egui]: https://github.com/emilk/egui

#[cfg(feature = "svg")]
pub mod art;
#[cfg(feature = "chat")]
pub mod chat;
pub mod paint;
#[cfg(feature = "shell")]
pub mod shell;
mod text;
pub mod theme;
pub mod widgets;
pub mod window;

#[doc(no_inline)]
#[cfg(feature = "svg")]
pub use art::{Icon, Svg};
/// The egui version SpaceUI is built on, so apps can depend on one egui.
pub use egui;
#[doc(no_inline)]
#[cfg(feature = "shell")]
pub use shell::{RailItem, Regions, Shell, WindowLabels};
#[doc(no_inline)]
pub use theme::{Backdrop, Fonts, Metrics, Palette, Theme};

/// The names most apps need: `use spaceui::prelude::*;`.
pub mod prelude {
    #[doc(no_inline)]
    #[cfg(feature = "svg")]
    pub use crate::art::{Icon, Svg};
    #[doc(no_inline)]
    #[cfg(feature = "chat")]
    pub use crate::chat;
    #[doc(no_inline)]
    #[cfg(feature = "shell")]
    pub use crate::shell::{RailItem, Regions, Shell, WindowLabels};
    #[doc(no_inline)]
    pub use crate::theme::{Backdrop, FontFile, Fonts, Metrics, Palette, Theme, mono, regular, semibold};
    #[doc(no_inline)]
    pub use crate::widgets::Tone;
    #[doc(no_inline)]
    pub use crate::{paint, widgets, window};
}

// The README's examples compile and run as doc tests.
#[cfg(all(doctest, feature = "shell", feature = "native", feature = "chat"))]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

#[cfg(test)]
pub(crate) mod test_util {
    use egui::{Context, RawInput, Ui};

    /// One frame of `add` on a fresh context with the default theme.
    pub(crate) fn run_ui(add: impl FnMut(&mut Ui)) -> Context {
        let ctx = Context::default();
        crate::Theme::default().install(&ctx);
        ctx.run_ui(RawInput::default(), add).drop_without_applying_deltas();
        ctx
    }
}
