//! The window: a custom title bar, an optional icon rail, a carded main panel
//! and resizable columns on either side.
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────────┐
//! │ ◉ Title                               status…    ─  ☐  ✕       │  title bar (own, draggable)
//! ├────┬────────────┬────────────────────────────┬─────────────────┤
//! │ ▣  │            │                            │                 │
//! │ ◇  │    left    │         main card          │      right      │
//! │ ◇  │   column   │                            │     column      │
//! │ ── │            │                            │   (e.g. chat)   │
//! │ ⚙  │            │                            │                 │
//! └────┴────────────┴────────────────────────────┴─────────────────┘
//!  rail          splitter                    splitter
//! ```
//!
//! Only the title bar and the main card are always there: add a rail, a
//! left column, a right column and a status line as you need them.
//! [`Shell::show`] paints the backdrop, title bar, rail and card plates,
//! handles window dragging and resizing, the splitters and the rail
//! selection, and returns the [`Regions`] the app fills. Use it with an
//! undecorated viewport ([`window::viewport`](crate::window::viewport)).
//!
//! The title bar lives on egui's tooltip layer, above modal dialogs, so the
//! window can always be moved, maximized and closed, even while a modal
//! blocks the workspace.
mod rail;
mod splitter;
mod title_bar;
mod window_edges;

use crate::{Icon, Svg, Theme, theme::paint_backdrop, widgets};
use egui::{Rect, Ui, pos2, vec2};
use std::{fmt, ops::RangeInclusive};

/// Translatable labels for window controls and column dividers. The strings
/// are borrowed for the current frame and may come from a locale catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowLabels<'a> {
    /// Minimize action.
    pub minimize: &'a str,
    /// Maximize action.
    pub maximize: &'a str,
    /// Restore a maximized window.
    pub restore: &'a str,
    /// Close action.
    pub close: &'a str,
    /// The draggable title region.
    pub move_window: &'a str,
    /// Left column's resizable width.
    pub left_column_width: &'a str,
    /// Right column's resizable width.
    pub right_column_width: &'a str,
    /// Pointer resize regions, in northwest, northeast, southwest, southeast,
    /// north, south, west, east order.
    pub resize: [&'a str; 8],
}

impl Default for WindowLabels<'_> {
    fn default() -> Self {
        Self {
            minimize: "Minimize window",
            maximize: "Maximize window",
            restore: "Restore window",
            close: "Close window",
            move_window: "Move window",
            left_column_width: "Left column width",
            right_column_width: "Right column width",
            resize: [
                "Resize top left",
                "Resize top right",
                "Resize bottom left",
                "Resize bottom right",
                "Resize top",
                "Resize bottom",
                "Resize left",
                "Resize right",
            ],
        }
    }
}

/// An entry in the icon rail. Labels may borrow a runtime localization catalog
/// or application-owned strings for the frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RailItem<'a> {
    /// A selectable page.
    Page {
        /// The tile's icon.
        icon: Icon,
        /// The tooltip and accessible page name.
        label: &'a str,
    },
    /// A short rule separating groups (Settings from the pages, say).
    Divider,
}

impl<'a> RailItem<'a> {
    /// A page with its icon and tooltip label.
    #[must_use]
    pub const fn page(icon: Icon, label: &'a str) -> Self {
        Self::Page { icon, label }
    }
}

/// Where the app draws, as returned by [`Shell::show`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Regions {
    /// The main card's content area, inside its padding.
    pub main: Rect,
    /// The left column's whole card, if the shell has one. Columns are
    /// returned unpadded, since a chat column draws its own padding.
    pub left: Option<Rect>,
    /// The right column's whole card, if the shell has one.
    pub right: Option<Rect>,
    /// The status line under the cards, if the shell has one.
    pub status: Option<Rect>,
}

/// Content drawn into the title bar's status area.
type TitleStatus<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

/// A resizable column: its width (kept by the app) and allowed range.
type Column<'a> = (&'a mut f32, RangeInclusive<f32>);

