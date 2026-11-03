use super::{RailItem, finite_size};
use crate::{Icon, Theme, paint};
use egui::{
    Color32, CursorIcon, EventFilter, Id, Key, Modifiers, Rect, Response, Role, ScrollArea, Sense, Stroke, StrokeKind,
    Ui, UiBuilder, Vec2, WidgetInfo, pos2, vec2,
};

/// Icon tiles down the left edge; a clipped scroll area keeps overflow reachable.
pub(super) fn show(ui: &mut Ui, rect: Rect, items: &[RailItem], page: &mut usize, enabled: bool) {
    let count = items.iter().filter(|item| matches!(item, RailItem::Page { .. })).count();
    *page = (*page).min(count.saturating_sub(1));
    if !rect.is_positive() || count == 0 {
        return;
    }
    let group = ui.make_persistent_id("spaceui-rail");
    let mut child = ui.new_child(UiBuilder::new().max_rect(rect).id_salt("spaceui-rail-viewport"));
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    if !enabled {
        child.disable();
    }
    // Overlay the scrollbar in the rail's outer gutter, leaving tile centres
    // unchanged when overflow appears or disappears.
    child.spacing_mut().scroll.floating = true;
    child.spacing_mut().scroll.floating_allocated_width = 0.0;
    ScrollArea::vertical()
        .id_salt("spaceui-rail-scroll")
        .auto_shrink([false, false])
        .max_height(rect.height())
        .min_scrolled_height(0.0)
        .content_margin(0)
        .show(&mut child, |ui| render(ui, items, page, group, rect.width()));
}

fn render(ui: &mut Ui, items: &[RailItem], page: &mut usize, group: Id, width: f32) {
    let theme = Theme::of(ui.ctx());
    let tile_size = finite_size(theme.metrics.tile).max(1.0).min(width);
    let origin = ui.max_rect().min;
    let x = origin.x + width * 0.5;
    let mut height = 4.0;
    let offsets: Vec<f32> = items
        .iter()
        .filter_map(|item| match item {
            RailItem::Page { .. } => {
                let top = height;
                height += tile_size + 10.0;
                Some(top)
            }
            RailItem::Divider => {
                height += 12.0;
                None
            }
        })
        .collect();
    ui.set_min_size(vec2(width, height));
    let rect_at = |index: usize| {
        Rect::from_min_size(pos2(x - tile_size * 0.5, origin.y + offsets[index]), Vec2::splat(tile_size))
    };
    let selected_id = group.with("selected");
    let previous = ui.ctx().data(|data| data.get_temp::<usize>(selected_id));
    let mut y = origin.y + 4.0;
    let mut index = 0;
    let mut choices = Vec::with_capacity(offsets.len());
    for item in items {
        let RailItem::Page { icon, label } = *item else {
            let half_width = 16.0_f32.min(width * 0.5);
            ui.painter().hline((x - half_width)..=(x + half_width), y + 4.0, Stroke::new(1.0, theme.palette.border));
            y += 12.0;
            continue;
        };
        let tile = rect_at(index);
        let mut response = ui.interact(tile, group.with(index), Sense::click());
        let old = *page;
        if response.clicked() {
            *page = index;
        }
        if let Some(next) = keyboard_choice(ui, &response, index, offsets.len()) {
            *page = next;
            ui.memory_mut(|memory| memory.request_focus(group.with(next)));
            ui.scroll_to_rect(rect_at(next), None);
        }
        if old != *page {
            response.mark_changed();
        }
        choices.push((icon, label, tile, response));
        index += 1;
        y += tile_size + 10.0;
    }
    // Publish the final selection only after every tile has handled input.
    for (index, (icon, label, tile, response)) in choices.into_iter().enumerate() {
        let active = *page == index;
        response.widget_info(|| WidgetInfo::labeled(Role::Tab, ui.is_enabled(), label));
        ui.ctx().accesskit_node_builder(response.id, |node| node.set_selected(active));
        if response.gained_focus() || (active && previous != Some(*page)) {
            response.scroll_to_me(None);
        }
        paint_tile(ui, tile, icon, active, &response);
        response.on_hover_text(label);
    }
    ui.ctx().data_mut(|data| data.insert_temp(selected_id, *page));
}

fn keyboard_choice(ui: &Ui, response: &Response, index: usize, count: usize) -> Option<usize> {
    if !ui.is_enabled() || !response.has_focus() {
        return None;
    }
    ui.memory_mut(|memory| {
        memory.set_focus_lock_filter(response.id, EventFilter { vertical_arrows: true, ..Default::default() });
    });
    ui.input_mut(|input| {
        if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
            Some((index + 1) % count)
        } else if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
            Some(if index == 0 { count - 1 } else { index - 1 })
        } else if input.consume_key(Modifiers::NONE, Key::Home) {
            Some(0)
        } else if input.consume_key(Modifiers::NONE, Key::End) {
            Some(count - 1)
        } else {
            None
        }
    })
}

fn paint_tile(ui: &Ui, tile: Rect, icon: Icon, active: bool, response: &Response) {
    let theme = Theme::of(ui.ctx());
    let palette = &theme.palette;
    let hot = ui.is_enabled() && response.hovered();
    let painter = ui.painter();
    let radius = finite_size(theme.metrics.tile_radius).min(tile.width() * 0.5);
    if active {
        paint::glow(painter, tile, radius, palette.accent, 9.0);
        let top = paint::mix(palette.accent, palette.accent_hi, 0.4);
        paint::gradient(painter, tile, top, palette.accent_lo, radius);
        painter.rect_stroke(tile, radius, Stroke::new(1.0, Color32::from_white_alpha(46)), StrokeKind::Inside);
    } else if hot {
        painter.rect_filled(tile, radius, palette.field_hover);
    }
    let ink = match (active, hot) {
        (true, _) => palette.on_accent,
        (false, true) => palette.text,
        (false, false) => palette.text_2,
    };
    icon.paint(painter, Rect::from_center_size(tile.center(), tile.size() * 0.5), ink);
    if response.has_focus() {
        painter.rect_stroke(tile.expand(2.0), radius, Stroke::new(2.0, palette.accent_hi), StrokeKind::Outside);
    }
    if hot {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
}
