//! The window: a custom title bar, an icon rail, a carded main panel and a
//! resizable side column.
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────┐
//! │ ◉ Title                         status…    ─  ☐  ✕       │  title bar (own, draggable)
//! ├────┬─────────────────────────────────────┬───────────────┤
//! │ ▣  │                                     │               │
//! │ ◇  │            main card                │  side column  │
//! │ ◇  │                                     │  (e.g. chat)  │
//! │ ── │                                     │               │
//! │ ⚙  │                                     │               │
//! └────┴─────────────────────────────────────┴───────────────┘
//!  rail                                  splitter
//! ```
//!
//! [`Shell::show`] paints the backdrop, title bar, rail and card plates,
//! handles window dragging, resizing, the splitter and the rail selection,
//! and returns the [`Regions`] the app fills. Use it with an undecorated
//! viewport ([`window::viewport`](crate::window::viewport)).
//!
//! The title bar lives on egui's tooltip layer, above modal dialogs, so the
//! window can always be moved, maximized and closed, even while a modal
//! blocks the workspace.
mod rail;
mod title_bar;
mod window_edges;

use crate::{Icon, Svg, Theme, theme::paint_backdrop, widgets};
use egui::{CursorIcon, Rect, Sense, Stroke, Ui, pos2, vec2};
use std::{fmt, ops::RangeInclusive};

/// An entry in the icon rail.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RailItem {
    /// A selectable page.
    Page {
        /// The tile's icon.
        icon: Icon,
        /// The tooltip.
        label: &'static str,
    },
    /// A short rule separating groups (Settings from the pages, say).
    Divider,
}

impl RailItem {
    /// A page with its icon and tooltip label.
    #[must_use]
    pub const fn page(icon: Icon, label: &'static str) -> Self {
        Self::Page { icon, label }
    }
}

/// Where the app draws, as returned by [`Shell::show`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Regions {
    /// The main card's content area, inside its padding.
    pub main: Rect,
    /// The side column's whole card (a chat column draws its own padding).
    pub side: Option<Rect>,
    /// The status line under the cards.
    pub status: Option<Rect>,
}

/// Content drawn into the title bar's status area.
type TitleStatus<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

/// Builder for the window shell. Build and show it from the root `Ui` every
/// frame.
///
/// ```
/// use spaceui::{Icon, RailItem, Shell, Theme, egui, widgets, widgets::Tone};
///
/// const RAIL: &[RailItem] = &[RailItem::page(Icon::Home, "Home"), RailItem::page(Icon::Settings, "Settings")];
///
/// # let ctx = egui::Context::default();
/// # Theme::carbon_copper().install(&ctx);
/// # let (mut page, mut side) = (0, 380.0);
/// # ctx.run_ui(Default::default(), |ui| {
/// let regions = Shell::new("My App")
///     .rail(RAIL)
///     .side_column(&mut side, 320.0..=560.0)
///     .title_status(|ui| {
///         widgets::pill(ui, Some(Tone::Positive), "Connected");
///     })
///     .show(ui, &mut page);
/// assert!(regions.side.is_some());
/// # }).drop_without_applying_deltas();
/// ```
#[must_use = "a shell does nothing until `show` is called"]
pub struct Shell<'a> {
    title: &'a str,
    logo: Option<Svg>,
    rail: &'a [RailItem],
    side: Option<(&'a mut f32, RangeInclusive<f32>)>,
    status: f32,
    enabled: bool,
    title_status: Option<TitleStatus<'a>>,
}

impl fmt::Debug for Shell<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Shell")
            .field("title", &self.title)
            .field("logo", &self.logo.map(|svg| svg.name))
            .field("rail", &self.rail)
            .field("side", &self.side)
            .field("status", &self.status)
            .field("enabled", &self.enabled)
            .field("title_status", &self.title_status.is_some())
            .finish()
    }
}

