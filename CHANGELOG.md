# Changelog

All notable changes to SpaceUI are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-11-04

First public release. Requires Rust 1.95 and integrates with egui 0.37.

### Added

- `theme`: semantic palettes, layout metrics, system or bundled fonts, custom font files and backdrops;
  the Carbon / Copper and Deep Space themes; egui visuals.
- `paint`: anti-aliased gradients, polygon fills, glows and a sparkle glyph.
- `art`: 70 line icons and custom SVGs, rasterized at the display's pixel density, with window icons,
  fallible rasterization diagnostics and bounded texture caches.
- `widgets`: buttons, status badges and pills, tabs, segmented controls, cards, metrics, switches,
  text fields and notched sliders. Status widgets accept semantic or custom tones.
- `shell`: a custom title bar, optional icon rail, main panel, resizable columns on either side,
  a status bar and window resize edges.
- `chat`: conversation tabs, transcript bubbles, Markdown replies, collapsible thinking, tool steps,
  notices, an empty state and a composer with model and effort controls. Applications own their backends.
- `window`: an undecorated viewport, Windows 11 rounded corners and native screenshot capture.
- Optional SVG, shell, chat, native-window and screenshot features, with a core-only configuration.
- Astronomy examples `minimal`, `custom_theme` and `gallery`, including a selectable illustrative sky map,
  observing plans, sample capture controls and a scripted assistant. The gallery defaults to Deep Space;
  the custom theme demonstrates a Lunar palette.
- Rendered regression baselines, interaction and geometry tests, parser/geometry fuzz targets,
  timing and allocation benchmarks, and an external-consumer smoke check.
- Windows, Linux and macOS stable/MSRV CI; feature-subset checks; dependency auditing, update automation,
  published-API compatibility checks and an explicit release dry-run/publish workflow.
- Integration, platform-support, compatibility and security-reporting documentation.
- Translatable chat/window labels, light-mode installation for custom palettes, reduced-motion settings,
  and labeled controls with keyboard and assistive-technology actions.
- Built-in accent gradients with dark foregrounds and brighter low stops for 4.5:1 text contrast.
- Bundled fallback fonts enabled independently of application/development dependencies.

### Fixed

The first release includes these fixes from development and review:

- Repeated segmented-control IDs, text-field identity during layout changes, and slider change notifications.
- Inaccessible overflowing chat tabs and rail pages, constrained-region geometry and decorative animation
  when reduced motion is requested.
- Repeated suffix scans and incomplete-delimiter handling in inline Markdown, with keyboard-accessible links.
- A Markdown panic on overlapping emphasis delimiters, including a hosted-fuzzer reproducer.
- Same-frame selection consistency for tabs, segmented controls and rail pages.
- Font-name collisions between roles, custom regular-font fallback, unnecessary owned-font/link copies,
  and animated scrolling while reduced motion is enabled.
- Screenshot capture lifecycle and original-viewport targeting; malformed screenshot buffers, overflowing
  paint geometry and text that exceeded very narrow regions.
- Composer submission when Shift changes in the same frame as Enter, and Markdown emphasis leaking
  from delimiters inside code spans or links.
- Same-frame splitter geometry, translated polygon precision, translucent radial-glow colors,
  redundant reduced-motion repaints, narrow/invalid card plates and overflowing slider interpolation.

### Limits and compatibility

- The API is pre-1.0. Patch releases preserve the supported public API; incompatible changes require a minor release.
- `Svg::rasterize` and `Svg::window_icon` return `None` for dimensions above 2048 pixels. Their
  `try_` counterparts report `SvgError::InvalidSize`; use a separate export rasterizer for larger output.
  Painting still caps raster resolution at 2048 pixels.
- Default features enable the complete kit. Disabling defaults removes the SVG,
  shell, chat, native-window and screenshot APIs unless their feature is selected.
- Automated platform checks do not establish native window-manager or screen-reader compatibility.
  Browser/WASM and mobile are not currently covered; see the support matrix for validation boundaries.
- Astronomy examples use illustrative data; they do not calculate live sky positions, connect to hardware
  or call an AI service.

[Unreleased]: https://github.com/lero-aom/spaceui/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/lero-aom/spaceui/tree/v0.1.0
