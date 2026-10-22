# Replace waveform dropdown and quiet completed analysis status

Status: done
Type: task
Assignee: Codex
Blocked by: 59, 61
Labels: ready-for-agent

## Owner request

2026-10-22: reconsider the waveform dropdown as icon buttons and move it to a more suitable location. Use Impeccable design guidance. The Processed audio ready panel contains too much text and timing detail.

## Scope and acceptance

- Replace the three-choice waveform dropdown with equal-size segmented icon buttons grouped at the right of the timeline toolbar, distinct from Lock/Center/Meters. Preserve Original/Processed/Both semantics, selected state, keyboard focus and descriptive hover hints.
- Reduce completed status to a quiet Ready indicator. Put timing detail behind a Details disclosure; remove the redundant normal-state instruction to audition and export. Preserve actionable setup/stale guidance, active analysis progress/cancellation and errors.
- Reuse native Iced controls, theme roles and waveform cache behavior. No audio, persistence or export changes; no toolkit migration or new app dependencies.
- Verify selection and disclosure behavior with relevant tests and compact/wide native inspection. Run scripts/check.ps1 -Local and scripts/gui-dev.ps1. Record source/build identity and unavailable checks before marking done.

## Results

Final follow-up validation: scripts/check.ps1 -Local passed 353 tests, zero failures, 20 existing ignored, formatting, strict all-target/all-feature Clippy and script fixtures. An overlapping check initially could not replace the reopened GUI executable (Windows access denied); closing the unchanged test window and rerunning passed. The final scripts/gui-dev.ps1 rebuild/launch succeeded, with executable SHA256 4405D8275B827854C75B0C00B8EF6AD091584E2DBBFEC58A0795FDE1E7C59024, superseding the earlier follow-up binary hash below. Source is development base f9fbe33 plus this correction. No project edits were saved.

2026-10-22 follow-up: reopened for the owner's request to organize expanded Details into aligned labeled timing rows instead of a wrapped sentence. Keep Details collapsed by default; preserve informational timing data separately from actionable messages.

Follow-up delivered: total analysis time is separated from five aligned stage/duration rows. Native Dark checks at 1026×932 and 2560×1392 show all rows fitting with Details open. The completed-analysis test checks structured stage data and collapsed/reset behavior. Impeccable layout detector returned no findings; native Rust coverage is not implied. Rebuilt through scripts/gui-dev.ps1 from development base f9fbe33 plus this correction, then opened the retained development project. Executable C:/Users/User/AppData/Local/CleanSlate/BuildCache/debug/rust-app.exe; SHA256 97016A8C6646CA8FEB4BB7485BD1FD3CC44BB8A5EA863403A7E217DD5725993E. Light/enlarged-text follow-up checks were not repeated.

Claimed on development before editing; dependencies 59/61 are done. Installed user-level Impeccable 4.7.0 using the requested official npx installer with Codex provider and no hooks. Its scoped distill guidance is applied to the incumbent Rust/Iced interface; no new product/design onboarding is needed for this refinement. Menu redesign remains in ticket 60.

Implemented three native icon buttons with selected states and delayed descriptive hover hints at the right of the timeline toolbar, separated from navigation. Authored waveform SVGs use the existing themed SVG path; no application dependency was added. Completed analysis now stores its informational timing summary separately from actionable feedback, keeping Ready compact and exposing timings through Details. Errors, stale/source guidance and active analysis/cancellation remain visible.

Final scripts/check.ps1 -Local passed: 352 tests, zero failures, 20 existing ignored across 38 targets, plus formatting, strict all-target/all-feature Clippy and local-build/CI/packaging/release-status script fixtures. New completion/disclosure tests verify collapsed timing storage and reset; prior waveform drawing/cache and transport tests still pass. The Impeccable detector returned no findings for the named Rust files; this does not establish native Rust UI coverage. Native verification/build identity follows below.

Native verification passed at 1026×932 and 2560×1392: the three waveform buttons fit and Processed selected feedback/hover hint works. Completed analysis presents Ready with collapsed Details; opening/closing Details exposes/hides timings. The compact panel also fits without the former instruction/timing block. Light and Dark wide-window inspections passed; Dark was restored. Enlarged-text and a complete keyboard-only pass were not performed.

scripts/gui-dev.ps1 built and opened the existing three-track development project from development base 8c6a151 plus this change. Executable: C:/Users/User/AppData/Local/CleanSlate/BuildCache/debug/rust-app.exe; SHA256: 8EE855EAFA695F2D35D6946D31A3923CF41FA8637CC1E07E812DA9DF6504AA7C. The app remains open, stopped, ready for owner testing. No source audio or project edits were saved during these checks.
