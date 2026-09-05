# 03: Make controls and processed waveforms honest

**Working branch:** codex/responsive-controls
**Blocked by:** None
**Status:** resolved
**What to build:** Correct sensitivity presentation, actual padding increments, accurate bleed explanation and gain-aware waveform summaries.

- [x] Higher displayed sensitivity admits weaker speech; old stored thresholds retain their meaning and sound.
- [x] Padding control uses 32 ms effective increments and handles legacy values deliberately.
- [x] Bleed helper text explains margin above each track's noise floor, not direct microphone competition.
- [x] Processed waveform accounts for changing gain within a bucket rather than blindly using gain at the start; document conservative approximation if used.
- [x] Behavioral checks cover transitions and parameter round-trip; retain accessibility and compact layouts.

## Answer

Sensitivity displays `1 - speech_sensitivity` and converts changes back at the UI message boundary. Serialization and detector thresholds are unchanged. Padding displays the nearest effective 32 ms step, with an aligned 0–992 ms range. A legacy value such as 300 ms remains stored as 300 ms, displays 288 ms effective, and only changes when padding is edited. The bleed explanation names each track's own noise floor. Visible helper text and a scrollable inspector retain access to controls in compact windows.

Processed buckets multiply source peak bounds by exact gain extrema across the bucket's half-open interval. The new crate-local `GainEnvelope::gain_bounds` checks monotonic segment endpoints and both sides of internal retargets, including protection edges. Existing envelope methods and representation remain compatible. This bounds calculation visits envelope edges rather than every audio sample.

Approximation: cached source peaks do not retain their positions, so source and gain extrema may occur at different times. The bright waveform can overstate the processed peak; it is a conservative bound, not exact processed PCM. This is stated in the UI and code. Original remains faintly visible. No source audio or evidence WAV was copied or retained.

## Validation

- Worktree-local `CARGO_TARGET_DIR=target` for every Cargo invocation.
- Red/green: sensitivity regression failed on the old message behavior, then passed after inversion. Padding, gain-bound and bucket-bound tests failed before their implementation and passed afterward.
- `cargo test --quiet`: 40 passed, 2 existing reference-session tests ignored.
- `cargo test --lib app::tests --quiet`: 9 passed after the final inspector scrolling and saved-threshold assertion changes.
- `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, and `git diff --check`: passed.
- Independent standards and spec reviews against base `528c65de57e5b78759f763934194a000a51132bf`: zero findings on each axis.
- No human listening or native GUI visual audit claimed. Parent task owns integrated GUI verification, actual-session benchmarks, and the release EXE.
