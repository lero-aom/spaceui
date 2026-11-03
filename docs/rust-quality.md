# Rust code quality review

Reviewed on 2026-11-04, starting from `d7f5097`. This review covers every library module,
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

## Verification

The fixes have targeted regressions plus the full unit, interaction, example and documentation suites.
Strict Clippy covers all targets/features on stable Rust and Rust 1.95. Core-only tests and doctests
exercise the optional-module boundaries. Visual tests compare committed images without automatically
accepting changes, and benchmarks measure allocation behavior separately from timing.

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
- The Markdown renderer remains a documented subset, not a complete CommonMark implementation.
- Safe Rust and passing lint/tests do not prove semantic correctness or exhaustively verify native
  window behavior. Fuzzing and platform tests remain ongoing checks.
- Serialization, extra generics, blanket trait derives and banning every internal assertion are not
  universal Rust requirements. They were not added without a concrete use case.
