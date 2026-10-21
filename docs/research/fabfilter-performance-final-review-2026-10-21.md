# FabFilter performance branch: final review and native checks

Integration branch: `codex/fabfilter-performance`. The owner requested that these changes remain isolated; development/main are unchanged. This receipt supplements the [accepted eight-ticket scope](../../.scratch/fabfilter-performance/spec.md), rather than reopening the closed formal detector evaluation or authorizing release publication.

## Integrated validation

At integrated source `798594260a19c30e2824f197c70c757177d3c6a1`, `scripts/check.ps1 -Local` passed all script fixtures, strict Clippy across all targets/features and the complete suite: **338 passed, 0 failed, 19 ignored**. Ignored tests retain their explicit recording/device requirements. The GUI was built through `scripts/gui-dev.ps1` by the profiling wrapper with the shared pinned development settings. The native executable SHA-256 was `feffb695e346bc303cedc713b3f7ed2b516b3ddfa271445f7606f53331060c7d`.

Restart fix `de5a0a9e12c47ac1268d87c8237520fb1b761a61` was integrated at `cd2a0d92aa472bc61dacfbdbecb4b888b1aa07ef`. Its scoped validation passed 11 audition unit tests, three native-playback integration tests and one public reclamation integration test, plus strict Clippy, formatting and whitespace checks. The earlier native measurements remain labeled with their actual source.

Final combined source `e2f51846527b256ce32835c173d326f004e82196` passed `scripts/check.ps1 -Local`: **339 passed, 0 failed, 20 ignored** across 38 test targets, all script fixtures, formatting and strict all-target/all-feature Clippy. This includes the new real-canvas theme/size check and the restart fix. The ignored native restart probe was also executed explicitly in its real-device RED/GREEN check described below; the default suite does not imply hardware qualification.

## Standards review

An independent review of `7be3fe319689fe7bb7a5fdb46025f7a2ae9264d0...7985942` found **0 actionable Standards findings**. It examined the documented architecture, bounded optional observation, diagnostics privacy, shared-envelope authority, selective drawing, visibility scheduling and audio ownership. Graph transport was unavailable to the reviewers; they used the documented source fallback.

## Spec review

The independent Spec review found **1 P2 issue**, now resolved: `LiveAudition::restart` replaced its envelope Arc directly, bypassing writer-side retirement and allowing the old snapshot's final free inside the device iterator after Stop → Play. The existing regular edit path already used the retirement helper. One implementer made the minimal helper-call fix and inspected the other writer assignments. On the actual default ZOOM F8n Pro device at 48 kHz, the same nonempty-envelope Stop → restart → next-edit test observed `freed_in_sample=true` before the fix and `false` afterward, with eventual freeing still confirmed. The observer is test-only; no production API or audio setting was added. The public consumer probe retained 0 allocations/reallocations/frees for steady playback, edits and rapid seeks; its explicitly separate teardown still recorded 11 frees.

An independent follow-up review found **0 actionable findings** in the fix and the new theme/size canvas acceptance check. It confirmed the actual app restart call reaches the helper and that the check compares real production drawing geometry. Geometry equality is not raster, physical DPI or presentation proof; native deadlines remain unqualified.

## Eight deliveries

| Ticket | Result and evidence |
| --- | --- |
| 01: observations | Opt-in bounded capture; passive by default. Native/software/OS measurements are labeled separately. |
| 02: invalidation | Unchanged worker polls retain drawing caches; real data/style/viewport changes remain checked against fresh drawing. |
| 03: audio lifetime | Bounded decoded-block retirement and writer-owned envelope retirement; device restart review fix receives its own native regression receipt. |
| 04: graphics | About → Copy version info reports the actual active backend/adapter/scale; unsupported refresh remains explicit. Native capture confirmed the fields. |
| 05: edits | Reuses unchanged effective track preparation. [Warm-edit evidence](fabfilter-warm-edit-measurements-2026-10-21.md) includes ordinary and dense logical long-session workloads and their limits. |
| 06: minimized work | Suspends optional visual work only after actual minimized-state observation; audio and job processing remain independent. Native restore/end checks passed. |
| 07: follow | Exact-flat curve vertex pruning, bounded by the existing allocation. [Follow evidence](fabfilter-follow-measurements-2026-10-21.md) shows modest median gains, variable tails and the remaining wide-canvas budget miss. The final real-canvas check passed across both themes and 1x/2x logical bounds: waveform/gain retained and fresh geometry agree, unchanged draws reuse geometry, theme/size changes rebuild it, and gain clipping/source endpoints remain correct. This is software geometry evidence, not a physical DPI or raster comparison. |
| 08: contention | Reproducible opt-in benchmark and [two real analysis comparisons](fabfilter-background-contention-2026-10-21.md). No speculative worker/DSP rewrite was justified. The final native Add tracks/playback-restart retry succeeded; its limits are below. |

## Native observations

