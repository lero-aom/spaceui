# Compatibility and releases

## API policy

SpaceUI is pre-1.0. Patch releases preserve the supported public API; incompatible changes require a minor
version increase and migration notes in the changelog. Once 1.0 is released, incompatible changes require a
major version increase. Public fields and exhaustive enums are part of the API. An upstream egui update can
change exposed types and therefore needs compatibility review even if SpaceUI's function names stay the same.

Prefer an additive replacement and a documented deprecation before removing a commonly used API. Removal
must wait for the next breaking release; there is no promised time-based deprecation window. Internal rendering
details, exact pixels and benchmark timings are not stable API, but changes still need regression review.

The MSRV is 1.95. It is raised only in a minor release and only when the supported egui version requires it.
The latest release receives maintenance; older release lines have no promised backport period.

CI resolves the latest non-yanked stable release from crates.io and checks default and all-feature APIs with
`cargo-semver-checks`. Before the first publication, an actual registry 404 produces an explicit first-release
message. Network failures, malformed responses and all-yanked releases fail the job rather than disabling
checking. If only prereleases exist, the newest published prerelease is used. Maintainers must review changes
the tool cannot infer, including behavior, native integration, default feature changes and MSRV changes.

## Prepare a release

1. Update the manifest version, lockfile and changelog. Describe breaking changes, migration steps and supported
   egui/MSRV changes. Update the README's version-pinned repository and screenshot links to the new tag.
   Check the worktree and commit only the intended release files.
2. Run the checks in [CONTRIBUTING.md](../CONTRIBUTING.md), inspect visual baseline changes and manually smoke-test
   the native gallery on supported platforms. Record any checks that were not run.
3. Run the **Release** workflow with `publish = false` on the intended revision. It runs CI, lists package files,
   performs `cargo publish --dry-run`, and uploads the `.crate` for inspection. It does not publish a version.
4. Inspect the packaged sources and generated API documentation. The repository carries screenshots;
   the package excludes `docs/` and `.github/`. The Release workflow builds documentation and examples and
   runs doctests from the extracted archive.
5. Create and push the matching `vX.Y.Z` tag, then verify the README's version-pinned links. For the first
   publication, follow the local publishing steps below. For later releases with Trusted Publishing configured,
   run **Release** on that tag with `publish = true` only when ready to publish. The workflow rejects a branch
   or a tag that differs from `Cargo.toml`, reruns verification and uses the `crates-io` environment before
   requesting a short-lived publishing token.
6. Verify the published crate and docs.rs build, then write release notes that include migration instructions.
   Add publication badges only once their targets exist and have been verified.

## First publication: 0.1.0

The [crates.io Trusted Publishing prerequisites](https://crates.io/docs/trusted-publishing) require an existing
published crate. The first release therefore uses a crates.io API token; the workflow's `publish = true` mode
is for subsequent releases after configuration. The `publish = false` verification mode needs no token.

1. Sign in to [crates.io](https://crates.io/) with the account that should own SpaceUI and verify its email address.
   Create a short-lived API token with permission to publish the new `spaceui` crate.
2. Use a clean checkout of the prepared tag. These commands work in PowerShell and a POSIX shell:

   ```sh
   git fetch origin --tags
   git switch --detach v0.1.0
   git status --short
   cargo publish --locked --dry-run
   cargo login
   ```

   `git status --short` must print nothing, and the dry run must succeed. Enter the token only at Cargo's
   interactive prompt; do not put it in a command argument, commit, issue or chat message.
3. When ready to upload the checked version, run:

   ```sh
   cargo publish --locked
   ```

4. Verify [spaceui on crates.io](https://crates.io/crates/spaceui) and
   [the 0.1.0 API documentation](https://docs.rs/spaceui/0.1.0/spaceui/), including README images and links.
   Publication and the docs.rs build can take time to appear. Revoke the bootstrap token when finished and
   run `cargo logout` to remove its local copy. Use `git switch main` to return to development.

The release is pre-1.0. Automated checks cover the platforms in [support.md](support.md); native Linux/macOS
interaction, IME, screen-reader and multi-monitor testing still need suitable systems and human validation.
Passing CI does not mark those checks complete.

## Repository-owner setup for subsequent automated releases

The workflow files cannot configure account settings. Before automated publication, the crate owner must:

- Enable GitHub private vulnerability reporting, or publish another maintained private contact route.
- Configure the GitHub `crates-io` environment to allow selected tags matching `v*`. Required reviewers are an
  optional additional control; the workflow itself requires an explicit `publish = true` dispatch.
- After the first publication, add a GitHub Trusted Publisher in the crate's settings using repository owner
  `lero-aom`, repository name `spaceui`, workflow filename `release.yml`, and environment `crates-io`, following
  the [crates.io trusted publishing guide](https://crates.io/docs/trusted-publishing). The filename field takes
  `release.yml`, not the full `.github/workflows/release.yml` path.
- Enable Actions/Dependabot as appropriate, select required checks in branch protection, and review the first
  macOS, visual and dependency-audit runs. These settings are not established by this change.

Publishing is irreversible for that version number. A dry run and a successful CI run establish build and
package evidence, not permission to publish. The workflow only publishes when a maintainer explicitly requests it.
