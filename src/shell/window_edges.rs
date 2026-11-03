//! Resize zones along the edges and corners of an undecorated window.
use egui::{Area, Context, CursorIcon, Id, Order, Rect, Sense, ViewportCommand, pos2, viewport::ResizeDirection};

pub(super) fn show(ctx: &Context) {
    // Edges are `EDGE` deep; corners are `CORNER` square.
    const EDGE: f32 = 5.0;
    const CORNER: f32 = 14.0;
    if ctx.input(|i| i.viewport().maximized.unwrap_or(false) || i.viewport().fullscreen.unwrap_or(false)) {
        return;
    }
    let window = ctx.content_rect();
    let (l, r, t, b) = (window.left(), window.right(), window.top(), window.bottom());
    let (m, k) = (EDGE, CORNER);
    let zones = [
        (Rect::from_min_max(pos2(l, t), pos2(l + k, t + k)), ResizeDirection::NorthWest, CursorIcon::ResizeNorthWest),
        (Rect::from_min_max(pos2(r - k, t), pos2(r, t + k)), ResizeDirection::NorthEast, CursorIcon::ResizeNorthEast),
        (Rect::from_min_max(pos2(l, b - k), pos2(l + k, b)), ResizeDirection::SouthWest, CursorIcon::ResizeSouthWest),
        (Rect::from_min_max(pos2(r - k, b - k), pos2(r, b)), ResizeDirection::SouthEast, CursorIcon::ResizeSouthEast),
        (Rect::from_min_max(pos2(l + k, t), pos2(r - k, t + m)), ResizeDirection::North, CursorIcon::ResizeVertical),
        (Rect::from_min_max(pos2(l + k, b - m), pos2(r - k, b)), ResizeDirection::South, CursorIcon::ResizeVertical),
        (Rect::from_min_max(pos2(l, t + k), pos2(l + m, b - k)), ResizeDirection::West, CursorIcon::ResizeHorizontal),
        (Rect::from_min_max(pos2(r - m, t + k), pos2(r, b - k)), ResizeDirection::East, CursorIcon::ResizeHorizontal),
    ];
    // Small areas on the tooltip layer: above everything, yet covering only the edges.
    for (i, (zone, direction, cursor)) in zones.into_iter().enumerate() {
        Area::new(Id::unique(("spaceui-resize", i))).order(Order::Tooltip).fixed_pos(zone.min).constrain(false).show(
            ctx,
            |ui| {
                let response = ui.allocate_rect(zone, Sense::drag());
                if response.hovered() || response.dragged() {
                    ui.ctx().set_cursor_icon(cursor);
                }
                if response.drag_started() {
                    ui.ctx().send_viewport_cmd(ViewportCommand::BeginResize(direction));
                }
            },
        );
    }
}
