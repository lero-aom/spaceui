//! Building blocks for an assistant chat column.
//!
//! The app owns its conversations, models and backends; these pieces lay out
//! and draw:
//!
//! - [`ChatLayout`] splits a column into tab strip, transcript and composer;
//! - [`tab_strip`] draws closable conversation tabs and a "+";
//! - [`transcript`] holds the message pieces: bubbles, replies, thinking and
//!   tool-call [`Step`](transcript::Step)s;
//! - [`Composer`] is the message box, with accessory dropdowns and a send button;
//! - [`markdown`] renders replies.
//!
//! ```
//! use spaceui::chat::{self, ChatLayout, Composer, Tab, transcript};
//! use spaceui::{Theme, egui};
//!
//! # let ctx = egui::Context::default();
//! # Theme::carbon_copper().install(&ctx);
//! # let mut draft = String::new();
//! # ctx.run_ui(Default::default(), |ui| {
//! let column = ui.max_rect(); // e.g. `Regions::side` from the shell
//! let hint = "Ask anything";
//! let layout = ChatLayout::new(column, Composer::measure(ui, column.width(), &draft, hint));
//!
//! let tabs = [Tab { id: 1, title: "Welcome", busy: false, closable: false }];
//! let _action = chat::tab_strip(ui, layout.strip, &tabs, 0);
//!
//! let mut body = ui.new_child(egui::UiBuilder::new().max_rect(layout.body));
//! transcript::user_bubble(&mut body, "How is SPY trading?", "09:41");
//! transcript::reply(&mut body, "Above its **daily** trend.", false);
//!
//! let response = Composer::new(1, &mut draft).hint(hint).show(ui, layout.composer, |_accessories| {});
//! if response.send {
//!     // take the draft and start a reply
//! }
//! # }).drop_without_applying_deltas();
//! ```
mod composer;
pub mod markdown;
pub mod transcript;

pub use composer::{Composer, ComposerResponse};

use crate::{Icon, Theme, text::truncated};
use egui::{CursorIcon, FontId, Id, Rect, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};

/// The parts of a chat column, top to bottom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChatLayout {
    /// The conversation tabs along the top.
    pub strip: Rect,
    /// The transcript between the strip and the composer.
    pub body: Rect,
    /// The message box at the foot.
    pub composer: Rect,
}

impl ChatLayout {
    /// Splits `column` (a side column from the shell, say) for a composer
    /// `composer_height` tall, as from [`Composer::measure`].
    #[must_use]
    pub fn new(column: Rect, composer_height: f32) -> Self {
        let strip = Rect::from_min_size(column.min + vec2(10.0, 9.0), vec2(column.width() - 20.0, 34.0));
        let composer = Rect::from_min_max(
            pos2(column.left() + 12.0, column.bottom() - 12.0 - composer_height),
            pos2(column.right() - 12.0, column.bottom() - 12.0),
        );
        let body = Rect::from_min_max(
            pos2(column.left() + 8.0, strip.bottom() + 12.0),
            pos2(column.right() - 6.0, composer.top() - 8.0),
        );
        Self { strip, body, composer }
    }
}

/// One conversation tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tab<'a> {
    /// Stable across frames; it keys the tab's widgets.
    pub id: u64,
    /// Shown, cut with an ellipsis if long.
    pub title: &'a str,
    /// Shows a pulsing dot while a reply runs.
    pub busy: bool,
    /// Shows a close button on hover and allows middle-click to close.
    pub closable: bool,
}

/// What the owner did in [`tab_strip`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TabAction {
    /// Open the tab at this index.
    Select(usize),
    /// Close the tab at this index.
    Close(usize),
    /// Start a new conversation.
    New,
}

