# Rust code quality review

Reviewed on 2026-11-04, initially from `d7f5097`, with a follow-up from `f47b7bf`.
This review covers every library module,
its public contracts, and the supporting Rust tests, examples and feature configurations.
It records concrete findings, not a certification that every possible defect has been found.

## Review standard

Memory safety, correct foreign-function contracts, valid UTF-8 indexing, arithmetic invariants,
and accurate error behavior are requirements. Naming, trait coverage, ownership ergonomics,
documentation and lint choices are design guidance that must fit this crate's immediate-mode GUI API.

The primary references were the [Rust API Guidelines checklist](https://rust-lang.github.io/api-guidelines/checklist.html),
[argument validation](https://rust-lang.github.io/api-guidelines/dependability.html),
[failure documentation](https://rust-lang.github.io/api-guidelines/documentation.html#function-docs-include-error-panic-and-safety-considerations-c-failure),
[panic versus recoverable errors](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html),
[borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html),
[FFI safety](https://doc.rust-lang.org/nomicon/ffi.html),
[Cargo compatibility](https://doc.rust-lang.org/cargo/reference/semver.html), and
[Clippy's configuration guidance](https://doc.rust-lang.org/clippy/usage.html).

## Findings and changes

| Category | Assessment before this review | Correction or retained design |
| --- | --- | --- |
| Parsing and panic safety | **Defect:** `****` and `____` could create an inverted byte range and panic. Hosted fuzzing independently found the same problem in a longer input. | Validate the emphasis body with checked string slicing; regress delimiter runs, Unicode/mixed input and the exact CI reproducer. Retain each minimal case as a fuzz seed. |
| Numeric/input validation | **Gaps:** finite endpoints could produce infinite rectangle extents; decorative painting accepted invalid dimensions; tiny text regions could still display a glyph wider than the region. | Reject invalid/overflowing geometry before submitting it, check SVG rectangles after pixel rounding, and return an empty text galley when nothing fits. Validate screenshot dimensions and pixel count before copying the buffer. |
| Asynchronous state and concurrency | **Defect:** screenshot capture repainted continuously while waiting and after completion; its callback closed whichever viewport was current. | Use explicit settling/requested/finished states. Request exactly one screenshot, wake on completion, target the captured viewport and consume the outcome once. Filesystem work stays outside the result lock. |
| UI state consistency | **Defect:** tab, segmented and rail controls could expose both old and new choices as selected in the click frame. | Resolve all group interactions before publishing selection metadata and painting. Regression tests inspect the same frame's accessibility tree. |
| Ownership and allocation | **Avoidable work:** runtime font bytes were deep-cloned; parser links copied destinations and accessible labels. | Share immutable font blobs and cached font configuration with `Arc`; borrow link metadata from the input. Cache the default theme for repeated lookups. Public signatures remain unchanged. |
| Font identity and fallback | **Defects:** same-named fonts in different roles overwrote each other; system semibold could precede an explicitly supplied regular fallback. | Give each role a distinct font key and honor custom regular fallback first. Verify stored bytes and fallback ordering. |
| Behavioral contracts | **Gap:** reduced motion disabled transitions but left programmatic scrolling animated. | Save, disable and restore both transition and scroll animation settings for light and dark styles, including repeated calls and theme changes. |
| Public API/documentation | **Mostly sound:** borrowed builders, owned retained state, conventional names, useful standard traits and typed errors already existed. Some normalization and font prerequisites were implicit. | Document index/width normalization, stable widget identity, accessibility labels, explicit theme installation and deferred invalid-font panics. Narrow SVG error wording so it does not promise recovery from global allocator exhaustion. Mark screenshot runs as values that must be driven. |
| Unsafe code | **Contained:** one private Windows DWM wrapper with the system ABI, a live borrowed window handle and a four-byte value valid throughout the call. No new unsafe operation was needed. | Keep the crate-wide unsafe-code denial and the narrow wrapper exception. Add checks requiring explicit unsafe blocks and documented safety arguments. |
| Compatibility and build hygiene | **Already strong:** locked dependencies, explicit MSRV, feature tests, strict Clippy/rustdoc, platform CI and package verification. | Preserve existing public fields, function signatures and error variants. Keep the scoped graphics-cast allowances; avoid blanket restriction lints and unnecessary new dependencies. |

## Follow-up findings

The follow-up checked the official guidance again and began with passing format, strict Clippy,
88 unit/interaction tests and 25 documentation tests. Manual review still found these defects:

| Area | Defect | Correction and regression |
| --- | --- | --- |
| Keyboard events | Composer submission used the final frame's Shift state rather than the state when Enter was pressed. Releasing Shift in the same frame could send a newline draft. | Read modifiers from the Enter event. Exercise both modifier-change directions and IME text commits. |
| Markdown tokens | Emphasis could close on a delimiter inside code or a link, then remain active after the renderer skipped over that delimiter. | Exclude token contents from the emphasis index; preserve unmatched markers. Cover code, link labels/destinations and bare URLs, including a fuzz seed. Skip the extra pass when there are no emphasis markers. |
| Splitter state | Input updated the stored column width after its returned rectangle was already calculated. | Build column/main regions, divider indicators and accessible bounds from the updated width in the same frame; test keyboard and accessibility adjustments on both sides. |
| Polygon arithmetic | Absolute-coordinate cross products lost the area of small polygons translated far from the origin. | Calculate area relative to the first vertex; verify translated polygons retain their fill and winding. |
| Alpha handling | A translucent radial-glow color was premultiplied twice when replacing its opacity. | Unmultiply RGB before applying the requested alpha; regress a translucent saturated color. |
| Repaint lifecycle | Calling the reduced-motion setter each frame requested perpetual repainting even when nothing changed. | Repaint only when motion settings change; test that repeated preferences settle and actual changes still wake the UI. |
| Card geometry | Invalid rectangles still produced border/sheen shapes, and narrow plates could invert the sheen endpoints. | Reject invalid geometry and clamp finite radii/sheens to the rectangle; test invalid and tiny positive rectangles. |
| Slider arithmetic | Multiplying width by a notch index before dividing could overflow for large finite inputs. | Multiply by the normalized fraction; verify finite positions while retaining the 512-marker paint bound. |

No public signatures, feature definitions, dependency versions or unsafe operations changed.
Existing lint exceptions remain scoped to intentional graphics conversions and naming.

## Verification

The fixes have targeted regressions plus the full unit, interaction, example and documentation suites.
Strict Clippy covers all targets/features on stable Rust and Rust 1.95. Core-only tests and doctests
exercise the optional-module boundaries. Visual tests compare committed images without automatically
accepting changes, and benchmarks measure allocation behavior separately from timing.

The follow-up passed 98 unit/interaction tests, all example targets, 25 doctests and four rendered
regression tests on Windows with Rust 1.97.1. Rust 1.95 passed strict Clippy, the behavior suite,
doctests, and check/test/rustdoc for core-only and each individual optional feature. External
core/desktop consumers and local package verification passed. Both lockfiles passed `cargo audit`;
`cargo deny` passed advisory, license, source and ban policies with the existing duplicate-version
warnings. Benchmark results and the parser correction's allocation cost are recorded in
[performance.md](performance.md). Hosted platform and fuzz checks run on the pushed commit.

The previous hosted run passed Windows/Linux/macOS stable and MSRV builds, all feature configurations,
external-consumer checks and dependency auditing. Its parser fuzz failure supplied a permanent regression.
Its Linux shell snapshot failure was also inspected: the rendered layout was unchanged, with small
renderer-dependent color differences. The visual fixture now distinguishes the reviewed Linux Vulkan
baseline from the original local baseline while retaining the original comparison threshold.

## Deliberate limits

- No new parser or font-validation dependency is added. Invalid app-supplied font data follows egui's
  deferred panic contract, now documented. Applications must supply valid TTF/OTF assets.
- SVG raster dimensions and retained cache bytes are bounded; arbitrary SVG source complexity and
  process-wide allocation exhaustion are not made recoverable by those bounds.
- At the time of this review the Markdown renderer was a documented subset. It has since been replaced
  with `egui_commonmark`; see [integration.md](integration.md) for the current rendering contract.
- Safe Rust and passing lint/tests do not prove semantic correctness or exhaustively verify native
  window behavior. Fuzzing and platform tests remain ongoing checks.
- Serialization, extra generics, blanket trait derives and banning every internal assertion are not
  universal Rust requirements. They were not added without a concrete use case.
