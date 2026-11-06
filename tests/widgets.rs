//! Widgets clicked headlessly.
mod common;

use common::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use spaceui::egui::{Event, Key, Pos2, Rect, Role, TextStyle, Ui};
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
    let mut changed = false;
    let harness = click_widget(|ui| {
        let response = widgets::notches(ui, 200.0, 5, &mut index);
        changed |= response.changed();
        // Aim at the last notch rather than the centre.
        Rect::from_center_size(
            Pos2::new(response.rect.right() - 7.0, response.rect.center().y),
            spaceui::egui::Vec2::ZERO,
        )
    });
    drop(harness);
    assert_eq!(index, 4);
    assert!(changed);
}

fn accessible_harness<State>(
    state: State,
    app: impl FnMut(&mut Ui, &mut State) + 'static,
) -> egui_kittest::Harness<'static, State>
where
    State: 'static,
{
    egui_kittest::Harness::builder()
        .with_context_setup(|ctx| {
            spaceui::Theme { fonts: spaceui::Fonts::BUNDLED, ..Default::default() }.install(ctx);
        })
        .build_ui_state(app, state)
}

#[test]
fn stacked_segmented_controls_keep_independent_state() {
    let mut harness = accessible_harness([0, 0], |ui, selected| {
        widgets::segmented(ui, &mut selected[0], &["One", "Two", "Three"]);
        widgets::segmented(ui, &mut selected[1], &["Red", "Green", "Blue"]);
    });
    harness.get_by_label("Two").click();
    harness.run();
    assert_eq!(*harness.state(), [1, 0]);
    harness.get_by_label("Blue").click();
    harness.run();
    assert_eq!(*harness.state(), [1, 2]);
}

#[test]
fn a_slider_supports_keyboard_and_screen_reader_changes() {
    let mut harness = accessible_harness((2, 0), |ui, (index, changes)| {
        let response = widgets::notches_labeled(ui, 200.0, 5, index, "Effort");
        *changes += usize::from(response.changed());
    });
    let slider = harness.get_by_label("Effort");
    assert_eq!(slider.accesskit_node().role(), Role::Slider);
    assert_eq!(slider.accesskit_node().numeric_value(), Some(2.0));
    slider.focus();
    harness.run();
    for (key, expected) in [(Key::ArrowRight, 3), (Key::End, 4), (Key::Home, 0), (Key::ArrowUp, 1)] {
        let before = harness.state().1;
        harness.key_press(key);
        harness.run();
        assert_eq!(harness.state().0, expected);
        assert_eq!(harness.state().1, before + 1);
        assert!(harness.get_by_label("Effort").is_focused());
    }
    harness.get_by_label("Effort").increment_accesskit();
    harness.run();
    assert_eq!(harness.state().0, 2);
    harness.get_by_label("Effort").set_value_accesskit(99.0);
    harness.run();
    assert_eq!(harness.state().0, 4);
    harness.get_by_label("Effort").decrement_accesskit();
    harness.run();
    assert_eq!(harness.state().0, 3);
    assert_eq!(harness.get_by_label("Effort").accesskit_node().numeric_value(), Some(3.0));
}

#[test]
fn a_switch_exposes_its_name_role_and_state() {
    use spaceui::egui::accesskit::Toggled;
    let mut harness = accessible_harness(false, |ui, on| {
        widgets::switch_labeled(ui, on, "Notifications");
    });
    let switch = harness.get_by_label("Notifications");
    assert_eq!(switch.accesskit_node().role(), Role::Switch);
    assert_eq!(switch.accesskit_node().toggled(), Some(Toggled::False));
    switch.click_accesskit();
    harness.run();
    assert!(*harness.state());
    assert_eq!(harness.get_by_label("Notifications").accesskit_node().toggled(), Some(Toggled::True));
}

#[test]
fn a_button_can_be_focused_and_activated_with_the_keyboard() {
    let mut harness = accessible_harness(0, |ui, clicks| {
        *clicks += usize::from(widgets::primary(ui, "Save").clicked());
    });
    assert_eq!(harness.get_by_label("Save").accesskit_node().role(), Role::Button);
    harness.key_press(Key::Tab);
    harness.run();
    assert!(harness.get_by_label("Save").is_focused());
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(*harness.state(), 1);
}

