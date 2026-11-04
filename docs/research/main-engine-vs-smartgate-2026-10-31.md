# Desktop ownership engine versus Smart Gate — source audit

2026-10-31, source-only. No build, inference, audio comparison, GUI or installed
artifact change was performed for this audit.

The Desktop engine has a real information advantage for **synchronized,
dedicated-microphone podcast bleed decisions**: it sees the other microphones
and receives human truth for this recording. Our current Gate prototype lacks
both. This does not establish better sound than sonible, and much of that
advantage disappears when a Gate receives only one microphone.

## Exact snapshots and discovery

- Desktop `development`: `a2606de537ad99e1b36d6cd7748d131e9d5d5485`.
  Inspected checkout `../lusca` is `e2a5f8d8de3e23735d9dd4d4dc6ba6143ce54c2c`;
  `git diff development HEAD -- src Cargo.toml` is empty. Existing untracked
  research files there were not edited.
- Gate production snapshot `4afa18a371cd811f8f91a4504c1a612ae38e3af0`, combined
  only in this isolated checkout as `198dbf6e365debd564c070f6c9d01deb89aad0e0`.
  That merge adds the previously verified test-only backend observations, not
  another production algorithm. This is **4afa-based**, not necessarily the
  owner's latest GUI.
- Read `CONTEXT.md`, ADR-0006 and the accepted migration spec/amendments first.
  Graph search returned `Transport closed`; named source/call-chain inspection
  supplied the fallback. No index or compiler ran.

## What Desktop actually runs

`EditorAnalysisWorker::analyze_with_feedback` / `analyze_evidence`
(`src/editor_workflow.rs:193,286`) validate sources, prepare separate16kHz
analysis audio, require usable speaker/room references, run acoustic ownership,
apply temporal/human constraints, then publish a completed result only after
cancellation and source/reference identity checks. Playback/render use original
native-rate audio with envelopes; this is offline analysis, not inference in
the playback callback.

There are two distinct model layers:

1. **Fixed embedded Silero VAD v6.2** supplies speech-presence evidence on512-sample
   /32ms frames with64 past-context samples and recurrent state. Its weights are
   not trained by labels. `src/vad.rs::silero_frames`,
   `src/session_ownership.rs::analyze_acoustic_with_feedback`.

   Version correction: the initial audit repeated the repository's `v5` label.
   Subsequent [official tag/byte verification](silero-model-provenance-2026-10-31.md)
   proves the unchanged asset matches v6.2. This corrects metadata, not weights.
2. **One binary logistic regression per assigned track** learns local ownership
   from this session's labels using pinned `linfa-logistic0.8.1`. It standardizes
   varying features, fits L2-regularized coefficients/intercept, and verifies
   finite inputs, positive/negative examples, convergence/usefulness and class
   orientation. This is genuine per-session supervised fitting, not neural
   fine-tuning, an LLM, speech recognition or a voiceprint.
   `crates/ownership-model/src/lib.rs::OwnershipModel::train`,
   `src/session_ownership.rs::features/feature_names`.

The input count is `3 + 5*(tracks-1)`: raw VAD, room-noise-relative level,
dominance over the strongest other mic, then each directed peer's bleed
explanation, expected level, excess residual, fit reliability and availability.
`AcousticEvidenceCache::analyze` learns room floors and directed transfer fits
from trusted solos. `acoustic_transfer::qualify` permits a regularized two-tap
FIR with±12ms lag only after separate held-out checks versus a delayed scalar.
These are **features**, not destructive bleed subtraction or source separation.
`src/acoustics.rs:64,172`; `src/acoustic_transfer.rs:77`.

