# SpaceUI

[![Crates.io](https://img.shields.io/crates/v/spaceui.svg)](https://crates.io/crates/spaceui)
[![Documentation](https://docs.rs/spaceui/badge.svg)](https://docs.rs/spaceui)
[![MSRV](https://img.shields.io/badge/MSRV-1.95-blue.svg)](#minimum-supported-rust-version)
[![License](https://img.shields.io/crates/l/spaceui.svg)](#license)

A design system for [egui](https://github.com/emilk/egui) desktop apps: themes, anti-aliased painting, vector icons,
widgets, a custom-window shell and a chat kit. SpaceUI was extracted from **Rover** (a trading workspace) and **Probe**
(an AI-driven browser), so new apps start from the same parts.

![The gallery in the Carbon / Copper theme](docs/screenshots/carbon-copper.png)

## Highlights

- **One theme, everywhere.** A `Theme` is a semantic `Palette`, `Metrics`, fonts and a `Backdrop`. Install it once;
  every widget reads it back, so nothing hard-codes a colour. Two ship: *Carbon / Copper* (Rover) and *Deep Space*
  (Probe).
- **Smooth at any scale.** Gradients and fills get the same one-pixel feathered edge egui gives its own shapes. Icons
  and logos stay SVG and are rasterized at the exact pixel size they're drawn at.
- **A whole window.** Your own title bar (usable even above modal dialogs), an icon rail, a carded main panel, a
  resizable side column, a status line and window resize edges, in one builder.
- **A chat kit.** Conversation tabs, user bubbles, Markdown replies, collapsible thinking, a tool-call timeline, starters
  for an empty chat and a composer with model and effort dropdowns. Your app keeps the conversations and the backend.
- **Plain data, plain functions.** Widgets are functions returning egui's `Response`; themes are structs. There is no
  framework to adopt.

![The gallery in the Deep Space theme](docs/screenshots/deep-space.png)

## Getting started

```toml
[dependencies]
spaceui = "0.1"
eframe = "0.37"
```

A themed window with a title bar, an icon rail, a main card and a side column
([`examples/minimal.rs`](examples/minimal.rs)):

```rust,no_run
use spaceui::egui;
use spaceui::prelude::*;

const RAIL: &[RailItem] = &[
    RailItem::page(Icon::Home, "Home"),
    RailItem::page(Icon::Chart, "Markets"),
    RailItem::Divider,
    RailItem::page(Icon::Settings, "Settings"),
];

#[derive(Default)]
struct App {
    page: usize,
    side: f32,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let regions = Shell::new("My App")
            .logo(Icon::Orbit.svg())
            .rail(RAIL)
            .side_column(&mut self.side, 320.0..=560.0)
            .show(ui, &mut self.page);

        ui.scope_builder(egui::UiBuilder::new().max_rect(regions.main), |ui| {
            ui.heading(["Home", "Markets", "Settings"][self.page]);
            if widgets::primary(ui, "Connect").clicked() {
                // ...
            }
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        // Undecorated: the shell draws the title bar and the resize edges.
        viewport: window::viewport("My App", "my-app"),
        ..Default::default()
    };
    eframe::run_native(
        "My App",
        options,
        Box::new(|cc| {
            let theme = Theme::carbon_copper();
            window::round_corners(cc, theme.palette.border);
            theme.install(&cc.egui_ctx);
            Ok(Box::new(App::default()))
        }),
    )
}
```

SpaceUI re-exports egui as `spaceui::egui`, so your app and SpaceUI always agree on its version.

## Examples

| Example | Shows |
|---|---|
| [`minimal`](examples/minimal.rs) | The window above: shell, rail, card and side column |
| [`custom_theme`](examples/custom_theme.rs) | A theme of your own from a few palette changes |
| [`gallery`](examples/gallery.rs) | Every widget in both themes, and a chat column with canned replies |

```sh
cargo run --example gallery
```

![Widgets in the gallery](docs/screenshots/controls.png)

## Organisation

The modules build on each other, from colours to whole windows:

| Module | Holds |
|---|---|
| [`theme`](https://docs.rs/spaceui/latest/spaceui/theme/) | `Theme`, `Palette`, `Metrics`, `Backdrop` and the fonts (`regular`, `semibold`, `mono`) |
| [`paint`](https://docs.rs/spaceui/latest/spaceui/paint/) | Anti-aliased gradients, polygon fills, glows and a sparkle glyph |
| [`art`](https://docs.rs/spaceui/latest/spaceui/art/) | The built-in `Icon` set (70 line icons) and your own `Svg` art, with window icons |
| [`widgets`](https://docs.rs/spaceui/latest/spaceui/widgets/) | Buttons, pills, badges, chips, tabs, segmented controls, cards, metrics, a switch, a field and a slider |
| [`shell`](https://docs.rs/spaceui/latest/spaceui/shell/) | The `Shell` builder, `RailItem` and the `Regions` it returns |
| [`chat`](https://docs.rs/spaceui/latest/spaceui/chat/) | `ChatLayout`, `tab_strip`, transcript pieces, `Composer` and a Markdown renderer |
| [`window`](https://docs.rs/spaceui/latest/spaceui/window/) | An undecorated viewport, rounded corners on Windows 11 and screenshot runs |

## Your own theme

A theme is plain data: start from a built-in one and change what you need
([`examples/custom_theme.rs`](examples/custom_theme.rs)).

```rust
use spaceui::egui::Color32;
use spaceui::{Backdrop, Palette, Theme};

let arctic = Theme {
    name: "Arctic",
    palette: Palette {
        accent: Color32::from_rgb(64, 156, 255),
        accent_hi: Color32::from_rgb(130, 196, 255),
        accent_lo: Color32::from_rgb(28, 98, 196),
        ..Palette::DEEP_SPACE
    },
    backdrop: Backdrop::Plain,
    ..Theme::deep_space()
};
assert_eq!(arctic.metrics, Theme::deep_space().metrics);
```

![Palette roles and the icon set](docs/screenshots/palette-and-icons.png)

## Checking your UI

`window::Screenshot` renders a few frames, saves the window as a PNG and closes the app, for native visual checks of
pages and themes. The gallery wires it to environment variables:

```sh
SPACEUI_SCREENSHOT=out.png SPACEUI_THEME=deep_space SPACEUI_PAGE=2 cargo run --example gallery
```

SpaceUI's own tests drive the widgets headlessly with simulated clicks and key presses; see [`tests/`](tests).

## Platform support

SpaceUI runs wherever egui does.

- **Fonts.** On Windows, Segoe UI, Segoe UI Semibold, Segoe UI Symbol and Segoe UI Emoji are read from the system at
  runtime, never bundled. Elsewhere egui's bundled fonts stand in under the same family names.
- **Window corners.** `window::round_corners` asks Windows 11 for rounded, anti-aliased corners and a border colour;
  other systems keep their defaults.
- **Window.** The shell expects an undecorated viewport (`window::viewport`).

## Minimum supported Rust version

SpaceUI's MSRV is 1.95, the minimum of egui 0.37. It is raised only in minor releases, and only when the egui version
SpaceUI builds on requires it.

## Contributing

Contributions are welcome; [CONTRIBUTING.md](CONTRIBUTING.md) explains the checks, the design rules and how to add
an icon.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option. Third-party notices are in [NOTICE.md](NOTICE.md).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in SpaceUI by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
