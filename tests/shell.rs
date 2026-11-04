//! The shell's layout and rail, driven headlessly.
#![cfg(feature = "shell")]
mod common;

use common::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use spaceui::egui::{Event, Key, Modifiers, Pos2, Role, Shape, Ui, Vec2, ViewportId, pos2, vec2};
use spaceui::shell::WindowLabels;
use spaceui::{Icon, Metrics, RailItem, Regions, Shell, Theme};

const RAIL: &[RailItem] = &[
    RailItem::page(Icon::Home, "Home"),
    RailItem::page(Icon::Star, "Targets"),
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

#[test]
fn constrained_windows_keep_regions_ordered_and_inside_the_viewport() {
    for width in [40.0, 160.0, 320.0, 800.0] {
        for height in [30.0, 100.0, 600.0] {
            let mut harness = Harness::new();
            harness.window = spaceui::egui::Rect::from_min_size(Pos2::ZERO, vec2(width, height));
            let (mut page, mut left, mut right) = (0, f32::NAN, 2000.0);
            let shown = regions(&mut harness, |ui| {
                Shell::new("Small")
                    .rail(RAIL)
                    .left_column(&mut left, 320.0..=560.0)
                    .right_column(&mut right, 320.0..=560.0)
                    .status_bar(22.0)
                    .show(ui, &mut page)
            });
            for part in [Some(shown.main), shown.left, shown.right, shown.status].into_iter().flatten() {
                assert!(part.is_finite() && part.width() >= 0.0 && part.height() >= 0.0, "{part:?}");
                assert!(harness.window.contains_rect(part), "{part:?} outside {:?}", harness.window);
            }
        }
    }
}

const LONG_RAIL: &[RailItem] = &[
    RailItem::page(Icon::Home, "First page"),
    RailItem::page(Icon::Home, "Second page"),
    RailItem::page(Icon::Home, "Third page"),
    RailItem::Divider,
    RailItem::page(Icon::Home, "Fourth page"),
    RailItem::page(Icon::Home, "Fifth page"),
    RailItem::page(Icon::Home, "Sixth page"),
    RailItem::page(Icon::Home, "Last page"),
];

fn accessible_shell<State: 'static>(
    size: Vec2,
    state: State,
    app: impl FnMut(&mut Ui, &mut State) + 'static,
) -> egui_kittest::Harness<'static, State> {
    egui_kittest::Harness::builder()
        .with_size(size)
        .with_context_setup(|ctx| {
            Theme { fonts: spaceui::Fonts::BUNDLED, ..Default::default() }.install(ctx);
        })
        .build_ui_state(app, state)
}

fn assert_visible_in_rail(rect: spaceui::egui::Rect, window_height: f32) {
    let metrics = Metrics::default();
    let bottom = window_height - metrics.gap * 1.5;
    assert!(rect.top() >= metrics.title_bar - 1.0, "tile is above the rail: {rect:?}");
    assert!(rect.bottom() <= bottom + 1.0, "tile is below the rail: {rect:?}");
}

#[test]
fn rail_keyboard_navigation_reveals_overflow_pages() {
    let mut harness = accessible_shell(vec2(420.0, 240.0), 0, |ui, page| {
        let _ = Shell::new("Small window").rail(LONG_RAIL).show(ui, page);
    });
    harness.get_by_label("First page").focus();
    harness.run();
    harness.key_press(Key::End);
    harness.run();
    assert_eq!(*harness.state(), 6);
    let last = harness.get_by_label("Last page");
    assert!(last.is_focused());
    assert_eq!(last.accesskit_node().role(), Role::Tab);
    assert_eq!(last.accesskit_node().is_selected(), Some(true));
    assert_visible_in_rail(last.rect(), 240.0);

    harness.key_press(Key::Home);
    harness.run();
    assert_eq!(*harness.state(), 0);
    let first = harness.get_by_label("First page");
    assert!(first.is_focused());
    assert_visible_in_rail(first.rect(), 240.0);
    harness.key_press(Key::ArrowDown);
    harness.run();
    assert_eq!(*harness.state(), 1);
    assert!(harness.get_by_label("Second page").is_focused());
}

#[test]
fn a_screen_reader_can_scroll_to_and_activate_an_overflow_page() {
    let mut harness = accessible_shell(vec2(420.0, 240.0), 0, |ui, page| {
        let _ = Shell::new("Small window").rail(LONG_RAIL).show(ui, page);
    });
    assert!(harness.get_by_label("Last page").rect().top() > 240.0);
    harness.get_by_label("Last page").scroll_to_me();
    harness.run();
    assert_visible_in_rail(harness.get_by_label("Last page").rect(), 240.0);
    harness.get_by_label("Last page").click();
    harness.run();
    assert_eq!(*harness.state(), 6);
}