Solo/overlap/room references supply positives and negatives for every assigned
track; unlabeled frames are not presumed negative. Training caps each class
at2048 selected frames, then predicts the recording. Independent probabilities
permit simultaneous owners. Temporal decisions use0.8/0.2 hysteresis;
intermediate scores retain the previous state, **not automatic uncertainty
protection**. Missing/invalid regional evidence is preserved; exact human
labels override statistical decisions across their full intervals.
`src/session_ownership.rs::truth_frames/selected_training_frames`;
`src/temporal_ownership.rs::TemporalOwnershipCache`.

Manual edits/protections and finite S-shaped fades compose the effective
envelope used for display, audition and export. Source changes/corrupt or
cancelled analyses do not silently become new accepted results. Nemotron is
removed from execution/dependencies, as ADR-0006 specifies; the legacy
`nemotron_enabled` deserialization field in `Project` is compatibility metadata,
not a second selectable inference mode. See `src/effective_envelope.rs`,
`src/editor_workflow.rs`, `src/project.rs:149` and
`docs/supervised-ownership-models.md`.

## Strength, limitation and unknown matrix

Vendor facts below are the companion researcher's verified official claims;
this audit does not inspect proprietary weights or infer their internals.

| Question | Desktop | Current Gate4afa prototype | sonible smart:gate / comparison limit |
| --- | --- | --- | --- |
| What is the target? | Local ownership of each assigned mic; direct speech versus another speaker's bleed. **Strength:** peer evidence plus explicit truth. | Generic speech presence on one input. Stereo uses maximum confidence from two independent VADs, not peer ownership. | Official targets are source classes such as Speech/Vocals. Reviewed material does not establish assigned-mic ownership or personal voice identity. |
| What adapts? | Per-session room/bleed fits and independently trained logistic models; source/label-bound caches. | Fixed Silero; no per-session trainer, references or acoustic transfer features. | Profiles initialize on representative playback; FAQ says no Learn button or automatic parameter setting. Neural architecture, training data and personal adaptation are **not established**. |
| Timing | Whole-recording offline analysis and gain placement around known preserved regions. **Potential strength:** future context/manual correction; **cost:** references and Analyze workflow. | Causal32ms inference frames, past FIR/recurrent context, asynchronous bounded worker. Zero audio delay is not zero evidence delay. | Official NN processing uses temporal context/~80ms latency and is not recommended for live/recording use. Latency alone cannot rank preservation quality. |
| Safety/correction | Explicit labels and authoritative manual edits; missing regional evidence preserves, failed jobs do not replace accepted results. | Default bypass, temporary explicit arm, stale/invalid/overflow fallback, reset recovery; Range0..6dB in4afa. | Exact comparable fallback/manual-protection semantics are not established by this source audit; do not claim ours superior without testing. |
| Known limits | Reference correctness and microphone/room changes matter; simple linear ownership boundary can miss novel/quiet overlap. Known ignored quiet-overlap qualification test remains. | Existing whisper/laughter low VAD scores are preservation risks; no speaker identity/bleed discrimination. Original stressed mono128 latch remains causally unresolved. | Proprietary detection accuracy, whisper/laughter/overlap behavior versus our recordings are **not established**. |
| Actual evidence | Numerical/source/synthetic tests and owner listening acceptance exist; no matched vendor comparison. | Functional/fallback tests and bounded observations exist; no universal realtime/quality qualification. | Product claims and controls establish available behavior, not a measured win over Desktop or our Gate. |

