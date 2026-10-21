# FabFilter-inspired performance follow-up

2026-10-21 owner integration amendment: the owner subsequently authorized merging the complete reviewed GUI/icon/performance batch into development for testing. Merge `1235ca1` preserves the reviewed history; [ticket09](issues/09-development-integration.md) and [the development receipt](../../docs/research/development-integration-2026-10-21.md) record combined checks, the preserved build and launch. This supersedes the initial branch-only restriction below. Main/release promotion remains unauthorized pending the owner's development-build test.

Status: eight scoped tickets complete at the owner-confirmed software/report boundaries; native checks and remaining physical-measurement limits recorded on 2026-10-21.
Completion receipt: [integrated validation and final review](../../docs/research/fabfilter-performance-final-review-2026-10-21.md). All eight scoped deliveries are integrated on the performance branch. Ticket07's final theme/size software-canvas check passed, and ticket08's native Add tracks/playback restart retry succeeded. Physical-input/display timing and listening quality remain outside those software/functional proofs. Measurement-dependent no-change decisions and the remaining wide-canvas budget miss are explicit.
Integration branch: `codex/fabfilter-performance`.
Base: `7be3fe319689fe7bb7a5fdb46025f7a2ae9264d0` on the existing isolated GUI review branch. Development/main and preserved GUI builds remain separate.

## Accepted scope

The owner requested one ticket for each P1–P8 proposal in [the investigation](../../docs/research/fabfilter-performance-investigation-2026-10-21.md), followed by implementation of all tickets using to-tickets and implement-spec. This authorizes the eight scopes, including their explicit measurement-dependent decisions. It does not authorize speculative changes when an optimization's condition is absent, or closing an inconclusive investigation as if its acceptance checks passed.

Existing acoustic/source/timebase/editor contracts remain in [the accepted architecture](../offline-supervised-migration/spec.md) and ADR-0006. Preserve source validation, cancellation, native rates, stale-result rejection, manual-edit authority, exact preview/export envelope parity and safe publication. No language/renderer replacement, processing-quality modes, audio algorithm change, dependency upgrade, release promotion or merge to development is included.

## Task graph

| Ticket | Proposal | Blocked by | Delivers |
| --- | --- | --- | --- |
| [01 — Measure application frame pacing and input feedback](issues/01-frame-and-input-measurements.md) | P2 | None | Provide an opt-in, bounded performance capture that exercises the real editor and reports reproducible frame/interaction observations alongside existing processing and playback measurements. |
| [02 — Retain unchanged waveform geometry during worker polling](issues/02-selective-drawing-invalidation.md) | P1 | None | Keep the existing waveform/gain caches valid when a background poll or unrelated interface action cannot change their contents, while every visible data, style, size and viewport change still redraws correctly. |
| [03 — Audit and harden audio-device buffer reclamation](issues/03-audio-buffer-lifetime.md) | P3 | None | Keep the playback device path predictable by measuring heap-owning block/envelope lifetime transitions and moving demonstrated reclamation work off it without compromising bounded, nonblocking streaming. |
| [04 — Report the actual graphics environment safely](issues/04-graphics-diagnostics.md) | P7 | None | Expose useful graphics backend/adapter and available display scaling/timing information through the established diagnostic or version-information flow so machine-specific rendering issues can be reproduced. |
| [05 — Keep long-session sound and gain edits responsive](issues/05-responsive-long-session-edits.md) | P4 | 01 | Measure warm dial/gain edits on long recordings and dense effective envelopes, then eliminate demonstrated synchronous rebuild/copy bottlenecks while preserving immediate, authoritative editing and preview/export parity. |
| [06 — Avoid optional visual work in a minimized editor](issues/06-inactive-visual-scheduling.md) | P5 | 01 | Measure visible versus minimized playback and background work, then suspend optional visuals with no visible consumer where needed and resume truthfully without pausing useful audio or jobs. |
| [07 — Measure and improve continuous-follow drawing reuse](issues/07-locked-follow-geometry-reuse.md) | P6 | 01 | Use complete-editor and geometry measurements to decide whether continuously following the locked playhead needs additional retained geometry reuse, and implement the smallest justified improvement. |
| [08 — Protect foreground work during background processing](issues/08-foreground-background-budget.md) | P8 | 01 | Measure playback/edit responsiveness during representative imports and analysis, then tune only demonstrated contention using bounded independent work while retaining processing correctness and useful throughput. |