/// Conversation tabs in `rect` (close on hover or middle-click) with a "+" for
/// a new one, ruled off from the transcript below. `active` is the open tab's
/// index; the app applies the returned action.
pub fn tab_strip(ui: &mut Ui, rect: Rect, tabs: &[Tab<'_>], active: usize) -> Option<TabAction> {
    let p = Theme::of(ui.ctx()).palette;
    ui.painter().hline(rect.x_range().expand(9.0), rect.bottom() + 8.0, Stroke::new(1.0, p.border_soft));
    let plus_width = 34.0;
    let tab_width = ((rect.width() - plus_width - 6.0) / tabs.len().max(1) as f32).clamp(72.0, 190.0);
    let time = ui.input(|i| i.time) as f32;
    let mut x = rect.left();
    let mut action = None;
    for (i, tab) in tabs.iter().enumerate() {
        let tab_rect = Rect::from_min_size(pos2(x, rect.top()), vec2(tab_width - 4.0, rect.height()));
        if tab_rect.right() > rect.right() - plus_width {
            break;
        }
        let response = ui.interact(tab_rect, Id::unique(("spaceui-chat-tab", tab.id)), Sense::click());
        let is_active = i == active;
        if is_active {
            ui.painter().rect(tab_rect, 10.0, p.field, Stroke::new(1.0, p.border), StrokeKind::Inside);
        } else if response.hovered() {
            ui.painter().rect_filled(tab_rect, 10.0, p.field.gamma_multiply(0.6));
        }
        let mut text_x = tab_rect.left() + 12.0;
        if tab.busy {
            let dot = pos2(text_x + 3.0, tab_rect.center().y);
            ui.painter().circle_filled(dot, 6.0, p.accent.gamma_multiply(0.15 + 0.2 * (time * 4.0).sin().abs()));
            ui.painter().circle_filled(dot, 3.0, p.accent_hi);
            text_x += 14.0;
            ui.ctx().request_repaint();
        }
        let show_close = tab.closable && (is_active || response.hovered());
        let text_right = if show_close { tab_rect.right() - 28.0 } else { tab_rect.right() - 10.0 };
        let ink = if is_active { p.text } else { p.text_2 };
        let galley = truncated(ui, tab.title, FontId::proportional(13.0), ink, text_right - text_x);
        ui.painter().galley(pos2(text_x, tab_rect.center().y - galley.size().y / 2.0), galley, ink);
        let mut closing = false;
        if show_close {
            let close = Rect::from_center_size(pos2(tab_rect.right() - 15.0, tab_rect.center().y), Vec2::splat(22.0));
            let x_button = ui.interact(close, Id::unique(("spaceui-chat-close", tab.id)), Sense::click());
            if x_button.hovered() {
                ui.painter().rect_filled(close, 6.0, p.field_hover);
            }
            let x_ink = if x_button.hovered() { p.text } else { p.muted };
            Icon::Close.paint(ui.painter(), Rect::from_center_size(close.center(), Vec2::splat(13.0)), x_ink);
            closing = x_button.clicked();
        }
        if closing || (response.middle_clicked() && tab.closable) {
            action = Some(TabAction::Close(i));
        } else if response.clicked() {
            action = Some(TabAction::Select(i));
        }
        x += tab_width;
    }
    let plus = Rect::from_center_size(pos2(x + 15.0, rect.center().y), Vec2::splat(30.0));
    let response = ui.interact(plus, Id::unique("spaceui-chat-new"), Sense::click()).on_hover_text("New chat");
    if response.hovered() {
        ui.painter().rect_filled(plus, 8.0, p.field_hover);
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    let plus_ink = if response.hovered() { p.text } else { p.text_2 };
    Icon::Plus.paint(ui.painter(), Rect::from_center_size(plus.center(), Vec2::splat(18.0)), plus_ink);
    if response.clicked() {
        action = Some(TabAction::New);
    }
    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layout_stacks_strip_body_and_composer_inside_the_column() {
        let column = Rect::from_min_size(pos2(1000.0, 50.0), vec2(380.0, 800.0));
        let layout = ChatLayout::new(column, 120.0);
        for part in [layout.strip, layout.body, layout.composer] {
            assert!(column.contains_rect(part), "{part:?}");
        }
        assert!(layout.strip.bottom() < layout.body.top());
        assert!(layout.body.bottom() < layout.composer.top());
        assert!((layout.composer.height() - 120.0).abs() < 0.01);
    }
}