/// Builder for the window shell. Build and show it from the root `Ui` every
/// frame, using one shell per egui viewport.
///
/// ```
/// use spaceui::{Icon, RailItem, Shell, Theme, egui, widgets, widgets::Tone};
///
/// const RAIL: &[RailItem<'static>] = &[RailItem::page(Icon::Home, "Home"), RailItem::page(Icon::Settings, "Settings")];
///
/// # let ctx = egui::Context::default();
/// # Theme::carbon_copper().install(&ctx);
/// # let (mut page, mut chat_width) = (0, 380.0);
/// # ctx.run_ui(Default::default(), |ui| {
/// let regions = Shell::new("My App")
///     .rail(RAIL)
///     .left_column(&mut chat_width, 320.0..=560.0) // or `right_column`
///     .title_status(|ui| {
///         widgets::pill(ui, Some(Tone::Positive), "Connected");
///     })
///     .show(ui, &mut page);
/// assert!(regions.left.is_some() && regions.right.is_none());
/// # }).drop_without_applying_deltas();
/// ```
#[must_use = "a shell does nothing until `show` is called"]
pub struct Shell<'a> {
    title: &'a str,
    logo: Option<Svg<'a>>,
    rail: &'a [RailItem<'a>],
    left: Option<Column<'a>>,
    right: Option<Column<'a>>,
    status: f32,
    enabled: bool,
    title_status: Option<TitleStatus<'a>>,
    labels: WindowLabels<'a>,
}

impl fmt::Debug for Shell<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Shell")
            .field("title", &self.title)
            .field("logo", &self.logo.map(|svg| svg.name))
            .field("rail", &self.rail)
            .field("left", &self.left)
            .field("right", &self.right)
            .field("status", &self.status)
            .field("enabled", &self.enabled)
            .field("title_status", &self.title_status.is_some())
            .field("labels", &self.labels)
            .finish()
    }
}

