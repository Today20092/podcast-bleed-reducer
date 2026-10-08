# Carry native rates through playback and WAV rendering

Status: done
Assignee: playback_finish
Blocked by: 03
Spec: ../spec.md

## Scope

Remove fixed 48 kHz timing from audition, sample reporting, timeline displays and WAV rendering. Prepare tracks at the selected common playback/device rate outside its callback. Apply envelopes in source-aware time. Preserve transport synchronization, Original/Processed/Removed semantics and monitor-only headroom.

Render each individual WAV from native-rate decoded samples using that source's rate and frame count. Reuse existing WAV format/BEXT preservation where valid; define a valid WAV output format for other decoded sources. Validate quantization/output integrity instead of claiming compressed files are byte-identical to WAVs. Keep safe output transactions, collision checks and cancellation.

## Reach

src/audition.rs SAMPLE_RATE/AuditionSource/LiveAudition, src/audio_io.rs Stem::write, src/export.rs, src/engine.rs run_with_ownership/render wrappers and source/envelope time mapping, app/waveform timing, src/bin/{ownership_diagnose,ownership_evaluate,preview_benchmark,release_gate}.rs and tests/render.rs/export.rs. There is no separate src/render.rs file.

## Acceptance

- A mixed-rate impulse session stays synchronized in playback and corresponding native-rate exports.
- Export rate/frame count and valid BWF metadata are preserved; unity paths meet the chosen WAV format's exactness/quantization contract.
- Original minus Processed equals Removed apart from common monitoring/output quantization; monitor gain never reaches exports.
- The device callback performs no decode, inference, disk I/O or heavy resampling.
- Cancelled render, invalid samples and destination collisions preserve sources and prior deliverables.

## Implementation and evidence

- Preparation resamples one source at a time into verified disk-backed float32 artifacts at the selected output-device rate. Keys include the source fingerprint, decoder/resampler recipes and explicit output rate. Native source bytes and frame counts remain separate. Playback retains at most four queued 4,096-frame multitrack blocks, the current block and one worker block; it does not retain full-session PCM.
- The device iterator reads only the bounded worker queue and evaluates the shared gain envelope at project time on its declared playback grid. It performs no file reads, decoding, inference or resampling. An underrun emits silence and freezes every track's position. Seeks tag both successful blocks and errors by generation, discard old buffers and restart the producer. Mode/envelope changes retain transport position. Producer cancellation is nonblocking in the private callback source; joining belongs to the editor/headless off-device lifetime owner.
- Device rate selection happens on the preparation worker. Opening playback requests that exact rate without configuration fallback; an unavailable configuration produces an error rather than silently resampling in the callback. Original, Processed and Removed share monitoring headroom. Native rendering never reads that monitor setting.
- Rendering streams decoded native samples through the same time-based gain evaluator. Each WAV retains its own native rate and decoded length. PCM WAV/BWF formats and BEXT metadata are retained where supported; non-WAV and float64 inputs deliver native-rate float32 WAV (internal audio is float32). This is an explicit quantization contract, not compressed-source or float64 bit identity. Checked RIFF size arithmetic rejects unsupported oversized output before creating a WAV. Existing atomic transactions, source rechecks, collision and cancellation handling remain.
- Render reports retain their compatibility fields for the first source's actual native grid and add `input.track_grids` / `output.track_grids` for every source. The named virtual 48 kHz detector-envelope adapter remains only until ticket11 replaces automatic envelope generation; its sample count is never reported as a source-native count.
- Public tests verify mixed 44.1/48/96 kHz impulse timing in common-grid preparation and native exports, native gain application, seek/mode behavior, bounded worker shutdown, valid envelope/monitor values, PCM16/24 exact representable unity samples, BWF references, source-byte preservation and per-track report grids. Existing unit tests for live envelope replacement, selection seeks and cancellation were adapted to actual disk-backed preparation.
- Common-grid audition applies gain after resampling Original; native delivery applies gain before any playback resampling. Shared gain/time parity is required, but the resulting waveforms are not asserted bit-identical around transitions (`R(x*g)` need not equal `R(x)*g`). Ticket13 owns the complete current-effective-envelope parity check. No physical-device listening, independent ownership accuracy or eight-track/two-hour release performance approval is claimed here.
- Final integration-base checks pass: `cargo fmt --all -- --check`, strict serial `cargo clippy --locked --all-targets -- -D warnings`, `git diff --check`, and locked serial all-target tests: **125 passed, 0 failed, 9 intentionally ignored**. Tests include existing invalid-sample, failed/cancelled render, destination-collision, source-change and export transaction regressions. Application-only test debug information was disabled to bound build memory; assertions stayed enabled. The base is `ee6d92b`; no new dependency was added.
