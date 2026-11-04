//! An illustrative sky field for the astronomy gallery, not an ephemeris.

use spaceui::egui::{
    Align2, CursorIcon, FontId, Painter, Pos2, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2, WidgetInfo,
    pos2, vec2,
};
use spaceui::{Palette, Theme, paint, theme::semibold};

const TARGETS: [(&str, [f32; 2]); 4] = [
    ("M31 · Andromeda", [0.25, 0.46]),
    ("M45 · Pleiades", [0.83, 0.78]),
    ("Double Cluster", [0.59, 0.31]),
    ("M33 · Triangulum", [0.39, 0.78]),
];

/// Select an example target by clicking its marker or activating it with the keyboard.
pub(super) fn show(ui: &mut Ui, selected: &mut usize, zoom: usize) {
    let palette = Theme::of(ui.ctx()).palette;
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width().max(1.0), 290.0), Sense::hover());
    let painter = ui.painter_at(rect);
    paint::gradient(&painter, rect, palette.well, palette.bg_bottom, 10.0);
    painter.rect_stroke(rect, 10.0, Stroke::new(1.0, palette.border_soft), StrokeKind::Inside);
    let field = rect.shrink2(vec2(34.0_f32.min(rect.width() * 0.1), 37.0));
    let scale = [0.84, 1.0, 1.2][zoom.min(2)];
    let project = |point: [f32; 2]| {
        field.center() + vec2((point[0] - 0.5) * field.width(), (point[1] - 0.5) * field.height()) * scale
    };
    stars(&painter.with_clip_rect(field), field, &palette);
    grid(&painter, field, &palette);
    constellations(&painter.with_clip_rect(field), project, &palette);

    *selected = (*selected).min(TARGETS.len() - 1);
    let responses: [Response; 4] = std::array::from_fn(|index| {
        let at = project(TARGETS[index].1);
        let hit = Rect::from_center_size(at, Vec2::splat(34.0)).intersect(field);
        let mut target = ui.interact(hit, response.id.with(index), Sense::click());
        if target.clicked() && *selected != index {
            *selected = index;
            target.mark_changed();
        }
        target
    });
    let field_painter = painter.with_clip_rect(field);
    for (index, target) in responses.into_iter().enumerate() {
        let (name, point) = TARGETS[index];
        let active = index == *selected;
        target.widget_info(|| WidgetInfo::selected(Role::RadioButton, ui.is_enabled(), active, name));
        marker(&field_painter, project(point), index, active || target.has_focus(), &palette);
        let color = if active { palette.text } else { palette.text_2 };
        let align = if index == 1 { Align2::RIGHT_TOP } else { Align2::LEFT_TOP };
        let offset = if index == 1 { vec2(-14.0, 15.0) } else { vec2(14.0, 15.0) };
        field_painter.text(project(point) + offset, align, name, semibold(11.5), color);
        target.on_hover_cursor(CursorIcon::PointingHand).on_hover_text(format!("Select {name}"));
    }
    painter.text(rect.left_top() + vec2(15.0, 13.0), Align2::LEFT_TOP, "NORTHERN SKY", semibold(10.5), palette.text_2);
    painter.text(
        rect.right_top() + vec2(-15.0, 13.0),
        Align2::RIGHT_TOP,
        "N",
        FontId::proportional(11.0),
        palette.accent_hi,
    );
    painter.arrow(rect.right_top() + vec2(-33.0, 24.0), vec2(0.0, -10.0), Stroke::new(1.0, palette.accent_hi));
    painter.text(
        rect.left_bottom() + vec2(15.0, -12.0),
        Align2::LEFT_BOTTOM,
        "Illustrative field · demo positions",
        FontId::proportional(10.5),
        palette.muted,
    );
}

fn stars(painter: &Painter, rect: Rect, palette: &Palette) {
    paint::radial_glow(painter, rect.center(), rect.width() * 0.45, palette.accent_lo, 12);
    let mut seed = 0x78AB_5623_u32;
    let mut next = || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) as f32 / 16_777_215.0
    };
    for _ in 0..150 {
        let at = pos2(rect.left() + next() * rect.width(), rect.top() + next() * rect.height());
        let radius = 0.45 + next().powi(5) * 1.1;
        let alpha = 0.15 + next() * 0.45;
        painter.circle_filled(at, radius, palette.text_2.gamma_multiply(alpha));
    }
}

fn grid(painter: &Painter, rect: Rect, palette: &Palette) {
    let stroke = Stroke::new(0.7, palette.border_soft.gamma_multiply(0.65));
    for column in 0..=8 {
        let x = rect.left() + column as f32 * rect.width() / 8.0;
        painter.vline(x, rect.y_range(), stroke);
    }
    for row in 0..=4 {
        let y = rect.top() + row as f32 * rect.height() / 4.0;
        painter.hline(rect.x_range(), y, stroke);
    }
    for (fraction, label, align, offset) in [
        (0.0, "+60°", Align2::LEFT_TOP, 4.0),
        (0.5, "+40°", Align2::LEFT_BOTTOM, -3.0),
        (1.0, "+20°", Align2::LEFT_BOTTOM, -3.0),
    ] {
        painter.text(
            pos2(rect.left() + 4.0, rect.top() + fraction * rect.height() + offset),
            align,
            label,
            FontId::monospace(9.0),
            palette.muted,
        );
    }
    for (fraction, label) in [(0.2, "00h"), (0.5, "02h"), (0.8, "04h")] {
        painter.text(
            pos2(rect.left() + fraction * rect.width(), rect.bottom() + 6.0),
            Align2::CENTER_TOP,
            label,
            FontId::monospace(9.0),
            palette.muted,
        );
    }
}

