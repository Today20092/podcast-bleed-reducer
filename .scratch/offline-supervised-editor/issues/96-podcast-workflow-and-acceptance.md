# Podcast workflow parity and autonomous acceptance evidence

Status: claimed
Assignee: Codex
Readiness: ready-for-agent
Type: task
Blocked by: 90, 95
Related: 85, 86

## Scope and acceptance

Continue Podcast ticket 80's acceptance and UI workflow with a machine-checkable/native split. Apply the UI contract only within the product workspace and preserve 84's navigation regressions. Build a parity checklist against the desktop workflow, fixing demonstrated gaps in capture/label/analyze/review and recovery rather than cloning the whole application shell.

Acceptance:
- [ ] Test source-time capture, looping/replayed changed/unchanged audio, gaps, labels and fingerprint invalidation, explicit analysis/cancellation and shared participants.
- [ ] Actual-format hosts verify cached envelope playback versus host offline rendering, first/last sample positions, latency reporting, state restore, changed-source fallback and Original/Processed/Removed output.
- [ ] Headless compact/full/DPI and interaction tests cover quick selection, zoom/pan/fit/navigator, Center/Catch up/Lock, label edits and actionable errors.
- [ ] Record exact machine evidence against the unresolved 80 checklist; retain subjective quiet-speech/overlap/transition listening and unavailable native host checks as pending.
- [ ] Do not mark 80 done or unblock 81–83 merely because synthetic tests pass. If native listening acceptance is the only blocker, checkpoint it and continue independent Gate/research tasks.
- [ ] No owner source overwrite, project save or export replacement; tests use disposable copies and explicitly named outputs. All heavy steps use 88.

## Results

2026-10-27 reviewed checkpoint: production source `4558474` and test-only follow-up `22b2d9f` passed 69 tests, strict Clippy, normal CLAP/VST3 ABI checks and the separately guarded debug-only acceptance-driver workflow. The driver demonstrates non-unity shared-envelope/offline parity, alignment, monitoring, source invalidation and real-format state recreation; it does not qualify native end-user Analyze/listening. Both artifacts were copied outside the worktree with every file hash checked: `%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/plugin-products/Podcast/4558474-normal-b2c3a2332976` and `4558474-acceptance-only-d7c4fd823fbb`. Six actual-widget CPU captures are retained under `Podcast/Workflow96Captures/final-4558474-d8f666d03c4f49879bc66f7490da9fed/`; these are synthetic-data captures, not native DAW screenshots.

Independent review found two remaining issues: a pending conflicting-label confirmation can target the old range after selection changes, and offscreen lanes still generate geometry. Fixes and focused regressions are in progress in `codex/podcast-workflow-96`; this ticket stays claimed until verification/re-review and product-lane integration. The prior preserved artifacts/captures predate those fixes. Latest light measurement was 14.10 GiB available physical / 12.17 GiB commit headroom, below the unchanged 14 GiB heavy-work start requirement for a new job. Do not relax policy or loop retries; record the fix-validation guard result. Gate 93/94 and native/listening ticket 80 remain open; 81–83 remain blocked. Earlier intermediate 65-test/unrun-driver descriptions below are historical.

Claimed 2026-10-26 after 90/95 completion, based on Podcast integration da9b297. Scoped branch codex/podcast-workflow-96 reuses the preserved Podcast worktree and candidate; no whole-desktop merge. Implementation is queued behind the single active Gate detector compiler/measurement job to keep local RAM low.

Existing 95 evidence supplies 62 passing Session/event tests and basic format/state/latency coverage, not this ticket's complete acceptance. Confirmed gaps include labeling/sound actions clipped by eight lanes, the accepted independently scrolling lane viewport and compact/full/DPI interactions, and actual-format cached-envelope versus offline-render/changed-source/monitoring qualification. Keep every unchecked item open until demonstrated. Native listening, real-host PDC/save/export and ticket 80 remain separate; no owner source or installed output will be changed.

