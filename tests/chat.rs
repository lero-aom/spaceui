//! The chat kit's tab strip and composer, driven headlessly.
#![cfg(feature = "chat")]
mod common;
#[path = "chat/composer_sequences.rs"]
mod composer_sequences;

use common::Harness;
use spaceui::chat::{self, ChatLayout, Composer, ComposerResponse, Tab, TabAction};
use spaceui::egui::{Event, ImeEvent, Key, Modifiers, Rect, pos2, vec2};

fn column() -> Rect {
    Rect::from_min_size(pos2(1000.0, 60.0), vec2(380.0, 800.0))
}

const TABS: [Tab<'static>; 2] = [
    Tab { id: 1, title: "First", busy: false, closable: true },
    Tab { id: 2, title: "Second", busy: true, closable: true },
];

/// Tabs share the strip, less 40 points for the "+", within 72 to 190 points each.
fn tab_width(strip: Rect) -> f32 {
    ((strip.width() - 40.0) / TABS.len() as f32).clamp(72.0, 190.0)
}

#[test]
fn the_tab_strip_selects_tabs_and_starts_new_ones() {
    let strip = ChatLayout::new(column(), 120.0).strip;
    let width = tab_width(strip);
    let second = pos2(strip.left() + width + 40.0, strip.center().y);
    let plus = pos2(strip.left() + 2.0 * width + 15.0, strip.center().y);
    for (at, expected) in [(second, TabAction::Select(1)), (plus, TabAction::New)] {
        let mut harness = Harness::new();
        let mut action = None;
        harness.click(at, |ui| action = action.or(chat::tab_strip(ui, strip, &TABS, 0)));
        assert_eq!(action, Some(expected), "clicking {at:?}");
    }
}

#[test]
fn the_active_tabs_close_button_closes_it() {
    let strip = ChatLayout::new(column(), 120.0).strip;
    // The close button sits 15 points inside the tab's right edge (tabs keep a 4-point gap).
    let close = pos2(strip.left() + tab_width(strip) - 4.0 - 15.0, strip.center().y);
    let mut harness = Harness::new();
    let mut action = None;
    harness.click(close, |ui| action = action.or(chat::tab_strip(ui, strip, &TABS, 0)));
    assert_eq!(action, Some(TabAction::Close(0)));
}

#[test]
fn enter_sends_the_draft_and_shift_enter_does_not() {
    for (modifiers, sends) in [(Modifiers::NONE, true), (Modifiers::SHIFT, false)] {
        let mut harness = Harness::new();
        let rect = ChatLayout::new(column(), 120.0).composer;
        let mut draft = String::new();
        let mut response = ComposerResponse::default();
        let mut first = true;
        let mut show = |ui: &mut spaceui::egui::Ui| {
            let focus = std::mem::take(&mut first);
            let shown = Composer::new(1, &mut draft).focus(focus).show(ui, rect, |_| {});
            response.send |= shown.send;
        };
        harness.frame(Vec::new(), &mut show);
        harness.type_then("Where is M31?", Key::Enter, modifiers, show);
        assert_eq!(response.send, sends, "with {modifiers:?}");
        assert!(draft.starts_with("Where is M31?"), "the app takes the draft, not the composer: {draft:?}");
    }
}

#[test]
fn composer_submission_uses_the_enter_events_modifiers() {
    for (pressed, after, sends) in
        [(Modifiers::SHIFT, Modifiers::NONE, false), (Modifiers::NONE, Modifiers::SHIFT, true)]
    {
        let mut harness = Harness::new();
        let rect = ChatLayout::new(column(), 120.0).composer;
        let mut draft = "Hello".to_owned();
        let mut response = ComposerResponse::default();
        let mut first = true;
        let mut show = |ui: &mut spaceui::egui::Ui| {
            response = Composer::new(1, &mut draft).focus(std::mem::take(&mut first)).show(ui, rect, |_| {});
        };
        harness.frame(Vec::new(), &mut show);
        harness.frame_with(
            vec![
                Event::Key { key: Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: pressed },
                Event::ModifiersChanged(after),
            ],
            pressed,
            &mut show,
        );
        assert_eq!(response.send, sends, "pressed with {pressed:?}, released with {after:?}");
        assert_eq!(draft.contains('\n'), pressed.shift);
    }
}