impl<'a> Shell<'a> {
    /// A shell titled `title`, with no logo, rail, side column or status line yet.
    pub fn new(title: &'a str) -> Self {
        Self { title, logo: None, rail: &[], side: None, status: 0.0, enabled: true, title_status: None }
    }

    /// The app's logo, beside the title.
    pub fn logo(mut self, logo: Svg) -> Self {
        self.logo = Some(logo);
        self
    }

    /// The rail's pages and dividers. [`show`](Self::show) takes the
    /// selected page as an index counting pages only.
    pub fn rail(mut self, items: &'a [RailItem]) -> Self {
        self.rail = items;
        self
    }

    /// A column on the right, resizable within `range` (and at most half the
    /// window). The app keeps, and may persist, its width.
    pub fn side_column(mut self, width: &'a mut f32, range: RangeInclusive<f32>) -> Self {
        self.side = Some((width, range));
        self
    }

    /// A status line `height` points tall under the cards.
    pub fn status_bar(mut self, height: f32) -> Self {
        self.status = height;
        self
    }

    /// `false` makes the rail inert, as under a blocking modal. The title bar
    /// stays usable either way, so the window can always be moved and closed.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Right-aligned content in the title bar (status pills, account menus),
    /// laid out right to left.
    pub fn title_status(mut self, add: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.title_status = Some(Box::new(add));
        self
    }

    /// Paints the window and handles its interaction. `page` is the selected
    /// rail page, updated when the owner clicks another one.
    pub fn show(self, ui: &mut Ui, page: &mut usize) -> Regions {
        let ctx = ui.ctx().clone();
        let theme = Theme::of(&ctx);
        let metrics = theme.metrics;
        paint_backdrop(&ctx);
        let window = ctx.content_rect();
        title_bar::show(&ctx, window, self.title, self.logo, self.title_status);

        let top = window.top() + metrics.title_bar;
        let bottom = window.bottom() - metrics.gap - self.status.max(metrics.gap * 0.5);
        let rail =
            Rect::from_min_max(pos2(window.left() + metrics.gap, top), pos2(window.left() + metrics.rail, bottom));
        rail::show(ui, rail, self.rail, page, self.enabled);

        let right = window.right() - metrics.gap * 1.5;
        let (main, side) = if let Some((width, range)) = self.side {
            let max = (window.width() * 0.5).max(*range.start()).min(*range.end());
            *width = width.clamp(*range.start(), max);
            let side = Rect::from_min_max(pos2(right - *width, top), pos2(right, bottom));
            let main =
                Rect::from_min_max(pos2(rail.right() + 6.0, top), pos2(side.left() - metrics.gap * 1.75, bottom));
            let handle = Rect::from_min_max(pos2(main.right(), top + 20.0), pos2(side.left(), bottom - 20.0));
            splitter(ui, handle, width, *range.start(), max);
            (main, Some(side))
        } else {
            (Rect::from_min_max(pos2(rail.right() + 6.0, top), pos2(right, bottom)), None)
        };
        widgets::plate(ui.painter(), main);
        if let Some(side) = side {
            widgets::plate(ui.painter(), side);
        }
        window_edges::show(&ctx);
        Regions {
            main: main.shrink2(vec2(18.0, 14.0)),
            side,
            status: (self.status > 0.0).then(|| {
                Rect::from_min_max(pos2(rail.left() + 12.0, bottom + 2.0), pos2(right - 6.0, window.bottom() - 2.0))
            }),
        }
    }
}

/// The drag handle between the main card and the side column.
fn splitter(ui: &mut Ui, rect: Rect, width: &mut f32, min: f32, max: f32) {
    let response = ui.interact(rect, ui.scope_id().with("spaceui-splitter"), Sense::drag());
    if response.hovered() || response.dragged() {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
        let accent = Theme::of(ui.ctx()).palette.accent;
        ui.painter().vline(rect.center().x, rect.y_range().shrink(40.0), Stroke::new(2.0, accent.gamma_multiply(0.5)));
    }
    if response.dragged() {
        *width = (*width - response.drag_delta().x).clamp(min, max);
    }
}
