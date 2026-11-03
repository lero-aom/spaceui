//! The shell's layout and rail, driven headlessly.
mod common;

use common::Harness;
use spaceui::egui::{Pos2, Ui, pos2, vec2};
use spaceui::{Icon, Metrics, RailItem, Regions, Shell, Theme};

const RAIL: &[RailItem] = &[
    RailItem::page(Icon::Home, "Home"),
    RailItem::page(Icon::Chart, "Markets"),
    RailItem::Divider,
    RailItem::page(Icon::Settings, "Settings"),
];

/// One frame of `show`, returning the regions it produced.
fn regions(harness: &mut Harness, mut show: impl FnMut(&mut Ui) -> Regions) -> Regions {
    let mut shown = None;
    harness.frame(Vec::new(), &mut |ui| shown = Some(show(ui)));
    shown.expect("shown")
}

#[test]
fn a_right_column_sits_between_the_main_card_and_the_window_edge() {
    let mut harness = Harness::new();
    let window = harness.window;
    let (mut page, mut width) = (0, 400.0);
    let shown = regions(&mut harness, |ui| {
        Shell::new("Test").rail(RAIL).right_column(&mut width, 320.0..=560.0).status_bar(22.0).show(ui, &mut page)
    });
    let (main, right, status) =
        (shown.main, shown.right.expect("a right column"), shown.status.expect("a status line"));
    let metrics = Theme::of(&harness.ctx).metrics;
    for part in [main, right, status] {
        assert!(window.contains_rect(part), "{part:?} leaves the window");
    }
    assert!(shown.left.is_none());
    assert!(main.left() > metrics.rail, "the main card starts after the rail");
    assert!(main.right() < right.left(), "the column sits right of the main card");
    assert!(main.top() > metrics.title_bar, "the title bar keeps its height");
    assert!((right.width() - 400.0).abs() < 0.5);
    assert!(status.top() > main.bottom() && status.top() > right.bottom() - 1.0);
}

#[test]
fn a_left_column_sits_between_the_rail_and_the_main_card() {
    let mut harness = Harness::new();
    let (mut page, mut width) = (0, 360.0);
    let shown = regions(&mut harness, |ui| {
        Shell::new("Test").rail(RAIL).left_column(&mut width, 320.0..=560.0).show(ui, &mut page)
    });
    let left = shown.left.expect("a left column");
    let metrics = Theme::of(&harness.ctx).metrics;
    assert!(shown.right.is_none());
    assert!(left.left() > metrics.rail - 1.0, "the column starts after the rail");
    assert!(left.right() < shown.main.left(), "the main card follows the column");
    assert!((left.width() - 360.0).abs() < 0.5);
}

#[test]
fn columns_on_both_sides_share_the_window() {
    let mut harness = Harness::new();
    let (mut page, mut left_width, mut right_width) = (0, 2000.0, 2000.0);
    let shown = regions(&mut harness, |ui| {
        Shell::new("Test")
            .left_column(&mut left_width, 200.0..=900.0)
            .right_column(&mut right_width, 200.0..=900.0)
            .show(ui, &mut page)
    });
    let (left, right) = (shown.left.expect("left"), shown.right.expect("right"));
    // With two columns each is capped at 35% of the 1400-point window.
    assert!((left_width - 490.0).abs() < 0.5 && (right_width - 490.0).abs() < 0.5);
    assert!(left.right() < shown.main.left() && shown.main.right() < right.left());
    assert!(shown.main.width() > 200.0);
}

#[test]
fn without_a_rail_or_columns_the_main_card_takes_the_window() {
    let mut harness = Harness::new();
    let mut page = 0;
    let shown = regions(&mut harness, |ui| Shell::new("Test").show(ui, &mut page));
    assert!(shown.left.is_none() && shown.right.is_none() && shown.status.is_none());
    assert!(shown.main.left() < 40.0, "no space is kept for a rail: {:?}", shown.main);
    assert!(shown.main.width() > harness.window.width() - 80.0);
}

#[test]
fn a_column_is_kept_within_its_range_and_half_the_window() {
    let mut harness = Harness::new();
    let mut page = 0;
    for (asked, expected) in [(10.0, 320.0), (2000.0, 700.0)] {
        let mut width = asked;
        let _ =
            regions(&mut harness, |ui| Shell::new("Test").right_column(&mut width, 320.0..=900.0).show(ui, &mut page));
        assert!((width - expected).abs() < 0.5, "asked {asked}, got {width}");
    }
}

#[test]
fn dragging_a_splitter_resizes_its_column() {
    let mut harness = Harness::new();
    let (mut page, mut width) = (0, 400.0);
    let mut show = |ui: &mut Ui| Shell::new("Test").right_column(&mut width, 320.0..=600.0).show(ui, &mut page);
    let main = regions(&mut harness, &mut show).main;
    // The handle is the gutter right of the main card's plate (18 points of padding).
    let handle = pos2(main.right() + 18.0 + 7.0, 450.0);
    harness.drag(handle, handle - vec2(60.0, 0.0), &mut |ui| {
        let _ = show(ui);
    });
    assert!((width - 460.0).abs() < 1.0, "dragging left by 60 widens a right column: {width}");
}

fn rail_tile(index: usize, dividers_above: usize) -> Pos2 {
    let metrics = Metrics::default();
    let first = metrics.title_bar + 4.0 + metrics.tile / 2.0;
    let step = metrics.tile + 10.0;
    pos2(f32::midpoint(metrics.gap, metrics.rail), first + index as f32 * step + dividers_above as f32 * 12.0)
}

#[test]
fn clicking_a_rail_tile_selects_its_page_and_dividers_do_not_count() {
    let mut harness = Harness::new();
    let mut page = 0;
    // Settings is the third page, below two tiles and a divider.
    harness.click(rail_tile(2, 1), |ui| {
        let _ = Shell::new("Test").rail(RAIL).show(ui, &mut page);
    });
    assert_eq!(page, 2);
}

#[test]
fn a_disabled_shell_ignores_rail_clicks() {
    let mut harness = Harness::new();
    let mut page = 0;
    harness.click(rail_tile(1, 0), |ui| {
        let _ = Shell::new("Test").rail(RAIL).enabled(false).show(ui, &mut page);
    });
    assert_eq!(page, 0);
}
