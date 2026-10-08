# Carry native rates through playback and WAV rendering

Status: ready-for-agent
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
