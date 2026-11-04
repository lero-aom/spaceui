//! A moonlit observatory: Deep Space's surfaces with a cool blue accent.
//! The sky and capture settings use illustrative demo data.
//!
//! ```text
//! cargo run --example custom_theme
//! ```
use spaceui::egui::{self, Color32};
use spaceui::prelude::*;

#[path = "gallery/sky.rs"]
mod sky;

fn lunar() -> Theme {
    Theme {
        name: "Lunar",
        palette: Palette {
            accent: Color32::from_rgb(125, 211, 252),
            accent_hi: Color32::from_rgb(186, 230, 253),
            accent_lo: Color32::from_rgb(82, 160, 220),
            bubble: Color32::from_rgb(28, 44, 78),
            ..Palette::DEEP_SPACE
        },
        backdrop: Backdrop::Starfield,
        ..Theme::deep_space()
    }
}

#[derive(Default)]
struct App {
    page: usize,
    on: bool,
    level: usize,
    target: usize,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        const RAIL: &[RailItem] = &[RailItem::page(Icon::Orbit, "Sky"), RailItem::page(Icon::Camera, "Imaging")];
        let regions = Shell::new("Lunar Observatory").logo(Icon::Orbit.svg()).rail(RAIL).show(ui, &mut self.page);
        ui.scope_builder(egui::UiBuilder::new().max_rect(regions.main), |ui| {
            ui.heading(if self.page == 0 { "A moonlit view of the sky" } else { "Imaging sequence" });
            widgets::caption(ui, "Lunar palette · a custom astronomy theme · illustrative demo data");
            ui.add_space(12.0);
            if self.page == 0 {
                sky::show(ui, &mut self.target, self.level);
                ui.add_space(12.0);
            }
            widgets::card(ui, |ui| {
                ui.horizontal(|ui| {
                    if widgets::primary(ui, if self.on { "Pause sequence" } else { "Start sequence" }).clicked() {
                        self.on = !self.on;
                    }
                    if widgets::secondary(ui, "Reset").clicked() {
                        self.on = false;
                        self.level = 0;
                    }
                    widgets::switch_labeled(ui, &mut self.on, "Capture sequence enabled");
                    widgets::segmented(ui, &mut self.level, &["Wide", "Field", "Detail"]);
                    widgets::chip(ui, if self.on { "CAPTURING · DEMO" } else { "READY" }, Tone::Accent);
                    widgets::chip(ui, "LUNAR", Tone::Custom(Color32::from_rgb(255, 196, 92)));
                });
            });
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: window::viewport("Lunar Observatory", "spaceui-lunar"),
        ..Default::default()
    };
    eframe::run_native(
        "Lunar Observatory",
        options,
        Box::new(|cc| {
            let theme = lunar();
            window::round_corners(cc, theme.palette.border);
            theme.install(&cc.egui_ctx);
            Ok(Box::new(App::default()))
        }),
    )
}
