//! The app's own title bar. It lives on the tooltip layer, above modal
//! dialogs, so the window can always be moved, maximized and closed.
use super::TitleStatus;
use crate::{Icon, Svg, Theme, paint, theme::semibold};
use egui::{
    Align, Align2, Area, Color32, Context, Id, Layout, Order, PointerButton, Rect, Sense, Ui, UiBuilder, Vec2,
    ViewportCommand, pos2, vec2,
};

const BUTTON_WIDTH: f32 = 46.0;

pub(super) fn show(ctx: &Context, window: Rect, title: &str, logo: Option<Svg>, status: Option<TitleStatus<'_>>) {
    let rect = Rect::from_min_size(window.min, vec2(window.width(), Theme::of(ctx).metrics.title_bar));
    let area = Area::new(Id::unique("spaceui-title-bar")).order(Order::Tooltip).fixed_pos(rect.min).constrain(false);
    area.show(ctx, |ui| {
        ui.set_min_size(rect.size());
        drag_to_move(ui, rect);
        let title_right = brand(ui, rect, title, logo);
        window_buttons(ui, rect);
        if let Some(status) = status {
            let area = Rect::from_min_max(
                pos2(title_right + 24.0, rect.top() + 7.0),
                pos2(rect.right() - 3.0 * BUTTON_WIDTH - 14.0, rect.bottom() - 7.0),
            );
            if area.width() > 0.0 {
                ui.scope_builder(UiBuilder::new().max_rect(area).layout(Layout::right_to_left(Align::Center)), |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    status(ui);
                });
            }
        }
    });
}

/// Dragging the bar moves the window; double-clicking maximizes or restores it.
/// Registered first, so the buttons drawn later take precedence.
fn drag_to_move(ui: &Ui, rect: Rect) {
    let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    let drag = ui.interact(rect, Id::unique("spaceui-title-drag"), Sense::click_and_drag());
    if drag.double_clicked() {
        ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(!maximized));
    } else if drag.drag_started_by(PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }
}

/// The logo (with a soft accent glow) and the title; returns the title's right edge.
fn brand(ui: &Ui, rect: Rect, title: &str, logo: Option<Svg>) -> f32 {
    let p = Theme::of(ui.ctx()).palette;
    let mut text_x = rect.left() + 18.0;
    if let Some(logo) = logo {
        let logo_rect = Rect::from_center_size(pos2(rect.left() + 38.0, rect.center().y + 1.0), Vec2::splat(34.0));
        paint::radial_glow(ui.painter(), logo_rect.center(), 30.0, p.accent, 30);
        logo.paint(ui.painter(), logo_rect, Color32::WHITE);
        text_x = logo_rect.right() + 10.0;
    }
    ui.painter().text(pos2(text_x, rect.center().y + 1.0), Align2::LEFT_CENTER, title, semibold(18.0), p.text).right()
}

/// Minimize, maximize or restore, and close, at the right end.
fn window_buttons(ui: &Ui, rect: Rect) {
    let p = Theme::of(ui.ctx()).palette;
    let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    let buttons = [
        (Icon::WinMin, ViewportCommand::Minimized(true)),
        (if maximized { Icon::WinRestore } else { Icon::WinMax }, ViewportCommand::Maximized(!maximized)),
        (Icon::WinClose, ViewportCommand::Close),
    ];
    for (i, (icon, command)) in buttons.into_iter().enumerate() {
        let button = Rect::from_min_size(
            pos2(rect.right() - BUTTON_WIDTH * (3 - i) as f32, rect.top()),
            vec2(BUTTON_WIDTH, rect.height() - 6.0),
        );
        let response = ui.interact(button, Id::unique(("spaceui-window-button", i)), Sense::click());
        let close = i == 2;
        if response.hovered() {
            let plate = if close { Color32::from_rgb(214, 64, 58) } else { p.field_hover };
            ui.painter().rect_filled(button.shrink2(vec2(4.0, 5.0)), 8.0, plate);
        }
        let ink = if response.hovered() { Color32::WHITE } else { p.text_2 };
        icon.paint(ui.painter(), Rect::from_center_size(button.center(), Vec2::splat(20.0)), ink);
        if response.clicked() {
            ui.ctx().send_viewport_cmd(command);
        }
    }
}
