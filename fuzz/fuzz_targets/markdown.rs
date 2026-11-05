#![no_main]

use libfuzzer_sys::fuzz_target;
use spaceui::{chat::markdown, egui};

thread_local! {
    static CONTEXT: egui::Context = egui::Context::default();
}

fuzz_target!(|data: &[u8]| {
    // Bound a single input's allocation while still exercising long malformed
    // runs, Unicode, links and mixed emphasis. Lossy decoding exercises every
    // input rather than rejecting most mutated UTF-8 byte strings.
    if data.len() > 65_536 {
        return;
    }
    let text = String::from_utf8_lossy(data);
    CONTEXT.with(|ctx| {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(640.0, 480.0))),
            ..Default::default()
        };
        ctx.run_ui(input, |ui| {
            markdown::render(ui, &text);
            assert!(ui.min_rect().is_finite(), "CommonMark layout must remain finite");
        }).drop_without_applying_deltas();
        markdown::clear_cache(ctx);
    });
});
