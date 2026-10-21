# 05: Keep long-session sound and gain edits responsive

Status: done
Assignee: perf01_implementer
Type: task
Blocked by: 01 — Measure application frame pacing and input feedback
Source proposal: P4

**What to build:** Measure warm dial/gain edits on long recordings and dense effective envelopes, then eliminate demonstrated synchronous rebuild/copy bottlenecks while preserving immediate, authoritative editing and preview/export parity.

## Acceptance criteria

- [x] Record repeated warm edit latency and preparation reuse on representative long/dense sessions using ticket01 and existing editor/preview probes.
- [x] Apply a targeted recomputation, coalescing or preparation change only where measurements show a bottleneck; record a supported no-change decision if the path is already within its measured budget.
- [x] Verify last-edit wins, one undoable drag, source/result generation validation and cancellation. Shared audition/render envelope evaluation stays numerically equivalent.
- [x] No inference rerun for a warm sound edit, silent quality reduction or new user quality mode. Record before/after results and any hardware feedback limits.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. The owner confirmed the existing application, playback-output, shared preview/export envelope and benchmark/report test boundaries on 2026-10-21. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

2026-10-21 preparation checkpoint (perf01_implementer): claimed after merging integration `a530f4f` with01 done. The existing effective `GainEnvelope` holds an `Arc<EffectiveTrackEnvelope>` and an empty legacy point vector, so its outer `Vec` delivery does not deep-copy dense curves. Preserve ticket03's private playback snapshots; no ownership redesign is justified from that source observation alone.

Prepared `tests/warm_edit_measurements.rs` as an explicitly ignored public cache/component measurement:25 repeated attenuation updates, draft composition with automatic-layer reuse, and the app's two envelope-vector deliveries. Scenarios separate three-track60-second48kHz sources from eight-track two-hour sparse/dense evidence. The long fixtures use valid8Hz owned WAVs and source validation, not fake source metadata. They qualify time-coordinate/density behavior only, not ordinary-rate two-hour PCM/import/render or physical feedback. Existing effective-envelope and editor-workflow tests already cover source rejection/cancellation, latest transient drag and one undo. No Cargo run, timing result, production optimization or acceptance pass is claimed at this checkpoint; waiting for the coordinated build lease before measurement and decisions.

2026-10-21 RED: after explicit worktree metadata verification and a no-op native save of `src/lib.rs`, the scoped test `a_single_track_draft_retains_other_tracks_complete_preparation` failed at its numerical reuse-report assertion: expected1 unchanged track, actual0. The reporting field initially returned0, accurately describing the pre-optimization implementation. This was an executed assertion failure, not a missing-symbol/compiler failure. The unoptimized path's second baseline retained all 25 chronological samples per metric in `docs/research/fabfilter-warm-edit-baseline-2026-10-21.json`; dense draft median31.024ms/p9543.496ms, versus tiny private-vector deliveries (p950.009917ms). This demonstrated unnecessary effective-edge preparation across untouched tracks, rather than deep envelope copying or a need for general async work.

2026-10-21 GREEN/completion: merged current integration `9446d3e` (including 03 reclamation and 04 graphics diagnostics) before final checks. Retain complete per-track effective preparation only when its source grid, automatic preparation and normalized relevant manual/protection constraints match. Drafts retain the accepted cache and remain provisional; source/cancellation checks still precede acceptance. No external shared playback-vector API or audio/inference algorithm changed. `reused_tracks` reports actual retained preparation through the existing public result/report seam.

All 25 final single-track drafts reused seven of eight complete tracks in the long fixtures and two of three in the ordinary-rate fixture. Retained baseline → final draft median/p95: native 48 kHz 0.412/0.588 → 0.405/0.538 ms; two-hour sparse 2.948/4.259 → 2.051/3.027 ms; two-hour dense 31.024/43.496 → 19.545/22.396 ms. Dense dial p95 remains 25.690 ms. The targeted reuse removes demonstrated work without a global latency guarantee or speculative scheduler; dense residuals exceed the 16.7 ms diagnostic review threshold and remain explicit. Full scenarios, point/region counts, build identities, all 25 chronological repetitions, hashes and reproduction commands are portable in `docs/research/fabfilter-warm-edit-measurements-2026-10-21.md` and its two adjacent JSON reports.

Final checks: `cargo test --test effective_envelopes --test supervised_delivery` passed 14+5 tests; `cargo test --test editor_workflow gain` passed 4; exact `supervised_completion_composes_latest_edits_and_rejects_obsolete_or_failed_installation` passed 1; ignored `measure_source_bound_warm_edits` passed with 25 repetitions per boundary/scenario. Strict scoped Clippy (`--lib --test effective_envelopes --test warm_edit_measurements -- -D warnings`), `cargo fmt --all --check` and `git diff --check` passed. Commands used the explicit perf-05 worktree/manifest, pinned toolchain, shared target/config and locked/offline single-job settings under an exclusive build lease. The added reuse test's supplemental absence assertion initially failed because its synthetic global-speech row incorrectly requested uncertain-unity preservation; correcting that fixture to known silence made all 14 envelope tests pass, without a production behavior change.

Limits: three 60-second 48 kHz owned WAVs cover ordinary-rate composition; eight two-hour 8 Hz owned WAVs and synthetic 32 ms evidence cover genuine source-bound long coordinates and sparse/dense boundaries with small files. Long fixtures do not qualify ordinary-rate two-hour PCM/import/inference/export, large manual-point collections or listening quality. Each draft has one manual edit with two points. The ticket01 native run includes an actual 600-second 48 kHz warm dial gesture, but its mixed rolling input/update window cannot isolate that edit. No physical input-to-photon, device onset, GPU presentation or universal hardware deadline is claimed. Larger runtime qualifications remain separate from this completed bounded preparation change. Cargo lease released with no running or queued build.

