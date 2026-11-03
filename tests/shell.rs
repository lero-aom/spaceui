//! The shell's layout and rail, driven headlessly.
mod common;

use common::Harness;
use spaceui::egui::pos2;
use spaceui::{Icon, Metrics, RailItem, Regions, Shell, Theme};

const RAIL: &[RailItem] = &[
    RailItem::page(Icon::Home, "Home"),
    RailItem::page(Icon::Chart, "Markets"),
    RailItem::Divider,
    RailItem::page(Icon::Settings, "Settings"),
];

#[test]
fn regions_fit_the_window_between_rail_and_side_column() {
    let mut harness = Harness::new();
    let window = harness.window;
    let (mut page, mut side) = (0, 400.0);
    let mut regions = None;
    harness.frame(Vec::new(), &mut |ui| {
        let shown =
            Shell::new("Test").rail(RAIL).side_column(&mut side, 320.0..=560.0).status_bar(22.0).show(ui, &mut page);
        regions = Some(shown);
    });
    let Regions { main, side: side_rect, status } = regions.expect("shown");
    let side_rect = side_rect.expect("a side column");
    let status = status.expect("a status line");
    let metrics = Theme::of(&harness.ctx).metrics;
    for part in [main, side_rect, status] {
        assert!(window.contains_rect(part), "{part:?} leaves the window");
    }
    assert!(main.left() > metrics.rail, "the main card starts after the rail");
    assert!(main.right() < side_rect.left(), "the side column sits right of the main card");
    assert!(main.top() > metrics.title_bar, "the title bar keeps its height");
    assert!((side_rect.width() - 400.0).abs() < 0.5);
    assert!(status.top() > main.bottom() && status.top() > side_rect.bottom() - 1.0);
}

#[test]
fn without_a_side_column_the_main_card_takes_the_width() {
    let mut harness = Harness::new();
    let mut page = 0;
    let mut regions = None;
    harness.frame(Vec::new(), &mut |ui| regions = Some(Shell::new("Test").show(ui, &mut page)));
    let regions = regions.expect("shown");
    assert!(regions.side.is_none() && regions.status.is_none());
    assert!(regions.main.width() > harness.window.width() - 160.0);
}

#[test]
fn the_side_column_is_kept_within_its_range_and_half_the_window() {
    let mut harness = Harness::new();
    let mut page = 0;
    for (asked, expected) in [(10.0, 320.0), (2000.0, 700.0)] {
        let mut side = asked;
        harness.frame(Vec::new(), &mut |ui| {
            let _ = Shell::new("Test").side_column(&mut side, 320.0..=900.0).show(ui, &mut page);
        });
        assert!((side - expected).abs() < 0.5, "asked {asked}, got {side}");
    }
}

#[test]
fn clicking_a_rail_tile_selects_its_page_and_dividers_do_not_count() {
    let metrics = Metrics::default();
    let x = f32::midpoint(metrics.gap, metrics.rail);
    let first = metrics.title_bar + 4.0 + metrics.tile / 2.0;
    let step = metrics.tile + 10.0;
    // Settings is the third page, below two tiles and a divider.
    let settings = pos2(x, first + 2.0 * step + 12.0);

    let mut harness = Harness::new();
    let mut page = 0;
    harness.click(settings, |ui| {
        let _ = Shell::new("Test").rail(RAIL).show(ui, &mut page);
    });
    assert_eq!(page, 2);
}

#[test]
fn a_disabled_shell_ignores_rail_clicks() {
    let metrics = Metrics::default();
    let markets = pos2(f32::midpoint(metrics.gap, metrics.rail), metrics.title_bar + 4.0 + metrics.tile * 1.5 + 10.0);
    let mut harness = Harness::new();
    let mut page = 0;
    harness.click(markets, |ui| {
        let _ = Shell::new("Test").rail(RAIL).enabled(false).show(ui, &mut page);
    });
    assert_eq!(page, 0);
}
