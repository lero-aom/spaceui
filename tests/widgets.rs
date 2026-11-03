//! Widgets clicked headlessly.
mod common;

use common::Harness;
use spaceui::egui::{Pos2, Rect, Ui};
use spaceui::widgets;

/// Runs `add` once to find where its widget lands, then clicks there.
fn click_widget(mut add: impl FnMut(&mut Ui) -> Rect) -> Harness {
    let mut harness = Harness::new();
    let mut at = Pos2::ZERO;
    harness.frame(Vec::new(), &mut |ui| at = add(ui).center());
    harness.click(at, |ui| {
        add(ui);
    });
    harness
}

#[test]
fn a_primary_button_reports_its_click() {
    let mut clicks = 0;
    click_widget(|ui| {
        let response = widgets::primary(ui, "Connect");
        clicks += usize::from(response.clicked());
        response.rect
    });
    assert_eq!(clicks, 1);
}

#[test]
fn a_disabled_button_does_not_click() {
    let mut clicked = false;
    click_widget(|ui| {
        ui.add_enabled_ui(false, |ui| {
            let response = widgets::primary(ui, "Connect");
            clicked |= response.clicked();
            response.rect
        })
        .inner
    });
    assert!(!clicked);
}

#[test]
fn the_switch_flips_and_reports_a_change() {
    let mut on = false;
    let mut changed = false;
    click_widget(|ui| {
        let response = widgets::switch(ui, &mut on);
        changed |= response.changed();
        response.rect
    });
    assert!(on && changed);
}

#[test]
fn segmented_controls_select_the_clicked_segment() {
    let mut harness = Harness::new();
    let mut selected = 0;
    let mut outer = Rect::NOTHING;
    harness.frame(Vec::new(), &mut |ui| {
        let before = ui.cursor().min;
        widgets::segmented(ui, &mut selected, &["M5", "H1", "D1"]);
        outer = Rect::from_min_max(before, ui.min_rect().max);
    });
    let mut changed = false;
    // The labels are equally short, so the segments are about equally wide.
    harness.click(common::slot_center(outer.shrink2(spaceui::egui::vec2(3.0, 0.0)), 2, 3), |ui| {
        changed |= widgets::segmented(ui, &mut selected, &["M5", "H1", "D1"]);
    });
    assert_eq!(selected, 2);
    assert!(changed);
}

#[test]
fn the_slider_jumps_to_the_clicked_notch() {
    let mut index = 0;
    let harness = click_widget(|ui| {
        let response = widgets::notches(ui, 200.0, 5, &mut index);
        // Aim at the last notch rather than the centre.
        Rect::from_center_size(
            Pos2::new(response.rect.right() - 7.0, response.rect.center().y),
            spaceui::egui::Vec2::ZERO,
        )
    });
    drop(harness);
    assert_eq!(index, 4);
}
