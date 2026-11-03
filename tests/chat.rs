//! The chat kit's tab strip and composer, driven headlessly.
mod common;

use common::Harness;
use spaceui::chat::{self, ChatLayout, Composer, ComposerResponse, Tab, TabAction};
use spaceui::egui::{Key, Modifiers, Rect, pos2, vec2};

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
        harness.type_then("How is SPY?", Key::Enter, modifiers, show);
        assert_eq!(response.send, sends, "with {modifiers:?}");
        assert!(draft.starts_with("How is SPY?"), "the app takes the draft, not the composer: {draft:?}");
    }
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