Official vendor sources: [product](https://www.sonible.com/smartgate/),
[temporal context/live FAQ](https://help.sonible.com/hc/en-us/articles/9636748686620-Can-I-use-smart-gate-while-recording-or-in-live-situations),
[no Learn/automatic parameter setting FAQ](https://help.sonible.com/hc/en-us/articles/9636750686364-Why-is-there-no-learning-button-and-no-auto-parametrization-in-smart-gate),
[manual](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf),
[vocal workflow](https://www.sonible.com/blog/smartgate-vocal-track/).

## Reuse and recommendation

Gate reuses the **same embedded Silero bytes** (SHA256
`1a153a22f4509e292a94e67d6f9b85e8deb25b4988682b7e174c65279d8788e3`)
and pinned ORT runtime, plus the recurrent invocation contract. Its new causal
resampler/detector, bounded worker and gain policy are separate from Desktop.
`plugins/smart-gate/Cargo.toml` explicitly excludes the Desktop/trainer graph;
it depends on `plugins/gate-detector`, not `cleanslate-ownership-model`.
It does **not** reuse Desktop's learned ownership models, session reference
trainer, directed bleed calibration, temporal truth overlay or effective manual
envelope engine. Sharing model bytes therefore does not make their decisions
equivalent. Current Gate source also supersedes the historical fixed-floor
description with the real0..6dB Range parameter (`lib.rs`, `gain.rs`).

For podcast-vocal Gate, first decide whether the product is single-mic speech
gating or synchronized multi-mic bleed ownership. The latter is where Desktop's
existing evidence/labels offer a concrete differentiated design; it belongs to
the supervised offline/Podcast path unless a new host-input/latency contract is
accepted. For single-mic Gate, retain the causal safety boundary and assess
onset/tail, quiet speech, laughter/interjections and bleed-as-speech on matched
human-labeled recordings before choosing a new detector or learning control.
Optional user correction/session learning is a possible future design, not an
already implemented advantage for Gate. Do not port offline full-session
allocation, future context or peer-dependent features into its callback.

No quality-superiority conclusion follows from model size, neural versus
logistic architecture, richer feature count, stable Desktop status or numerical
test success. `tests/session_ownership.rs::unlabelled_quiet_overlap_remains_a_quality_qualification_gate`
is explicitly ignored; owner acceptance did not turn it into passing evidence.

## Later native-host diagnosis and unselected design ideas

Read-only follow-up inspected canonical Gate `a6ca1f6`. Its detector/envelope
still matches the earlier production snapshot; the host now paces real-time
callbacks. The latest preserved Live candidate is
`GuiPreviews/SmartGate/Live/124870c09ed24883bd715318b9c831f9`, plugin SHA256
`6f74cc0182976c1386e7a39eabf6c7decd2c01b1e67fe67b83ef42234d48d2d5`.
Installed preview links still target the earlier `2e3adbf` source/hash recorded
in the [controls report](sonible-smartgate-controls-2026-10-31.md).

Latest native attempts explicitly remain unqualified with unity output. One
records a real consent transition, but neither that transition nor unity gain
establishes whether Bypass, Range, detector readiness and classification allowed
attenuation. The logs do not capture enough public parameter/readiness state to
identify a unique cause. No post-consent reset was found in the latest host path.

Two concrete host-contract findings are separate from that unresolved cause:

- Pinned nice-plug `0.4.2` constructs its buffer configuration using the previous
  processing mode before installing the requested mode. A first Offline setup
  can consequently activate the plugin as Realtime. Latest paced-Realtime host
  work avoids that specific mismatch; it does not prove successful gating.
- Canonical native qualification still assumes a fixed floor and gain below
  `0.6`. Correct 3 dB Range attenuation is approximately `0.708`, so this oracle
  can reject working partial attenuation. Reviewed host source `56feace` contains
  Range-aware qualification and SDK metadata discovery, but was not integrated
  into the inspected canonical host. The latest real-time path also retains an
  outdated offline label in its success text.

The narrow diagnostic proposed, but not run, was to characterize the exact
36,000-frame native fixture through the actual Rust processing callback, then
record public host Bypass/Range readback and readiness. The 0.75-second native
fixture is shorter than the ordinary 1.024-second floor test. This would
distinguish configuration, fixture timing and protected fallback before changing
thresholds. No private consent ABI or fabricated native pass was proposed.

Additional source limitations: after the gate has closed, zero audio delay cannot
recover a speech onset preceding delivery of fresh detector evidence. Range
changes can clamp existing attenuation upward immediately; smoothing and click
behavior need attention before extending depth or making automation claims.

No final parameter ranges or defaults were selected. Candidate latency values,
deeper attenuation and level/ML blending were brainstorming, not measured
recommendations. A proposed log-odds hysteresis-width control was not adopted as
Sonible-like Tolerance: hysteresis and a gradual activity transition are distinct.
The [full control outline](sonible-smartgate-controls-2026-10-31.md) retains that
distinction and lists the actual routing, crossover, state and monitoring work.
The owner's latest instruction stops after saving research; no implementation
ticket, additional build, installation change or GUI change follows this audit.

## Shipped-model comparison after authenticated artifact recovery

The [authenticated model report](smartgate-authenticated-model-2026-10-31.md)
now supplies direct artifact evidence for both sonible profiles and our bundled
Silero model. This supersedes comparisons based only on vendor documentation or
the old, incorrect Silero v5 label. The audited Desktop model asset and the
core VAD/acoustic/ownership source files are unchanged between `a2606de` and the
later primary development checkpoint `348a9cf` (scoped Git diff is empty).

| Layer | CleanSlate Desktop | Current Smart Gate prototype | sonible Speech/Vocals artifacts |
| --- | --- | --- | --- |
| Neural component | Embedded Silero v6.2 | Same model bytes | Two authenticated ORT-format profiles with the same serialized topology and different tensor contents |
| Neural architecture | STFT/Conv plus an active LSTM path, hidden128 | Same | Conv plus sequential forward GRUs, hidden192 and96 |
| Model interface | Raw16kHz analysis frames and recurrent state | Same detector interface, behind a bounded worker | Complex spectrum, two48-band feature rows, convolution cache and recurrent state |
| Neural output | Speech-presence evidence for later ownership analysis | Speech-presence evidence for the gain policy | Scalar sigmoid head transformed as `10^((72*a-72)/20)`; spectral output is constant complex unity |
| Recording-specific supervised fitting | Per-track logistic ownership models fitted from session labels | Not implemented | No fitting is present in the recovered inference graphs or loader path; broader native initialization/calibration must be considered separately |
| Cross-microphone evidence | Directed peer-mic acoustic features and human solo/overlap/room truth | No peer ownership engine | No peer ownership inputs or labels in these recovered neural graphs; external sidechain routing is a separate product feature |

The Silero file serializes sample-rate and decoder alternatives. Multiple LSTM
nodes in mutually exclusive branches must not be counted as multiple active
layers per frame. The sonible initializer-element total also includes constants;
it is not a directly comparable learned-parameter count.

The actual sonible preprocessor forms48 weighted spectral-power bands, maintains
a per-band running average and constructs a feature divided by the square root
of that average plus a small epsilon. That is feature adaptation, not itself a
neural weight update. Both spectral power and mel features reach its activity
head. Consequently neither "entirely level independent" nor "just another raw
VAD probability" accurately describes all recovered model mathematics. Its
native Level Bias, threshold and envelope consumers still need their own trace.

For a new single-input vocal/podcast Gate, the current evidence supports retaining
the already integrated, MIT-licensed Silero v6.2 and improving the independently
implemented detector-to-gain policy and controls. Recovering a different vendor
network does not establish a reason to replace it, and the recovered proprietary
weights are research artifacts, not CleanSlate product assets. A soft activity
transition, stable temporal behavior, smooth reduction depth, audible difference
monitoring and explicit onset/latency handling are separate DSP responsibilities.
The complete control outline remains in the linked research bundle; its numeric
ranges and defaults have not become a finalized implementation specification.

Desktop's differentiated design is the additional synchronized microphone and
human-reference evidence, not merely a different recurrent cell. Generic speech
activity can retain another person's bleed as speech. Neither a GRU versus LSTM
comparison nor an inactive tensor named `h_emb` proves speaker recognition,
whisper/laughter preservation or better sound. No matched listening or inference
benchmark was run in this research tranche, so no quality-superiority claim is
made.
