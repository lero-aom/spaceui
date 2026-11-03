# Fuzzing

The isolated `fuzz/` Cargo workspace contains two `cargo-fuzz` targets:

- `markdown` mutates Unicode and malformed inline markup, checking that parsing
  does not panic and every returned formatting range is valid UTF-8.
- `geometry` supplies arbitrary floating-point bits to rounded outlines,
  gradients and polygon meshes, checking finite vertices, valid triangle indices
  and bounded rounded-rectangle subdivision. This includes invalid values and
  extreme finite values; rejected geometry may return an empty mesh.

These targets supplement deterministic regression and GUI tests. They do not
prove Markdown conformance, accessibility, visual correctness or the absence of
bugs. Input sizes are bounded to keep individual fuzz iterations useful.

## Run locally

Use a nightly Rust toolchain and `cargo-fuzz`. The
[official setup guide](https://rust-fuzz.github.io/book/cargo-fuzz/setup.html)
describes platform requirements; Linux is the CI baseline. The commands below
use `cargo-fuzz` 0.13.2, verified against the
[upstream manifest](https://github.com/rust-fuzz/cargo-fuzz/blob/main/Cargo.toml).
The `libfuzzer-sys = "0.4"` dependency follows the
[upstream project template](https://github.com/rust-fuzz/cargo-fuzz/blob/main/src/templates.rs).

```sh
cargo install cargo-fuzz --version 0.13.2 --locked
cargo +nightly fuzz run markdown fuzz/seeds/markdown -- -max_total_time=60 -max_len=65536
cargo +nightly fuzz run geometry fuzz/seeds/geometry -- -max_total_time=60 -max_len=284
```

For a longer run, omit `-max_total_time=60`. Seed directories are small checked-in
starting inputs. Copy them to `fuzz/corpus/<target>` before a long campaign if you
want generated corpus inputs kept in the ignored directory. To compile the Rust
harnesses without running a fuzz campaign:

```sh
cargo check --manifest-path fuzz/Cargo.toml --bins
```

That check is not equivalent to a sanitizer-instrumented fuzz run. A nightly
toolchain and supported native C++ build tools are needed for fuzzing; see the
[official platform guidance](https://rust-fuzz.github.io/book/cargo-fuzz/setup.html).

## Handle a finding

`cargo-fuzz` reports the crashing input under `fuzz/artifacts/<target>/`. Replay
the exact file, then minimize it before fixing the underlying behavior:

```sh
cargo +nightly fuzz run markdown fuzz/artifacts/markdown/<crash-file>
cargo +nightly fuzz tmin markdown fuzz/artifacts/markdown/<crash-file>
```

Use `geometry` for geometry findings. Add the minimized input or an equivalent
focused case to the ordinary test suite so future changes exercise the
regression without requiring the fuzz toolchain. See the
[official tutorial](https://rust-fuzz.github.io/book/cargo-fuzz/tutorial.html)
and [command guide](https://rust-fuzz.github.io/book/cargo-fuzz/guide.html).