#[test]
fn choice_groups_support_arrow_home_and_end_keys() {
    let mut harness = accessible_harness([0, 0], |ui, selected| {
        widgets::tabs(ui, &mut selected[0], &["Overview", "Settings"]);
        widgets::segmented(ui, &mut selected[1], &["Day", "Week", "Month"]);
    });
    harness.get_by_label("Overview").focus();
    harness.run();
    harness.key_press(Key::ArrowRight);
    harness.run();
    assert_eq!(harness.state()[0], 1);
    assert!(harness.get_by_label("Settings").is_focused());
    harness.get_by_label("Day").focus();
    harness.run();
    harness.key_press(Key::End);
    harness.run();
    assert_eq!(harness.state()[1], 2);
    assert!(harness.get_by_label("Month").is_focused());
    harness.key_press(Key::Home);
    harness.run();
    assert_eq!(harness.state()[1], 0);
}

#[test]
fn text_field_keeps_focus_when_its_layout_moves() {
    let mut harness = accessible_harness((0.0, String::new()), |ui, (offset, text)| {
        ui.add_space(*offset);
        widgets::text_field(ui, text, "Type here").accessible_name("Draft");
    });
    harness.get_by_label("Draft").focus();
    harness.run();
    let id = harness.get_by_label("Draft").accesskit_node().id();
    harness.state_mut().0 = 80.0;
    harness.run();
    assert_eq!(harness.get_by_label("Draft").accesskit_node().id(), id);
    assert!(harness.get_by_label("Draft").is_focused());
    // Send text directly, without refocusing the field through the test API.
    harness.event(Event::Text("Still focused".into()));
    harness.run();
    assert_eq!(harness.state().1, "Still focused");
}

#[test]
fn explicit_text_field_identity_survives_inserted_widgets() {
    let mut harness = accessible_harness((false, String::new()), |ui, (extra, text)| {
        if *extra {
            widgets::secondary(ui, "New action");
        }
        widgets::text_field_id(ui, "draft", text, "Type here").accessible_name("Draft");
    });
    harness.get_by_label("Draft").focus();
    harness.run();
    harness.state_mut().0 = true;
    harness.run();
    assert!(harness.get_by_label("Draft").is_focused());
    harness.event(Event::Text("Preserved".into()));
    harness.run();
    assert_eq!(harness.state().1, "Preserved");
}

#[test]
fn empty_single_and_invalid_width_sliders_stay_finite() {
    for count in [0, 1, 5] {
        for width in [0.0, -20.0, 14.0, f32::NAN, f32::INFINITY] {
            let mut harness = Harness::new();
            let mut index = usize::MAX;
            harness.frame(Vec::new(), &mut |ui| {
                let response = widgets::notches(ui, width, count, &mut index);
                assert!(response.rect.is_finite());
                assert!(response.rect.is_positive());
                assert!(index < count.max(1));
            });
        }
    }
}

#[test]
fn large_slider_ranges_do_not_overflow_marker_positions() {
    use spaceui::egui::{Context, RawInput, Shape};
    let ctx = Context::default();
    spaceui::Theme { fonts: spaceui::Fonts::BUNDLED, ..Default::default() }.install(&ctx);
    let mut index = usize::MAX;
    let output = ctx.run_ui(RawInput::default(), |ui| {
        widgets::notches(ui, 1.0e25, usize::MAX, &mut index);
    });
    let mut markers = 0;
    for clipped in &output.shapes {
        if let Shape::Circle(circle) = &clipped.shape {
            assert!(circle.center.is_finite(), "slider markers must stay finite for finite dimensions");
            markers += 1;
        }
    }
    assert_eq!(markers, 512, "large ranges retain bounded paint work");
    output.drop_without_applying_deltas();
}

#[test]
fn disabled_sliders_ignore_keyboard_and_screen_reader_input() {
    let mut harness = accessible_harness(2, |ui, index| {
        ui.add_enabled_ui(false, |ui| widgets::notches_labeled(ui, 200.0, 5, index, "Effort"));
    });
    assert!(harness.get_by_label("Effort").accesskit_node().is_disabled());
    harness.get_by_label("Effort").increment_accesskit();
    harness.get_by_label("Effort").set_value_accesskit(4.0);
    harness.key_press(Key::End);
    harness.run();
    assert_eq!(*harness.state(), 2);
}

#[test]
fn custom_text_sizes_expand_controls() {
    let mut harness = Harness::new();
    let mut before = Rect::NOTHING;
    harness.frame(Vec::new(), &mut |ui| before = widgets::primary(ui, "Readable").rect);
    harness.ctx.global_style_mut(|style| {
        style.text_styles.get_mut(&TextStyle::Body).expect("body font").size = 28.0;
    });
    harness.frame(Vec::new(), &mut |ui| {
        let after = widgets::primary(ui, "Readable").rect;
        assert!(after.width() > before.width());
        assert!(after.height() > before.height());
    });
}