#[test]
fn ime_composition_commits_text_without_sending_the_draft() {
    let mut harness = Harness::new();
    let rect = ChatLayout::new(column(), 120.0).composer;
    let mut draft = String::new();
    let mut response = ComposerResponse::default();
    let mut first = true;
    let mut show = |ui: &mut spaceui::egui::Ui| {
        let shown = Composer::new(1, &mut draft).focus(std::mem::take(&mut first)).show(ui, rect, |_| {});
        response.send |= shown.send;
    };
    harness.frame(Vec::new(), &mut show);
    harness.frame(
        vec![Event::Ime(ImeEvent::Preedit { text: "にほん".to_owned(), active_range_chars: Some(0..3) })],
        &mut show,
    );
    // Integrations withhold key events consumed by IME, including the Enter
    // that accepts a candidate; egui receives only the resulting commit.
    harness.frame(vec![Event::Ime(ImeEvent::Commit("日本".to_owned()))], &mut show);
    assert_eq!(draft, "日本");
    assert!(!response.send);
}

#[test]
fn an_empty_draft_cannot_be_sent_and_a_busy_composer_offers_stop() {
    let rect = ChatLayout::new(column(), 120.0).composer;
    // The send button sits at the right end of the bottom row.
    let button = pos2(rect.right() - 8.0 - 16.0, rect.bottom() - 23.0);
    for (draft, busy, expected) in [
        ("", false, ComposerResponse { send: false, stop: false }),
        ("Hello", false, ComposerResponse { send: true, stop: false }),
        ("Hello", true, ComposerResponse { send: false, stop: true }),
    ] {
        let mut harness = Harness::new();
        let mut draft = draft.to_owned();
        let mut response = ComposerResponse::default();
        harness.click(button, |ui| {
            let shown = Composer::new(1, &mut draft).busy(busy).show(ui, rect, |_| {});
            response.send |= shown.send;
            response.stop |= shown.stop;
        });
        assert_eq!(response, expected, "draft {draft:?}, busy {busy}");
    }
}

#[test]
fn overflowing_tabs_expose_every_conversation_and_keep_the_active_one_visible() {
    use egui_kittest::kittest::Queryable as _;
    let tabs: Vec<_> = (0..12)
        .map(|id| Tab {
            id,
            title: if id == 0 {
                "Hidden first"
            } else if id == 11 {
                "Active last"
            } else {
                "Other"
            },
            busy: false,
            closable: true,
        })
        .collect();
    let mut harness = egui_kittest::Harness::new_ui_state(
        |ui, action| {
            let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(280.0, 34.0));
            if let Some(next) = chat::tab_strip(ui, rect, &tabs, 11) {
                *action = Some(next);
            }
        },
        None::<TabAction>,
    );
    assert!(harness.query_by_label("Active last").is_some());
    assert!(harness.query_by_label("Hidden first").is_none());
    harness.get_by_label("All chats").focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    harness.get_by_label("Hidden first").focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(*harness.state(), Some(TabAction::Select(0)));
}

#[test]
fn two_tab_strips_can_reuse_conversation_ids_and_close_with_the_keyboard() {
    use egui_kittest::kittest::{NodeT as _, Queryable as _};
    let mut harness = egui_kittest::Harness::new_ui_state(
        |ui, action| {
            let first = [Tab { id: 1, title: "Left", busy: false, closable: true }];
            let second = [Tab { id: 1, title: "Right", busy: false, closable: true }];
            let _ = chat::tab_strip(ui, Rect::from_min_size(pos2(20.0, 20.0), vec2(260.0, 34.0)), &first, 0);
            if let Some(next) =
                chat::tab_strip(ui, Rect::from_min_size(pos2(20.0, 80.0), vec2(260.0, 34.0)), &second, 0)
            {
                *action = Some(next);
            }
        },
        None::<TabAction>,
    );
    assert_ne!(
        harness.get_by_label("Close chat: Left").accesskit_node().id(),
        harness.get_by_label("Close chat: Right").accesskit_node().id()
    );
    harness.get_by_label("Close chat: Right").focus();
    harness.run();
    harness.key_press(Key::Space);
    harness.run();
    assert_eq!(*harness.state(), Some(TabAction::Close(0)));
}

#[test]
fn composer_controls_have_translated_names_and_disable_empty_sends() {
    use egui_kittest::kittest::{NodeT as _, Queryable as _};
    let labels = chat::ChatLabels { send: "Senden", stop: "Anhalten", ..Default::default() };
    let mut harness = egui_kittest::Harness::new_ui_state(
        |ui, state: &mut (String, bool)| {
            let response = Composer::new(1, &mut state.0).labels(labels).show(
                ui,
                Rect::from_min_size(pos2(20.0, 20.0), vec2(360.0, 120.0)),
                |_| {},
            );
            state.1 |= response.send;
        },
        (String::new(), false),
    );
    assert!(harness.get_by_label("Senden").accesskit_node().is_disabled());
    harness.state_mut().0 = "Hallo".to_owned();
    harness.run();
    harness.get_by_label("Senden").focus();
    harness.run();
    harness.key_press(Key::Space);
    harness.run();
    assert!(harness.state().1);
}

