//! A headless egui harness: frames with simulated pointer and keyboard input.
#![allow(dead_code, unreachable_pub)] // Each test file uses a different part.

use spaceui::Theme;
use spaceui::egui::{Context, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Ui, pos2, vec2};

pub struct Harness {
    pub ctx: Context,
    pub window: Rect,
    time: f64,
}

impl Harness {
    /// A 1400 x 900 window with the default theme installed.
    pub fn new() -> Self {
        let ctx = Context::default();
        Theme::default().install(&ctx);
        Self { ctx, window: Rect::from_min_size(Pos2::ZERO, vec2(1400.0, 900.0)), time: 0.0 }
    }

    /// One frame with `events`.
    pub fn frame(&mut self, events: Vec<Event>, ui: &mut impl FnMut(&mut Ui)) {
        self.frame_with(events, Modifiers::NONE, ui);
    }

    /// One frame with `events` while `modifiers` are held, as a window reports them.
    pub fn frame_with(&mut self, mut events: Vec<Event>, modifiers: Modifiers, ui: &mut impl FnMut(&mut Ui)) {
        self.time += 1.0 / 60.0;
        events.insert(0, Event::ModifiersChanged(modifiers));
        let input = RawInput { screen_rect: Some(self.window), time: Some(self.time), events, ..Default::default() };
        self.ctx.run_ui(input, |root| ui(root)).drop_without_applying_deltas();
    }

    /// Lays out once, then clicks at `at` (press, then release). `ui` runs
    /// every frame; the click registers in the last one.
    pub fn click(&mut self, at: Pos2, mut ui: impl FnMut(&mut Ui)) {
        self.frame(Vec::new(), &mut ui);
        self.frame(vec![Event::PointerMoved(at), button(at, true)], &mut ui);
        self.frame(vec![button(at, false)], &mut ui);
    }

    /// Types `text`, then presses `key` with `modifiers`, one frame each.
    pub fn type_then(&mut self, text: &str, key: Key, modifiers: Modifiers, mut ui: impl FnMut(&mut Ui)) {
        self.frame(vec![Event::Text(text.to_owned())], &mut ui);
        let press = Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers };
        self.frame_with(vec![press], modifiers, &mut ui);
    }
}

fn button(at: Pos2, pressed: bool) -> Event {
    Event::PointerButton { pos: at, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE }
}

/// The centre of the `index`th of `count` equal slots across `rect`.
pub fn slot_center(rect: Rect, index: usize, count: usize) -> Pos2 {
    let width = rect.width() / count as f32;
    pos2(rect.left() + width * (index as f32 + 0.5), rect.center().y)
}
