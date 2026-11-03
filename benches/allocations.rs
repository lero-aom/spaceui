//! Allocation measurements are separate from timing to avoid instrumentation bias.
use spaceui::{Fonts, Icon, Theme, art, chat::markdown, egui, widgets};
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use std::{alloc::System, hint::black_box};

#[global_allocator]
static ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn profile(name: &str, mut work: impl FnMut()) {
    for _ in 0..10 {
        work();
    }
    let region = Region::new(ALLOCATOR);
    for _ in 0..100 {
        work();
    }
    let stats = region.change();
    println!(
        "{name:32} allocations/op {:10.2}   reallocations/op {:10.2}   allocated bytes/op {:12.2}",
        stats.allocations as f64 / 100.0,
        stats.reallocations as f64 / 100.0,
        stats.bytes_allocated as f64 / 100.0
    );
}

fn main() {
    let ctx = egui::Context::default();
    Theme { fonts: Fonts::BUNDLED, ..Theme::default() }.install(&ctx);
    spaceui::theme::set_reduced_motion(&ctx, true);
    profile("cached SVG frame", || {
        ctx.run_ui(egui::RawInput::default(), |ui| {
            Icon::Orbit.paint(
                ui.painter(),
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(64.0)),
                egui::Color32::WHITE,
            );
        })
        .drop_without_applying_deltas();
    });
    profile("button frame", || {
        ctx.run_ui(egui::RawInput::default(), |ui| {
            let _ = widgets::primary(ui, "Connect");
        })
        .drop_without_applying_deltas();
    });
    let text = "A **formatted** paragraph with [a link](https://example.com). ".repeat(100);
    profile("100-paragraph inline parse", || {
        black_box(markdown::inline_job(
            &text,
            &egui::FontId::proportional(14.0),
            egui::Color32::WHITE,
            &Theme::default().palette,
        ));
    });
    for px in 16..400 {
        ctx.run_ui(egui::RawInput::default(), |ui| {
            Icon::Orbit.paint(
                ui.painter(),
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(px as f32)),
                egui::Color32::WHITE,
            );
        })
        .drop_without_applying_deltas();
    }
    println!("SVG cache after 384 sizes: {:?}", art::cache_stats(&ctx));
    art::clear_cache(&ctx);
    println!("SVG cache after clear: {:?}", art::cache_stats(&ctx));
}
