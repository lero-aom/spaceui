//! Exercise complete input sequences, retaining the caller's draft after sends.
use super::common::Harness;
use spaceui::{
    chat::{Composer, ComposerResponse},
    egui::{self, Event, Id, ImeEvent, Key, Modifiers, PointerButton, Rect, pos2, vec2},
};

// These flags intentionally span the composer's public input-state matrix.
#[allow(clippy::struct_excessive_bools)]
struct Driver {
    harness: Harness,
    draft: String,
    other: String,
    busy: bool,
    stoppable: bool,
    enabled: bool,
    focus: bool,
    actions: Vec<ComposerResponse>,
}

fn rect() -> Rect {
    Rect::from_min_size(pos2(20.0, 80.0), vec2(380.0, 120.0))
}

fn key(key: Key, pressed: bool, repeat: bool, modifiers: Modifiers) -> Event {
    Event::Key { key, physical_key: None, pressed, repeat, modifiers }
}

fn enter() -> Event {
    key(Key::Enter, true, false, Modifiers::NONE)
}
fn release() -> Event {
    key(Key::Enter, false, false, Modifiers::NONE)
}

impl Driver {
    fn new(draft: &str) -> Self {
        Self {
            harness: Harness::new(),
            draft: draft.to_owned(),
            other: String::new(),
            busy: false,
            stoppable: true,
            enabled: true,
            focus: true,
            actions: Vec::new(),
        }
    }

    fn frame(&mut self, events: Vec<Event>, move_focus: bool) {
        let Self { harness, draft, other, busy, stoppable, enabled, focus, actions } = self;
        harness.frame(events, &mut |ui| {
            let other = ui.add(egui::TextEdit::singleline(other).id(Id::unique("sequence-other")));
            if move_focus {
                other.request_focus();
            }
            let response = ui
                .add_enabled_ui(*enabled, |ui| {
                    Composer::new("sequence", draft)
                        .busy(*busy)
                        .stoppable(*stoppable)
                        .focus(std::mem::take(focus))
                        .show(ui, rect(), |_| {})
                })
                .inner;
            assert!(!(response.send && response.stop), "actions are mutually exclusive");
            actions.push(response);
        });
    }

    fn sent(&self) -> usize {
        self.actions.iter().filter(|action| action.send).count()
    }
    fn stopped(&self) -> usize {
        self.actions.iter().filter(|action| action.stop).count()
    }
}

#[test]
fn enter_matrix_sends_once_per_press_and_never_stops() {
    for draft in ["", " \n\t", "Hello"] {
        for busy in [false, true] {
            for stoppable in [false, true] {
                for enabled in [false, true] {
                    for focus in [false, true] {
                        let mut driver = Driver::new(draft);
                        driver.busy = busy;
                        driver.stoppable = stoppable;
                        driver.enabled = enabled;
                        driver.focus = focus;
                        driver.frame(vec![], false);
                        driver.frame(vec![enter()], false);
                        driver.frame(vec![key(Key::Enter, true, true, Modifiers::NONE)], false);
                        driver.frame(vec![release()], false);
                        driver.frame(vec![], false);
                        let expected = usize::from(enabled && focus && !busy && !draft.trim().is_empty());
                        assert_eq!(
                            driver.sent(),
                            expected,
                            "draft={draft:?}, busy={busy}, stoppable={stoppable}, enabled={enabled}, focus={focus}"
                        );
                        assert_eq!(driver.stopped(), 0);
                        assert_eq!(driver.draft, draft, "sending never consumes the caller's draft");
                    }
                }
            }
        }
    }
}

#[test]
fn pointer_matrix_obeys_busy_stoppable_and_enabled_state() {
    let at = pos2(rect().right() - 24.0, rect().bottom() - 23.0);
    for draft in ["", " \n\t", "Hello"] {
        for busy in [false, true] {
            for stoppable in [false, true] {
                for enabled in [false, true] {
                    let mut driver = Driver::new(draft);
                    driver.busy = busy;
                    driver.stoppable = stoppable;
                    driver.enabled = enabled;
                    driver.frame(vec![], false);
                    for pressed in [true, false] {
                        driver.frame(
                            vec![
                                Event::PointerMoved(at),
                                Event::PointerButton {
                                    pos: at,
                                    button: PointerButton::Primary,
                                    pressed,
                                    modifiers: Modifiers::NONE,
                                },
                            ],
                            false,
                        );
                    }
                    driver.frame(vec![], false);
                    assert_eq!(driver.sent(), usize::from(enabled && !busy && !draft.trim().is_empty()));
                    assert_eq!(driver.stopped(), usize::from(enabled && busy && stoppable));
                }
            }
        }
    }
}

