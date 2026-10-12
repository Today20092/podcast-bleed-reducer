# Tokenize colors and add persistent dark/light appearance

Status: done
Type: task
Assignee: Codex appearance_37
Blocked by: 36
Spec: ../../offline-supervised-migration/specs/semantic-dark-light-themes.md

## Scope

Implement the [accepted appearance specification](../../offline-supervised-migration/specs/semantic-dark-light-themes.md) across application widgets and custom drawing. Keep the current Dark default, add Light and expose an immediately applied, app-wide saved choice in Settings. Preserve semantic timeline meaning, readable contrast, focus and all audio/project behavior.

Ticket 36 provides the Settings entry point. Coordinate token/palette changes with existing GUI polish and integrated navigation/gain-strip behavior; do not restyle or redesign their interactions. Existing completed GUI foundation is reused.

## Acceptance checks

- [x] Application-owned UI colors use semantic roles consistently across workspace, menus/dialogs/tooltips and custom waveform/gain drawing; platform-owned dialogs remain platform-controlled.
- [x] Settings offers Dark/Light only, applies immediately and defaults to existing Dark. Light remains readable and preserves semantic distinctions.
- [x] App-wide choice survives restart and project switching; missing/invalid preferences safely default, persistence failure is visible and the application remains usable.
- [x] Switching themes does not modify sources, settings/labels/edits, selection, audio/transport or export identity and does not rerun analysis/decoding.
- [x] Preference/theme behavioral tests cover defaults, valid round-trip, invalid data, failure and project independence. Token-coverage review includes custom drawing, not just built-in controls.
- [x] Measure text/focus/control contrast; review both themes in narrow/wide and selected/disabled/stale/loading/error/import/export/gain-edit states. Record unavailable checks.
- [x] Run appropriate Rust checks, formatting and strict Clippy and build/review with scripts/gui-dev.ps1. Record results before marking done.

## Results

Implemented in `7bcaaed754ea2ec891b1c6a7d436c750c2ca13d9`: internal semantic colors across widgets and waveform/gain drawing, Dark default and Light, immediate keyboard-accessible Settings choice, validated app-local preferences, visible load/save failures and drawing-only cache invalidation. New project preserves the app preference and any unsaved choice/error. No dependency or theme framework added.

Validation: 120 scoped tests passed (101 library, 11 editor workflow, 8 timeline), with 9 existing opt-in checks ignored; final exact-source appearance tests, strict all-target/all-feature Clippy, formatting and diff checks passed. `scripts/gui-dev.ps1 -BuildOnly` passed in 27.61 seconds. Tests measure at least 4.5:1 text and 3:1 focus/control contrast in both palettes, including actual composited overlays. Independent coverage review found no color literals left in waveform/gain drawing; app literals are contrast fixtures.

GUI review passed wide 2560×1392/restored 1026×932 Dark/Light, stale/disabled/empty states, Settings keyboard selection, gain drawing and manual points, retained selection through switching, Light label popup/preparing state, idle gain Escape deselection, About credits/reference gutters, and Light import text focus/Escape cancellation. Original playback stayed active and advanced through a theme switch. Light survived a full restart and New project reset. Returned the saved preference to Dark. The development project retained its 36 examples and sound values on reopening; no labels/imports/exports were committed during this review.

Unavailable visual checks: forced preference-save failure and separate malformed preference error presentation, export modal/running/error states, every tooltip/focus combination, enlarged text and cross-platform GUI. Automated tests cover preference failure, fallback and project/audio/export identity preservation. Physical listening and existing opt-in qualification checks were not run. OS-theme following, custom colors and theme imports remain deferred; stable release gates are unchanged.

## Comments

2026-10-12 integration review: Independent Standards and Spec reviews compared `main` (`0de48e4`) to `7bcaaed` across all six selected tickets and reported zero actionable findings. Broad all-target regression compilation initially hit Windows paging-file exhaustion; `cargo test --all-targets --locked -j 1` then passed 250 tests with zero failures and 15 existing opt-in checks ignored.

2026-10-12: Owner confirmed semantic tokens, Dark/Light and application-local appearance persistence, and authorized specification/ticket creation.
