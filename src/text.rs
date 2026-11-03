//! Text layout helpers shared by widgets and the chat kit.
use egui::{
    Color32, FontId, Galley, Ui,
    text::{LayoutJob, TextWrapping},
};
use std::sync::Arc;

/// Single-line text cut with an ellipsis at `max_width`.
pub(crate) fn truncated(ui: &Ui, text: &str, font: FontId, color: Color32, max_width: f32) -> Arc<Galley> {
    let mut job = LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap = TextWrapping::truncate_at_width(max_width.max(10.0));
    ui.fonts_mut(|f| f.layout_job(job))
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
}
