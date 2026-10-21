# 06: Avoid optional visual work in a minimized editor

Status: claimed
Assignee: perf06_implementer
Type: task
Blocked by: 01 — Measure application frame pacing and input feedback
Source proposal: P5

**What to build:** Measure visible versus minimized playback and background work, then suspend optional visuals with no visible consumer where needed and resume truthfully without pausing useful audio or jobs.

## Acceptance criteria

- [ ] Confirm the installed GUI stack's real visibility/minimization behavior; unfocused but visible windows retain normal updates.
- [ ] Compare visible/minimized/resumed observations; if the framework already suppresses the work, record evidence instead of adding duplicate polling or state.
- [ ] Where needed, gate optional drawing/meter updates and restore the current playhead/meters on resume without stale screens.
- [ ] Playback, accumulating measurements, analysis/export progress, cancellation and result acceptance remain correct through visibility transitions; verify through confirmed application/playback seams and native observation where possible.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. The owner confirmed the existing application, playback-output, shared preview/export envelope and benchmark/report test boundaries on 2026-10-21. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

### Implementation and evidence — 2026-10-21

Ticket01 was verified done before this claim. Its real three-track/600-second/48-kHz native capture observed 97.29% one-core CPU visible versus 105.47% minimized with gain editing and locked follow, with continuous playback after restore. The [baseline summary](../../../docs/research/fabfilter-performance-baseline-2026-10-21.md) records the five-second OS samples and their limits. The OS did not suppress optional work in that scenario; these numbers are not a post-change speedup claim.

Context7's pinned Iced0.14 documentation was fetched. The installed iced_runtime0.14.0 window source confirms is_minimized(Id) returns Task<Option<bool>>, latest() returns Task<Option<Id>>, and the installed window events have no minimization event. Focus loss triggers an actual state query, never a visibility inference. Unknown state retains visible behavior. Active playback checks actual state at500ms intervals, with focus/resize events prompting an immediate query; stopped idle has no new timer. Latest-query generations reject superseded responses, including stop/new-project invalidation.

Minimized playback unsubscribes from display frames and ignores queued visual position/follow work. Audio stays on its existing independent device path. The500ms check still handles playback failure/end; ordinary source, analysis, preview and export Tick polling/cancellation/result delivery remain independent. Restore samples the current device position before resuming frame updates, so meters and follow use the current playhead. No audio processing, envelope, renderer, dependency version or background-worker scheduling changed.

TDD RED: the new application-update/real-canvas test failed on the missing WindowVisibilityChecked message before production changes. It exercises unknown/minimized/restored observations without rebuilding unchanged waveform geometry, then verifies a hidden source-worker result is still accepted by Tick. Device listening, actual minimized/restored rendering, post-change CPU and full combined checks remain assigned to root; acceptance stays claimed until that evidence is recorded.
