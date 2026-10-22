# 06: Avoid optional visual work in a minimized editor

Status: done
Assignee: perf06_implementer
Type: task
Blocked by: 01 — Measure application frame pacing and input feedback
Source proposal: P5

**What to build:** Measure visible versus minimized playback and background work, then suspend optional visuals with no visible consumer where needed and resume truthfully without pausing useful audio or jobs.

## Acceptance criteria

- [x] Confirm the installed GUI stack's real visibility/minimization behavior; unfocused but visible windows retain normal updates.
- [x] Compare visible/minimized/resumed observations; if the framework already suppresses the work, record evidence instead of adding duplicate polling or state.
- [x] Where needed, gate optional drawing/meter updates and restore the current playhead/meters on resume without stale screens.
- [x] Playback, accumulating measurements, analysis/export progress, cancellation and result acceptance remain correct through visibility transitions; verify through confirmed application/playback seams and native observation where possible.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. The owner confirmed the existing application, playback-output, shared preview/export envelope and benchmark/report test boundaries on 2026-10-21. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

### Implementation and evidence — 2026-10-21

Ticket01 was verified done before this claim. Its real three-track/600-second/48-kHz native capture observed 97.29% one-core CPU visible versus 105.47% minimized with gain editing and locked follow, with continuous playback after restore. The [baseline summary](../../../docs/research/fabfilter-performance-baseline-2026-10-21.md) records the five-second OS samples and their limits. The OS did not suppress optional work in that scenario; these numbers are not a post-change speedup claim.

Context7's pinned Iced0.14 documentation was fetched. The installed iced_runtime0.14.0 window source confirms is_minimized(Id) returns Task<Option<bool>>, latest() returns Task<Option<Id>>, and the installed window events have no minimization event. Focus loss triggers an actual state query, never a visibility inference. Unknown state retains visible behavior. Active playback checks actual state at500ms intervals, with focus/resize events prompting an immediate query; stopped idle has no new timer. Latest-query generations reject superseded responses, including stop/new-project invalidation.

Minimized playback unsubscribes from display frames and ignores queued visual position/follow work. Audio stays on its existing independent device path. The500ms check still handles playback failure/end; ordinary source, analysis, preview and export Tick polling/cancellation/result delivery remain independent. Restore samples the current device position before resuming frame updates, so meters and follow use the current playhead. No audio processing, envelope, renderer, dependency version or background-worker scheduling changed.

TDD RED: the new application-update/real-canvas test failed on the missing WindowVisibilityChecked message before production changes. It exercises unknown/minimized/restored observations without rebuilding unchanged waveform geometry, then verifies a hidden source-worker result is still accepted by Tick. Device listening, actual minimized/restored rendering, post-change CPU and full combined checks remain assigned to root; acceptance stays claimed until that evidence is recorded.

Scoped GREEN after merging integration dd8df122:69 application tests passed,4 existing owner-recording tests ignored;3 performance report integration tests and the deterministic playback reclamation/output test passed. Strict Clippy --lib --tests with warnings denied, formatting and diff checks passed. All Cargo commands used scripts/local-build-settings.ps1, pinned1.99.0, locked/offline dependencies, one job and the shared application-data target under an exclusive build lease. No GUI executable was built in this worktree; root owns the integrated scripts/gui-dev.ps1 build, executable preservation and native acceptance.

The integration merge exposed one stale cfg(test) caller in the existing three-track zoom benchmark: ticket05 changed EffectiveTrackEnvelope::new into prepare returning Arc. The helper now calls prepare with previous=None and otherwise identical inputs, as authorized by root. This caller repair restores library-test compilation without changing production envelope behavior. Source implementation commit cc769d1 and its integration merge are retained on codex/perf-06-visibility; native minimized/resumed playback, stop/end and concurrent-job observations remain unclaimed here.

2026-10-21 root native acceptance at `7985942`: real Original playback continued during a 23.3-second Analyze and live attenuation/zoom/gain-editor changes. At 2560x1392 with three 600-second/48-kHz tracks, 480-second viewport and gain editor + Lock enabled, one 5.18-second visible sample used 106.14% of one logical core; a separately confirmed minimized 5.22-second sample used 5.39%. Restore showed the current position (02:35.092 after 02:11.962 before minimization) and current curves/meters. A second minimization near 09:51 completed playback while hidden; restore showed Play available and the cursor at the recording end. Native restored layout also exercised 1026x932 and maximized sizes. The first title-index attempts did not confirm minimization and are explicitly excluded from minimized comparisons. Raw `final-os.json` and the gracefully written mixed-session `final-native.json` are retained in the external investigation folder; see the final receipt for limitations. App/playback boundary checks cover job delivery, cancellation and failures; no live export, listening quality or device deadline claim is made. Integrated checks passed 338 tests with 19 ignored and strict Clippy. Short sequential samples demonstrate suspension in this scenario, not a universal CPU/presentation guarantee.
