//! A design system for [egui] desktop apps.
//!
//! SpaceUI gives an egui app a complete, consistent look and window: dark,
//! card-based layouts with an accent colour, an icon rail, a custom title bar
//! and a chat column, built from small parts you can also use on their own.
//!
//! - **One theme, everywhere.** A [`Theme`] (a semantic [`Palette`],
//!   [`Metrics`], fonts and a [`Backdrop`]) is installed once per context;
//!   every widget reads it back with [`Theme::of`]. Nothing hard-codes a colour.
//! - **Smooth at any scale.** Gradients and fills get a one-pixel feathered
//!   edge like egui's own shapes ([`paint`]); icons and logos are SVG,
//!   rasterized at the exact pixel size they're drawn at ([`art`]).
//! - **Shell, not framework.** [`Shell`] paints the window and returns the
//!   [`Regions`] to fill; your app keeps its own state, pages and backends.
//!
//! # Quick start
//!
//! ```no_run
//! use spaceui::egui;
//! use spaceui::prelude::*;
//!
//! const RAIL: &[RailItem] = &[
//!     RailItem::page(Icon::Home, "Home"),
//!     RailItem::page(Icon::Chart, "Markets"),
//!     RailItem::Divider,
//!     RailItem::page(Icon::Settings, "Settings"),
//! ];
//!
//! #[derive(Default)]
//! struct App {
//!     page: usize,
//!     side: f32,
//! }
//!
//! impl eframe::App for App {
//!     fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
//!         let regions = Shell::new("My App")
//!             .logo(Icon::Orbit.svg())
//!             .rail(RAIL)
//!             .side_column(&mut self.side, 320.0..=560.0)
//!             .show(ui, &mut self.page);
//!
//!         ui.scope_builder(egui::UiBuilder::new().max_rect(regions.main), |ui| {
//!             ui.heading(["Home", "Markets", "Settings"][self.page]);
//!             if widgets::primary(ui, "Connect").clicked() {
//!                 // ...
//!             }
//!         });
//!     }
//! }
//!
//! fn main() -> eframe::Result {
//!     let options = eframe::NativeOptions {
//!         // Undecorated: the shell draws the title bar and the resize edges.
//!         viewport: window::viewport("My App", "my-app"),
//!         ..Default::default()
//!     };
//!     eframe::run_native(
//!         "My App",
//!         options,
//!         Box::new(|cc| {
//!             let theme = Theme::carbon_copper();
//!             window::round_corners(cc, theme.palette.border);
//!             theme.install(&cc.egui_ctx);
//!             Ok(Box::new(App::default()))
//!         }),
//!     )
//! }
//! ```
//!
//! The repository's `examples/` directory has this app (`minimal`), a custom
//! theme (`custom_theme`) and a gallery of every part in both themes (`gallery`).
//!
//! # Organisation
//!
//! The modules build on each other, from colours to whole windows:
//!
//! | Module | Holds |
//! |---|---|
//! | [`theme`] | [`Theme`], [`Palette`], [`Metrics`], [`Backdrop`] and the fonts |
//! | [`paint`] | Anti-aliased gradients, polygon fills and glows |
//! | [`art`] | The built-in [`Icon`] set and app [`Svg`]s such as logos |
//! | [`widgets`] | Buttons, pills, tabs, cards, switches, fields and sliders |
//! | [`shell`] | The window: title bar, icon rail, cards and a side column |
//! | [`chat`] | A chat column: tabs, transcript, composer and Markdown |
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
//! SpaceUI runs wherever egui does. On Windows it uses the system's Segoe UI
//! fonts and asks Windows 11 for rounded window corners; elsewhere egui's
//! bundled fonts stand in and windows keep their system corners.
//!
//! # Minimum supported Rust version
//!
//! 1.95, the minimum of egui 0.37. Raising it is a minor-version change.
//!
//! [egui]: https://github.com/emilk/egui

pub mod art;
pub mod chat;
pub mod paint;
pub mod shell;
mod text;
pub mod theme;
pub mod widgets;
pub mod window;

#[doc(no_inline)]
pub use art::{Icon, Svg};
/// The egui version SpaceUI is built on, so apps can depend on one egui.
pub use egui;
#[doc(no_inline)]
pub use shell::{RailItem, Regions, Shell};
#[doc(no_inline)]
pub use theme::{Backdrop, Metrics, Palette, Theme};

/// The names most apps need: `use spaceui::prelude::*;`.
pub mod prelude {
    #[doc(no_inline)]
    pub use crate::art::{Icon, Svg};
    #[doc(no_inline)]
    pub use crate::shell::{RailItem, Regions, Shell};
    #[doc(no_inline)]
    pub use crate::theme::{Backdrop, Metrics, Palette, Theme, mono, regular, semibold};
    #[doc(no_inline)]
    pub use crate::widgets::Tone;
    #[doc(no_inline)]
    pub use crate::{chat, paint, widgets, window};
}

// The README's examples compile and run as doc tests.
#[cfg(doctest)]
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