fn constellations(painter: &Painter, project: impl Fn([f32; 2]) -> Pos2, palette: &Palette) {
    type Outline<'a> = (&'a str, [f32; 2], &'a [[f32; 2]]);
    let lines: &[Outline<'_>] = &[
        ("ANDROMEDA", [0.16, 0.28], &[[0.08, 0.66], [0.22, 0.55], [0.34, 0.40], [0.41, 0.46]]),
        ("CASSIOPEIA", [0.41, 0.03], &[[0.36, 0.16], [0.41, 0.23], [0.47, 0.11], [0.52, 0.20], [0.57, 0.12]]),
        ("PERSEUS", [0.72, 0.32], &[[0.64, 0.32], [0.68, 0.47], [0.75, 0.54], [0.70, 0.68], [0.62, 0.61]]),
    ];
    for (name, label, points) in lines {
        for [from, to] in points.array_windows::<2>() {
            painter
                .line_segment([project(*from), project(*to)], Stroke::new(1.0, palette.accent_hi.gamma_multiply(0.25)));
        }
        for point in *points {
            painter.circle_filled(project(*point), 2.0, palette.text_2.gamma_multiply(0.8));
        }
        painter.text(project(*label), Align2::CENTER_CENTER, name, FontId::proportional(9.5), palette.muted);
    }
}

fn marker(painter: &Painter, at: Pos2, index: usize, active: bool, palette: &Palette) {
    let ink = if active { palette.accent_hi } else { palette.info };
    if active {
        paint::radial_glow(painter, at, 39.0, palette.accent, 32);
        painter.circle_stroke(at, 17.0, Stroke::new(1.0, ink));
        painter.circle_stroke(at, 23.0, Stroke::new(0.7, ink.gamma_multiply(0.3)));
        for offset in [vec2(0.0, -1.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(-1.0, 0.0)] {
            painter.line_segment([at + offset * 25.0, at + offset * 30.0], Stroke::new(1.0, ink));
        }
    }
    if index == 0 || index == 3 {
        for layer in (1..=4).rev() {
            let radius = vec2(3.8, 1.5) * layer as f32;
            let points: Vec<_> = (0..=32)
                .map(|step| {
                    let angle = step as f32 / 32.0 * std::f32::consts::TAU;
                    at + tilt(vec2(angle.cos() * radius.x, angle.sin() * radius.y))
                })
                .collect();
            painter.add(spaceui::egui::Shape::closed_line(
                points,
                Stroke::new(1.0, ink.gamma_multiply(0.6 / layer as f32)),
            ));
        }
        painter.circle_filled(at, 2.5, palette.text);
    } else {
        for (offset, radius) in [
            (vec2(-7.0, 2.0), 2.0),
            (vec2(2.0, -5.0), 2.6),
            (vec2(7.0, 4.0), 1.7),
            (vec2(-2.0, 6.0), 1.3),
            (vec2(-4.0, -5.0), 1.0),
        ] {
            painter.circle_filled(at + offset, radius, ink);
        }
    }
}

fn tilt(point: Vec2) -> Vec2 {
    vec2(point.x * 0.866 - point.y * 0.5, point.x * 0.5 + point.y * 0.866)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::kittest::{NodeT, Queryable};
    use spaceui::egui::{Event, Key, Modifiers, accesskit::Toggled};

    #[test]
    fn sky_targets_remain_accessible_and_select_in_the_input_frame_at_every_zoom() {
        for width in [360.0, 800.0] {
            for zoom in 0..3 {
                let viewport = Rect::from_min_size(Pos2::ZERO, vec2(width, 330.0));
                let mut harness = egui_kittest::Harness::builder()
                    .with_size(viewport.size())
                    .with_context_setup(|ctx| {
                        Theme { fonts: spaceui::Fonts::BUNDLED, ..Theme::deep_space() }.install(ctx);
                    })
                    .build_ui_state(move |ui, selected| show(ui, selected, zoom), 0_usize);
                for (label, _) in TARGETS {
                    let target = harness.get_by_label(label);
                    let bounds = target.rect();
                    assert!(bounds.is_finite() && bounds.is_positive() && viewport.contains_rect(bounds));
                    assert_eq!(target.accesskit_node().role(), Role::RadioButton);
                }
                for (label, expected, keyboard) in [("M45 · Pleiades", 1, false), ("Double Cluster", 2, true)] {
                    if keyboard {
                        harness.get_by_label(label).focus();
                        harness.step();
                        harness.event(Event::Key {
                            key: Key::Enter,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: Modifiers::NONE,
                        });
                    } else {
                        harness.get_by_label(label).click();
                    }
                    // Observe the input frame, without an idle frame hiding stale selection metadata.
                    harness.step();
                    assert_eq!(*harness.state(), expected);
                    for (index, (name, _)) in TARGETS.into_iter().enumerate() {
                        let selected = harness.get_by_label(name).accesskit_node().toggled() == Some(Toggled::True);
                        assert_eq!(selected, index == expected, "{name} at zoom {zoom}");
                    }
                }
            }
        }
    }
}