Tickets01–04 are the initial frontier. Ticket02 has a direct repeatable invalidation regression and does not need the broader instrumentation to start. Ticket01 supplies the measured scenarios for05–08. Worktree isolation handles overlapping editor changes; integration must resolve and retest their combined behavior.

### Reviewable ticket breakdown

1. **Measure application frame pacing and input feedback.** Blocked by: None (can start immediately). Delivers: Provide an opt-in, bounded performance capture that exercises the real editor and reports reproducible frame/interaction observations alongside existing processing and playback measurements.

2. **Retain unchanged waveform geometry during worker polling.** Blocked by: None (can start immediately). Delivers: Keep the existing waveform/gain caches valid when a background poll or unrelated interface action cannot change their contents, while every visible data, style, size and viewport change still redraws correctly.

3. **Audit and harden audio-device buffer reclamation.** Blocked by: None (can start immediately). Delivers: Keep the playback device path predictable by measuring heap-owning block/envelope lifetime transitions and moving demonstrated reclamation work off it without compromising bounded, nonblocking streaming.

4. **Report the actual graphics environment safely.** Blocked by: None (can start immediately). Delivers: Expose useful graphics backend/adapter and available display scaling/timing information through the established diagnostic or version-information flow so machine-specific rendering issues can be reproduced.

5. **Keep long-session sound and gain edits responsive.** Blocked by: 01 — Measure application frame pacing and input feedback. Delivers: Measure warm dial/gain edits on long recordings and dense effective envelopes, then eliminate demonstrated synchronous rebuild/copy bottlenecks while preserving immediate, authoritative editing and preview/export parity.

6. **Avoid optional visual work in a minimized editor.** Blocked by: 01 — Measure application frame pacing and input feedback. Delivers: Measure visible versus minimized playback and background work, then suspend optional visuals with no visible consumer where needed and resume truthfully without pausing useful audio or jobs.

7. **Measure and improve continuous-follow drawing reuse.** Blocked by: 01 — Measure application frame pacing and input feedback. Delivers: Use complete-editor and geometry measurements to decide whether continuously following the locked playhead needs additional retained geometry reuse, and implement the smallest justified improvement.

8. **Protect foreground work during background processing.** Blocked by: 01 — Measure application frame pacing and input feedback. Delivers: Measure playback/edit responsiveness during representative imports and analysis, then tune only demonstrated contention using bounded independent work while retaining processing correctness and useful throughput.

The owner's request specifies one ticket per proposal, so the granularity is eight corresponding vertical slices. The task graph retains only the measurement dependencies that genuinely gate conditional work. The owner confirmed the observable TDD boundaries below; no further planning confirmation is required.

## Implementation and completion rules

Each ticket is claimed before editing, implemented in a separate worktree/branch based on the current integration tip and merged by a merger agent after its scoped checks. Implementers use TDD at confirmed existing public boundaries, fetch applicable library docs through Context7 and merge the current integration tip before delivery. The integration branch receives combined local checks, two-axis code review, a GUI build and a preserved executable/self-check receipt before temporary worktrees are retired.

Use measured conditional outcomes honestly: a justified no-change result resolves an investigation only with its requested evidence; missing hardware capability, inconclusive timing or failed checks remain explicit outstanding acceptance. Optional diagnostics must be bounded and inactive by default; no per-frame disk logging or recording-content collection.

## Test boundaries

Owner-confirmed boundaries (2026-10-21: “Use those existing boundaries”):

- Existing application update/subscription and real canvas output: catches stale redraws and visibility state; cannot prove physical display delivery.
- Playback source/output and lifetime probes: catches output/generation/starvation/reclamation regressions; deterministic checks cannot guarantee device deadlines or listening quality.
- Shared effective-envelope evaluation via preview/render: catches gain, manual edit and parity errors; cannot establish subjective sound quality.
- Existing benchmark/diagnostic command and copied report output: catches metadata/statistics/unsupported-field/privacy errors; synthetic workloads do not establish hardware performance.

The confirmation covers these existing boundaries for all eight tickets; do not ask again. Exact selected scenarios and limitations are retained with each ticket. Preparation notes are preserved outside the repo under `C:/Users/User/Documents/Codex/Investigations/cleanslate-performance-implementation-2026-10-21/` (editor-notes.md, audio-notes.md, graphics-notes.md).

## Review baseline

Review `git diff 7be3fe319689fe7bb7a5fdb46025f7a2ae9264d0...HEAD` against this scope and the eight ticket acceptance records. No PR is required by the local tracker.

