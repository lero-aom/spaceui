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

`Svg<'a>` and `RailItem<'a>` borrow strings for the call, including runtime-generated SVGs and translated
rail labels. Rebuild rail items from the current labels each frame; their positional IDs preserve focus
when labels change. Static declarations should spell the lifetime explicitly, for example
`const LOGO: Svg<'static>` and `const RAIL: &[RailItem<'static>]`. SVG painting owns any source text retained
by the cache and keys it by content, so editing or dropping the caller's string cannot leave a stale image.

The Composer returns actions without clearing the caller's draft. A nonempty, non-whitespace draft sends
on a fresh Enter press while the editor retains focus through the call; held-key repeats do not send.
Shift+Enter inserts a newline using the key event's modifiers. An IME preedit or commit frame does not
submit, and a later explicit Enter can submit the committed text. Requesting focus cannot submit an event
from another field. Busy state blocks sending; the stop button returns only `stop` when busy and stoppable.
A disabled Composer returns neither action. Applications still own when to clear the draft or mark a request
busy, and must give each Composer a stable ID. Sequence tests cover these combinations and transitions;
native IME/backend integration also needs application testing.

## Configurable controls and response ownership

`widgets::Button::primary(label)` and `Button::secondary(label)` implement `egui::Widget`, so they work
with `ui.add`, `ui.add_enabled` and reusable widget adapters. Minimum size, padding, corner radius,
solid fill and text colour are optional overrides. Disabled colours continue to come from the theme.
The existing button functions delegate to the same implementation.

For choice groups, `tabs_response` and `segmented_response` retain the same keyboard navigation,
accessibility roles and caller-owned selection as the boolean helpers. Their `ChoiceResponse.response`
is an egui response union with the group's layout ID. Use it for bounds, hover help, clicks and changes;
use `choices[index]` for focus or a particular choice's response. Do not call `interact` on the union.
Selection-state changes mark the old and new choices changed; clamping an invalid index marks the
aggregate changed even when the group is empty or disabled. Disabled groups ignore user input.

`theme::ControlMetrics::install` sets context-wide button, tab, segment and field dimensions. Per-button
options override those defaults. `chat::ComposerMetrics::install` provides the same convention for
composer defaults, with `.metrics(...)` for an individual composer. These settings deliberately survive
`Theme::install` so palette switching does not reset density or drafting preferences. Reinstall default
metrics to reset them. No fields were added to the existing public `Theme` or `Metrics` structs.

Always use a configured composer's `desired_height` when computing its layout. The static `measure`
method knows only the installed defaults. Longer drafts scroll; very small supplied rectangles may
leave no usable editing area. Non-finite dimensions use defaults and large/negative values are bounded
as described in the API docs. Font sizes and line limits have additional bounds.

The input controls also have `egui::Widget` builders, so reusable adapters and `ui.add_enabled` work
with `Switch::new(&mut enabled).label("Notifications")`,
`NotchedSlider::new(&mut level, 5).width(180.0).label("Effort")`, and
`TextField::new(&mut query).id_salt("query").hint_text("Search").width(240.0)`.
The explicit field salt produces the same scoped ID as `text_field_id`; migrating a field to a builder
does not lose its focus. Themes remain context-wide; these builders do not create independent theme scopes.

## Markdown and external content

Replies and user bubbles use `egui_commonmark` 0.26 (egui 0.37), including nested lists, reference links,
tables and fenced code with basic syntax highlighting. `chat::markdown::render` keeps the familiar
two-argument API and a shared per-context `CommonMarkCache`; `clear_cache` releases that cache.
`render_with_cache` accepts an application-owned cache for link hooks and explicit lifetime management.
The upstream `CommonMarkCache` and `CommonMarkViewer` types are re-exported for further customization.

Long plain paragraphs that the CommonMark parser identifies as text-only are rendered as one label when
fragmentation would otherwise create many widgets. The analysis cache retains at most 32 entries and 1 MiB
of source plus decoded text; `clear_cache` clears it too. A changed source is reanalyzed. Formatting, links,
line breaks, code and multiple paragraphs use the upstream viewer. This is not transcript virtualization.

For an application that accepts showing source when rich rendering would create too many widgets:

```rust
let shown = spaceui::chat::markdown::Markdown::new(message).max_events(4096).show(ui);
if shown.fallback {
    ui.small("Showing Markdown source; open the document for full formatting.");
}
```

The default has no event budget and preserves rich rendering. A budget counts parser events, including
block boundaries, before creating rich widgets. The fallback displays the complete original source as a
single wrapping label; markup is visible and links are inactive. Use `show_with_cache` for link hooks,
or `ui.add(Markdown::new(message))` when only the egui response is needed. Source length, parser work and
font layout still need separate application limits; an event budget is not a wall-clock deadline.
The directly re-exported upstream `CommonMarkViewer` does not use SpaceUI's analysis cache or budget.

