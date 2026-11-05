//! Rendered regressions with bundled fonts, fixed time, and reduced motion.
#![cfg(all(feature = "svg", feature = "chat", feature = "shell"))]

use egui_kittest::{Harness, SnapshotOptions, SnapshotResults};
use spaceui::{Fonts, Icon, Theme, chat, egui, theme, widgets};

fn shell_snapshot_options() -> SnapshotOptions {
    match std::env::var("SPACEUI_SNAPSHOT_PROFILE") {
        Err(std::env::VarError::NotPresent) => SnapshotOptions::new(),
        Ok(profile) if profile == "linux-vulkan" && cfg!(target_os = "linux") => {
            assert_eq!(
                std::env::var("WGPU_BACKEND").as_deref(),
                Ok("vulkan"),
                "linux-vulkan requires WGPU_BACKEND=vulkan"
            );
            // Layered translucent backdrops differ slightly between graphics drivers.
            // CI selects Mesa lavapipe explicitly and uses its reviewed shell baselines.
            // Both profiles retain kittest's default threshold and failed-pixel limit.
            SnapshotOptions::new().output_path("tests/snapshots/linux-vulkan")
        }
        profile => panic!("unsupported SPACEUI_SNAPSHOT_PROFILE on {}: {profile:?}", std::env::consts::OS),
    }
}

fn setup(deep: bool) -> Theme {
    Theme { fonts: Fonts::BUNDLED, ..if deep { Theme::deep_space() } else { Theme::carbon_copper() } }
}

#[test]
fn controls_across_themes_sizes_and_pixel_densities() {
    let mut results = SnapshotResults::new();
    for deep in [false, true] {
        for width in [320, 640] {
            for density in [1, 2] {
                let mut value = 2;
                let mut enabled = true;
                let mut selected = 0;
                let mut text = "Editable text".to_owned();
                let mut harness = Harness::builder()
                    .with_size(egui::vec2(width as f32, 600.0))
                    .with_pixels_per_point(density as f32)
                    .with_context_setup(move |ctx| {
                        setup(deep).install(ctx);
                        theme::set_reduced_motion(ctx, true);
                    })
                    .build_ui(|ui| {
                        egui::Frame::new().fill(Theme::of(ui.ctx()).palette.panel).inner_margin(12.0).show(ui, |ui| {
                            ui.heading("SpaceUI controls");
                            ui.horizontal(|ui| {
                                let _ = widgets::primary(ui, "Connect");
                                let _ = widgets::secondary(ui, "Cancel");
                                let _ = widgets::icon_button_labeled(ui, Icon::Settings, 30.0, "Settings");
                            });
                            ui.add_enabled_ui(false, |ui| { let _ = widgets::primary(ui, "Unavailable"); });
                            let _ = widgets::text_field(ui, &mut text, "Name");
                            let _ = widgets::switch_labeled(ui, &mut enabled, "Notifications");
                            let _ = widgets::notches_labeled(ui, 230.0, 5, &mut value, "Effort");
                            widgets::tabs(ui, &mut selected, &["Overview", "History"]);
                            widgets::segmented(ui, &mut selected, &["Day", "Week", "Month"]);
                            widgets::chip(ui, "Connected", widgets::Tone::Positive);
                            widgets::badge(ui, Icon::Shield, "Read only", widgets::Tone::Info);
                            chat::markdown::render(ui, "### Formatted text\nA **bold** word, *emphasis*, and `code`.\n- A list item\n> A quotation");
                        });
                    });
                harness.run();
                harness.snapshot(format!("controls_{}_w{width}_dpi{density}", if deep { "deep" } else { "carbon" }));
                results.extend_harness(&mut harness);
            }
        }
    }
    results.unwrap();
}

#[test]
fn chat_with_overflow_and_a_constrained_composer() {
    let mut results = SnapshotResults::new();
    for width in [320, 640] {
        let mut draft = "A draft that wraps naturally when the column is narrow.".to_owned();
        let tabs: Vec<_> =
            (0..12).map(|id| chat::Tab { id, title: "Conversation", busy: false, closable: true }).collect();
        let mut harness = Harness::builder()
            .with_size(egui::vec2(width as f32, 560.0))
            .with_context_setup(|ctx| {
                setup(true).install(ctx);
                theme::set_reduced_motion(ctx, true);
            })
            .build_ui(|ui| {
                ui.painter().rect_filled(ui.max_rect(), 0.0, Theme::of(ui.ctx()).palette.panel);
                let layout = chat::ChatLayout::new(
                    ui.max_rect(),
                    chat::Composer::measure(ui, ui.available_width(), &draft, "Message"),
                );
                let _ = chat::tab_strip(ui, layout.strip, &tabs, 10);
                ui.scope_builder(egui::UiBuilder::new().max_rect(layout.body), |ui| {
                    chat::transcript::user_bubble(ui, "How does this work?", "User message");
                    chat::transcript::reply(
                        ui,
                        "The active tab stays visible.\n\n**Other tabs** are available in the menu.",
                        false,
                    );
                });
                let _ = chat::Composer::new("visual", &mut draft).hint("Message").show(ui, layout.composer, |_| {});
            });
        harness.run();
        harness.snapshot(format!("chat_overflow_w{width}"));
        results.extend_harness(&mut harness);
    }
    results.unwrap();
}

#[test]
fn shell_at_normal_and_constrained_sizes() {
    let mut results = SnapshotResults::new();
    for deep in [false, true] {
        for width in [360, 1024] {
            let rail = [
                spaceui::RailItem::page(Icon::Home, "Home"),
                spaceui::RailItem::page(Icon::Chat, "Conversations"),
                spaceui::RailItem::Divider,
                spaceui::RailItem::page(Icon::Settings, "Settings"),
            ];
            let mut page = 0;
            let mut side = 320.0;
            let mut harness = Harness::builder()
                .with_options(shell_snapshot_options())
                .with_size(egui::vec2(width as f32, 500.0))
                .with_context_setup(move |ctx| {
                    setup(deep).install(ctx);
                    theme::set_reduced_motion(ctx, true);
                })
                .build_ui(|ui| {
                    let mut shell = spaceui::Shell::new("SpaceUI").logo(Icon::Orbit.svg()).rail(&rail);
                    if width > 600 {
                        shell = shell.right_column(&mut side, 240.0..=360.0);
                    }
                    let regions = shell.show(ui, &mut page);
                    ui.scope_builder(egui::UiBuilder::new().max_rect(regions.main), |ui| {
                        ui.heading("Workspace");
                        ui.label("Content adapts to the available space.");
                        let _ = widgets::primary(ui, "Continue");
                    });
                    if let Some(right) = regions.right {
                        ui.scope_builder(egui::UiBuilder::new().max_rect(right.shrink(12.0)), |ui| {
                            ui.heading("Side panel");
                            chat::transcript::reply(ui, "A **separate** region owned by your application.", false);
                        });
                    }
                });
            harness.run();
            harness.snapshot(format!("shell_{}_w{width}", if deep { "deep" } else { "carbon" }));
            results.extend_harness(&mut harness);
        }
    }
    results.unwrap();
}

#[test]
fn keyboard_focus_is_visible() {
    let mut value = 2;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(320.0, 150.0))
        .with_context_setup(|ctx| {
            setup(false).install(ctx);
            theme::set_reduced_motion(ctx, true);
        })
        .build_ui(|ui| {
            ui.label("Keyboard focus: effort");
            let response = widgets::notches_labeled(ui, 260.0, 5, &mut value, "Effort");
            response.request_focus();
        });
    harness.run();
    harness.snapshot("keyboard_slider_focus");
}
