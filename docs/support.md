# Support and validation

SpaceUI targets desktop applications built with egui 0.37. Rust 1.95 is the minimum supported Rust version.
The workflow definitions describe checks to run after a change is pushed; adding a job does not establish
that it has passed on the current revision.

| Target | Automated checks | Remaining manual validation |
|---|---|---|
| Windows hosted runner | Stable/MSRV compilation, unit and interaction tests, examples, docs; stable packaging | Window chrome, resizing, multi-monitor DPI, font fallback and assistive technology |
| Linux hosted runner | Same checks; rendered snapshots with Mesa Vulkan | X11/Wayland window managers, clipboard, IME and assistive technology |
| macOS hosted runner | Stable/MSRV compilation, unit and interaction tests, examples, docs; stable packaging | Native window behavior, font rendering, IME and VoiceOver |
| Browser/WASM, mobile and other targets | No current CI coverage or support promise | Target-specific integration and packaging |

The CI runner's architecture and OS image are determined by GitHub's `*-latest` labels. Coverage does not
extend automatically to older OS versions, every CPU architecture, every display scale or every renderer.
Native screenshot and window-handle helpers target desktop backends. A browser port needs its own backend
integration and checks; use `default-features = false` as a starting point, not as a compatibility guarantee.

## Accessibility and localization

Custom controls expose egui accessibility metadata and support documented keyboard interactions. Whether
those reach a screen reader depends on the application's egui integration and platform backend. A complete
application still needs keyboard-only and screen-reader testing on its supported systems.

Pass meaningful text to labeled controls and set application-specific labels and hints. Provide font files
covering the scripts your application uses. Bundled fallback fonts do not guarantee coverage of every script
or correct bidirectional shaping. Right-to-left layout, complete localization and OS accessibility preference
detection are not automatically provided by SpaceUI.

## Reporting bugs

Use [GitHub issues](https://github.com/lero-aom/spaceui/issues). Include SpaceUI and egui versions, selected
features, `rustc -Vv`, OS, display scaling, rendering backend and a small reproducer. A screenshot helps with
rendering problems; keyboard and accessibility reports should include the exact sequence and assistive tool.
Use [SECURITY.md](../SECURITY.md) for vulnerabilities. Support is best effort, with no response-time guarantee.
