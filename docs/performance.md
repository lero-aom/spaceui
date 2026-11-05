# Performance measurements

The historical measurements below predate the `egui_commonmark` migration and describe the removed custom parser.
The current benchmarks render CommonMark frames, including the malformed-input and allocation workloads;
their results cannot be compared directly with the former parser-only measurements.

Run the timing and allocation workloads separately:

```sh
cargo bench --locked --bench workloads --all-features
cargo bench --locked --bench allocations --all-features
cargo bench --locked --bench svg_resize --all-features
```

`workloads` warms each case, then reports median and p95 wall-clock time. UI
measurements cover egui frame construction and layout, not GPU drawing or a
native event loop. The 100-message case renders a fixed Markdown transcript;
it does not claim transcript virtualization. `allocations` uses `stats_alloc`
only in its own executable, so instrumentation does not bias the timing run.
Counts include egui/framework work in the measured frame, not just SpaceUI.
Requested allocation bytes are allocation traffic, not peak resident memory.

## Initial local baseline

Recorded 2026-11-04 on Windows, AMD Ryzen 9 5950X, Rust 1.97.1, optimized bench
profile, bundled fonts, reduced motion. These are observations from one run,
not cross-platform performance guarantees or thresholds for shared CI runners.

| CPU workload | Median | p95 |
|---|---:|---:|
| 100 rows of three controls | 2,492.2 us | 2,990.5 us |
| 100-message Markdown frame | 548.4 us | 730.0 us |
| 16 KiB of unmatched opening brackets | 104.5 us | 117.7 us |
| SVG rasterization, 64 pixels | 48.3 us | 54.6 us |
| Cached SVG frame | 3.4 us | 3.5 us |
| Gradient mesh, 2x pixel density | 4.7 us | 5.0 us |

| Warm workload | Allocations/op | Reallocations/op | Requested bytes/op |
|---|---:|---:|---:|
| Cached SVG frame | 20 | 0 | 8,846 |
| Button frame | 24 | 10 | 11,397 |
| Inline parse of 100 repeated paragraphs | 108 | 38 | 80,788 |

After painting 384 different SVG sizes, the cache retained 61 entries and
33,298,924 RGBA bytes, below its 32 MiB (33,554,432-byte) budget. Clearing it
reported zero entries and zero retained bytes. Unit tests enforce both the
entry and byte limits; renderer resources and transient uploads are separate.

## Rust code review follow-up

Rerun on 2026-11-04 with the same machine, Rust version, optimized profile and
allocation workload after borrowing Markdown link metadata:

| Warm workload | Allocations/op | Reallocations/op | Requested bytes/op |
|---|---:|---:|---:|
| Cached SVG frame | 20 | 0 | 8,846 |
| Button frame | 24 | 10 | 11,397 |
| Inline parse of 100 repeated paragraphs | 8 | 38 | 79,912 |

The inline workload removes 100 allocations per operation; its layout text is
still owned. This is an allocation measurement, not a new timing claim. Runtime
font clones also share their immutable bytes; unit tests verify that installation
and cloning retain the same backing allocation. Cache retention and clearing
matched the initial measurement.

## Token-boundary correctness follow-up

Measured again on 2026-11-04, Windows, Rust 1.97.1, optimized bench profile with the same workloads.
The existing pre-change allocation executable was run immediately before rebuilding the fixes.
Cached SVG/button allocation counts and cache retention were unchanged. The 100-paragraph parser
changed from 8 allocations, 38 reallocations and 79,912 requested bytes per operation to 9 allocations,
43 reallocations and 81,960 bytes. The additional temporary vector records code/link token ranges so
their contents cannot incorrectly close emphasis. Inputs without emphasis markers skip this pass.

The post-change timing smoke run reported the following values. These are single-run observations,
not a controlled speed comparison with the earlier baseline:

| CPU workload | Median | p95 |
|---|---:|---:|
| 100 rows of three controls | 2,929.2 us | 4,426.4 us |
| 100-message Markdown frame | 610.1 us | 898.0 us |
| 16 KiB of unmatched opening brackets | 168.0 us | 202.7 us |
| SVG rasterization, 64 pixels | 53.2 us | 78.2 us |
| Cached SVG frame | 3.4 us | 3.6 us |
| Gradient mesh, 2x pixel density | 5.3 us | 5.8 us |

