# Retain complete Nemotron evidence for long recordings

Status: claimed
Assignee: diarization_finish
Blocked by: 03, 05
Spec: ../spec.md

## Scope

Wrap the pinned standalone C diarization ABI narrowly in Rust with checked statuses, owned lifecycle and copied results. Use CPU mode and v3-offline preset through one continuous stream per recording, with bounded pushes and cancellation on a worker. Do not use long-input full-attention inference or restart chunks while assuming speaker IDs persist.

Copy newly available probability frames into project-owned cache using global frame indices before native compaction. Include the final tail and keep all required history for two-hour/eight-speaker analysis. Record runtime/model/preset/cadence/mix provenance. Compare deterministic nonclipping analysis downmix recipes on development fixtures and freeze one; preserve separate per-track measurements.

## Reach

New native diarization module, src/prepared_analysis.rs fixed-evidence cache, source preparation workers and fixtures. C buffers/errors must be copied while valid; never expose opaque handles to the GUI or audio callback.

## Acceptance

- Native load/infer/destroy succeeds and failures/cancellation/stale completion cannot install partial current results.
- A recording exceeding native compaction duration retains early, boundary and final frames with no duplicates or gaps; a two-hour run measures memory/cache growth.
- Eight-channel capacity and too-many-speaker policy are checked; channels remain hypotheses until human mapping.
- Long-gap speaker recurrence, processing boundaries, quiet speech and overlap are tested against references.
- Chosen downmix and repeat-run behavior are recorded; native API support is not substituted for measured accuracy or throughput.

## Implementation checkpoint, 2026-10-08

Continuous `v3-offline` owned-helper processing now reads prepared 16 kHz copies in one-second blocks, copies new global probability rows before native compaction, retains the finish tail, and records source/runtime/model/preset/mix provenance. Complete history is disk-backed with bounded reads, length/hash validation and atomic cache publication. Parent cancellation/native abort cannot publish a partial result. The project consumer must still check its current source/job generation before installing fixed evidence into a preview.

Three regular public-boundary tests passed after fresh compilation in the native worktree. Actual CPU native inference on a 100 ms fixture retained all 11 frames/eight columns with verified history and provenance (3.57 s). The independent cache lifecycle probe passed cold acquisition, reuse with an absent helper proving no second inference, and cancellation retaining accepted bytes (3.67 s). These are deployment/infrastructure checks, not speech accuracy or long-session qualification.

Ticket remains claimed: native compaction, two-hour memory/cache measurement, repeat/downmix comparison and controlled speech/overlap/recurrence checks are pending. Scripted Windows SAPI development fixtures are local at `C:/Users/User/.codex/backups/podcast-automix-rust-gui/ticket06-development`; they do not replace independent real recording labels or owner listening approval.