#[test]
fn shift_enter_inserts_one_newline_and_a_later_enter_sends_once() {
    let mut driver = Driver::new("Hello");
    driver.frame(vec![], false);
    driver.frame(vec![key(Key::Enter, true, false, Modifiers::SHIFT)], false);
    driver.frame(vec![release()], false);
    assert_eq!(driver.sent(), 0);
    assert_eq!(driver.draft, "Hello\n");
    driver.frame(vec![enter()], false);
    driver.frame(vec![release()], false);
    assert_eq!(driver.sent(), 1);
}

#[test]
fn losing_focus_in_an_enter_frame_does_not_send() {
    let mut driver = Driver::new("Hello");
    driver.frame(vec![], false);
    driver.frame(vec![enter()], true);
    driver.frame(vec![release()], false);
    assert_eq!(driver.sent(), 0);
}

#[test]
fn requesting_focus_does_not_submit_an_event_from_another_field() {
    let mut driver = Driver::new("Hello");
    driver.focus = false;
    driver.frame(vec![], true);
    driver.focus = true;
    driver.frame(vec![enter()], false);
    driver.frame(vec![release()], false);
    assert_eq!(driver.sent(), 0);
    driver.frame(vec![enter()], false);
    assert_eq!(driver.sent(), 1);
}

#[test]
fn ime_preedit_and_commit_enter_do_not_submit_composition() {
    for commit_first in [false, true] {
        let mut driver = Driver::new("");
        driver.frame(vec![], false);
        driver.frame(
            vec![Event::Ime(ImeEvent::Preedit { text: "にほん".to_owned(), active_range_chars: Some(0..3) })],
            false,
        );
        driver.frame(vec![enter()], false);
        driver.frame(vec![release()], false);
        assert_eq!(driver.sent(), 0, "Enter during preedit must not submit");
        let commit = Event::Ime(ImeEvent::Commit("日本".to_owned()));
        driver.frame(if commit_first { vec![commit, enter()] } else { vec![enter(), commit] }, false);
        driver.frame(vec![release()], false);
        assert_eq!(driver.draft, "日本");
        assert_eq!(driver.sent(), 0, "the commit frame must not submit");
        driver.frame(vec![enter()], false);
        driver.frame(vec![release()], false);
        assert_eq!(driver.sent(), 1, "a subsequent explicit press sends the committed draft");
    }
}

#[test]
fn a_held_enter_does_not_send_when_busy_finishes() {
    let mut driver = Driver::new("Hello");
    driver.busy = true;
    driver.frame(vec![], false);
    driver.frame(vec![enter()], false);
    driver.busy = false;
    driver.frame(vec![key(Key::Enter, true, true, Modifiers::NONE)], false);
    driver.frame(vec![release()], false);
    assert_eq!(driver.sent(), 0);
    driver.frame(vec![enter()], false);
    assert_eq!(driver.sent(), 1);
}

#[test]
fn cancelled_composition_allows_a_later_explicit_send() {
    let mut driver = Driver::new("Hello");
    driver.frame(vec![], false);
    driver.frame(vec![Event::Ime(ImeEvent::Preedit { text: "x".to_owned(), active_range_chars: None })], false);
    driver.frame(vec![Event::Ime(ImeEvent::Preedit { text: String::new(), active_range_chars: None }), enter()], false);
    driver.frame(vec![release()], false);
    assert_eq!(driver.sent(), 0, "cancelling a composition must not send");
    assert_eq!(driver.draft, "Hello");
    driver.frame(vec![enter()], false);
    assert_eq!(driver.sent(), 1);
}
