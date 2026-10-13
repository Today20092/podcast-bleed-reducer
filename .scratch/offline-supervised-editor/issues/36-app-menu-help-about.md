# Add the app menu, Help and About

Status: done
Type: task
Assignee: Codex app_menu_36
Blocked by: 34
Spec: ../../offline-supervised-migration/specs/app-menu-help-about.md

## Scope

Implement the [accepted app-menu specification](../../offline-supervised-migration/specs/app-menu-help-about.md): top-bar Menu with Settings, Keyboard shortcuts, Help, GitHub, Report a bug and About; centered Settings/About dialogs; accurate credits/version/license and allowlisted Copy version info. Reuse ticket 34's reference instead of creating duplicate shortcut help. Settings provides the entry point for ticket 37 without adding speculative controls.

Ticket 34 gates integration of the keyboard-reference entry and modal keyboard behavior. Coordinate shared GUI changes with ticket 23; its broader appearance review does not gate this behavior.

## Acceptance checks

- [x] Menu is discoverable and keyboard accessible; reference entry opens the ticket-34 reference and Ctrl+/ remains functional.
- [x] Settings/About use centered dialogs, preserve visible workspace context, manage focus/Escape and prevent background edits.
- [x] Help/repository/issue-chooser destinations match repository documentation; user-initiated actions open them or show an actionable failure. No issue is automatically submitted.
- [x] About shows actual version, available build identity, license and credits/references with usable long-content scrolling.
- [x] Copied text includes only app/build/platform fields and excludes private paths/project/audio/log data; clipboard failures are handled.
- [x] App tests cover routing, focus/modal suppression, allowed copy fields, missing build identity and action failures without network-dependent assertions.
- [x] Run appropriate Rust checks, formatting and strict Clippy; build/review through scripts/gui-dev.ps1. Inspect narrow/wide menu/dialog views and keyboard-only behavior, recording unavailable checks.

## Results

Implemented through `bb5ec90beb45530ad36135c67552f7441bde5c4c`: top-bar Menu, centered Settings/About, existing shortcut reference and legacy Help route, verified fixed repository destinations, allowlisted app/version/platform copy, actionable clipboard/link failure handling and scrollable license/credits. Missing build identity is omitted. No dependency added.

Validation: 97 library and 8 timeline tests passed, with 8 existing opt-in tests ignored. Strict all-target/all-feature Clippy, formatting and diff checks passed. `scripts/gui-dev.ps1 -BuildOnly` passed, including the final layout build (17.01 seconds). GUI review confirmed Alt+M/Enter Settings and Escape, About actual version `0.1.0-alpha.1`, Windows x86_64 platform, copy success feedback and End scrolling. About credits initially clipped; bounded content and scrollbar gutters fixed it, and wide 2560×1392/restored 1026×932 review passed. Ctrl+/ opens the reused reference at restored width; its matching gutter correction passed source checks/build and will be visually rechecked in ticket 37's final pass. Workspace retained 36 examples and stopped transport.

Unavailable checks: macOS/Linux external-launch behavior, forced physical clipboard/browser failures and a complete keyboard-only pass through every external action. Failure and routing tests are network independent. Diagnostics and refresh-rate controls remain deferred.

## Comments

2026-10-12 follow-up: Ticket 37 GUI review passed the final reference gutter at restored width, including End scrolling and Enter dismissal. Light About credits and narrow Menu/Settings also passed.

2026-10-12: Owner confirmed Q25–Q33 and authorized this specification and ticket. No implementation performed while creating records.
