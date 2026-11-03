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
//! let column = ui.max_rect(); // e.g. `Regions::right` from the shell
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
use egui::{
    ComboBox, CursorIcon, FontId, Id, Rect, Role, Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2, WidgetInfo, pos2,
    vec2,
};

/// Labels shared by the chat controls. Supply translated strings with
/// [`tab_strip_with_options`] and [`Composer::labels`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChatLabels<'a> {
    /// The new-conversation button.
    pub new_chat: &'a str,
    /// The close-conversation button.
    pub close_chat: &'a str,
    /// The menu containing every conversation when tabs overflow.
    pub all_chats: &'a str,
    /// The send button's accessible name.
    pub send: &'a str,
    /// The send button's keyboard-help tooltip.
    pub send_help: &'a str,
    /// The stop button.
    pub stop: &'a str,
    /// A running reply that cannot be stopped.
    pub busy: &'a str,
}

impl Default for ChatLabels<'_> {
    fn default() -> Self {
        Self {
            new_chat: "New chat",
            close_chat: "Close chat",
            all_chats: "All chats",
            send: "Send",
            send_help: "Send (Enter) · Shift+Enter for a new line",
            stop: "Stop",
            busy: "Reply in progress",
        }
    }
}

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
        // Shrink spacing as the column shrinks, keeping every rectangle ordered
        // and contained even during window resizing or when the composer grows.
        let width = column.width().max(0.0);
        let height = column.height().max(0.0);
        let padding_x = 12.0_f32.min(width / 2.0);
        let padding_y = 12.0_f32.min(height / 4.0);
        let top = column.top() + padding_y;
        let bottom = column.bottom() - padding_y;
        let available = (bottom - top).max(0.0);
        let strip_height = 34.0_f32.min(available / 3.0);
        let gap = 10.0_f32.min((available - strip_height) / 4.0);
        let requested = if composer_height.is_finite() { composer_height.max(0.0) } else { 0.0 };
        let composer_height = requested.min((available - strip_height - 2.0 * gap).max(0.0));
        let strip = Rect::from_min_size(
            pos2(column.left() + padding_x, top),
            vec2((width - 2.0 * padding_x).max(0.0), strip_height),
        );
        let composer = Rect::from_min_max(
            pos2(column.left() + padding_x, bottom - composer_height),
            pos2(column.right() - padding_x, bottom),
        );
        let body_top = (strip.bottom() + gap).min(composer.top());
        let body = Rect::from_min_max(
            pos2(column.left() + padding_x, body_top),
            pos2(column.right() - padding_x, (composer.top() - gap).max(body_top)),
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
    /// Shows a close button and allows middle-click to close.
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

/// Conversation tabs in `rect` (close button or middle-click) with a "+" for
/// a new one, ruled off from the transcript below. `active` is the open tab's
/// index; the app applies the returned action.
pub fn tab_strip(ui: &mut Ui, rect: Rect, tabs: &[Tab<'_>], active: usize) -> Option<TabAction> {
    let id = ui.next_auto_id();
    ui.skip_ahead_auto_ids(1);
    tab_strip_with_options(ui, id, rect, tabs, active, &ChatLabels::default())
}

/// Conversation tabs with a stable, parent-scoped identity and translated labels.
///
/// Give each strip its own `id_salt`; each tab's `id` must be unique within it.
/// Overflow keeps the active tab visible and offers all tabs in a keyboard-
/// accessible menu. An out-of-range `active` index selects no tab.
pub fn tab_strip_with_options(
    ui: &mut Ui,
    id_salt: impl std::hash::Hash + std::fmt::Debug,
    rect: Rect,
    tabs: &[Tab<'_>],
    active: usize,
    labels: &ChatLabels<'_>,
) -> Option<TabAction> {
    if !rect.is_positive() {
        return None;
    }
    let id = ui.make_persistent_id(("spaceui-chat-tabs", id_salt));
    let p = Theme::of(ui.ctx()).palette;
    ui.painter().hline(rect.x_range().expand(9.0), rect.bottom() + 8.0, Stroke::new(1.0, p.border_soft));
    let overflow = tabs.len() as f32 * 72.0 > rect.width() - 40.0;
    let controls = if overflow { 74.0 } else { 40.0 };
    let available = (rect.width() - controls).max(0.0);
    let count = if overflow { (available / 72.0) as usize } else { tabs.len() }.min(tabs.len());
    let start = if count > 0 { active.min(tabs.len().saturating_sub(1)).saturating_sub(count - 1) } else { 0 };
    let tab_width = (available / count.max(1) as f32).min(190.0);
    let mut x = rect.left();
    let mut action = None;
    for (i, tab) in tabs.iter().enumerate().skip(start).take(count) {
        let tab_rect = Rect::from_min_size(pos2(x, rect.top()), vec2(tab_width - 4.0, rect.height()));
        match draw_tab(ui, tab_rect, tab, i == active, labels, id.with(tab.id)) {
            Some(TabInteraction::Close) => action = Some(TabAction::Close(i)),
            Some(TabInteraction::Select) => action = Some(TabAction::Select(i)),
            None => {}
        }
        x += tab_width;
    }
    if overflow {
        let menu_width = 34.0_f32.min(rect.width() / 2.0);
        let menu = Rect::from_min_size(
            pos2(rect.right() - menu_width - 34.0_f32.min(rect.width() / 2.0), rect.top()),
            vec2(menu_width, rect.height()),
        );
        let mut child = ui.new_child(UiBuilder::new().max_rect(menu).id_salt(id.with("overflow")));
        child.set_clip_rect(menu.intersect(ui.clip_rect()));
        ComboBox::from_id_salt("all")
            .selected_text("…")
            .width(menu_width)
            .show_ui(&mut child, |ui| {
                for (i, tab) in tabs.iter().enumerate() {
                    ui.horizontal(|ui| {
                        if ui.selectable_label(i == active, tab.title).clicked() {
                            action = Some(TabAction::Select(i));
                            ui.close();
                        }
                        if tab.closable && ui.small_button(labels.close_chat).clicked() {
                            action = Some(TabAction::Close(i));
                            ui.close();
                        }
                    });
                }
            })
            .response
            .widget_info(|| WidgetInfo::labeled(Role::ComboBox, ui.is_enabled(), labels.all_chats));
        x = rect.right() - 34.0_f32.min(rect.width() / 2.0);
    }
    let plus = Rect::from_min_size(pos2(x, rect.top()), vec2(30.0_f32.min((rect.right() - x).max(0.0)), rect.height()));
    let response = ui.interact(plus, id.with("new"), Sense::click()).on_hover_text(labels.new_chat);
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), labels.new_chat));
    if response.hovered() || response.has_focus() {
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

enum TabInteraction {
    Select,
    Close,
}

fn draw_tab(
    ui: &mut Ui,
    rect: Rect,
    tab: &Tab<'_>,
    active: bool,
    labels: &ChatLabels<'_>,
    id: Id,
) -> Option<TabInteraction> {
    let p = Theme::of(ui.ctx()).palette;
    let response = ui.interact(rect, id.with("tab"), Sense::click());
    response.widget_info(|| WidgetInfo::selected(Role::Tab, ui.is_enabled(), active, tab.title));
    if active {
        ui.painter().rect(rect, 10.0, p.field, Stroke::new(1.0, p.border), StrokeKind::Inside);
    } else if response.hovered() || response.has_focus() {
        ui.painter().rect_filled(rect, 10.0, p.field.gamma_multiply(0.6));
    }
    if response.has_focus() {
        ui.painter().rect_stroke(rect, 10.0, Stroke::new(1.5, p.accent_hi), StrokeKind::Inside);
    }
    let mut text_x = rect.left() + 12.0;
    if tab.busy {
        let time = ui.input(|i| i.time) as f32;
        let animate = ui.style().animation_time > 0.0;
        let dot = pos2(text_x + 3.0, rect.center().y);
        let alpha = if animate { 0.15 + 0.2 * (time * 4.0).sin().abs() } else { 0.25 };
        ui.painter().circle_filled(dot, 6.0, p.accent.gamma_multiply(alpha));
        ui.painter().circle_filled(dot, 3.0, p.accent_hi);
        text_x += 14.0;
        if animate {
            ui.ctx().request_repaint();
        }
    }
    let text_right = if tab.closable { rect.right() - 28.0 } else { rect.right() - 10.0 };
    let ink = if active { p.text } else { p.text_2 };
    let galley = truncated(ui, tab.title, FontId::proportional(13.0), ink, (text_right - text_x).max(0.0));
    ui.painter().galley(pos2(text_x, rect.center().y - galley.size().y / 2.0), galley, ink);
    let mut closing = false;
    if tab.closable {
        let close = Rect::from_center_size(pos2(rect.right() - 15.0, rect.center().y), Vec2::splat(22.0));
        let label = format!("{}: {}", labels.close_chat, tab.title);
        let button = ui.interact(close, id.with("close"), Sense::click());
        button.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), &label));
        if button.hovered() || button.has_focus() {
            ui.painter().rect_filled(close, 6.0, p.field_hover);
        }
        let ink = if button.hovered() || button.has_focus() { p.text } else { p.muted };
        Icon::Close.paint(ui.painter(), Rect::from_center_size(close.center(), Vec2::splat(13.0)), ink);
        closing = button.on_hover_text(label).clicked();
    }
    if closing || (response.middle_clicked() && tab.closable) {
        Some(TabInteraction::Close)
    } else if response.clicked() {
        Some(TabInteraction::Select)
    } else {
        None
    }
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

    #[test]
    fn tiny_columns_and_extreme_composer_sizes_stay_contained() {
        for width in [0.0, 1.0, 12.0, 64.0, 320.0] {
            for height in [0.0, 1.0, 20.0, 80.0, 800.0] {
                for requested in [-10.0, 0.0, 120.0, 10_000.0, f32::NAN, f32::INFINITY] {
                    let column = Rect::from_min_size(pos2(12.0, 15.0), vec2(width, height));
                    let layout = ChatLayout::new(column, requested);
                    for part in [layout.strip, layout.body, layout.composer] {
                        assert!(part.width() >= 0.0 && part.height() >= 0.0, "{part:?}");
                        assert!(column.contains_rect(part), "{column:?}: {part:?}");
                    }
                    assert!(layout.strip.bottom() <= layout.body.top());
                    assert!(layout.body.bottom() <= layout.composer.top());
                }
            }
        }
    }
}
