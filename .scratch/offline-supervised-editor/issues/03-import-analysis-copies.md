# Decode common mono formats and prepare aligned analysis copies

Status: done
Assignee: import_implementation
Blocked by: 02
Spec: ../spec.md

## Scope

Generalize import to actual mono WAV/BWF PCM, AIFF PCM, FLAC, MP3, M4A/MP4 AAC-LC or ALAC, and Ogg Vorbis profiles using pinned Symphonia features. Keep the WAV/BWF metadata reader rather than assuming general decoding preserves BEXT. Do not advertise Opus or HE-AAC. Accept supported native rates and mixed-rate sessions.

Validate physical duration and available normalized timing metadata. Missing metadata shows alignment unverified and requires the supplied-alignment declaration; conflicting available metadata blocks work. There is no automatic alignment or silent source stretching/trimming/padding. Generalize source waveform caching and source identities.

Prepare antialiased 16 kHz analysis copies using pinned Rubato, compensating startup delay and flushing tails. Process in bounded work units with cancellation and bounded retained data. Record decoder/resampler/source identities in cache keys.

## Reach

src/audio_io.rs Stem::read_cancellable, src/engine.rs load_and_validate, src/vad.rs input preparation, src/prepared_analysis.rs source/preprocessing cache, src/waveform.rs cache timing, Cargo manifests and import/metadata fixtures.

## Acceptance

- A real fixture for each advertised container/codec profile decodes as expected; stereo, corrupt/truncated and unsupported profiles fail clearly.
- Mixed 44.1/48/96 kHz impulses at both ends align on the 16 kHz analysis timeline.
- BEXT at differing rates is compared in time; matching/conflicting/missing and lossy-delay cases exercise the policy.
- Original files/hashes are unchanged and decoded native frame counts are retained.
- Cancellation and allocation/read failures do not install incomplete evidence or draw incomplete waveform caches as silence.

## Implementation and evidence

- Pinned Symphonia 0.6.1 and Rubato 0.16.2. Real encoded fixtures cover every advertised container/codec; accepted WAV PCM widths are 16/24/32 and IEEE float32/64, AIFF PCM widths 8/16/24/32. Unsupported WAV PCM8, stereo, multiple audio streams, Opus and truncated inputs fail. FFmpeg generates fixtures only; decoding at runtime is Rust.
- Native rate, decoded frame count, BEXT origin and strong source fingerprint are retained. Common mixed rates compare physical duration and normalized origins; missing timing requires a saved, inspectable supplied-alignment declaration. Changed source bytes revoke that declaration. Conflicting metadata/duration remains an error even when declared aligned.
- AAC-LC currently retains decoder priming/padding: the 4,410-frame presentation fixture decodes to 6,144 frames. Import warns that onset alignment must be verified; incompatible session durations fail. No gapless AAC promise or automatic trimming is made. Float64 sources decode to internal float32; legacy 48 kHz export explicitly produces float32 WAV rather than calling the unsupported float64 writer. Original bytes/rate/count stay unchanged; this is not a bit-exact float64 round trip.
- Disk-backed 16 kHz copies use source/decoder/resampler identities, checked geometry, startup-delay compensation and exact ceiling-duration tails. Conversion has no blanket rate range: bounded rational FFT input/output lengths are 1,048,576/65,536, chunks at most 4,096 frames. Safe 1/7,000/48,001/768,000 Hz and unsafe geometry are tested. Invalid owned caches are quarantined/rebuilt; cancellation, permission/sharing failures preserve accepted artifacts. Missing payloads regenerate. No incomplete artifact is accepted.
- Actual owner recording: all three 48 kHz float32 tracks retain 154,128,376 native frames and produce 51,376,126 analysis frames. The initial v1 geometry pass took 711.13 seconds in the unoptimized test profile, observed peak working set 11,669,504 bytes; a small geometry-test build overlapped this pass. Final v2 geometry was separately validated on track 1: 107.19 seconds, observed peak 11,476,992 bytes, unchanged source SHA. These are preparation checks, not release speed/quality benchmarks. Copies are approximately 205.5 MB per track on disk; native session PCM is not retained by preparation. The private owner recording remains Git-ignored.
- VAD and joint evidence read actual 16 kHz windows and filters. Until ticket04, existing envelope coordinates use an explicit virtual 48 kHz adapter, never the source native frame count. Every non-48-kHz audition/render is blocked before full PCM loading with an actionable message, preventing tempo/gain indexing errors. Ticket04 owns source-time playback and native-rate rendering.
- Editor import/relink/drop extensions and waveform caches now decode common mono formats with native timing. Waveform readiness follows complete source decode; a failed decode cannot install a partial waveform as silence. Protected rendering/project validation remains in place.
- Final merged-base checks: `cargo fmt --all -- --check`, strict `cargo clippy --locked --all-targets -- -D warnings`, and serial locked `cargo test --all-targets -- --test-threads=1` pass: 119 tests passed, 9 intentionally ignored fixtures/benchmarks. The owner recording test was run explicitly as recorded above. Package debug information was disabled for the application only to bound build memory; assertions remain enabled.
