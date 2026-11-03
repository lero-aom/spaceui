# Security policy

Security fixes target the latest published SpaceUI release. Older release lines do not have a promised
backport or support period. When feasible, fixes retain compatibility within the affected release line;
the advisory and changelog describe required upgrades and any workarounds.

## Reporting a vulnerability

Check the repository's [Security page](https://github.com/lero-aom/spaceui/security) for the **Report a
vulnerability** option. If private reporting is enabled, use it to provide affected versions, a minimal
reproducer, impact, platform and feature selection. This file does not imply that GitHub private reporting
has already been enabled.

If the option is unavailable, open an issue titled **Private security contact requested**, without exploit
details or sensitive data, and ask the repository owner to enable a private reporting channel. Do not put
credentials, private application data or an uncoordinated exploit in a public issue. No response-time SLA
is currently promised.

## Scope and checks

SpaceUI renders UI; applications own network requests, authorization, storage, file access and AI backends.
Raw HTML in the compact Markdown renderer is not executed. Link destinations still lead to external content;
an application is responsible for its own navigation policy. Review untrusted text and assets at the application boundary.

Dependency CI checks RustSec advisories, yanked versions, licenses and sources on changes and weekly.
Checks can identify known issues; they are not a security audit. Exceptions require a specific justification
and should be removed when an update is available. The only intended unsafe Rust is the documented Windows
window-attribute call, which should remain small and separately reviewed.