The former `inline_job` API has been removed: CommonMark is rendered as widgets, not reduced to a single
text layout job. Migrate custom bubbles to `render` or `render_with_cache` inside a constrained egui UI.
User bubbles now support block Markdown as well as inline formatting.

The upstream code-block copy button uses the Unicode symbol U+1F5D0. Install a symbol font if needed;
egui's bundled fonts alone render its fallback box. With egui's strict missing-glyph policy, supply the
font or use the normal `MissingGlyphPolicy::Tofu` policy for arbitrary Markdown content.

HTML is displayed as text. Images use loaders installed in the egui context; SpaceUI disables the
dependency's default features and does not enable file or network loaders. Consumers can opt into those
through `egui_extras` or `egui_commonmark` features. Links follow upstream egui navigation behavior,
including schemes beyond HTTP(S); use cache link hooks or filter egui output commands for an application's
navigation policy. The old renderer's HTTP(S)-only restriction no longer applies.

Use `Svg::try_paint`, `Svg::try_rasterize` and `Svg::try_window_icon` when invalid art must be diagnosed;
they preserve a `SvgError` rather than silently omitting an icon. Explicit rasterization accepts dimensions
from 1 through `art::MAX_RASTER_SIZE` (2048 pixels). Painting caps raster resolution at that size while
preserving the destination's aspect ratio. Painting fits and centers the SVG inside the destination;
square raster and window-icon exports likewise center non-square artwork with transparent padding. Larger
exports need a separate rasterizer or tiled pipeline. Parsing is synchronous: the raster limit does not bound
the complexity of an SVG document, so an application accepting untrusted art must also limit its input.

The per-context cache retains at most 256 raster entries and 32 MiB of RGBA texture data, plus 128 parsed
documents and 4 MiB of source text. Parsed documents, including parse failures, are reused across sizes.
Evicting a document also removes its textures; sources larger than 4 MiB render without cache retention.
The source-byte budget does not measure or bound parsed-tree allocations. These limits exclude renderer
overhead, parsing intermediates and texture handles retained elsewhere by the application, and are not a
total-process memory or latency bound. Use `art::cache_stats` and `art::clear_cache` to inspect or release
both caches. Different documents may share a diagnostic name without sharing a cached texture; identical
content shares an entry regardless of its name. Standalone raster/window-icon exports do not use this cache.
See [performance.md](performance.md) for measured resize costs and workload limits.

For runtime artwork on native targets, keep one `art::SvgLoader::new(&context)?` in application state.
Its `load(svg, [width_px, height_px])` method returns `Result<Poll<TextureHandle>, SvgError>`. While pending,
reserve the image's rectangle and show a placeholder; on completion, paint the returned texture into that
rectangle. Supply both raster dimensions to preserve the intended aspect ratio, and recompute them for DPI
or size changes. `SvgError::QueueFull` means retry on a later frame; completed work requests a repaint.
The loader is bound to its creation context, so use its textures only in that context.

One worker accepts at most two outstanding requests, even across `clear()` calls. It rejects sources over
4 MiB and raster dimensions outside `1..=2048`; pending source bytes and retained cache source bytes each
have a 4 MiB limit. The ready cache holds at most 32 entries/32 MiB of RGBA. Failed documents are cached.
Source content and both dimensions identify results. Clearing discards old tickets so stale completions
cannot replace new requests; externally held textures remain valid. Dropping does not join the worker.

Parsing and rasterization run on the worker. Source copying, cache lookup, conversion into texture deltas
and subsequent GPU upload remain foreground costs. Active resvg calls cannot be forcibly cancelled, and
parsed-tree/filter allocations have no process-wide memory bound. This loader is an opt-in native path;
built-in icons and synchronous exports retain their existing behavior. Its cache is separate from
`art::cache_stats`/`art::clear_cache`; use the loader's `clear` method to release its retained results.

`Screenshot::try_update` preserves a typed `ScreenshotError` and its underlying cause; the existing
`update` helper retains its string-error interface.

## Diagnosing a problem

- Missing text: check font coverage and verify the same theme is installed on the context that draws the UI.
- A component loses focus or state: check stable IDs and avoid reusing an ID for unrelated components.
- Native controls behave differently: reproduce with `examples/minimal.rs` and record the OS and backend.
- A feature subset fails: run a separate consuming crate; development dependencies can accidentally enable
  features that an application does not have. CI's consumer smoke check exists for this reason.

See [support.md](support.md) for the validation matrix and [CONTRIBUTING.md](../CONTRIBUTING.md) for regression tests.
