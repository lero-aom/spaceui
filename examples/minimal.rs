//! The smallest SpaceUI app: a themed window with a title bar, an icon rail,
//! a main card and a side column. The same code as the crate's quick start.
//!
//! ```text
//! cargo run --example minimal
//! ```
use spaceui::egui;
use spaceui::prelude::*;

const RAIL: &[RailItem] = &[
    RailItem::page(Icon::Home, "Home"),
    RailItem::page(Icon::Chart, "Markets"),
    RailItem::Divider,
    RailItem::page(Icon::Settings, "Settings"),
];

#[derive(Default)]
struct App {
    page: usize,
    side: f32,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let regions = Shell::new("My App")
            .logo(Icon::Orbit.svg())
            .rail(RAIL)
            .side_column(&mut self.side, 320.0..=560.0)
            .show(ui, &mut self.page);

        ui.scope_builder(egui::UiBuilder::new().max_rect(regions.main), |ui| {
            ui.heading(["Home", "Markets", "Settings"][self.page]);
            if widgets::primary(ui, "Connect").clicked() {
                // ...
            }
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        // Undecorated: the shell draws the title bar and the resize edges.
        viewport: window::viewport("My App", "my-app"),
        ..Default::default()
    };
    eframe::run_native(
        "My App",
        options,
        Box::new(|cc| {
            let theme = Theme::carbon_copper();
            window::round_corners(cc, theme.palette.border);
            theme.install(&cc.egui_ctx);
            Ok(Box::new(App::default()))
        }),
    )
}