#[test]
fn choice_groups_report_one_selection_in_the_click_frame() {
    use spaceui::egui::accesskit::Toggled;
    for segmented in [false, true] {
        let mut harness = accessible_harness(0, move |ui, selected| {
            if segmented {
                widgets::segmented(ui, selected, &["First", "Second", "Third"]);
            } else {
                widgets::tabs(ui, selected, &["First", "Second", "Third"]);
            }
        });
        harness.get_by_label("Third").click();
        harness.step();
        assert_eq!(*harness.state(), 2);
        for label in ["First", "Second", "Third"] {
            let node = harness.get_by_label(label).accesskit_node();
            let selected =
                if segmented { node.toggled() == Some(Toggled::True) } else { node.is_selected() == Some(true) };
            assert_eq!(selected, label == "Third", "{label}, segmented={segmented}");
        }
    }
}

#[test]
fn primary_buttons_render_the_on_accent_foreground() {
    use spaceui::egui::{Color32, Context, epaint::Primitive};
    let ctx = Context::default();
    let ink = Color32::from_rgb(4, 241, 5);
    let theme = spaceui::Theme {
        palette: spaceui::Palette { on_accent: ink, ..Default::default() },
        fonts: spaceui::Fonts::BUNDLED,
        ..Default::default()
    };
    theme.install(&ctx);
    let output = ctx.run_ui(spaceui::egui::RawInput::default(), |ui| {
        widgets::primary(ui, "Connect");
    });
    let primitives = ctx.tessellate(output.shapes.clone(), output.pixels_per_point);
    assert!(
        primitives.iter().any(|clipped| match &clipped.primitive {
            Primitive::Mesh(mesh) => mesh.vertices.iter().any(|vertex| vertex.color == ink),
            Primitive::Callback(_) => false,
        }),
        "the requested foreground must reach the rendered text mesh"
    );
    output.drop_without_applying_deltas();
}

#[cfg(feature = "svg")]
#[test]
fn icon_buttons_expose_the_action_name() {
    let harness = accessible_harness((), |ui, ()| {
        widgets::icon_button_labeled(ui, spaceui::Icon::Trash, 32.0, "Delete draft");
    });
    assert_eq!(harness.get_by_label("Delete draft").accesskit_node().role(), Role::Button);
}

#[test]
fn configured_buttons_keep_keyboard_accessibility_and_disabled_behavior() {
    for primary in [false, true] {
        let mut harness = accessible_harness((true, 0), move |ui, (enabled, clicks)| {
            let button = if primary { widgets::Button::primary("Save") } else { widgets::Button::new("Save") };
            let response = ui.add_enabled(
                *enabled,
                button
                    .min_size(spaceui::egui::vec2(120.0, 44.0))
                    .padding(spaceui::egui::vec2(20.0, 10.0))
                    .corner_radius(10.0),
            );
            assert!(response.rect.width() >= 120.0 && response.rect.height() >= 44.0);
            *clicks += usize::from(response.clicked());
        });
        assert_eq!(harness.get_by_label("Save").accesskit_node().role(), Role::Button);
        harness.get_by_label("Save").click_accesskit();
        harness.run();
        assert_eq!(harness.state().1, 1);
        harness.get_by_label("Save").focus();
        harness.run();
        harness.key_press(Key::Enter);
        harness.run();
        assert_eq!(harness.state().1, 2);
        harness.state_mut().0 = false;
        harness.run();
        harness.key_press(Key::Enter);
        harness.run();
        assert_eq!(harness.state().1, 2);
    }
}

#[test]
fn control_metrics_survive_palette_changes_and_button_overrides_are_local() {
    use spaceui::{
        Theme,
        egui::vec2,
        theme::{ControlMetrics, ControlSize},
    };
    let mut harness = Harness::new();
    let large = ControlSize { min_size: vec2(180.0, 60.0), padding: vec2(20.0, 12.0) };
    let metrics = ControlMetrics { button: large, tab: large, segmented: large, field: large };
    metrics.install(&harness.ctx);
    Theme::deep_space().install(&harness.ctx);
    assert_eq!(ControlMetrics::of(&harness.ctx), metrics);
    assert_eq!(ControlMetrics::of(&spaceui::egui::Context::default()), ControlMetrics::default());
    harness.frame(Vec::new(), &mut |ui| {
        let custom = ui.add(widgets::Button::secondary("Go").min_size(vec2(50.0, 25.0)).padding(vec2(1.0, 1.0)));
        assert!(custom.rect.width() < 180.0 && custom.rect.height() < 60.0);
        let standard = widgets::primary(ui, "Go");
        assert_eq!(standard.rect.size(), large.min_size);
        for response in
            [widgets::tabs_response(ui, &mut 0, &["A", "B"]), widgets::segmented_response(ui, &mut 0, &["C", "D"])]
        {
            assert!(response.response.rect.height() >= 60.0);
            assert!(response.choices.iter().all(|choice| choice.rect.width() >= 180.0));
        }
        let field = ui.scope(|ui| widgets::text_field(ui, &mut String::new(), "Name"));
        assert!(field.response.rect.height() >= 60.0);
    });
}

