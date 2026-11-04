//! A small observatory app: a themed sky view, a target catalog and an
//! observation plan, using demo data. The same code as the crate's quick start.
//!
//! ```text
//! cargo run --example minimal
//! ```
use spaceui::egui;
use spaceui::prelude::*;

const RAIL: &[RailItem] = &[
    RailItem::page(Icon::Orbit, "Sky"),
    RailItem::page(Icon::Star, "Targets"),
    RailItem::Divider,
    RailItem::page(Icon::Settings, "Settings"),
];

#[derive(Default)]
struct App {
    page: usize,
    side: f32,
    queued: bool,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let regions = Shell::new("Observatory")
            .logo(Icon::Orbit.svg())
            .rail(RAIL)
            .right_column(&mut self.side, 320.0..=560.0)
            .show(ui, &mut self.page);

        ui.scope_builder(egui::UiBuilder::new().max_rect(regions.main), |ui| {
            ui.heading(["Sky", "Targets", "Settings"][self.page]);
            widgets::caption(ui, "Demo observing session · sample data only");
            match self.page {
                0 => {
                    ui.label("M31 · Andromeda Galaxy");
                    if widgets::primary(ui, "Add M31 to plan").clicked() {
                        self.queued = true;
                    }
                }
                1 => {
                    for target in ["M31 · Andromeda Galaxy", "M42 · Orion Nebula", "M45 · Pleiades"] {
                        ui.label(target);
                    }
                }
                _ => {
                    ui.label("Deep Space theme · no telescope connected");
                }
            }
        });
        if let Some(plan) = regions.right {
            ui.scope_builder(egui::UiBuilder::new().max_rect(plan.shrink(18.0)), |ui| {
                ui.heading("Observation plan");
                ui.label(if self.queued { "01 · M31 · Andromeda Galaxy" } else { "Add a target from the sky view." });
            });
        }
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        // Undecorated: the shell draws the title bar and the resize edges.
        viewport: window::viewport("Observatory", "spaceui-observatory"),
        ..Default::default()
    };
    eframe::run_native(
        "Observatory",
        options,
        Box::new(|cc| {
            let theme = Theme::deep_space();
            window::round_corners(cc, theme.palette.border);
            theme.install(&cc.egui_ctx);
            Ok(Box::new(App::default()))
        }),
    )
}
