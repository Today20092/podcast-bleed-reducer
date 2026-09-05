# 01: Teach room tone and solo speech in the editor

**Working branch:** codex/ownership-calibration
**What to build:** Select a track/range, save room-tone or solo-speech examples, inspect/remove examples, persist mode/calibration, and invalidate stale examples. Keep protection gestures intact.
**Blocked by:** None
**Status:** ready-for-agent
**Implementation:** complete, awaiting parent integration

- [x] Optional, understandable calibration interaction for each track, with usable selection controls and visible examples.
- [x] New projects enable joint mode; old projects retain legacy until explicitly enabled.
- [x] Settings save/reopen, invalidate on source changes, and mark renders stale.
- [x] Bounds/tracks/minimum usable ranges and source identities validated; mistaken examples removable.
- [x] Editor requests preview refresh without UI audio reads; preserve existing play/protect controls.
- [x] Project/editor behavior tests, relevant full checks and two-axis review pass.

Read the spec, decisions and shared interface contract. Parent wires the new detector API after ticket02 lands. Do not modify detector/engine internals in this task.

## Implementation decisions

The delegated project lifecycle and headless editor seams are approved. Test mode defaults and round trips, example validation/removal, source invalidation, and editor messages that save examples without protections and refresh preview. Source checks read metadata and WAV headers, never decode PCM during calibration edits. Examples require at least 12,000 samples at 48 kHz, or 250 ms. Precise seconds fields, playhead capture, and selection capture provide keyboard-accessible range entry.

## Completion evidence

Implementation commits: `3c69856` and `edda647` on `codex/ownership-calibration`, based on verified shared contract `10f2cd0`. Canonical ticket status is unchanged; implementation completion is tracked separately above.

`CARGO_BUILD_JOBS=1 cargo test --quiet` passed: 66 tests passed, 3 existing tests ignored. `cargo fmt --check` and `git diff --check` passed. TDD observed missing mode API, invalid range acceptance, solo ranges extending beyond another source, and first-example acceptance after replacing a loaded recording before their fixes. Two headless editor tests cover exact drag bounds, non-protection, room-tone/solo saving, removal, mode persistence, preview requests, invalid numeric input, playhead/selection capture, and stale-source rejection.

Standards review: pass after fixing one source-identity finding with cached loaded stamps and regression tests. Spec review: pass. No outstanding findings. Settings edits use metadata and WAV headers; existing import/open fingerprint behavior is unchanged.

User interaction: choose Select calibration range, drag a waveform, then Save room tone or Save solo speech under the intended track. Alternatively enter seconds, capture start/end at the playhead, or use the current timeline selection. Saved examples appear in the inspector with track, kind, bounds, and Remove example. Existing Shift-drag selection and the explicit Protect selected range action remain unchanged.

Parent work remains exactly as assigned: connect preview/render workers to ownership APIs and guard render completion against changed settings. No detector, cache, engine, or worker calls were modified here. No application launch, source-recording changes, or human listening claims.
