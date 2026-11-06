//! Colours, sizes, fonts and backdrops: everything that makes a look.
//!
//! A [`Theme`] bundles a [`Palette`] (semantic colours), [`Metrics`] (sizes
//! and radii), [`Fonts`] and a [`Backdrop`]. Install one per context with
//! [`Theme::install`]; widgets read it back with [`Theme::of`].
//!
//! ```
//! use spaceui::{Theme, egui};
//!
//! let ctx = egui::Context::default();
//! Theme::deep_space().install(&ctx);
//! assert_eq!(Theme::of(&ctx).name, "Deep Space");
//! ```
mod backdrop;
mod controls;
mod fonts;
mod palette;

pub use backdrop::paint_backdrop;
pub use controls::{ControlMetrics, ControlSize};
pub(crate) use controls::{normalize, normalize_vec};
pub use fonts::{FontFile, Fonts, SEMIBOLD, mono, regular, semibold};
pub use palette::Palette;

use egui::{Color32, Context, CornerRadius, Id, Stroke, TextStyle, Visuals, epaint::Shadow, vec2};
use std::sync::Arc;

#[derive(Clone, Copy, Default, PartialEq)]
struct MotionStyle {
    animation_time: f32,
    scroll_animation: egui::style::ScrollAnimation,
}

/// Disable decorative motion and egui transitions in this context.
/// Call after installing a theme. Disabling the preference restores the
/// transition and programmatic-scroll settings active before it was enabled.
/// Repeated calls preserve the original settings for both light and dark mode and
/// request a repaint only when the motion settings change. Applications
/// can connect this to an OS preference or an in-app accessibility setting.
pub fn set_reduced_motion(ctx: &Context, reduced: bool) {
    let id = Id::unique("spaceui-animation-duration");
    let modes = [egui::Theme::Dark, egui::Theme::Light];
    let mut changed = false;
    if reduced {
        let current = modes.map(|mode| {
            let style = ctx.style_of(mode);
            MotionStyle { animation_time: style.animation_time, scroll_animation: style.scroll_animation }
        });
        ctx.data_mut(|d| {
            if d.get_temp::<[MotionStyle; 2]>(id).is_none() {
                d.insert_temp(id, current);
            }
        });
        let disabled = MotionStyle { animation_time: 0.0, scroll_animation: egui::style::ScrollAnimation::none() };
        changed = current.iter().any(|style| *style != disabled);
        if changed {
            ctx.all_styles_mut(|style| {
                style.animation_time = disabled.animation_time;
                style.scroll_animation = disabled.scroll_animation;
            });
        }
    } else if let Some(settings) = ctx.data_mut(|d| d.remove_temp::<[MotionStyle; 2]>(id)) {
        for (mode, saved) in modes.into_iter().zip(settings) {
            ctx.style_mut_of(mode, |style| {
                let current =
                    MotionStyle { animation_time: style.animation_time, scroll_animation: style.scroll_animation };
                changed |= current != saved;
                style.animation_time = saved.animation_time;
                style.scroll_animation = saved.scroll_animation;
            });
        }
    }
    if changed {
        ctx.request_repaint();
    }
}

/// Sizes the shell and widgets share, in points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    /// Height of the app's own title bar.
    pub title_bar: f32,
    /// Width of the icon rail on the left.
    pub rail: f32,
    /// Side of a rail tile.
    pub tile: f32,
    /// Space between the window edge, the rail and the cards.
    pub gap: f32,
    /// Corner radius of the main card and side column.
    pub card_radius: f32,
    /// Corner radius of rail tiles.
    pub tile_radius: f32,
    /// Corner radius of buttons and fields.
    pub control_radius: f32,
    /// Corner radius of pills and badges.
    pub pill_radius: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            title_bar: 46.0,
            rail: 74.0,
            tile: 48.0,
            gap: 8.0,
            card_radius: 16.0,
            tile_radius: 14.0,
            control_radius: 8.0,
            pill_radius: 14.0,
        }
    }
}

impl Metrics {
    /// Corner radius of cards nested inside the main card.
    #[must_use]
    pub fn inner_radius(&self) -> f32 {
        (self.card_radius - 4.0).max(0.0)
    }
}

