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

2026-10-27 latest resumption: source remains `6070569`; no code changed. Default guard readiness receipt `20261027-093309-2983749bfe4e46f383b2142619d5ba65.json` refused startup before a process launched, at 3.59 GiB available physical / 1.30 GiB commit headroom; cleanup verified. The corrected focused regression, final Clippy/build/captures and integration remain unchecked. No retry loop, threshold relaxation, service restart or additional reboot occurred. This supersedes older resource observations only; prior source-specific passing evidence remains unchanged.

Claimed 2026-10-26 after 90/95 completion, based on Podcast integration da9b297. Scoped branch codex/podcast-workflow-96 reuses the preserved Podcast worktree and candidate; no whole-desktop merge. Implementation is queued behind the single active Gate detector compiler/measurement job to keep local RAM low.

Existing 95 evidence supplies 62 passing Session/event tests and basic format/state/latency coverage, not this ticket's complete acceptance. Confirmed gaps include labeling/sound actions clipped by eight lanes, the accepted independently scrolling lane viewport and compact/full/DPI interactions, and actual-format cached-envelope versus offline-render/changed-source/monitoring qualification. Keep every unchecked item open until demonstrated. Native listening, real-host PDC/save/export and ticket 80 remain separate; no owner source or installed output will be changed.

2026-10-26 implementation checkpoint: 65 Session/event tests pass serially and through the targeted normal build, including six compact/full native-DPI combinations, visible Analyze bounds, explicit conflicting-example replacement, explicit cancellation retention and settled-selection Escape. The normal candidate at `dist/podcast-candidate/d8bfb90f14154b71a401f2c8785aba62` has SHA256 `b5b71d527af0f15162557c63e832948f64f9df9760866a87f427964a0b5e2383`; strict Clippy and both actual-format basic hosts pass (CLAP 1,858 / VST3 4,048 blocks). Both hosts assert that the debug acceptance-driver symbol is absent. Source is 13bcc2c plus the uncommitted 96 diff; this is a preserved intermediate candidate, not final source attestation. No installation or owner media/project write occurred.

2026-10-27 machine delivery: production checkpoint `4558474` passed 67 tests, strict normal/driver Clippy, both normal actual-format hosts, explicit normal-driver/release-feature rejection, and actual CLAP/VST3 non-unity processing versus independent maintained Float32 final renders. Realtime/offline Original/Processed/Removed, nonzero first/last markers, delay reporting, changed-source fallback and real format save/destroy/recreate checks pass. Test-only follow-up `22b2d9f` passed 69 tests and strict Clippy, adding Center/follow/Fit/navigator event coverage. [Full receipt and qualification limits](../../../docs/research/podcast-workflow-acceptance-2026-10-27.md) distinguish the normal artifact, acceptance-only driver and unavailable native/listening checks.

Independent source review found and cleared two scoped fixes in current commit `47e83bb`: discard a pending replacement when the highlighted range changes, and cull offscreen lanes before peak scans/geometry. Their first test attempt passed 69 cases, including the selection-change regression, but its new lane counter included a four-vertex scrollbar quad. The corrected exact-waveform counter remains unrun. Final focused tests/Clippy/UI build were refused before process start by unchanged guard receipt `20261027-004212-5e39d1247dc343ac804ddb95ca04accd.json`: 13.82 GiB available versus the 14 GiB start requirement. No retry, service kill or threshold relaxation followed.

The pre-review 4558474 normal and acceptance-only directories were copied to `%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/plugin-products/Podcast/4558474-normal-b2c3a2332976` and `4558474-acceptance-only-d7c4fd823fbb`; every copied file hash matches. Six pre-review PNGs and source metadata are retained at the exact paths in the report. They do not represent the review-fix source.

Remaining unchecked: corrected review-fix regression, post-review Clippy, final normal UI build/source/hash preservation, refreshed six PNGs and integration. Status stays claimed. Native ticket 80 remains open and 81–83 stay blocked; prior machine successes do not clear those dependencies.

