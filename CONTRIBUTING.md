# Contributing to SpaceUI

Thanks for helping. Bug reports, fixes, new widgets and new themes are all welcome.

## Before you open a pull request

Run what CI runs:

```sh
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features --lib --test widgets --test shell --test chat --examples
cargo test --locked --all-features --doc
RUSTDOCFLAGS="-D warnings" cargo doc --locked --all-features --no-deps
cargo check --locked --lib --no-default-features
cargo test --locked --lib --no-default-features
cargo test --locked --test visual --all-features
cargo bench --locked --bench workloads --all-features
cargo bench --locked --bench allocations --all-features
python .github/scripts/check_consumer.py
```

The environment-variable syntax above is for a POSIX shell. In PowerShell, set
`$env:RUSTDOCFLAGS = '-D warnings'` before running `cargo doc`, and use `$env:NAME = 'value'`
for the screenshot variables below. Remove an override when finished with `Remove-Item Env:NAME`.

CI also checks each optional feature by itself on the MSRV, packages the crate, checks dependency
advisories/licenses/sources with `cargo deny --all-features --locked check`, and compares the public API
with the latest published version. The external consumer check creates temporary applications outside
the workspace; it checks the minimal example and runs a core-only frame without development dependencies.

The crate enables `missing_docs`, `missing_debug_implementations`, `unreachable_pub` and clippy's `pedantic` group
(see `[lints]` in `Cargo.toml`), and denies `unsafe_code` outside the one Windows call in `window.rs`.

For visual changes, attach screenshots of the gallery in both themes:

```sh
SPACEUI_SCREENSHOT=carbon.png cargo run --example gallery
SPACEUI_SCREENSHOT=deep.png SPACEUI_THEME=deep_space cargo run --example gallery
```

The rendered regression tests use bundled fonts and committed PNG baselines under `tests/snapshots/`.
They require a usable wgpu graphics adapter; Linux CI uses Mesa's software Vulkan renderer. Review
generated difference images when a test fails. If the change is intentional, run
`UPDATE_SNAPSHOTS=force cargo test --test visual --all-features`, inspect every changed PNG, and commit
the approved baselines with the code. Never set `UPDATE_SNAPSHOTS` in CI. A passing snapshot is not a
substitute for checking keyboard navigation, screen-reader labels, IME composition or native window behavior.

For performance changes, record the benchmark output, Rust version, build mode, platform and workload.
Compare before and after on the same machine. Hosted-runner timings are not stable enough for a universal
frame-time threshold; the benchmarks provide measurements, not a performance guarantee.
Allocation instrumentation runs in a separate benchmark so it does not bias the timing workload.
For build costs, run `cargo build --release --timings` in a fresh target directory for each feature set.
See [docs/performance.md](docs/performance.md) for the workload definitions and recorded baseline,
and [docs/fuzzing.md](docs/fuzzing.md) for the parser/geometry fuzz targets run briefly in CI.

## Design rules

- **Colours come from the palette.** Widgets read `Theme::of(ctx)` and use a `Palette` role (`accent`, `field`,
  `border_soft`, ...), never a literal hue. If no role fits, propose one.
- **Sizes come from `Metrics`** where they are shared (radii, rail and title bar sizes).
- **Edges stay smooth.** Paint gradients and custom fills with `paint`, which feathers their edges; egui's own shapes
  are already anti-aliased.
- **Art is SVG.** Draw icons on a 24 x 24 grid with white 1.8-point strokes, round caps and joins, so tinting works.
- **Widgets are functions** that allocate their own space and return egui's `Response` when interactive. Builders
  (`Shell`, `Composer`) are for things with several options.
- **Document every public item**, with an example where usage isn't obvious. Examples run as doc tests.
- **Test behaviour headlessly.** `tests/common` simulates clicks and key presses; add a test for new interaction.
- **Keep controls accessible.** Supply names for icon-only controls, preserve keyboard focus and activation,
  and test disabled states. Prefer the labeled switch/slider helpers when an application-specific name is available.
- **Keep feature subsets independent.** A core-only consumer must not depend on SVG or native-window code.
- **Review dependency changes.** Keep `Cargo.lock` updated and explain any narrowly scoped advisory exception.

## Adding an icon

1. Add `assets/icons/<name>.svg` (24 x 24 viewBox, white strokes).
2. In `src/art/icons.rs`, add a variant and its two match arms (`name` and `svg`), keeping alphabetical order.
3. `cargo test` checks that the enum and the folder agree and that every icon renders.

If an icon comes from a third-party set, record its license in `NOTICE.md`.

## Releasing

Follow [docs/releases.md](docs/releases.md). The Release workflow defaults to a package dry run and
requires an explicit publish selection, a matching version tag and successful CI to publish.

## Collaboration

Open a focused issue or pull request with a reproducer, the affected platform and feature selection, and
the expected behavior. Discuss API changes before expanding the public surface. Treat contributors
respectfully, keep feedback about the work, and avoid personal attacks. The repository owner reviews
contributions; there is no guaranteed review time or staffed support service. Security reports use
[SECURITY.md](SECURITY.md), not the ordinary issue template.
