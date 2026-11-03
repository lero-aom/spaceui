# Integrating SpaceUI

## Choose the required pieces

The default features provide the desktop kit. With `default-features = false`, themes, fonts, painting and
basic widgets remain available without `resvg` or native window handles. Add `svg`, `shell`, `chat`, `native`
or `screenshot` as needed. `shell`, `chat` and `screenshot` each enable `svg`; features are additive.
Some icon-bearing widget helpers require `svg`. Build documentation with the same feature set as the app:

```sh
cargo doc --no-deps --no-default-features --features chat
```

Use `spaceui::egui` for shared types, and keep any direct egui/eframe dependency on the same compatible version.
The library does not depend on eframe at runtime; the examples use it as one backend.

## Theme, font and scale settings

Install a theme once when creating the context, then reinstall when the user changes it. Widgets read the
theme from the context. Font changes take effect through egui's next frame. Use `Fonts::BUNDLED` for repeatable
rendering across machines, or provide your own font bytes with suitable language coverage. Windows system
fonts are read at runtime and are not redistributed with the crate.

Use egui's `Context::set_zoom_factor` for proportional UI enlargement, including text and geometry. Changing
individual text sizes does not proportionally enlarge every custom-painted control. After installing a theme,
call `spaceui::theme::set_reduced_motion(ctx, true)` to disable decorative motion and egui transitions. The
application owns this preference and any connection to the operating system's setting.

Both bundled palettes are dark. A custom light palette should call
`theme.install_with_mode(ctx, spaceui::egui::Theme::Light)` so egui's remaining defaults match the palette.
This selects light-mode defaults; it does not convert a dark palette into a light one.

## State, IDs and layout

Keep application state outside SpaceUI. Shell regions are rectangles for the application to fill; chat
components do not create network requests, store conversations or authorize tool calls. Use `ui.push_id`
around repeated component groups and stable identifiers for conversations and items so egui can preserve
focus and state across frames. Labels that describe different actions should remain distinct.

The shell is intended for one undecorated desktop viewport. Start with `window::viewport` and test the
window controls on each supported backend. Keep scrollable content inside the returned regions. Extremely
small windows can leave no usable content area; adapt or hide application panels rather than assuming every
region has a useful minimum size. Prefer egui-native layout containers for components placed outside the shell.

Use labeled switch and notch-slider helpers to give application-specific names to assistive technology.
Keyboard focus and activation are part of the control contract, but the application must keep a coherent
focus order, name surrounding regions and test its backend's screen-reader bridge.

## Markdown and external content

The chat renderer intentionally supports a compact subset: headings, simple lists, quotes, rules, fenced
code, table-like rows and inline formatting/links. It is not a CommonMark implementation. It does not provide
HTML execution, images, reference links, nested lists, syntax highlighting or aligned table columns. Simple
inline destinations cannot contain nested parentheses; unmatched delimiters remain literal and backslash-
escaped punctuation is displayed as punctuation without the escape backslash.
Only HTTP(S) destinations activate as links. Applications that require a complete Markdown standard should
choose a parser and renderer for that contract.

Use `Svg::try_paint`, `Svg::try_rasterize` and `Svg::try_window_icon` when invalid art must be diagnosed;
they preserve a `SvgError` rather than silently omitting an icon. Explicit rasterization accepts dimensions
from 1 through `art::MAX_RASTER_SIZE` (2048 pixels). Painting caps raster resolution at that size. Larger
exports need a separate rasterizer or tiled pipeline. Parsing is synchronous: the raster limit does not bound
the complexity of an SVG document, so an application accepting untrusted art must also limit its input.

The per-context cache retains at most 256 entries and 32 MiB of RGBA texture data. These limits exclude
renderer overhead, parsing intermediates and texture handles retained elsewhere by the application. Use
`art::cache_stats` and `art::clear_cache` to inspect or release it. Different documents may share a diagnostic
name without sharing a cached texture. `Screenshot::try_update` preserves a typed `ScreenshotError` and its
underlying cause; the existing `update` helper retains its string-error interface.

## Diagnosing a problem

- Missing text: check font coverage and verify the same theme is installed on the context that draws the UI.
- A component loses focus or state: check stable IDs and avoid reusing an ID for unrelated components.
- Native controls behave differently: reproduce with `examples/minimal.rs` and record the OS and backend.
- A feature subset fails: run a separate consuming crate; development dependencies can accidentally enable
  features that an application does not have. CI's consumer smoke check exists for this reason.

See [support.md](support.md) for the validation matrix and [CONTRIBUTING.md](../CONTRIBUTING.md) for regression tests.