## CommonMark migration smoke run

On 2026-11-05, Windows, Rust 1.97.1, optimized bench profile, the updated workloads completed with:

| CPU workload | Median | p95 |
|---|---:|---:|
| 100-message CommonMark frame | 2,900.3 us | 4,716.4 us |
| 16 KiB unmatched brackets, including layout | 26,183.5 us | 45,081.5 us |
| SVG rasterization, 64 pixels | 55.2 us | 111.3 us |
| Cached SVG frame | 5.7 us | 6.5 us |

These are smoke measurements taken while other validation builds ran on the same machine, not a controlled
comparison. The malformed-input workload now includes font layout and rendering; the former workload only
parsed inline markup. The allocation workload now renders 100 actual paragraphs, recording 3,042 allocations,
7 reallocations and 508,461 requested bytes per frame. Cached SVG and button allocation counts remain at
20 and 24 per frame. SVG cache retention remains bounded and clearing reports zero retained bytes.

## Parsed SVG reuse and resize workload

Recorded 2026-11-05 on Windows, AMD Ryzen 9 5950X, Rust 1.97.1, optimized bench profile, with no concurrent
validation builds. The same `svg_resize` workload ran immediately before and after parsed-document reuse.
Every measured frame uses a new square raster size: 32–231 pixels for one icon/the gallery logo (200 frames),
and 32–111 pixels for all 70 icons (80 frames). One initial frame warms egui and the documents. The cold
variant clears the entire SVG cache before each timed frame; the resize variant retains it. Timings include
CPU parsing, rasterization, texture-delta creation and egui frame construction, but exclude GPU uploads,
drawing and a native event loop. Assets overlap intentionally so only their creation costs are measured.

| Workload | Before median / p95 | After median / p95 |
|---|---:|---:|
| One icon, cold | 146.2 / 264.3 us | 132.6 / 239.2 us |
| One icon, resize | 137.4 / 277.4 us | 129.9 / 238.6 us |
| 70 icons, cold | 4,208.4 / 6,048.7 us | 4,087.6 / 6,023.7 us |
| 70 icons, resize | 5,038.6 / 7,154.6 us | 3,194.6 / 4,496.9 us |
| Gallery logo, cold | 356.7 / 641.0 us | 338.0 / 647.4 us |
| Gallery logo, resize | 348.1 / 637.8 us | 287.2 / 560.1 us |

The 70-icon resize median improved by about 37% in this run. Its maximum was 4,840.3 us after versus
8,144.1 us before. The cold results illustrate measurement variation and the cost that remains when documents
cannot be reused. These simple assets do not establish performance for arbitrary downloaded SVGs, filters,
embedded images, or larger rasters. Parsing and rasterization still run synchronously, without a latency bound.

After the 70-icon resize run the cache retained 256 raster entries (12,310,736 RGBA bytes), 70 parsed
documents and 20,754 source bytes. The document cache is limited to 128 documents and 4 MiB of source text;
the byte figure excludes parsed-tree allocations. Texture retention remains limited to 32 MiB. Tests cover
both eviction policies, parse failures, reuse after raster eviction, runtime source mutation and clearing.
Sources exceeding the source budget bypass retention. Applications accepting untrusted documents must
still constrain source length and complexity; bounded cache counts do not imply bounded process memory.

The separate allocation benchmark still reports 20 allocations, zero reallocations and 8,846 requested
bytes per cached SVG frame, matching the previous implementation. Clearing the cache resets texture,
document and source-byte counts to zero.

## Compare changes

Use the same toolchain, hardware, feature set and workload before and after a
change. Investigate repeatable increases rather than accepting a single noisy
sample. For application performance, also profile GPU time, visible versus
off-screen content, event processing, and large scrolling transcripts.

Build cost is separate. Use `cargo build --release --timings` with a fresh
target directory for each configuration, and record whether dependency downloads
and caches were warm. Compare `--no-default-features` against the components the
consumer actually enables. The external consumer CI check prevents development
dependencies from concealing missing runtime features.
