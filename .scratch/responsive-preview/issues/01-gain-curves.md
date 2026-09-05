# 01: Construct shared gain curves at target changes

**Working branch:** codex/responsive-gain
**Blocked by:** None
**Status:** resolved
**What to build:** Fast exact envelope construction for preview and final render.

- [x] Sweep region/protection target changes rather than every PCM sample.
- [x] Preserve initial conditions, one-pole retargeting, gaps, overlaps, per-track/all-track protections, session-end sentinels.
- [x] Differential and closed-form checks verify gain behavior; long-session construction benchmark shows reduced work.
- [x] Existing callers remain compatible; commit tested implementation and evidence.

## Evidence

The constructor sorts region edges and applicable protection edges, counts active
protections, and evaluates targets only at distinct edges. One-pole point creation,
gain evaluation, constructor arguments and the shared envelope representation stay
unchanged. Construction work depends on interval count rather than PCM length.

- `cargo test --lib engine::preview_tests --target-dir target`: differential checks
  passed against the legacy sample scan before and after the change. Exact sample
  equality covers empty sessions, empty inputs, initial gaps, tied region starts,
  overlaps, gaps, empty intervals, all-track/per-track protections, adjacent and
  overlapping protections, clipped session ends, and `usize::MAX` protection ends.
  The existing independent closed-form retargeting check also passed.
- `cargo test --all-targets --target-dir target`: 37 passed, 3 ignored. The ignored
  tests are the explicit construction benchmark and two local reference UI checks.
- `cargo clippy --all-targets --target-dir target -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- Separate standards and spec reviews found no code findings. The spec review
  noted only pending benchmark evidence and ticket completion bookkeeping.

All build artifacts use this worktree's `target` directory. No GUI changes,
dependencies, source WAV copies, or human listening claims were added.

### Long-session construction benchmark

Synthetic alternating ownership regions at one-second intervals: 239,660,221
samples, 4,993 regions, three tracks, 2,400-sample Open and 24,000-sample Close.
The benchmark asserts exact equality of the complete old and new envelopes.
It allocates no PCM and does not open the reference WAVs.

Optimized engine and test code built with
`cargo rustc --lib --profile test --target-dir target -- -C opt-level=3`, then run
with the generated test executable's
`long_session_gain_construction_benchmark --ignored --nocapture` arguments:

- Legacy sample scan: **22.621147521 seconds**.
- Edge sweep: **1.366305 milliseconds**.
- Single-run speedup: **16,556.4x**.

The Cargo test profile retains debug assertions and existing dependency builds;
`-C opt-level=3` explicitly optimizes this crate, including both measured
constructors. This is a construction-only synthetic comparison, not an actual
source-analysis or end-to-end warm preview latency claim. Integration owns those
measurements. A separate ordinary debug run measured 192.382137115 seconds versus
15.898669 milliseconds and also passed exact envelope equality.

Reproduce the debug comparison with
`cargo test --lib long_session_gain_construction_benchmark --target-dir target -- --ignored --nocapture`.
