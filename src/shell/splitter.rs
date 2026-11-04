use crate::Theme;
use egui::accesskit::{Action, ActionData, Orientation};
use egui::{CursorIcon, EventFilter, Key, Modifiers, Rect, Response, Role, Sense, Stroke, Ui, WidgetInfo, vec2};
use std::ops::RangeInclusive;

#[derive(Clone, Copy)]
pub(super) enum Side {
    Left,
    Right,
}

impl Side {
    fn id(self) -> &'static str {
        match self {
            Self::Left => "spaceui-splitter-left",
            Self::Right => "spaceui-splitter-right",
        }
    }

    fn direction(self) -> f32 {
        match self {
            Self::Left => 1.0,
            Self::Right => -1.0,
        }
    }
}

pub(super) fn show(
    ui: &Ui,
    rect: Rect,
    width: &mut f32,
    range: RangeInclusive<f32>,
    side: Side,
    label: &str,
    enabled: bool,
) {
    if !rect.is_positive() {
        return;
    }
    let (min, max) = (*range.start(), *range.end());
    let editable = enabled && ui.is_enabled() && min < max;
    let sense = if editable { Sense::click_and_drag() } else { Sense::hover() };
    let mut response = ui.interact(rect, ui.make_persistent_id(side.id()), sense);
    let old = *width;
    if editable {
        if response.dragged() {
            *width += side.direction() * response.drag_delta().x;
            response.request_focus();
        }
        adjust(ui, &response, width, &range, side);
        *width = width.clamp(min, max);
    }
    if old.to_bits() != width.to_bits() {
        response.mark_changed();
    }
    // Interact at the previous position, then publish and paint the handle
    // beside the column's updated edge in this same frame.
    let rect = rect.translate(vec2(side.direction() * (*width - old), 0.0));
    response.rect = rect;
    response.interact_rect = rect.intersect(ui.clip_rect());
    response.widget_info(|| {
        let mut info = WidgetInfo::labeled(Role::Splitter, editable, label);
        info.value = Some(f64::from(*width));
        info
    });
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_orientation(Orientation::Vertical);
        node.set_min_numeric_value(f64::from(min));
        node.set_max_numeric_value(f64::from(max));
        node.set_numeric_value_step(10.0);
        if editable {
            node.add_action(Action::SetValue);
            if *width < max {
                node.add_action(Action::Increment);
            }
            if *width > min {
                node.add_action(Action::Decrement);
            }
        }
    });
    if editable && (response.hovered() || response.dragged() || response.has_focus()) {
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
        }
        let accent = Theme::of(ui.ctx()).palette.accent_hi;
        let ink = if response.has_focus() { accent } else { accent.gamma_multiply(0.5) };
        ui.painter().vline(
            rect.center().x,
            rect.y_range().shrink(40.0_f32.min(rect.height() * 0.25)),
            Stroke::new(2.0, ink),
        );
    }
    response.on_hover_text(label);
}

fn adjust(ui: &Ui, response: &Response, width: &mut f32, range: &RangeInclusive<f32>, side: Side) {
    if response.has_focus() {
        ui.memory_mut(|memory| {
            memory.set_focus_lock_filter(response.id, EventFilter { horizontal_arrows: true, ..Default::default() });
        });
        ui.input_mut(|input| {
            let right = input.consume_key(Modifiers::NONE, Key::ArrowRight);
            let left = input.consume_key(Modifiers::NONE, Key::ArrowLeft);
            *width += side.direction() * (f32::from(right) - f32::from(left)) * 10.0;
            if input.consume_key(Modifiers::NONE, Key::Home) {
                *width = *range.start();
            }
            if input.consume_key(Modifiers::NONE, Key::End) {
                *width = *range.end();
            }
        });
    }
    ui.input(|input| {
        let increase = input.num_accesskit_action_requests(response.id, Action::Increment);
        let decrease = input.num_accesskit_action_requests(response.id, Action::Decrement);
        *width += (increase as f32 - decrease as f32) * 10.0;
        for request in input.accesskit_action_requests(response.id, Action::SetValue) {
            if let Some(ActionData::NumericValue(value)) = request.data
                && value.is_finite()
            {
                *width = value.clamp(f64::from(*range.start()), f64::from(*range.end())) as f32;
            }
        }
    });
}
