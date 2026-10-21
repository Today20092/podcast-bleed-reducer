# FabFilter performance branch: final review and native checks

Integration branch: `codex/fabfilter-performance`. The owner requested that these changes remain isolated; development/main are unchanged. This receipt supplements the [accepted eight-ticket scope](../../.scratch/fabfilter-performance/spec.md), rather than reopening the closed formal detector evaluation or authorizing release publication.

## Integrated validation

At integrated source `798594260a19c30e2824f197c70c757177d3c6a1`, `scripts/check.ps1 -Local` passed all script fixtures, strict Clippy across all targets/features and the complete suite: **338 passed, 0 failed, 19 ignored**. Ignored tests retain their explicit recording/device requirements. The GUI was built through `scripts/gui-dev.ps1` by the profiling wrapper with the shared pinned development settings. The native executable SHA-256 was `feffb695e346bc303cedc713b3f7ed2b516b3ddfa271445f7606f53331060c7d`.

Final restart-fix validation and preserved executable identity are recorded below when available; the earlier native measurements remain labeled with their actual source.

## Standards review

An independent review of `7be3fe319689fe7bb7a5fdb46025f7a2ae9264d0...7985942` found **0 actionable Standards findings**. It examined the documented architecture, bounded optional observation, diagnostics privacy, shared-envelope authority, selective drawing, visibility scheduling and audio ownership. Graph transport was unavailable to the reviewers; they used the documented source fallback.

## Spec review

The independent Spec review found **1 P2 issue**: `LiveAudition::restart` replaced its envelope Arc directly, bypassing writer-side retirement and allowing the old snapshot's final free inside the device iterator after Stop → Play. The existing regular edit path already used the retirement helper. One implementer owns all review fixes; the real-device regression and resolution receipt follow below. No other actionable mismatch was reported.

## Eight deliveries

| Ticket | Result and evidence |
| --- | --- |
| 01: observations | Opt-in bounded capture; passive by default. Native/software/OS measurements are labeled separately. |
| 02: invalidation | Unchanged worker polls retain drawing caches; real data/style/viewport changes remain checked against fresh drawing. |
| 03: audio lifetime | Bounded decoded-block retirement and writer-owned envelope retirement; device restart review fix receives its own native regression receipt. |
| 04: graphics | About → Copy version info reports the actual active backend/adapter/scale; unsupported refresh remains explicit. Native capture confirmed the fields. |
| 05: edits | Reuses unchanged effective track preparation. [Warm-edit evidence](fabfilter-warm-edit-measurements-2026-10-21.md) includes ordinary and dense logical long-session workloads and their limits. |
| 06: minimized work | Suspends optional visual work only after actual minimized-state observation; audio and job processing remain independent. Native restore/end checks passed. |
| 07: follow | Exact-flat curve vertex pruning, bounded by the existing allocation. [Follow evidence](fabfilter-follow-measurements-2026-10-21.md) shows modest median gains, variable tails and the remaining wide-canvas budget miss. |
| 08: contention | Reproducible opt-in benchmark and [two real analysis comparisons](fabfilter-background-contention-2026-10-21.md). No speculative worker/DSP rewrite was justified. Native queued-import acceptance is still unverified. |

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

## Limits and outstanding acceptance

- Native queued Add tracks was attempted during Original playback using a private WAV copy. The native file picker did not reliably accept automation; it was closed without confirming an import. Do not close ticket08's full import/current-mix/restart/cancellation acceptance from the headless analysis comparison alone.
- Physical DPI transitions, middle-button release gestures, listening quality, audio-device deadlines, GPU presentation and other platforms were not measured by these native checks. Public application/canvas/playback tests cover their stated software boundaries only.
- The 4096-column follow probe still exceeds the inherited 16.7-ms geometry budget. The accepted small pruning improvement does not resolve all wide-canvas performance costs.
- The end-position time label displayed `09:60.000` for the 600-second recording while the cursor and stopped state were correct. This formatting observation is recorded separately from performance acceptance; no unrelated time-format change is included here.

## Preserved build and cleanup

Final artifact identity, self-check, fix-review disposition and worktree cleanup receipt will be appended after the restart regression passes and its commit is integrated.