#[test]
fn splitters_support_keyboard_and_accessibility_adjustments() {
    let mut harness = accessible_shell(vec2(1400.0, 900.0), (0, [300.0, 300.0]), |ui, (page, widths)| {
        let [left, right] = widths;
        let _ =
            Shell::new("Columns").left_column(left, 200.0..=400.0).right_column(right, 200.0..=400.0).show(ui, page);
    });
    for (index, name, after_right) in [(0, "Left column width", 310.0), (1, "Right column width", 290.0)] {
        let splitter = harness.get_by_label(name);
        assert_eq!(splitter.accesskit_node().role(), Role::Splitter);
        splitter.focus();
        harness.run();
        for (key, expected) in [(Key::ArrowRight, after_right), (Key::Home, 200.0), (Key::End, 400.0)] {
            harness.key_press(key);
            harness.run();
            assert!((harness.state().1[index] - expected).abs() < 0.01);
            assert!(harness.get_by_label(name).is_focused());
        }
        harness.get_by_label(name).set_value_accesskit(-100.0);
        harness.run();
        assert!((harness.state().1[index] - 200.0).abs() < 0.01);
        harness.get_by_label(name).increment_accesskit();
        harness.run();
        assert!((harness.state().1[index] - 210.0).abs() < 0.01);
        harness.get_by_label(name).decrement_accesskit();
        harness.run();
        assert!((harness.state().1[index] - 200.0).abs() < 0.01);
        harness.get_by_label(name).set_value_accesskit(1000.0);
        harness.run();
        assert!((harness.state().1[index] - 400.0).abs() < 0.01);
        assert_eq!(harness.get_by_label(name).accesskit_node().numeric_value(), Some(400.0));
    }
}

#[test]
fn splitter_changes_update_regions_in_the_input_frame() {
    let mut harness =
        accessible_shell(vec2(1400.0, 900.0), (0, [300.0, 300.0], None), |ui, (page, widths, regions)| {
            let [left, right] = widths;
            let shown = Shell::new("Columns")
                .left_column(left, 200.0..=400.0)
                .right_column(right, 200.0..=400.0)
                .show(ui, page);
            let left_rect = shown.left.expect("left column");
            let right_rect = shown.right.expect("right column");
            assert!((left_rect.width() - *left).abs() < 0.01, "left geometry must reflect this frame's input");
            assert!((right_rect.width() - *right).abs() < 0.01, "right geometry must reflect this frame's input");
            assert!(left_rect.right() < shown.main.left() && shown.main.right() < right_rect.left());
            *regions = Some(shown);
        });
    for (index, label) in [(0, "Left column width"), (1, "Right column width")] {
        harness.get_by_label(label).focus();
        harness.run();
        for (keyboard, expected) in [(true, 400.0), (false, 250.0)] {
            if keyboard {
                harness.event(Event::Key {
                    key: Key::End,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                });
            } else {
                harness.get_by_label(label).set_value_accesskit(expected);
            }
            // Process just the input event, without an idle frame masking stale geometry.
            harness.step();
            assert!((f64::from(harness.state().1[index]) - expected).abs() < 0.01);
            let shown = harness.state().2.expect("shell regions");
            let divider = harness.get_by_label(label).rect();
            let (divider_edge, column_edge) = if index == 0 {
                (divider.left(), shown.left.expect("left column").right())
            } else {
                (divider.right(), shown.right.expect("right column").left())
            };
            assert!((divider_edge - column_edge).abs() < 0.01, "accessible divider must follow the column");
            assert!(
                harness.output().shapes.iter().any(|clipped| {
                    matches!(&clipped.shape, Shape::LineSegment { points, .. }
                    if (points[0].x - divider.center().x).abs() < 0.01
                        && (points[1].x - divider.center().x).abs() < 0.01
                        && points[0].y < points[1].y)
                }),
                "focused divider must be painted at its updated position"
            );
        }
    }
}

#[test]
fn disabled_shell_blocks_rail_and_splitter_actions_but_keeps_window_controls() {
    let mut harness = accessible_shell(vec2(1000.0, 600.0), (0, 300.0), |ui, (page, width)| {
        let _ = Shell::new("Blocked").rail(RAIL).right_column(width, 200.0..=400.0).enabled(false).show(ui, page);
    });
    assert!(harness.get_by_label("Right column width").accesskit_node().is_disabled());
    assert!(harness.get_by_label("Targets").accesskit_node().is_disabled());
    assert!(!harness.get_by_label("Close window").accesskit_node().is_disabled());
    harness.get_by_label("Right column width").set_value_accesskit(400.0);
    harness.get_by_label("Right column width").increment_accesskit();
    harness.get_by_label("Targets").click_accesskit();
    harness.key_press(Key::End);
    harness.run();
    assert_eq!(harness.state().0, 0);
    assert!((harness.state().1 - 300.0).abs() < 0.01);
}

