# Live preview scheduling

Implementation branch: `codex/responsive-scheduling`, based on `e3cd247`.

The editor retains one active worker and the newest pending target. Drag inputs coalesce for 40 ms without extending the first pending deadline. A completed active snapshot can install before that boundary, so continuous input does not cancel every result. Release and protection changes supersede immediately. Source replacement cancels immediately and retains the worker slot until completion; dropping the editor signals cancellation too.

Unchanged-source inference survives parameter edits. Once progress reaches the retained-facts stage, obsolete derivation can cancel and return its cache even on an error. The next request uses the newest parameters and protections. Both source generation and current file stamps guard installation. Successful relink schedules replacement analysis.

Active preview polling uses 10 ms, removing the previous 100 ms polling interval. Other existing processing and audition polling stays at 100 ms when preview work is idle. Warm updates show a short updating status instead of replaying the full preparation display.

## Regression evidence

- Final checks on `28527c4`: `cargo test -j 1` passed 47 tests, with 3 existing environment/benchmark tests ignored; `cargo clippy -j 1 --all-targets -- -D warnings`, `rustfmt --check --edition 2024 src/app.rs`, and `git diff --check` passed. Builds used the isolated target `C:/Users/User/.codex/targets/responsive-scheduling`.
- Dragging test initially failed because the latest value never installed. It now verifies one active worker, one preparation, final release persistence, and updated protections.
- A controlled worker-delivery and clock-boundary test repeatedly places two input events before each completion, representing 16 ms input and 30 ms work within a 40 ms boundary. Intermediate snapshots install during sustained input. This assertion failed before the review fix.
- Source-change test initially failed because a completed result installed after its file changed. It now checks rejection, replacement-source completion, cancellation without overlapping workers on workspace reset, and retained completed preview after malformed/missing source failures.
- Standards review found no actionable findings. Spec review found sustained-drag starvation and missing relink scheduling; both were fixed and re-reviewed without further findings.

## Measurements and limits

The integration parent measured the actual 83-minute three-stem session read-only: 20 warm runs per control; attenuation/Open/Close median about 1.7 ms and p95 below 2.3 ms; sensitivity/bleed/padding median 29–30 ms and p95 below 33 ms. Preparation count stayed one and restoring defaults was exact. These are derivation timings supplied by the parent, not end-to-end UI measurements from this task. The parent retains the raw benchmark evidence.

The 40 ms cadence and 10 ms polling interval are scheduling choices, not a hard real-time guarantee. Source loading, ownership derivation, and each individual gain-envelope construction stop at the existing cooperative checkpoints; they cannot be interrupted inside those operations. Detector inference checks cancellation per frame. Same-path validation uses existing metadata stamps and inherits their limitations. No human listening or manual GUI cadence claim is made here. No audio was copied, and audition.rs and detector quality were not changed.
