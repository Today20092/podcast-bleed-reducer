# 08: Protect foreground work during background processing

Status: done
Assignee: perf08_implementer
Type: task
Blocked by: 01 — Measure application frame pacing and input feedback
Source proposal: P8

**What to build:** Measure playback/edit responsiveness during representative imports and analysis, then tune only demonstrated contention using bounded independent work while retaining processing correctness and useful throughput.

## Acceptance criteria

- [x] Record foreground latency, preparation/analysis throughput, memory and available underrun observations for representative track counts, durations and rates with heavy work active.
- [x] Inspect actual worker/critical-path behavior before changing concurrency; record a measured no-change decision when more scheduling machinery is unjustified. Analysis/preparation decision below; native import coverage remains in the first criterion.
- [x] Any tuning preserves source validation, cancellation, stale-generation rejection, authoritative envelope/export results and bounded memory. No worker tuning; existing authority checks pass.
- [x] Add no speculative parallel DSP/SIMD rewrite or user worker-count setting; retain reproducible comparison evidence and verify the affected public job/playback/report boundaries.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. The owner confirmed the existing application, playback-output, shared preview/export envelope and benchmark/report test boundaries on 2026-10-21. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

### Scoped implementation and observations — 2026-10-21

Claimed after verifying ticket01 done; merged current integration through `6ea58183918ad077cf2f34afa982c354bee1e9b9`. Added only opt-in `preview_benchmark --contention` mode through the existing public command/report seam. It compares fresh-cache standalone versus fresh-cache concurrent full analysis while existing gain/effective-envelope/playback boundaries run, retains at most4096 observations per metric, validates equal regions/envelopes and immutable saved-project bytes, and refuses existing report/cache destinations. No worker concurrency, priority, DSP, dependency, profile or user setting changed.

See [the retained comparison and reproducible command](../../../docs/research/fabfilter-background-contention-2026-10-21.md) and its two portable raw reports. On the real three-track600-second48kHz recording, two active runs retained1055/1077 observations with completion p99 2.695/2.771ms and maxima3.256/3.285ms; full analysis took23.258/23.739s versus22.660/22.321s standalone. Source preparation itself took11.136/11.283s active. Regions/effective envelopes matched exactly and the private project stayed byte-identical. The mixed-rate two-track4-second44.1/48kHz test covers a second count/rate path without claiming recording quality or stable timing tails.

External250ms OS sampling retained185 observations per real run: whole-command maximum sampled working sets56.93/56.34MiB, including standalone/setup/baseline/active phases together; not phase-specific or guaranteed peaks. Each phase recorded960 aggregate software silent samples without position advancement, including its explicit seek; no instantaneous attribution or native underrun conclusion is possible. Native device underruns/peak memory are explicitly null in reports. The measured component boundary demonstrates no scheduling bottleneck warranting tuning; actual native UI/import/device acceptance remains below.

TDD RED was the existing command's expected rejection of `--contention`; GREEN passed the real command/report contract. Latest-integration scoped tests passed14 with one existing ignored: `background_contention`, `editor_workflow`, `playback_reclamation`. They cover source/stale/cancellation authority and playback outputs/reclamation as well as report bounds/privacy/immutability. Final strict Clippy for those targets plus command/library, final command build, formatting and diff checks passed. All Cargo starts used the exclusive lease, pinned/shared locked/offline single-job settings; source lib was re-saved identically before the initial build to avoid stale worktree selection. Lease released after unopposed real measurements; no benchmark or compiler remained active.

Status remains claimed for root's combined checks/native capture: actual Analyze+Original/edit response, queued Add tracks throughput/current-mix/restart/cancellation behavior, and native resource/device observations. Queued Add tracks retains playback through `reserve_import`/one active `start_next_import`/four-event `poll_import_jobs`; full Open/Relink/Validate is a separate path that stops audition. No import behavior was bypassed or changed. Do not mark the full foreground/import acceptance passed from this headless analysis/preparation comparison alone.

2026-10-21 integrated native receipt at `7985942`: Original playback continued while Analyze completed in 23.3 seconds; zoom and live attenuation/gain-editor changes updated the GUI. Combined local checks passed 338 tests (19 ignored), all script fixtures and strict Clippy. Native queued Add tracks was attempted with a private 600-second/48-kHz WAV duplicate, but the native file picker did not reliably accept automation and was closed without confirming import. No queued-import throughput, current-mix/restart/cancellation result is claimed from that attempt. The first criterion and ticket remain open for this specific native acceptance; do not infer native device underruns from the headless software-starvation aggregates. OS samples and the completed capture are linked in [the final receipt](../../../docs/research/fabfilter-performance-final-review-2026-10-21.md).

Final native retry supersedes that open handoff: the preserved `e2f5184` executable accepted the native Windows path through the file picker's Open shortcut. A private fourth 600-second/48-kHz recording reached Ready and displayed its waveform while Original playback continued from 00:15.522 to 00:52.362. The notice explicitly said the new recording joins on the next playback start. Stop → Play then prepared the expanded session and returned to `Playing`/`Playback ready`, advancing from 01:07.252 to 01:22.322. The earlier picker attempt is not counted as a successful import. This is native functional evidence; no separate import throughput, physical listening or device-underrun number is inferred from UI snapshots. Cancellation/source/stale-result authority is covered by the passing public job/playback checks and package self-check, rather than a new claim of native import-cancel observation. Final combined checks passed 339 tests (20 ignored), all script fixtures and strict all-target/all-feature Clippy. The representative measured contention reports plus the now-completed native import/playback flow resolve this scoped investigation without worker tuning. The original three-track review input was reopened afterward and the owner's saved project hash remained unchanged.

