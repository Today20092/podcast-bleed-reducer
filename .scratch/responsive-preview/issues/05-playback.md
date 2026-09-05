# 05: Prepare and audition without blocking the editor

**Working branch:** codex/responsive-playback
**Blocked by:** 01, 02, 03
**Status:** resolved
**What to build:** Background audio preparation and efficient equivalent playback of shared envelopes.

- [x] Starting audition does not read all stems synchronously on the UI thread.
- [x] Repeated audition avoids needless full-source reloads with bounded/shared resources and valid identities.
- [x] Remove per-output-sample lock overhead; sequential curve evaluation preserves output within documented numeric tolerance.
- [x] Preserve Original/Processed/Removed playhead, selection, source-minus-processed, monitor gain and envelope replacement.
- [x] Test cancel/relink/error paths and playback equivalence; document memory behavior.

## Answer

Background preparation owns one worker and a shared session PCM cache. Reads are cancellable in chunks. Generation checks reject stale results, source invalidation drops cached audio, and prepared audio must match the completed preview's source stamps. Playback snapshots controls every 256 samples and uses exact shared point evaluation with a sequential segment cursor.

Validation: `cargo test --target-dir target-playback -j 1` passed 51 tests, with 3 existing reference/benchmark tests ignored. `cargo clippy --all-targets --target-dir target-playback -j 1 -- -D warnings`, rustfmt checks for all five changed Rust files, and `git diff --check` passed. Standards and spec follow-up reviews have zero findings. The warm-cancellation regression failed before its fix and passed afterward.

Memory, numeric behavior, review fixes, and limitations: [playback evidence](../playback-evidence.md). Full-session PCM uses about 2.68 GiB for the supplied sources; this is shared session storage, not streaming. No human listening or audio-device latency claim. No reference WAVs copied or playback enabled on the user's app.
