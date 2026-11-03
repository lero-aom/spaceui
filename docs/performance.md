# Performance measurements

Run the timing and allocation workloads separately:

```sh
cargo bench --locked --bench workloads --all-features
cargo bench --locked --bench allocations --all-features
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
