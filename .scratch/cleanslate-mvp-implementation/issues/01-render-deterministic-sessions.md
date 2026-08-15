# 01 — Render deterministic sessions end to end

**What to build:** A user can submit a deterministic session of one or more supported stems, receive either precise render blockers or an atomically committed set of rendered stems and one JSON report, and verify that supplied ownership decisions and protections produce the settled gain policy without modifying sources.

**Blocked by:** None — can start immediately.

**Working branch:** `feature/cleanslate-mvp`

**Status:** done

- [x] Accept PCM and IEEE-float RIFF/RF64 48 kHz mono stems with equal sample counts, with no product-level track cap.
- [x] Enforce the settled BEXT alignment rules and distinguish render blockers from the unverified-alignment warning.
- [x] Render `Owned(track set)`, `Uncertain`, and `NoSpeech` decisions with fixed −18 dB attenuation, unity-gain protections, and continuous transitions.
- [x] Preserve sample count, alignment, sample rate, channel layout, and relevant metadata without introducing clipping or invalid samples.
- [x] Commit rendered stems and the required JSON report atomically; failure or cancellation leaves no partial result.
- [x] Cover single-track, multi-track, overlap, uncertainty, no-speech, protection, invalid-input, failure, and cancellation behavior with deterministic fixtures.

## Verification

- `cargo test` — 3 deterministic end-to-end tests passed.
- `cargo clippy --all-targets --all-features -- -D warnings` — passed.
- `cargo fmt --all -- --check` — passed.
- `cargo build --all-targets` — passed.
- Scoped standards/spec review completed; findings were addressed. RIFF/RF64 and BEXT handling use settled `bwavfile 2.0.1` rather than custom container code.
