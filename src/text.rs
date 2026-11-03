//! Text layout helpers shared by widgets and the chat kit.
use egui::{
    Color32, FontId, Galley, Ui,
    text::{LayoutJob, TextWrapping},
};
use std::sync::Arc;

/// Single-line text cut with an ellipsis at `max_width`.
/// Returns an empty galley when no text fits in the available width.
pub(crate) fn truncated(ui: &Ui, text: &str, font: FontId, color: Color32, max_width: f32) -> Arc<Galley> {
    let max_width = max_width.max(0.0);
    ui.fonts_mut(|fonts| {
        if max_width > 0.0 {
            let mut job = LayoutJob::simple_singleline(text.to_owned(), font.clone(), color);
            job.wrap = TextWrapping::truncate_at_width(max_width);
            let galley = fonts.layout_job(job);
            // egui may exceed the wrap width when a single glyph (including
            // the ellipsis) is wider than the region. Omit that glyph as well.
            if galley.size().x <= max_width {
                return galley;
            }
        }
        fonts.layout_no_wrap(String::new(), font, color)
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn long_text_is_cut_to_the_width() {
        let ctx = crate::test_util::run_ui(|ui| {
            let galley =
                super::truncated(ui, &"wide ".repeat(40), egui::FontId::proportional(13.0), egui::Color32::WHITE, 80.0);
            assert!(galley.size().x <= 81.0, "{}", galley.size().x);
            assert!(galley.elided, "the text is cut short");
        });
        drop(ctx);
    }

    #[test]
    fn tiny_text_regions_never_draw_past_their_width() {
        let ctx = crate::test_util::run_ui(|ui| {
            for width in [-4.0_f32, 0.0, 1.0, 5.0, 9.0, 13.0] {
                let galley =
                    super::truncated(ui, "wide text", egui::FontId::proportional(13.0), egui::Color32::WHITE, width);
                assert!(galley.size().x <= width.max(0.0), "width {width}: {}", galley.size().x);
            }
        });
        drop(ctx);
    }
}
