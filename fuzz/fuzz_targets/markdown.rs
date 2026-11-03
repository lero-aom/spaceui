#![no_main]

use libfuzzer_sys::fuzz_target;
use spaceui::{Palette, chat::markdown, egui::FontId};

fuzz_target!(|data: &[u8]| {
    // Bound a single input's allocation while still exercising long malformed
    // runs, Unicode, links and mixed emphasis. Lossy decoding exercises every
    // input rather than rejecting most mutated UTF-8 byte strings.
    if data.len() > 65_536 {
        return;
    }
    let text = String::from_utf8_lossy(data);
    let palette = Palette::CARBON_COPPER;
    let job = markdown::inline_job(&text, &FontId::proportional(13.0), palette.text, &palette);
    for section in &job.sections {
        let range = section.byte_range.start.0..section.byte_range.end.0;
        assert!(job.text.get(range).is_some(), "format range is not valid UTF-8");
        assert!(section.format.font_id.size.is_finite());
    }
    assert!(job.text.len() <= text.len(), "inline formatting unexpectedly expands input");
});