#[test]
fn window_labels_accept_runtime_translations() {
    let mut harness = accessible_shell(
        vec2(1000.0, 600.0),
        (0, 300.0, String::from("Fermer la fenêtre")),
        |ui, (page, width, close)| {
            let labels = WindowLabels {
                close: close.as_str(),
                minimize: "Réduire",
                maximize: "Agrandir",
                restore: "Restaurer",
                move_window: "Déplacer",
                left_column_width: "Largeur du panneau",
                resize: [
                    "Coin supérieur gauche",
                    "Coin supérieur droit",
                    "Coin inférieur gauche",
                    "Coin inférieur droit",
                    "Bord supérieur",
                    "Bord inférieur",
                    "Bord gauche",
                    "Bord droit",
                ],
                ..Default::default()
            };
            let _ = Shell::new("Traduit").labels(labels).left_column(width, 200.0..=400.0).show(ui, page);
        },
    );
    for name in ["Fermer la fenêtre", "Réduire", "Agrandir"] {
        assert_eq!(harness.get_by_label(name).accesskit_node().role(), Role::Button);
    }
    assert_eq!(harness.get_by_label("Largeur du panneau").accesskit_node().role(), Role::Splitter);
    assert!(harness.query_by_label("Close window").is_none());
    assert!(harness.query_by_label("Move window").is_none());
    assert!(harness.query_by_label("Resize top left").is_none());
    let title_bar = harness.get_by_label("Traduit");
    assert_eq!(title_bar.accesskit_node().role(), Role::TitleBar);
    assert_eq!(title_bar.accesskit_node().description().as_deref(), Some("Déplacer"));
    let _ = harness.get_by_label("Coin supérieur gauche");

    harness.input_mut().viewports.get_mut(&ViewportId::ROOT).expect("root viewport").maximized = Some(true);
    harness.run();
    assert_eq!(harness.get_by_label("Restaurer").accesskit_node().role(), Role::Button);
}

#[test]
fn title_buttons_remain_usable_above_a_modal() {
    let mut harness = accessible_shell(vec2(900.0, 600.0), 0, |ui, page| {
        let _ = Shell::new("Modal test").rail(RAIL).enabled(false).show(ui, page);
        spaceui::egui::Modal::new(spaceui::egui::Id::unique("blocking-modal")).show(ui.ctx(), |ui| {
            ui.label("The workspace is blocked");
        });
    });
    harness.get_by_label("Minimize window").click();
    // Inspect the release frame before subsequent idle frames clear commands.
    harness.step();
    let commands = &harness.output().viewport_output[&ViewportId::ROOT].commands;
    assert!(commands.iter().any(|command| matches!(command, spaceui::egui::ViewportCommand::Minimized(true))));
}

#[test]
fn the_rail_reports_one_selection_in_the_click_frame() {
    let mut harness = accessible_shell(vec2(900.0, 600.0), 0, |ui, page| {
        let _ = Shell::new("Selection").rail(RAIL).show(ui, page);
    });
    harness.get_by_label("Settings").click();
    harness.step();
    assert_eq!(*harness.state(), 2);
    for label in ["Home", "Targets", "Settings"] {
        assert_eq!(harness.get_by_label(label).accesskit_node().is_selected(), Some(label == "Settings"));
    }
}

#[test]
fn shell_normalizes_invalid_state_without_user_input() {
    let mut harness = Harness::new();
    let mut page = usize::MAX;
    let mut width = f32::NAN;
    let shown = regions(&mut harness, |ui| {
        Shell::new("Bounds")
            .rail(RAIL)
            .left_column(&mut width, 400.0..=200.0)
            .status_bar(f32::INFINITY)
            .show(ui, &mut page)
    });
    assert_eq!(page, 2);
    assert!((width - 400.0).abs() < f32::EPSILON);
    assert!(shown.status.is_none());
    let _ = regions(&mut harness, |ui| Shell::new("Dividers").rail(&[RailItem::Divider]).show(ui, &mut page));
    assert_eq!(page, 0);
    page = 7;
    let _ = regions(&mut harness, |ui| Shell::new("No rail").rail(&[]).show(ui, &mut page));
    assert_eq!(page, 7);
}
