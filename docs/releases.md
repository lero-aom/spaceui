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
   egui/MSRV changes. Check the worktree and commit only the intended release files.
2. Run the checks in [CONTRIBUTING.md](../CONTRIBUTING.md), inspect visual baseline changes and manually smoke-test
   the native gallery on supported platforms. Record any checks that were not run.
3. Run the **Release** workflow with `publish = false` on the intended revision. It runs CI, lists package files,
   performs `cargo publish --dry-run`, and uploads the `.crate` for inspection. It does not publish a version.
4. Verify that the README's images and repository links resolve publicly. The repository carries screenshots;
   the package excludes `docs/` and `.github/`. Verify generated API docs and example compilation.
5. Create the matching `vX.Y.Z` tag. Run **Release** on that tag with `publish = true` only when ready to publish.
   The workflow rejects a branch or a tag that differs from `Cargo.toml`, reruns verification and uses the
   `crates-io` environment before requesting a short-lived publishing token.
6. Verify the published crate and docs.rs build, then write release notes that include migration instructions.
   Add publication badges only once their targets exist and have been verified.

## One-time repository-owner setup

The workflow files cannot configure account settings. Before automated publication, the crate owner must:

- Enable GitHub private vulnerability reporting, or publish another maintained private contact route.
- Configure the GitHub `crates-io` environment with appropriate reviewers and version-tag restrictions.
- Register this repository, `.github/workflows/release.yml` and environment `crates-io` as a trusted publisher
  in crates.io, following the [crates.io trusted publishing guide](https://crates.io/docs/trusted-publishing).
  If the first publication cannot use the account's trusted-publishing setup, publish the inspected release
  locally with an authorized crates.io account, then configure it for subsequent releases.
- Enable Actions/Dependabot as appropriate, select required checks in branch protection, and review the first
  macOS, visual and dependency-audit runs. These settings are not established by this change.

Publishing is irreversible for that version number. A dry run and a successful CI run establish build and
package evidence, not permission to publish. The workflow only publishes when a maintainer explicitly requests it.