#[test]
fn invalid_control_sizing_stays_finite() {
    use spaceui::{
        egui::Vec2,
        theme::{ControlMetrics, ControlSize},
    };
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, 0.0, f32::MAX] {
        let mut harness = Harness::new();
        let size = ControlSize { min_size: Vec2::splat(value), padding: Vec2::splat(value) };
        ControlMetrics { button: size, tab: size, segmented: size, field: size }.install(&harness.ctx);
        harness.frame(Vec::new(), &mut |ui| {
            let response = ui.add(
                widgets::Button::primary("Finite")
                    .min_size(Vec2::splat(value))
                    .padding(Vec2::splat(value))
                    .corner_radius(value),
            );
            assert!(response.rect.is_finite() && response.rect.is_positive());
            for choice in [widgets::tabs_response(ui, &mut 0, &["A"]), widgets::segmented_response(ui, &mut 0, &["B"])]
            {
                assert!(choice.response.rect.is_finite());
            }
            assert!(widgets::text_field(ui, &mut String::new(), "Name").rect.is_finite());
        });
    }
}

#[test]
fn choice_responses_report_clicks_changes_and_individual_focus() {
    for segmented in [false, true] {
        let mut harness =
            accessible_harness((0, 0, 0, Vec::new()), move |ui, (selected, changes, clicks, responses)| {
                let shown = if segmented {
                    widgets::segmented_response(ui, selected, &["First", "Second", "Third"])
                } else {
                    widgets::tabs_response(ui, selected, &["First", "Second", "Third"])
                };
                *changes += usize::from(shown.response.changed());
                *clicks += usize::from(shown.response.clicked());
                for child in &shown.choices {
                    assert!(shown.response.rect.contains_rect(child.rect));
                }
                *responses = shown.choices;
            });
        harness.get_by_label("Third").click();
        harness.step();
        assert_eq!((harness.state().0, harness.state().1, harness.state().2), (2, 1, 1));
        let responses = &harness.state().3;
        assert!(responses[0].changed() && !responses[1].changed() && responses[2].changed());
        harness.run();
        harness.get_by_label("Third").click();
        harness.run();
        assert_eq!((harness.state().1, harness.state().2), (1, 2), "reselecting is a click but not a change");
        harness.get_by_label("Third").focus();
        harness.run();
        harness.key_press(Key::Home);
        harness.run();
        assert_eq!(harness.state().0, 0);
        assert_eq!(harness.state().1, 2);
        assert!(harness.state().3[0].has_focus());
    }
}

#[test]
fn choice_responses_clamp_empty_indices_and_ignore_disabled_input() {
    for segmented in [false, true] {
        let mut harness = Harness::new();
        let mut index = usize::MAX;
        let mut first = true;
        harness.frame(Vec::new(), &mut |ui| {
            let response = if segmented {
                widgets::segmented_response(ui, &mut index, &[])
            } else {
                widgets::tabs_response(ui, &mut index, &[])
            };
            assert_eq!(response.response.changed(), first);
            first = false;
            assert!(response.choices.is_empty());
        });
        assert_eq!(index, 0);
        let mut changes = 0;
        let mut at = Pos2::ZERO;
        let mut show = |ui: &mut Ui| {
            ui.add_enabled_ui(false, |ui| {
                let response = if segmented {
                    widgets::segmented_response(ui, &mut index, &["A", "B"])
                } else {
                    widgets::tabs_response(ui, &mut index, &["A", "B"])
                };
                at = response.choices[1].rect.center();
                changes += usize::from(response.response.changed());
                assert!(!response.response.clicked());
            });
        };
        harness.frame(Vec::new(), &mut show);
        let target = at;
        harness.click(target, |ui| {
            ui.add_enabled_ui(false, |ui| {
                let response = if segmented {
                    widgets::segmented_response(ui, &mut index, &["A", "B"])
                } else {
                    widgets::tabs_response(ui, &mut index, &["A", "B"])
                };
                assert!(!response.response.clicked() && !response.response.changed());
            });
        });
        assert_eq!((index, changes), (0, 0));
    }
}
