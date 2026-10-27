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

Claimed 2026-10-26 after 90/95 completion, based on Podcast integration da9b297. Scoped branch codex/podcast-workflow-96 reuses the preserved Podcast worktree and candidate; no whole-desktop merge. Implementation is queued behind the single active Gate detector compiler/measurement job to keep local RAM low.

Existing 95 evidence supplies 62 passing Session/event tests and basic format/state/latency coverage, not this ticket's complete acceptance. Confirmed gaps include labeling/sound actions clipped by eight lanes, the accepted independently scrolling lane viewport and compact/full/DPI interactions, and actual-format cached-envelope versus offline-render/changed-source/monitoring qualification. Keep every unchecked item open until demonstrated. Native listening, real-host PDC/save/export and ticket 80 remain separate; no owner source or installed output will be changed.

2026-10-26 implementation checkpoint: 65 Session/event tests pass serially and through the targeted normal build, including six compact/full native-DPI combinations, visible Analyze bounds, explicit conflicting-example replacement, explicit cancellation retention and settled-selection Escape. The normal candidate at `dist/podcast-candidate/d8bfb90f14154b71a401f2c8785aba62` has SHA256 `b5b71d527af0f15162557c63e832948f64f9df9760866a87f427964a0b5e2383`; strict Clippy and both actual-format basic hosts pass (CLAP 1,858 / VST3 4,048 blocks). Both hosts assert that the debug acceptance-driver symbol is absent. Source is 13bcc2c plus the uncommitted 96 diff; this is a preserved intermediate candidate, not final source attestation. No installation or owner media/project write occurred.

2026-10-27 machine delivery: production checkpoint `4558474` passed 67 tests, strict normal/driver Clippy, both normal actual-format hosts, explicit normal-driver/release-feature rejection, and actual CLAP/VST3 non-unity processing versus independent maintained Float32 final renders. Realtime/offline Original/Processed/Removed, nonzero first/last markers, delay reporting, changed-source fallback and real format save/destroy/recreate checks pass. Test-only follow-up `22b2d9f` passed 69 tests and strict Clippy, adding Center/follow/Fit/navigator event coverage. [Full receipt and qualification limits](../../../docs/research/podcast-workflow-acceptance-2026-10-27.md) distinguish the normal artifact, acceptance-only driver and unavailable native/listening checks.

Independent source review found and cleared two scoped fixes in current commit `47e83bb`: discard a pending replacement when the highlighted range changes, and cull offscreen lanes before peak scans/geometry. Their first test attempt passed 69 cases, including the selection-change regression, but its new lane counter included a four-vertex scrollbar quad. The corrected exact-waveform counter remains unrun. Final focused tests/Clippy/UI build were refused before process start by unchanged guard receipt `20261027-004212-5e39d1247dc343ac804ddb95ca04accd.json`: 13.82 GiB available versus the 14 GiB start requirement. No retry, service kill or threshold relaxation followed.

The pre-review 4558474 normal and acceptance-only directories were copied to `%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/plugin-products/Podcast/4558474-normal-b2c3a2332976` and `4558474-acceptance-only-d7c4fd823fbb`; every copied file hash matches. Six pre-review PNGs and source metadata are retained at the exact paths in the report. They do not represent the review-fix source.

2026-10-27 post-reboot verification: clean `6070569` passes all 70 tests, including the corrected exact-waveform geometry counter and changed-selection confirmation, strict normal all-target Clippy, formatting, final normal build and basic actual CLAP/VST3 hosts (1,858 / 4,048 blocks; driver symbol absent). Build receipt `20261027-100020-4c428ef490b64973902956e5d27f05d0.json` is done/exit 0/cleanup verified; peak owned commit 1,860,972,544 bytes, minimum available physical RAM 12,670,218,240 bytes during execution, above the 8 GiB reserve. The unchanged 14/8/6 policy permitted this fresh post-reboot attempt; no limit was relaxed. The coordinator proved and stopped only the lingering owned VCTIP helper after compiler exit.

Final normal candidate `dist/podcast-candidate/f0be4aab16654eea97364eb8517b56b4` has SHA256 `cf1ddab61ac90879ca49e9c0b4d43d33e327ad94bfd0fc5ae0c670de4cd89485` for both formats. All eight files are preserved outside the worktree with matching hashes at `%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/plugin-products/Podcast/6070569-normal-cf1ddab61ac9`. All six refreshed actual CPU PNGs and source/hash metadata are at `%LOCALAPPDATA%/CleanSlate/Podcast/Workflow96Captures/final-6070569-5d990b1e8a904b40b045517fa27e7a0d`; six DPI geometry cases and visual inspection pass. Capture receipt `20261027-100247-53eb8b43156143f18ffe091163dfafbd.json` is done/exit 0/cleanup verified. Previous acceptance-only processing evidence remains valid for unchanged processing source; the UI-only fixes did not trigger a duplicate full driver workflow. Prior artifacts remain preserved.

Ready for coordinator integration from fixed merge-base/permanent Podcast tip `da9b297b814802985b35a8704da12dffe36475a0`. Machine checks, final normal preservation and refreshed captures are complete; status stays claimed until integration. Native ticket 80 remains open and 81–83 stay blocked. No installation, owner project/audio write or publication occurred. The sole heavy slot was released after capture completion.