#[test]
fn markdown_links_are_named_and_keyboard_activatable() {
    use egui_kittest::kittest::Queryable as _;
    let mut harness = egui_kittest::Harness::new_ui_state(
        |ui, opened| {
            chat::markdown::render(ui, "Read [wiki](https://example.org/wiki)");
            *opened |= ui.ctx().output(|output| {
                output.commands.iter().any(|command| {
            matches!(command, spaceui::egui::OutputCommand::OpenUrl(url) if url.url == "https://example.org/wiki")
        })
            });
        },
        false,
    );
    harness.get_by_label("wiki").focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert!(*harness.state());
    *harness.state_mut() = false;
    harness.get_by_label("wiki").click_accesskit();
    harness.run();
    assert!(*harness.state(), "assistive technology can activate the link");
}

#[test]
fn commonmark_reference_links_work_in_replies_and_user_bubbles() {
    use egui_kittest::kittest::Queryable as _;
    type Render = fn(&mut spaceui::egui::Ui, &str);
    let renderers: [Render; 3] = [
        chat::markdown::render,
        |ui, text| chat::transcript::reply(ui, text, false),
        |ui, text| chat::transcript::user_bubble(ui, text, ""),
    ];
    for render in renderers {
        let mut harness = egui_kittest::Harness::new_ui_state(
            move |ui, opened| {
                render(ui, "[Reference][guide]\n\n[guide]: https://example.org/a_(b)");
                *opened |= ui.ctx().output(|output| output.commands.iter().any(|command| {
                    matches!(command, spaceui::egui::OutputCommand::OpenUrl(url) if url.url == "https://example.org/a_(b)")
                }));
            },
            false,
        );
        harness.get_by_label("Reference").click();
        harness.run();
        assert!(*harness.state());
    }
}

#[test]
fn commonmark_renders_nested_lists_tables_and_changing_streamed_content() {
    use egui_kittest::kittest::Queryable as _;
    let mut harness = egui_kittest::Harness::builder()
        // The upstream code toolbar uses a copy symbol absent from bundled fonts.
        .allow_missing_glyphs()
        .build_ui_state(
            |ui, text: &mut String| chat::markdown::render(ui, text),
            "- Parent\n  - Nested\n\n| Name | Value |\n| --- | --- |\n| Galaxy | M31 |\n\n```rust\nlet x = 1;"
                .to_owned(),
        );
    for label in ["Parent", "Nested", "Name", "Value", "Galaxy", "M31"] {
        harness.get_by_label(label);
    }
    *harness.state_mut() = "A completed **reply** with [Final](https://example.org).".to_owned();
    harness.run();
    harness.get_by_label("Final");
    chat::markdown::clear_cache(&harness.ctx);
    harness.run();
    harness.get_by_label("Final");
}

#[test]
fn commonmark_application_owned_cache_keeps_link_hooks() {
    use egui_kittest::kittest::Queryable as _;
    let mut cache = chat::markdown::CommonMarkCache::default();
    cache.add_link_hook("app://details");
    let mut harness = egui_kittest::Harness::new_ui_state(
        |ui, state: &mut (chat::markdown::CommonMarkCache, bool)| {
            chat::markdown::render_with_cache(ui, &mut state.0, "[Details](app://details)");
            state.1 |= state.0.get_link_hook("app://details") == Some(true);
            assert!(!ui.ctx().output(|output| output.commands.iter().any(|command| {
                matches!(command, spaceui::egui::OutputCommand::OpenUrl(url) if url.url == "app://details")
            })));
        },
        (cache, false),
    );
    harness.get_by_label("Details").click();
    harness.run();
    assert!(harness.state().1);
}

#[test]
fn reduced_motion_chat_settles_instead_of_repainting_forever() {
    let mut harness = egui_kittest::Harness::new_ui(|ui| {
        ui.style_mut().animation_time = 0.0;
        chat::transcript::reply(ui, "Streaming", true);
        let _ = chat::transcript::thinking_with_label(ui, "Reasoning", true, false, "Working");
        chat::transcript::Step {
            icon: spaceui::Icon::Search,
            title: "Searching",
            detail: "",
            time: "",
            status: chat::transcript::StepStatus::Running,
        }
        .show(ui, false, false);
        let tabs = [Tab { id: 1, title: "Running", busy: true, closable: true }];
        let _ = chat::tab_strip(ui, Rect::from_min_size(pos2(20.0, 180.0), vec2(280.0, 34.0)), &tabs, 0);
        let mut draft = String::new();
        let _ = Composer::new(1, &mut draft).busy(true).stoppable(false).show(
            ui,
            Rect::from_min_size(pos2(20.0, 240.0), vec2(360.0, 100.0)),
            |_| {},
        );
    });
    assert!(harness.run_ok().is_some());
}