The private fixture contains three synchronized 600-second/48-kHz tracks and 18 saved examples. Original playback continued while Analyze completed in 23.3 seconds. Zoom to a 480-second viewport, gain-editor activation, locked follow and an attenuation change from -11 to -10 dB updated the current screen. Restored 1026x932 and maximized 2560x1392 layouts were exercised. After confirmed minimization, playback advanced and restoring refreshed the position, curves and meters. Another minimized interval near 09:51 reached the recording end and returned to the Play state on restore.

About → Copy version info displayed Vulkan, NVIDIA GeForce RTX 3070 Ti, 1.00x scale and `Display refresh: unavailable (current GUI API)`, followed by copied feedback. No project/audio path appeared in that diagnostic display.

| Phase | Wall duration | One-core process CPU | Maximum sampled working set |
| --- | ---: | ---: | ---: |
| Visible stopped | 5.161 s | 1.51% | 199,188,480 bytes |
| Visible locked follow + gain editor | 5.182 s | 106.14% | 235,986,944 bytes |
| Confirmed minimized, same playback | 5.220 s | 5.39% | 246,185,984 bytes |

These are short sequential samples on a machine with 32 logical processors, not repeated controlled trials, total-machine CPU percentages or memory peaks. The earlier [baseline](fabfilter-performance-baseline-2026-10-21.md) showed 97.29% visible and 105.47% minimized one-core CPU. This supports effective minimized suspension in the checked scenario; it does not demonstrate improved visible CPU or universal frame pacing. Two initial title-index actions failed to confirm minimization; their samples are retained but excluded from minimized comparisons.

The [native capture](fabfilter-final-native-capture-2026-10-21.json) and [OS samples](fabfilter-final-native-os-2026-10-21.json) preserve identity and statistics. The native report covers a mixed 650-second session, retaining the latest 4096 observations per metric. Its long input/redraw gaps include minimization and a modal file picker; they are not active-frame tails or presented FPS, and independent canvas percentiles cannot be summed into a frame percentile. Source project SHA-256 remained `20BB912825964A2BA9B717525477F6AF365ABB3E434084CF7836E6B5725CBD98`.

## Limits and unmeasured behavior

- The first native file-picker attempt was inconclusive and not counted. In the preserved final executable, a retry using the native Windows path and Open shortcut accepted a private fourth 600-second/48-kHz WAV while Original playback continued (00:15.522 before the picker, 00:52.362 with the fourth waveform ready). The GUI stated the recording joins on the next start. Stop → Play then prepared the expanded session and returned to Playing/Playback ready, advancing from 01:07.252 to 01:22.322. This completes native functional import/restart evidence; no separate import throughput or native import-cancel timing is inferred from snapshots. Public source/cancellation/stale-result checks and the package self-check cover their confirmed software boundaries.
- Physical DPI transitions, middle-button release gestures, listening quality, audio-device deadlines, GPU presentation and other platforms were not measured by these native checks. Public application/canvas/playback tests cover their stated software boundaries only.
- The 4096-column follow probe still exceeds the inherited 16.7-ms geometry budget. The accepted small pruning improvement does not resolve all wide-canvas performance costs.
- The end-position time label displayed `09:60.000` for the 600-second recording while the cursor and stopped state were correct. This formatting observation is recorded separately from performance acceptance; no unrelated time-format change is included here.

## Preserved build and cleanup

The final local review executable is preserved outside the shared build cache and temporary worktrees:

- Source: `e2f51846527b256ce32835c173d326f004e82196` (subsequent changes are validation receipts only).
- File: `C:/Users/User/Documents/Codex/Builds/CleanSlate/fabfilter-performance-e2f5184/CleanSlate.exe`.
- SHA-256: `de509502bc0ec40921c7c8c5d119fe2ee141338644015a0f4ace5c89dc84d3f6`, identical to the canonical build.
- Built through `scripts/gui-dev.ps1 -BuildOnly`, 88.08 seconds. Both canonical and preserved executables passed `--package-self-check`: offline model inference, analysis, project save/reopen, render and cancellation.
- The preserved executable itself launched the GUI and passed Stop → Play plus the native fourth-source import/restart smoke check. `Open-GUI.ps1` beside it opens the owned `Review.cleanslate` three-track input; originals remain separate. This is a local development review build, not release publication or cross-platform packaging qualification.

Cleanup completed after executable preservation: exactly `C:/Users/User/Documents/Codex/Worktrees/cleanslate-perf-01` through `cleanslate-perf-08` were checked for resolved path boundaries, cleanliness including ignored artifacts and merged ancestry, then removed with non-force `git worktree remove`. Their eight `codex/perf-0N-*` branch names were removed with `git branch -d`; every delivery remains reachable from the performance integration history. No other branch or artifact was removed. Only the primary worktree remains, still on `codex/fabfilter-performance`. Development remains `91fa021efe84450e332f598bc8358ff4bfbda204`; main remains `0b9622a88643093fc4496cd4729be1d441b3a27c`. Nothing was pushed or published.

All eight ticket records now close their scoped software/report acceptance. The preserved build reopened the original three-track input, completed Analyze in 22.5 seconds, played Processed and switched to Removed without a playback error, and is left open/stopped in Processed mode for owner review. These are functional checks, not listening-quality qualification or a new controlled timing comparison.
