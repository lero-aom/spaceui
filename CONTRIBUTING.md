# Contributing to SpaceUI

Thanks for helping. Bug reports, fixes, new widgets and new themes are all welcome.

## Before you open a pull request

Run what CI runs:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo test --doc
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

The crate enables `missing_docs`, `missing_debug_implementations`, `unreachable_pub` and clippy's `pedantic` group
(see `[lints]` in `Cargo.toml`), and denies `unsafe_code` outside the one Windows call in `window.rs`.

For visual changes, attach screenshots of the gallery in both themes:

```sh
SPACEUI_SCREENSHOT=carbon.png cargo run --example gallery
SPACEUI_SCREENSHOT=deep.png SPACEUI_THEME=deep_space cargo run --example gallery
```

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

## Adding an icon

1. Add `assets/icons/<name>.svg` (24 x 24 viewBox, white strokes).
2. In `src/art/icons.rs`, add a variant and its two match arms (`name` and `svg`), keeping alphabetical order.
3. `cargo test` checks that the enum and the folder agree and that every icon renders.

If an icon comes from a third-party set, record its license in `NOTICE.md`.

## Releasing

1. Update `CHANGELOG.md` and the version in `Cargo.toml`.
2. Make sure `repository` is set in `Cargo.toml`, so crates.io links the README's screenshots.
3. `cargo package` and inspect the file list, then `cargo publish`.