impl<'a> Shell<'a> {
    /// A shell titled `title`: the title bar and the main card, with no logo,
    /// rail, columns or status line yet.
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            logo: None,
            rail: &[],
            left: None,
            right: None,
            status: 0.0,
            enabled: true,
            title_status: None,
            labels: WindowLabels::default(),
        }
    }

    /// Localized names for window actions and column dividers.
    ///
    /// ```
    /// use spaceui::shell::{Shell, WindowLabels};
    /// let shell = Shell::new("Mon application")
    ///     .labels(WindowLabels { close: "Fermer la fenêtre", ..Default::default() });
    /// # let _ = shell;
    /// ```
    pub fn labels(mut self, labels: WindowLabels<'a>) -> Self {
        self.labels = labels;
        self
    }

    /// The app's logo, beside the title.
    pub fn logo(mut self, logo: Svg<'a>) -> Self {
        self.logo = Some(logo);
        self
    }

    /// An icon rail down the left edge with these pages and dividers.
    /// [`show`](Self::show) takes the selected page as an index counting
    /// pages only. Without a rail (or with no items), the space goes to the
    /// cards. Overflow scrolls vertically. Up/Down and Home/End select and
    /// reveal a page while the rail has keyboard focus.
    /// When `items` is non-empty, an out-of-range page is clamped to the last
    /// page, or zero for a rail containing only dividers. An empty rail leaves
    /// the caller's page unchanged.
    pub fn rail(mut self, items: &'a [RailItem<'a>]) -> Self {
        self.rail = items;
        self
    }

    /// A column left of the main card (after the rail), resizable within
    /// `range`. The app keeps, and may persist, its width. A focused divider
    /// moves ten points with Left/Right; Home/End choose the width limits.
    /// Negative or non-finite widths and range endpoints are treated as zero
    /// before clamping. A reversed range collapses to its normalized lower
    /// bound. The available window space can reduce both bounds further.
    pub fn left_column(mut self, width: &'a mut f32, range: RangeInclusive<f32>) -> Self {
        self.left = Some((width, range));
        self
    }

    /// A column right of the main card, resizable within `range`. The app
    /// keeps, and may persist, its width. Keyboard behavior matches
    /// [`left_column`](Self::left_column); screen-reader increment/decrement
    /// actions always increase/decrease the column's width.
    /// Width and range normalization also match [`left_column`](Self::left_column).
    pub fn right_column(mut self, width: &'a mut f32, range: RangeInclusive<f32>) -> Self {
        self.right = Some((width, range));
        self
    }

    /// A status line `height` points tall under the cards.
    /// A non-positive or non-finite height hides it.
    pub fn status_bar(mut self, height: f32) -> Self {
        self.status = height;
        self
    }

    /// `false` makes the rail and column dividers inert, as under a blocking modal. The title bar
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
    /// rail page, updated by pointer or keyboard input and normalized as
    /// described by [`rail`](Self::rail).
    ///
    /// A column is clamped to its range and to half the window, or to 35% of
    /// it when there are columns on both sides. Available space takes priority
    /// over minimum widths in narrow windows; returned regions stay ordered.
    pub fn show(self, ui: &mut Ui, page: &mut usize) -> Regions {
        let ctx = ui.ctx().clone();
        let metrics = Theme::of(&ctx).metrics;
        paint_backdrop(&ctx);
        let window = ctx.content_rect();
        title_bar::show(&ctx, window, self.title, self.logo, self.title_status, &self.labels);

        let gap = finite_size(metrics.gap);
        let status_height = finite_size(self.status);
        let top = (window.top() + finite_size(metrics.title_bar)).min(window.bottom());
        let bottom = (window.bottom() - gap - status_height.max(gap * 0.5)).max(top);
        let mut left_edge = (window.left() + gap * 1.5).min(window.right());
        if !self.rail.is_empty() {
            let rail = Rect::from_min_max(
                pos2((window.left() + gap).min(window.right()), top),
                pos2((window.left() + finite_size(metrics.rail).max(gap)).min(window.right()), bottom),
            );
            rail::show(ui, rail, self.rail, page, self.enabled);
            left_edge = (rail.right() + 6.0).min(window.right());
        }
        let mut right_edge = (window.right() - gap * 1.5).max(left_edge);
        let share = if self.left.is_some() && self.right.is_some() { 0.35 } else { 0.5 };
        let count = usize::from(self.left.is_some()) + usize::from(self.right.is_some());
        let available = (right_edge - left_edge - 36.0).max(0.0);
        let gutter = (gap * 1.75).min(available / (count.max(1) * 2) as f32);
        let cap = (window.width() * share).min((available - gutter * count as f32).max(0.0) / count.max(1) as f32);

        let left = self.left.map(|(width, range)| {
            let (min, max) = limits(&range, cap);
            *width = finite_size(*width).clamp(min, max);
            let edge = left_edge + *width;
            let handle = Rect::from_min_max(pos2(edge, top + 20.0), pos2(edge + gutter, bottom - 20.0));
            splitter::show(
                ui,
                handle,
                width,
                min..=max,
                splitter::Side::Left,
                self.labels.left_column_width,
                self.enabled,
            );
            // Input can change the width in this frame. Return and paint the
            // same geometry represented by the caller's updated state.
            let column = Rect::from_min_max(pos2(left_edge, top), pos2(left_edge + *width, bottom));
            left_edge = column.right() + gutter;
            column
        });
        let right = self.right.map(|(width, range)| {
            let (min, max) = limits(&range, cap);
            *width = finite_size(*width).clamp(min, max);
            let edge = right_edge - *width;
            let handle = Rect::from_min_max(pos2(edge - gutter, top + 20.0), pos2(edge, bottom - 20.0));
            splitter::show(
                ui,
                handle,
                width,
                min..=max,
                splitter::Side::Right,
                self.labels.right_column_width,
                self.enabled,
            );
            let column = Rect::from_min_max(pos2(right_edge - *width, top), pos2(right_edge, bottom));
            right_edge = column.left() - gutter;
            column
        });

        let main = Rect::from_min_max(pos2(left_edge, top), pos2(right_edge, bottom));
        for card in [Some(main), left, right].into_iter().flatten() {
            widgets::plate(ui.painter(), card);
        }
        window_edges::show(&ctx, &self.labels.resize);
        Regions {
            main: main.shrink2(vec2(18.0_f32.min(main.width() * 0.5), 14.0_f32.min(main.height() * 0.5))),
            left,
            right,
            status: (status_height > 0.0)
                .then(|| Rect::from_min_max(pos2(window.left(), bottom), pos2(window.right(), window.bottom()))),
        }
    }
}

fn finite_size(value: f32) -> f32 {
    if value.is_finite() { value.max(0.0) } else { 0.0 }
}

/// Available space wins over a preferred minimum in a constrained window.
fn limits(range: &RangeInclusive<f32>, cap: f32) -> (f32, f32) {
    let cap = finite_size(cap);
    let min = finite_size(*range.start()).min(cap);
    (min, finite_size(*range.end()).max(min).min(cap))
}
