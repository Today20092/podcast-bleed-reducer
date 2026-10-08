# Retain complete Nemotron evidence for long recordings

Status: ready-for-agent
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
