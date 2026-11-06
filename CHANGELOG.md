# Changelog

All notable changes to SpaceUI are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.2] - 2026-11-06

Compatible with 0.2.1; no migration is required. Rust 1.95 and egui 0.37 remain unchanged.

### Added

- Added `Markdown` with an opt-in rich-rendering event budget and an explicit plain-source fallback response.
- Added native `art::SvgLoader`: one background parsing/rasterization worker, bounded queue/cache, and
  distinct pending, ready and failure states. Existing synchronous SVG APIs are unchanged.
- Added `Switch`, `NotchedSlider` and `TextField` values implementing `egui::Widget`; helpers share their
  implementation, including stable field identities, accessibility, keyboard input and changed reporting.

### Performance

- Coalesce parser-verified, fragmented plain paragraphs into a single label, with bounded analysis reuse.
  Rich Markdown still uses `egui_commonmark`; nothing is truncated or silently downgraded by default.
- Measure cold/streaming Markdown and complex SVG enqueue, polling and completion costs separately.

## [0.2.1] - 2026-11-06

Compatible with 0.2.0; no migration is required. Rust 1.95 and egui 0.37 remain unchanged.
The existing helpers and default visual appearance are preserved.

### Added

- Added `widgets::Button`, a configurable `egui::Widget`, while retaining `primary` and `secondary`.
- Added `tabs_response` and `segmented_response` with aggregate and per-choice interaction responses;
  the existing boolean helpers preserve their signatures and change-reporting behavior.
- Added context-wide `theme::ControlMetrics` for text-bearing control sizing, preserving default visuals
  and the existing `Theme` and `Metrics` struct shapes.
- Added `chat::ComposerMetrics`, per-composer overrides and `Composer::desired_height` for consistent
  configuration and measurement. Existing composer calls use the default geometry and input behavior.
- Extended benchmarks with plain-label and ordinary-text comparisons for the expensive repeated-bracket
  CommonMark workload. No Markdown rendering semantics or dependencies changed.

## [0.2.0] - 2026-11-05

Requires Rust 1.95 and integrates with egui 0.37. This minor release changes public APIs;
follow the migration notes below when upgrading from 0.1.

### Changed

- Replace the custom Markdown parser, layout and link handling with `egui_commonmark` 0.26 for replies
  and user bubbles. Nested lists, reference links, tables and basic code highlighting use upstream rendering.
- **Breaking:** remove `chat::markdown::inline_job`. Use `render` or the new `render_with_cache` inside
  a constrained UI. `CommonMarkCache` and `CommonMarkViewer` are re-exported; `clear_cache` releases
  the shared context cache. Links now follow egui navigation behavior instead of an HTTP(S)-only filter.
- Update Markdown benchmarks and fuzzing to exercise the CommonMark integration.
- **Breaking:** `Svg<'a>` and `RailItem<'a>` accept borrowed runtime strings. Use explicit `'static`
  lifetimes in constant/type declarations. SVG cache identity uses source content instead of storage identity.
- Reuse parsed SVG documents across raster sizes. Retain at most 128 documents and 4 MiB of source text
  alongside the existing texture budget; document eviction also releases its textures. Parsed-tree memory
  is not included in the byte budget. `CacheStats` adds `documents` and `source_bytes` fields.
- Add cold/resize SVG frame benchmarks and Composer input-sequence tests covering the send/stop state matrix.

### Fixed

- Non-square SVGs now retain their aspect ratio and are centered when painted into rectangular destinations.
  Texture dimensions, cache keys and memory accounting use both axes. Square exports center the artwork.
- Composer ignores held Enter repeats, Enter when focus changes, and Enter during IME preedit/commit.
  A later fresh Enter can send the draft; disabled, busy and stoppable states retain their action contracts.

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

[Unreleased]: https://github.com/lero-aom/spaceui/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/lero-aom/spaceui/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/lero-aom/spaceui/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/lero-aom/spaceui/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/lero-aom/spaceui/tree/v0.1.0
