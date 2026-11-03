//! A theme of your own: Deep Space's metrics and surfaces with an icy blue accent
//! and a plain backdrop. Every widget follows it without further changes.
//!
//! ```text
//! cargo run --example custom_theme
//! ```
use spaceui::egui::{self, Color32};
use spaceui::prelude::*;

fn arctic() -> Theme {
    Theme {
        name: "Arctic",
        palette: Palette {
            accent: Color32::from_rgb(64, 156, 255),
            accent_hi: Color32::from_rgb(130, 196, 255),
            accent_lo: Color32::from_rgb(28, 98, 196),
            bubble: Color32::from_rgb(28, 44, 78),
            ..Palette::DEEP_SPACE
        },
        backdrop: Backdrop::Plain,
        ..Theme::deep_space()
    }
}

#[derive(Default)]
struct App {
    page: usize,
    on: bool,
    level: usize,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        const RAIL: &[RailItem] = &[RailItem::page(Icon::Home, "Home"), RailItem::page(Icon::Palette, "Theme")];
        let regions = Shell::new("Arctic").logo(Icon::Orbit.svg()).rail(RAIL).show(ui, &mut self.page);
        ui.scope_builder(egui::UiBuilder::new().max_rect(regions.main), |ui| {
            ui.heading("A custom theme");
            widgets::caption(ui, "Deep Space's surfaces and metrics, an icy accent and a plain backdrop.");
            ui.add_space(12.0);
            widgets::card(ui, |ui| {
                ui.horizontal(|ui| {
                    widgets::primary(ui, "Primary");
                    widgets::secondary(ui, "Secondary");
                    widgets::switch(ui, &mut self.on);
                    widgets::segmented(ui, &mut self.level, &["Low", "Mid", "High"]);
                    widgets::chip(ui, "ACCENT", Tone::Accent);
                    widgets::chip(ui, "ONE-OFF", Tone::Custom(Color32::from_rgb(255, 196, 92)));
                });
            });
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions { viewport: window::viewport("Arctic", "arctic"), ..Default::default() };
    eframe::run_native(
        "Arctic",
        options,
        Box::new(|cc| {
            let theme = arctic();
            window::round_corners(cc, theme.palette.border);
            theme.install(&cc.egui_ctx);
            Ok(Box::new(App::default()))
        }),
    )
}
