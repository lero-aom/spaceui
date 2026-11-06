//! Repeatable CPU workloads; run with `cargo bench --bench workloads`.
use spaceui::{Fonts, Icon, Theme, chat::markdown, egui, paint, widgets};
use std::{hint::black_box, time::Instant};

fn measure(name: &str, iterations: usize, mut work: impl FnMut()) {
    for _ in 0..10 {
        work();
    }
    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        work();
        samples.push(start.elapsed().as_secs_f64() * 1_000_000.0);
    }
    samples.sort_by(f64::total_cmp);
    println!(
        "{name:32} median {:10.2} us   p95 {:10.2} us   n={iterations}",
        samples[iterations / 2],
        samples[iterations * 95 / 100]
    );
}

fn main() {
    let ctx = egui::Context::default();
    Theme { fonts: Fonts::BUNDLED, ..Theme::default() }.install(&ctx);
    spaceui::theme::set_reduced_motion(&ctx, true);
    let input = || egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000.0, 800.0))),
        ..Default::default()
    };
    measure("100 control rows / frame", 200, || {
        let mut on = false;
        ctx.run_ui(input(), |ui| {
            for i in 0..100 {
                ui.horizontal(|ui| {
                    black_box(widgets::primary(ui, "Connect"));
                    black_box(widgets::switch(ui, &mut on));
                    black_box(widgets::notches(ui, 160.0, 5, &mut (i % 5)));
                });
            }
        })
        .drop_without_applying_deltas();
    });
    let transcript =
        "## Reply\nA **formatted** paragraph with [a link](https://example.com).\n- A list item\n\n".repeat(100);
    measure("100-message Markdown / frame", 100, || {
        ctx.run_ui(input(), |ui| markdown::render(ui, black_box(&transcript))).drop_without_applying_deltas();
    });
    let malformed = "[".repeat(16_384);
    measure("16 KiB malformed Markdown", 100, || {
        ctx.run_ui(input(), |ui| markdown::render(ui, black_box(&malformed))).drop_without_applying_deltas();
    });
    // Compare the same repeated brackets as one plain label, then ordinary
    // CommonMark text of the same length. These are warm full-frame workloads.
    measure("16 KiB brackets / plain label", 100, || {
        ctx.run_ui(input(), |ui| {
            ui.label(egui::RichText::new(black_box(&malformed)).size(13.5));
        })
        .drop_without_applying_deltas();
    });
    let ordinary = "x".repeat(16_384);
    measure("16 KiB ordinary Markdown", 100, || {
        ctx.run_ui(input(), |ui| markdown::render(ui, black_box(&ordinary))).drop_without_applying_deltas();
    });
    for (name, text) in [("brackets", &malformed), ("ordinary", &ordinary)] {
        let output = ctx.run_ui(input(), |ui| markdown::render(ui, text));
        println!("16 KiB {name} CommonMark: {} paint shapes before tessellation", output.shapes.len());
        output.drop_without_applying_deltas();
    }
    measure("brackets / cold analysis", 100, || {
        markdown::clear_cache(&ctx);
        ctx.run_ui(input(), |ui| markdown::render(ui, black_box(&malformed))).drop_without_applying_deltas();
    });
    let mut streaming = malformed.clone();
    measure("brackets / streaming append", 100, || {
        streaming.push('[');
        ctx.run_ui(input(), |ui| markdown::render(ui, black_box(&streaming))).drop_without_applying_deltas();
    });
    let mut plain_stream = malformed.clone();
    measure("plain label / streaming append", 100, || {
        plain_stream.push('[');
        ctx.run_ui(input(), |ui| {
            ui.label(egui::RichText::new(black_box(&plain_stream)).size(13.5));
        })
        .drop_without_applying_deltas();
    });
    let mixed = format!("**bold** {malformed}");
    measure("fragmented rich / budget 1024", 100, || {
        ctx.run_ui(input(), |ui| {
            let shown = markdown::Markdown::new(black_box(&mixed)).max_events(1024).show(ui);
            assert!(shown.fallback);
        })
        .drop_without_applying_deltas();
    });
    measure("SVG rasterization 64 px", 100, || {
        black_box(Icon::Orbit.svg().rasterize(64));
    });
    measure("cached SVG / frame", 200, || {
        ctx.run_ui(input(), |ui| {
            Icon::Orbit.paint(
                ui.painter(),
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(64.0)),
                egui::Color32::WHITE,
            );
        })
        .drop_without_applying_deltas();
    });
    measure("gradient mesh 2x DPI", 1000, || {
        black_box(paint::gradient_mesh(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(300.0, 60.0)),
            egui::Color32::WHITE,
            egui::Color32::BLACK,
            16.0,
            2.0,
        ));
    });
}