/// What the window shows behind the cards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Backdrop {
    /// The palette's gradient alone.
    Plain,
    /// A faint 2x2 twill of carbon tows and an accent bloom in the upper left.
    Carbon {
        /// Opacity of the weave, 0-255.
        weave: u8,
        /// Opacity of the accent bloom, 0-255.
        bloom: u8,
    },
    /// Nebula glows and a fixed starfield.
    Starfield,
}

/// Everything that defines a look. A theme is plain data: start from a
/// built-in one and change what you need.
///
/// ```
/// use spaceui::egui::Color32;
/// use spaceui::{Backdrop, Palette, Theme};
///
/// let lunar = Theme {
///     name: "Lunar",
///     palette: Palette {
///         accent: Color32::from_rgb(125, 211, 252),
///         accent_hi: Color32::from_rgb(186, 230, 253),
///         accent_lo: Color32::from_rgb(82, 160, 220),
///         ..Palette::DEEP_SPACE
///     },
///     backdrop: Backdrop::Plain,
///     ..Theme::deep_space()
/// };
/// assert_eq!(lunar.metrics, Theme::deep_space().metrics);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    /// Shown in theme pickers.
    pub name: &'static str,
    /// The colours.
    pub palette: Palette,
    /// The sizes.
    pub metrics: Metrics,
    /// The text fonts.
    pub fonts: Fonts,
    /// What shows behind the cards.
    pub backdrop: Backdrop,
}

impl Default for Theme {
    fn default() -> Self {
        Self::carbon_copper()
    }
}

impl Theme {
    /// Near-black carbon with a faint weave and copper accents.
    #[must_use]
    pub fn carbon_copper() -> Self {
        Self {
            name: "Carbon / Copper",
            palette: Palette::CARBON_COPPER,
            metrics: Metrics::default(),
            fonts: Fonts::SYSTEM,
            backdrop: Backdrop::Carbon { weave: 7, bloom: 14 },
        }
    }

    /// Deep-space indigo with violet accents and a starfield.
    #[must_use]
    pub fn deep_space() -> Self {
        Self {
            name: "Deep Space",
            palette: Palette::DEEP_SPACE,
            metrics: Metrics { title_bar: 48.0, rail: 72.0, tile: 50.0, card_radius: 18.0, ..Metrics::default() },
            fonts: Fonts::SYSTEM,
            backdrop: Backdrop::Starfield,
        }
    }

    fn id() -> Id {
        Id::unique("spaceui-theme")
    }

    /// Makes this the context's theme: stores it for [`Theme::of`], sets
    /// egui's visuals and spacing to match, and installs its fonts (only when
    /// they differ from the installed ones). Takes effect from the next frame.
    ///
    /// # Panics
    ///
    /// egui parses fonts lazily. Invalid TTF/OTF bytes in a [`FontFile`] can
    /// cause a panic when text is first rendered in a later frame.
    pub fn install(self, ctx: &Context) {
        self.install_with_mode(ctx, egui::Theme::Dark);
    }

    /// Install a custom palette with matching egui light/dark defaults.
    /// Built-in palettes are dark; a light palette should use `Theme::Light`.
    ///
    /// # Panics
    ///
    /// egui parses fonts lazily. Invalid TTF/OTF bytes in a [`FontFile`] can
    /// cause a panic when text is first rendered in a later frame.
    pub fn install_with_mode(self, ctx: &Context, mode: egui::Theme) {
        fonts::install(ctx, &self.fonts);
        ctx.set_theme(match mode {
            egui::Theme::Dark => egui::ThemePreference::Dark,
            egui::Theme::Light => egui::ThemePreference::Light,
        });
        ctx.set_visuals_of(mode, self.visuals_with_mode(mode));
        ctx.global_style_mut(|style| {
            style.spacing.item_spacing = vec2(10.0, 8.0);
            style.spacing.button_padding = vec2(12.0, 6.0);
            style.spacing.interact_size = vec2(40.0, 28.0);
            style.spacing.icon_width = 16.0;
            style.spacing.icon_width_inner = 9.0;
            style.spacing.scroll = egui::style::ScrollStyle::thin();
            style.interaction.selectable_labels = false;
            style.text_styles.insert(TextStyle::Body, regular(14.0));
            style.text_styles.insert(TextStyle::Button, regular(13.5));
            style.text_styles.insert(TextStyle::Small, regular(11.5));
            style.text_styles.insert(TextStyle::Heading, semibold(24.0));
            style.text_styles.insert(TextStyle::Monospace, mono(13.0));
        });
        ctx.data_mut(|d| d.insert_temp(Self::id(), Arc::new(self)));
    }

