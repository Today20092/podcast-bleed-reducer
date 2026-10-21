# 04: Report the actual graphics environment safely

Status: done
Assignee: perf04_implementer
Type: task
Blocked by: None (can start immediately)
Source proposal: P7

**What to build:** Expose useful graphics backend/adapter and available display scaling/timing information through the established diagnostic or version-information flow so machine-specific rendering issues can be reproduced.

## Acceptance criteria

- [x] Fetch current documentation for the installed GUI stack and report observed active backend/adapter rather than assuming the configured default is active.
- [x] Reuse the established diagnostic flow; add no new renderer, fallback setting, dependency upgrade, network collection or automatic publication.
- [x] Handle unsupported/unavailable graphics/display fields explicitly and keep copied reports free of project/audio paths and recording contents.
- [x] Verify the public diagnostic/report boundary and perform a current-machine native capture when available; document which fields cannot be obtained.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. The owner confirmed the existing application, playback-output, shared preview/export envelope and benchmark/report test boundaries on 2026-10-21. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

2026-10-21 implementation: `7c81f6b` adds observed graphics diagnostics to the explicit About → Copy version info action. Iced 0.14 Context7 documentation and the pinned installed API sources confirm `system::information()` supplies the active compositor's backend/adapter; `window::latest()` and `window::scale_factor()` supply the current window scale. Reports retain the existing application/version fields and add only those graphics fields. Refresh is explicitly `unavailable (current GUI API)`; no redraw interval is presented as display refresh. The collected report also appears in About so clipboard failure retains a manual-copy path.

An identity token guards both collection and clipboard replies. Closing/reopening About invalidates old replies, preventing a stale collection from writing the clipboard or a stale clipboard acknowledgment from finishing a newer request. Clipboard verification failure retains actionable feedback. Hardware fields strip control characters before rejecting path-like input, are capped at 160 Unicode characters, and reject blank/private path values. Nonpositive/nonfinite scale is unavailable. No project/session/audio values or complete system inventory enter the report.

The existing Iced version enables its `sysinfo` feature. All 572 previously locked package name/version pairs were preserved; necessary new support packages only: sysinfo 0.33.1, ntapi 0.4.3, rayon 1.12.0, rayon-core 1.13.0, crossbeam-deque 0.8.8, crossbeam-epoch 0.9.21. No renderer or build-profile changes.

TDD evidence at the confirmed application/report boundary: the observed-report test first failed with the formatter missing; the stale-copy test failed because the old reply finished the newer request; the sanitizer test failed because a control character concealed a private `C:/` path. Each then passed after its corresponding implementation. `cargo test --lib app::app_menu_tests` passed all seven tests after merging integration `83ba484` (own merge `0038d2d`). `cargo fmt --all -- --check` and `git diff --check` passed. Shared target was exclusively leased; own library was re-saved without content changes before its fresh build.

Strict scoped validation also passed: `cargo clippy --lib --all-features -- -D warnings` using the locked/offline shared-target settings. The exclusive Cargo lease was then released.

Remaining acceptance: the parent will capture this machine's actual About report from the combined native build and run the combined checks. No actual adapter/backend/scale or physical timing measurement is claimed by these controlled tests. This ticket remains claimed until that native capture is recorded; the GUI API does not expose monitor refresh or physical presentation latency.

2026-10-21 integrated native acceptance supersedes that pending handoff: at source `7985942`, About → Copy version info displayed Vulkan, NVIDIA GeForce RTX 3070 Ti, window scale 1.00x and `Display refresh: unavailable (current GUI API)`, then confirmed `Version information copied.` No project/audio path appeared in the copied-report display. The integrated local checks passed (338 tests, 19 ignored; strict Clippy and script fixtures). Physical display timing and other-machine adapters remain unmeasured, rather than inferred from the software observations.

