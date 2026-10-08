# Offline analysis feasibility notes

Recorded 2026-10-08 from current primary documentation fetched during planning. These are capability findings, not measured project accuracy or performance.

## Required local runtime

[Silero](https://github.com/snakers4/silero-vad) supports ONNX CPU inference and 8/16 kHz inputs under MIT licensing. Its [offline sequence example](https://github.com/snakers4/silero-vad/blob/master/examples/onnx_sequence/README.md) supports processing recordings locally. Use analysis copies and reset recurrent state at independent recording boundaries. Preserve source samples on the playback/render path.

[ort deployment documentation](https://github.com/pykeio/ort/blob/main/docs/content/setup/linking.mdx) describes native runtime linking/loading. Retain the existing deployment approach and test missing-runtime failures and fully offline installation/runs. No Python runtime is required for ONNX deployment.

[Linfa's logistic implementation](https://github.com/rust-ml/linfa/blob/master/algorithms/linfa-logistic/README.md) demonstrates pure Rust binary logistic training. It is a candidate, not authorization to add a dependency. Inspect existing dependencies and training requirements before choosing implementation. Independent binary ownership estimates permit overlap; multinomial softmax over all speakers does not. Validate reliability before describing estimates as calibrated confidence.

## Bleed modeling

The initial required acoustic baseline reuses existing scalar leakage/noise/pairwise evidence. Regularized FIR is qualified against it on development data before adoption; its plausibility is not a release-success claim.

Regularized offline FIR fitting is a candidate for learning directed microphone relationships from trusted solo references. [System identification constraints](https://www.mathworks.com/help/ident/gs/about-system-identification.html) and [acoustic echo filtering](https://www.mathworks.com/help/audio/ug/acoustic-echo-cancellation-aec.html) support investigation of response models, excitation, model complexity and validation.

Microphone-specific limitations are engineering inferences: both tracks contain noisy observations; overlaps, microphone movement, gain changes, delays and contamination can invalidate fits. Use predicted bleed and residuals as evidence for ownership. Do not subtract predicted bleed from the audio deliverable. Exact estimator, regularization and filter length need project-data evaluation rather than arbitrary constants.

## Required diarization

[NVIDIA NeMo-Speech.cpp](https://github.com/NVIDIA/NeMo-Speech.cpp) lists native Nemotron 3 Diarization, Windows installation and CPU builds. Its [SDK](https://github.com/NVIDIA/NeMo-Speech.cpp/blob/main/docs/sdk.md) exposes native integration. Runtime licensing is Apache-2.0; the [model card](https://huggingface.co/nvidia/Nemotron-3-Diarization) specifies OpenMDW 1.1, 16 kHz mono input and generic speaker identities. Published GPU measurements do not establish CPU speed for this editor.

The owner requires Nemotron in the first release. Native integration, project benchmarks, human-reference identity mapping, model acquisition and distribution review are required early implementation gates. Diarization identifies speakers; it does not by itself determine whether a particular microphone hears its assigned person directly.

The official NVIDIA [model repository](https://huggingface.co/nvidia/Nemotron-3-Diarization/tree/main) publishes a Q8 GGUF model, so end users do not need Python conversion. Pin runtime/model revisions and verify downloaded bytes against a recorded hash. The user chose one download on first use, followed by local offline inference. Bundle native runtime libraries, not the model, with the application.

The [standalone C header](https://raw.githubusercontent.com/NVIDIA/NeMo-Speech.cpp/main/include/nemo_speech/diar.h) exposes CPU selection, the v3-offline preset, continuous stream lifecycle, frame count, global probability start and probability buffers. Nemotron V3 supports eight speakers at 10 ms cadence. Raw probabilities compact into segments during long sessions, approximately after twenty minutes. Copy new probability frames into project-owned storage after bounded input pushes using global frame indices; reading once after finish loses early probability data. Retain one stream across the recording for speaker identity continuity. Full-attention one-shot inference has a positional duration limit and is unsuitable for the required two-hour sessions; see the [CLI distinctions](https://raw.githubusercontent.com/NVIDIA/NeMo-Speech.cpp/main/docs/cli.md).

Speaker channels are first-arrival ordered and are not microphone identities. Use human solo references to resolve the mapping and flag ambiguous matches. No native cache guarantees error-free speaker identity. A deterministic nonclipping analysis downmix is an engineering proposal to evaluate on actual solo/bleed/overlap fixtures, not NVIDIA's documented multitrack recipe.

The [build instructions](https://raw.githubusercontent.com/NVIDIA/NeMo-Speech.cpp/main/docs/build.md) document Windows, Linux and macOS CPU builds. Build availability is not measured editor throughput or successful packaging. Windows is the first release; verify native loading, missing/corrupt model failures, cancellation between bounded pushes, long-gap speaker recurrence, complete frame retention and two-hour resource usage there before claiming support.

## Boundary recommendation

Source audio and immutable cached features feed labeled calibration and an independent-per-track session classifier. Whole-timeline smoothing produces ownership/uncertainty regions. Envelope generation remains separate. Protected manual edits are applied in one effective-envelope path shared by display, audition and render.

Gain settings regenerate envelopes; reference-label changes retrain/reanalyze without repeating source decode or fixed VAD where cache keys remain valid; source and fixed-model-version changes invalidate affected features. Do not use human labels as proof that the same regions generalize, and do not train automatically on predictions without an explicit later decision.

## Expanded import and native rates

The owner requires common mono formats and mixed native rates, including 44.1/48/96 kHz. The [Symphonia support matrix](https://github.com/pdeljanov/Symphonia/blob/master/README.md) supports initial profiles WAV PCM, AIFF PCM, FLAC, MP3, MP4/M4A AAC-LC or ALAC, and Ogg Vorbis with explicit feature selection where needed. It marks Opus and HE-AAC variants unfinished; do not promise every codec/container. MPL-2.0 applies to Symphonia. AAC/MP4 decoded delay/timing requires validation rather than a gapless assumption.

Keep the existing bwavfile metadata path for WAV/BWF because generic decoder support is not proof of preserved BEXT/timecode; see [the metadata issue](https://github.com/pdeljanov/Symphonia/issues/195). Test the pinned implementation with actual metadata fixtures.

[Rubato](https://github.com/henquist/rubato) supports arbitrary resampling ratios under MIT licensing. Account for startup delay, output length and flush-tail behavior using the pinned version's API. Use 16 kHz analysis copies and per-source native-rate exports. Prepare device-rate playback outside its callback.

Code inspection found fixed 48 kHz assumptions in Stem::read_cancellable, engine::load_and_validate sample-count/BEXT checks, audition source rates, app/waveform time displays, calibration/protection coordinates, quality labels and gain/report calculations. A decoder replacement alone is insufficient. Common time coordinates and explicit per-source rates must reach all those callers. Cache identities include decoder/resampler versions, rates and decoded lengths.