    /// The installed theme, or a cached [`Theme::default`] if none was installed.
    /// Cheap after the first lookup: it clones an `Arc`. This lookup does not
    /// install fonts or change egui's styles; call [`Theme::install`] for that.
    #[must_use]
    pub fn of(ctx: &Context) -> Arc<Theme> {
        ctx.data_mut(|d| Arc::clone(d.get_temp_mut_or_default::<Arc<Theme>>(Self::id())))
    }

    /// egui's own widgets (labels, text edits, combo boxes, windows) in this
    /// palette. [`Theme::install`] applies it for you.
    #[must_use]
    pub fn visuals(&self) -> Visuals {
        self.visuals_with_mode(egui::Theme::Dark)
    }

    /// Build egui visuals using light or dark base defaults and this palette.
    #[must_use]
    pub fn visuals_with_mode(&self, mode: egui::Theme) -> Visuals {
        let p = &self.palette;
        let mut v = match mode {
            egui::Theme::Dark => Visuals::dark(),
            egui::Theme::Light => Visuals::light(),
        };
        v.override_text_color = Some(p.text);
        v.panel_fill = Color32::TRANSPARENT;
        v.window_fill = p.panel_hi;
        v.extreme_bg_color = p.well;
        v.faint_bg_color = Color32::from_white_alpha(5);
        v.code_bg_color = p.field;
        v.window_stroke = Stroke::new(1.0, p.border);
        v.window_corner_radius = CornerRadius::same(14);
        v.menu_corner_radius = CornerRadius::same(10);
        v.popup_shadow = Shadow { offset: [0, 10], blur: 28, spread: 0, color: Color32::from_black_alpha(170) };
        v.window_shadow = v.popup_shadow;
        v.selection.bg_fill = p.accent_lo.gamma_multiply(0.75);
        v.selection.stroke = Stroke::new(1.0, p.accent_hi);
        v.hyperlink_color = p.accent_hi;
        v.warn_fg_color = p.warning;
        v.error_fg_color = p.negative;
        v.text_cursor.stroke = Stroke::new(2.0, p.text);
        let w = &mut v.widgets;
        for (state, fill) in [
            (&mut w.noninteractive, p.panel_hi),
            (&mut w.inactive, p.field),
            (&mut w.hovered, p.field_hover),
            (&mut w.active, p.accent_lo),
            (&mut w.open, p.field_hover),
        ] {
            state.bg_fill = fill;
            state.weak_bg_fill = fill;
            state.corner_radius = CornerRadius::same(6);
            state.fg_stroke = Stroke::new(1.0, p.text);
            state.bg_stroke = Stroke::new(1.0, p.border_soft);
        }
        w.noninteractive.fg_stroke = Stroke::new(1.0, p.text_2);
        w.inactive.bg_stroke = Stroke::new(1.0, p.border);
        w.hovered.bg_stroke = Stroke::new(1.0, p.accent.gamma_multiply(0.55));
        w.active.bg_stroke = Stroke::new(1.0, p.accent_hi);
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeating_motion_preferences_does_not_keep_requesting_frames() {
        let ctx = Context::default();
        for reduced in [false, true, false] {
            for _ in 0..4 {
                ctx.run_ui(egui::RawInput::default(), |_| set_reduced_motion(&ctx, reduced))
                    .drop_without_applying_deltas();
            }
            let output = ctx.run_ui(egui::RawInput::default(), |_| set_reduced_motion(&ctx, reduced));
            assert_ne!(output.viewport_output[&egui::ViewportId::ROOT].repaint_delay, std::time::Duration::ZERO);
            output.drop_without_applying_deltas();
        }
        set_reduced_motion(&ctx, true);
        assert!(ctx.has_requested_repaint(), "a changed preference still wakes the UI");
    }

    #[test]
    fn reduced_motion_restores_the_callers_duration() {
        let ctx = Context::default();
        let dark_scroll = egui::style::ScrollAnimation::duration(0.43);
        let light_scroll = egui::style::ScrollAnimation::duration(0.79);
        ctx.style_mut_of(egui::Theme::Dark, |style| {
            style.animation_time = 0.37;
            style.scroll_animation = dark_scroll;
        });
        ctx.style_mut_of(egui::Theme::Light, |style| style.scroll_animation = light_scroll);
        set_reduced_motion(&ctx, true);
        set_reduced_motion(&ctx, true);
        for mode in [egui::Theme::Dark, egui::Theme::Light] {
            let style = ctx.style_of(mode);
            assert!(style.animation_time.abs() < f32::EPSILON);
            assert_eq!(style.scroll_animation, egui::style::ScrollAnimation::none());
        }
        Theme::default().install_with_mode(&ctx, egui::Theme::Light);
        assert!(ctx.global_style().animation_time.abs() < f32::EPSILON);
        assert_eq!(ctx.global_style().scroll_animation, egui::style::ScrollAnimation::none());
        set_reduced_motion(&ctx, false);
        assert!((ctx.style_of(egui::Theme::Dark).animation_time - 0.37).abs() < f32::EPSILON);
        assert!(ctx.style_of(egui::Theme::Light).animation_time > 0.0);
        assert_eq!(ctx.style_of(egui::Theme::Dark).scroll_animation, dark_scroll);
        assert_eq!(ctx.style_of(egui::Theme::Light).scroll_animation, light_scroll);
        set_reduced_motion(&ctx, false);
        assert_eq!(ctx.style_of(egui::Theme::Light).scroll_animation, light_scroll);
    }

    #[test]
    fn a_custom_light_palette_can_use_light_egui_defaults() {
        let ctx = Context::default();
        Theme::default().install_with_mode(&ctx, egui::Theme::Light);
        assert!(!ctx.style_of(ctx.theme()).visuals.dark_mode);
    }

    #[test]
    fn installing_a_theme_replaces_the_one_widgets_read() {
        let ctx = Context::default();
        assert_eq!(Theme::of(&ctx).name, Theme::default().name);
        assert!(Arc::ptr_eq(&Theme::of(&ctx), &Theme::of(&ctx)));
        Theme::deep_space().install(&ctx);
        assert_eq!(Theme::of(&ctx).palette, Palette::DEEP_SPACE);
        Theme::carbon_copper().install(&ctx);
        assert_eq!(Theme::of(&ctx).palette, Palette::CARBON_COPPER);
        assert_eq!(ctx.style_of(egui::Theme::Dark).visuals.hyperlink_color, Palette::CARBON_COPPER.accent_hi);
    }

    #[test]
    fn built_in_palettes_keep_body_text_readable() {
        // WCAG relative luminance contrast; 7:1 is the AAA level for body text.
        fn luminance(c: Color32) -> f32 {
            let lin = |v: u8| {
                let v = f32::from(v) / 255.0;
                if v <= 0.040_45 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
            };
            0.2126 * lin(c.r()) + 0.7152 * lin(c.g()) + 0.0722 * lin(c.b())
        }
        let contrast = |a, b| {
            let (a, b) = (luminance(a), luminance(b));
            (a.max(b) + 0.05) / (a.min(b) + 0.05)
        };
        for p in [Palette::CARBON_COPPER, Palette::DEEP_SPACE] {
            assert!(contrast(p.text, p.panel) >= 7.0);
            assert!(contrast(p.text_2, p.panel) >= 4.5);
            for step in 0..=100 {
                let fill = crate::paint::mix(p.accent_lo, p.accent_hi, step as f32 / 100.0);
                assert!(contrast(p.on_accent, fill) >= 4.5, "accent text at gradient step {step}");
            }
        }
    }
}
