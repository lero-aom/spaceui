//! The app's own title bar. It lives on the tooltip layer, above modal
//! dialogs, so the window can always be moved, maximized and closed.
use super::{TitleStatus, WindowLabels};
use crate::{Icon, Svg, Theme, paint, text::truncated, theme::semibold};
use egui::{
    Align, Area, Color32, Context, Id, Layout, Order, PointerButton, Rect, Role, Sense, Ui, UiBuilder, Vec2,
    ViewportCommand, WidgetInfo, pos2, vec2,
};

const BUTTON_WIDTH: f32 = 46.0;

pub(super) fn show(
    ctx: &Context,
    window: Rect,
    title: &str,
    logo: Option<Svg<'_>>,
    status: Option<TitleStatus<'_>>,
    labels: &WindowLabels<'_>,
) {
    let height = super::finite_size(Theme::of(ctx).metrics.title_bar).min(window.height());
    let rect = Rect::from_min_size(window.min, vec2(window.width(), height));
    if !rect.is_positive() {
        return;
    }
    let area = Area::new(Id::unique("spaceui-title-bar")).order(Order::Tooltip).fixed_pos(rect.min).constrain(false);
    area.show(ctx, |ui| {
        ui.set_min_size(rect.size());
        ui.set_clip_rect(ui.clip_rect().intersect(rect));
        drag_to_move(ui, rect, title, labels.move_window);
        let title_right = brand(ui, rect, title, logo);
        window_buttons(ui, rect, labels);
        if let Some(status) = status {
            let area = Rect::from_min_max(
                pos2(title_right + 24.0, rect.top() + 7.0),
                pos2(rect.right() - 3.0 * button_width(rect) - 14.0, rect.bottom() - 7.0),
            );
            if area.is_positive() {
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
fn drag_to_move(ui: &Ui, rect: Rect, title: &str, description: &str) {
    let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    // This is a pointer gesture region, not an extra keyboard tab stop.
    let drag = ui.interact(rect, Id::unique("spaceui-title-drag"), Sense::CLICK | Sense::DRAG);
    drag.widget_info(|| WidgetInfo::labeled(Role::TitleBar, true, title));
    ui.ctx().accesskit_node_builder(drag.id, |node| node.set_description(description));
    if drag.double_clicked() {
        ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(!maximized));
    } else if drag.drag_started_by(PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }
}

/// The logo (with a soft accent glow) and the title; returns the title's right edge.
fn brand(ui: &Ui, rect: Rect, title: &str, logo: Option<Svg<'_>>) -> f32 {
    let p = Theme::of(ui.ctx()).palette;
    let right = rect.right() - 3.0 * button_width(rect) - 12.0;
    if right <= rect.left() {
        return rect.left();
    }
    let painter = ui.painter().with_clip_rect(Rect::from_min_max(rect.min, pos2(right, rect.bottom())));
    let mut text_x = rect.left() + 18.0;
    if let Some(logo) = logo.filter(|_| right - rect.left() >= 58.0) {
        let logo_rect = Rect::from_center_size(pos2(rect.left() + 38.0, rect.center().y + 1.0), Vec2::splat(34.0));
        paint::radial_glow(&painter, logo_rect.center(), 30.0, p.accent, 30);
        logo.paint(&painter, logo_rect, Color32::WHITE);
        text_x = logo_rect.right() + 10.0;
    }
    let font_size = egui::TextStyle::Body.resolve(ui.style()).size * 18.0 / 14.0;
    let galley = truncated(ui, title, semibold(font_size), p.text, (right - text_x).max(0.0));
    let title_right = text_x + galley.size().x;
    painter.galley(pos2(text_x, rect.center().y + 1.0 - galley.size().y / 2.0), galley, p.text);
    title_right
}

fn button_width(rect: Rect) -> f32 {
    BUTTON_WIDTH.min(rect.width() / 3.0)
}

/// Minimize, maximize or restore, and close, at the right end.
fn window_buttons(ui: &Ui, rect: Rect, labels: &WindowLabels<'_>) {
    let p = Theme::of(ui.ctx()).palette;
    let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    let buttons = [
        (Icon::WinMin, ViewportCommand::Minimized(true)),
        (if maximized { Icon::WinRestore } else { Icon::WinMax }, ViewportCommand::Maximized(!maximized)),
        (Icon::WinClose, ViewportCommand::Close),
    ];
    for (i, (icon, command)) in buttons.into_iter().enumerate() {
        let width = button_width(rect);
        let button = Rect::from_min_size(
            pos2(rect.right() - width * (3 - i) as f32, rect.top()),
            vec2(width, (rect.height() - 6.0).max(1.0).min(rect.height())),
        );
        let label = match i {
            0 => labels.minimize,
            1 if maximized => labels.restore,
            1 => labels.maximize,
            _ => labels.close,
        };
        let response =
            ui.interact(button, Id::unique(("spaceui-window-button", i)), Sense::click()).on_hover_text(label);
        response.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
        let close = i == 2;
        if response.hovered() || response.has_focus() {
            // Close turns a deeper shade of the palette's red, as destructive actions do.
            let plate = if close { paint::mix(p.negative, Color32::BLACK, 0.15) } else { p.field_hover };
            ui.painter().rect_filled(
                button.shrink2(vec2(4.0_f32.min(width * 0.2), 5.0_f32.min(button.height() * 0.2))),
                8.0,
                plate,
            );
        }
        let ink = match (response.hovered(), close) {
            (true, true) => p.on_accent,
            (true, false) => p.text,
            (false, _) => p.text_2,
        };
        let icon_size = 20.0_f32.min(width * 0.65).min(button.height() * 0.75);
        icon.paint(ui.painter(), Rect::from_center_size(button.center(), Vec2::splat(icon_size)), ink);
        if response.clicked() {
            ui.ctx().send_viewport_cmd(command);
        }
    }
}
