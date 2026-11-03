# Changelog

All notable changes to SpaceUI are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Optional SVG, shell, chat, native-window and screenshot features, with a core-only configuration.
- Rendered regression baselines, expanded interaction and geometry tests, parser/geometry fuzz targets,
  timing and allocation benchmarks, and an
  external-consumer smoke check.
- Windows, Linux and macOS stable/MSRV CI; feature-subset checks; dependency auditing, update automation,
  published-API compatibility checks and an explicit release dry-run/publish workflow.
- Integration, platform-support, compatibility and security-reporting documentation.
- Translatable chat/window labels, light-mode installation for custom palettes, reduced-motion settings,
  and labeled controls with keyboard and assistive-technology actions.

### Changed

- Public support claims distinguish automated checks from native-platform and accessibility validation.
- Improved keyboard/accessibility metadata, input handling and rendering robustness; fallible SVG diagnostics
  and bounded texture-cache behavior.
- Built-in accent gradients use dark foregrounds and brighter low stops to retain 4.5:1 text contrast.
- Bundled fallback fonts are enabled explicitly, independently of application/development dependencies.

### Fixed

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

### Migration notes

- `Svg::rasterize` and `Svg::window_icon` now return `None` for dimensions above 2048 pixels. Their new
  `try_` counterparts report `SvgError::InvalidSize`; use a separate export rasterizer for larger output.
  Painting still caps raster resolution at 2048 pixels.
- Existing default features preserve the complete kit. Disabling defaults deliberately removes the SVG,
  shell, chat, native-window and screenshot APIs unless their feature is selected.

## [0.1.0] - 2026-11-03

First release.

### Added

- `theme`: `Theme`, `Palette`, `Metrics`, `Fonts` (system Segoe UI, egui's
  bundled fonts or your own files) and `Backdrop`; the Carbon / Copper and
  Deep Space themes; egui visuals.
- `paint`: anti-aliased gradients, polygon fills, glows and a sparkle glyph.
- `art`: 70 built-in line icons and app SVGs, rasterized at exact pixel sizes,
  with window icons.
- `widgets`: primary, secondary and icon buttons; pills, badges and chips;
  underline tabs and segmented controls; cards, sections, captions and
  metrics; a switch, a text field and a notched slider. Status widgets take a
  `Tone`: a palette role or `Tone::Custom(color)`.
- `shell`: a custom title bar that stays usable above modal dialogs, an
  optional icon rail, a carded main panel, resizable columns on the left, the
  right or both, a status line and window resize edges.
- `chat`: `ChatLayout`, conversation tabs, transcript pieces (bubbles,
  replies, thinking, tool steps, notices, an empty state), `Composer` and a
  Markdown renderer.
- `window`: an undecorated viewport, rounded corners on Windows 11 and
  screenshot runs.
- Examples `minimal`, `custom_theme` and `gallery`.

[Unreleased]: https://github.com/lero-aom/spaceui/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/lero-aom/spaceui/releases/tag/v0.1.0
