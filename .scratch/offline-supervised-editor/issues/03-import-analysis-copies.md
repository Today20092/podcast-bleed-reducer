# Decode common mono formats and prepare aligned analysis copies

Status: claimed
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