#[test]
fn composer_metrics_share_measurement_and_preserve_send_behavior() {
    use chat::ComposerMetrics;
    use egui_kittest::kittest::Queryable;
    let metrics = ComposerMetrics {
        max_lines: 2,
        row_height: 54.0,
        button_size: 44.0,
        padding: vec2(20.0, 16.0),
        ..Default::default()
    };
    let mut harness = egui_kittest::Harness::new_ui_state(
        move |ui, (draft, sends): &mut (String, usize)| {
            let composer = Composer::new("configured", draft).metrics(metrics);
            let height = composer.desired_height(ui, 360.0);
            let rect = Rect::from_min_size(pos2(20.0, 40.0), vec2(360.0, height));
            *sends += usize::from(composer.show(ui, rect, |_| {}).send);
        },
        ("A draft".to_owned(), 0),
    );
    let button = harness.get_by_label("Send");
    assert!((button.rect().width() - 44.0).abs() < 0.01);
    button.click();
    harness.run();
    assert_eq!(harness.state().1, 1);
    assert_eq!(harness.state().0, "A draft");
}

#[test]
fn composer_growth_limits_and_installed_defaults_agree() {
    use chat::ComposerMetrics;
    let mut harness = Harness::new();
    let metrics = ComposerMetrics { max_lines: 2, row_height: 50.0, ..Default::default() };
    metrics.install(&harness.ctx);
    spaceui::Theme::deep_space().install(&harness.ctx);
    assert_eq!(ComposerMetrics::of(&harness.ctx), metrics);
    let mut draft = "line\n".repeat(40);
    harness.frame(Vec::new(), &mut |ui| {
        let measured = Composer::measure(ui, 360.0, &draft, "Hint");
        let composer = Composer::new("configured", &mut draft).hint("Hint");
        assert!((composer.desired_height(ui, 360.0) - measured).abs() < 0.01);
        assert!(measured <= 24.0 + 18.0 * 2.0 + 4.0 + 50.0);
        let taller = composer.metrics(ComposerMetrics { max_lines: 10, ..metrics });
        assert!(taller.desired_height(ui, 360.0) > measured);
    });
}

#[test]
fn invalid_composer_metrics_and_rectangles_are_safe() {
    use chat::ComposerMetrics;
    let mut harness = Harness::new();
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, 0.0, f32::MAX] {
        let metrics = ComposerMetrics {
            max_lines: usize::MAX,
            font_size: value,
            line_height: value,
            padding: spaceui::egui::Vec2::splat(value),
            row_height: value,
            button_size: value,
            corner_radius: value,
        };
        metrics.install(&harness.ctx);
        harness.frame(Vec::new(), &mut |ui| {
            let mut draft = "Draft".to_owned();
            let composer = Composer::new("invalid", &mut draft).metrics(metrics);
            assert!(composer.desired_height(ui, value).is_finite());
            let response = composer.show(ui, Rect::from_min_size(pos2(value, 0.0), vec2(0.0, 0.0)), |_| {});
            assert_eq!(response, ComposerResponse::default());
        });
    }
}

#[test]
fn fragmented_markdown_is_one_shape_and_streaming_can_restore_a_link() {
    use egui_kittest::kittest::Queryable as _;
    let ctx = spaceui::egui::Context::default();
    let brackets = "[".repeat(16_384);
    for _ in 0..2 {
        let output = ctx.run_ui(spaceui::egui::RawInput::default(), |ui| chat::markdown::render(ui, &brackets));
        assert_eq!(output.shapes.len(), 1, "a text fragment must not create a separate widget");
        output.drop_without_applying_deltas();
    }
    let mut harness =
        egui_kittest::Harness::new_ui_state(|ui, text: &mut String| chat::markdown::render(ui, text), "[".repeat(1024));
    *harness.state_mut() = "[Completed](https://example.org)".to_owned();
    harness.run();
    harness.get_by_label("Completed");
}

#[test]
fn explicit_markdown_budget_reports_fallback_without_truncating_source() {
    use egui_kittest::kittest::Queryable as _;
    let source = "**bold** [link](https://example.org)";
    let harness = egui_kittest::Harness::new_ui_state(
        move |ui, fallback| *fallback = chat::markdown::Markdown::new(source).max_events(1).show(ui).fallback,
        false,
    );
    assert!(*harness.state());
    harness.get_by_label(source);
    assert!(harness.query_by_label("link").is_none());
}
