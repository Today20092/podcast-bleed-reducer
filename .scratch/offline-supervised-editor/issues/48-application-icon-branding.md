# Install the approved CleanSlate identity

Status: claimed
Type: task
Assignee: Codex
Blocked by: none

Owner approved Open channel (direction C) on 2026-10-17 and requested adding it to GitHub and the application. This is an independent presentation/packaging change; no audio or project behavior changes.

## Scope

- Promote the approved mint/charcoal C and audio bars to versioned assets.
- Embed the Windows executable icon and configure the Iced window icon.
- Keep Linux packaging on the shared SVG and provide the macOS bundle icon.
- Add the theme-appropriate outlined logo to the GitHub README.
- Publish a scoped PR from current main, preserving unrelated checkout work and repository protections.

## Acceptance

- Windows GUI development build succeeds using scripts/gui-dev.ps1.
- Embedded executable resources include the approved multiresolution icon.
- Embedded window pixels construct a valid Iced icon.
- macOS package validation/fixtures require the bundled icon; packaging tests pass.
- README image references and SVGs resolve; formatting checks pass.
- Record native checks and any unavailable visual/platform verification before marking done.

## Results

- Promoted the approved vector and outlined wordmarks into assets, with reproducible ICO/RGBA/ICNS/PNG generation and structural assertions.
- Added Windows resource embedding through winresource 0.1.34, and embedded 64px Iced pixels without an image-decoder runtime dependency.
- Added the README theme variants, macOS bundle icon/plist entry and required-icon packaging fixtures. Existing Linux SVG packaging uses the updated master.
- Passed asset generation/assertions, formatting, diff whitespace, packaging integrity/download checks (including missing macOS icon rejection), native-workflow regressions, protected-branch automation regressions and release-status fixtures.
- Windows GUI build/resource inspection and the dedicated embedded-icon Rust test are in progress. Hosted four-platform checks and native macOS/Linux visual inspection remain pending.
