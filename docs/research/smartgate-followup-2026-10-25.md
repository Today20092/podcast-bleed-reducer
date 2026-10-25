# smart:gate follow-up: useful boundaries for CleanSlate

Owner correction after this report: stop the reference investigation here. No sonible installation, activation or runtime testing is requested or required for CleanSlate development. The proposed runtime stage below is retained as historical research planning only; it is not the next task. Continue the accepted Session implementation. Unmeasured vendor behavior remains unknown.

Continues [ticket 79](../../.scratch/offline-supervised-editor/issues/79-sonible-static-reference-investigation.md) and the [first static pass](sonible-static-pass-2026-10-24.md). This pass investigates the supplied smart:gate 1.0.1 Windows VST3 and Speech.nn without running sonible code. It informs the accepted [Session/Live Mic design](../../.scratch/offline-supervised-migration/plugin-modes-interview.md); it does not change the implementation or claim detector equivalence.

## Findings grounded in the supplied manual

Page numbers below are the manual's printed pages. Source SHA256 is recorded in the first-pass inventory.

| Finding | Source | Consequence for CleanSlate |
|---|---|---|
| Target activity, the trigger threshold, gate state and output are distinct. Activity is described as the amount of target activity, not calibrated certainty. | pp. 6–7 | Show activity separately from applied attenuation. Do not promise a probability of correctness. |
| Profiles select source categories. The manual does not claim personal speaker recognition. | pp. 7–8 | A speech detector suits Live Mic; it cannot identify which person owns speech across podcast microphones. |
| Level bias controls amplitude dependence. Low bias can retain detection in quiet target passages; high bias follows level more closely. | p. 9 | Separate speech detection from optional level sensitivity; quiet speech needs its own regression cases. |
| Tolerance creates a soft region around the activity threshold. At the documented 10% setting, opening starts at 90% of the threshold. | p. 9 | Do not describe this as recovered two-threshold hysteresis. CleanSlate's proposed hysteresis remains its own policy decision. |
| Attack, release hold and release have separate roles. Hold retains full opening before the closing envelope starts. | p. 9 | Keep the detector and temporal gain controller separate; test pauses, word onsets and endings independently. |
| Impact allows partial suppression; full impact closes completely. Three frequency bands can have independent impact. | p. 10 | Finite attenuation is a useful default. Multiband control is outside the current accepted Live Mic scope. |
| Profile selection waits for representative target audio, then initializes content-aware processing. Switching back to an initialized profile avoids another initialization; reset clears it. | p. 8 | Display waiting/ready/reset states. Initialization is not evidence of model training, enrollment or personal identity learning. Persistence across host restarts is unknown. |
| Difference monitoring auditions what processing removed. | p. 11 | A Removed audio audition would make excessive suppression and chopped syllables easier to hear. |
| External sidechain, stereo analysis modes and a multithreading setting are documented. | pp. 11, 13 | These do not establish instance coordination, callback scheduling, latency or measured CPU cost. |

The manual does not establish detector frame size, exact latency/lookahead, gate equations, calibrated confidence, or history identity when transport loops or seeks. Its source-category gate is not a replacement for CleanSlate's synchronized Session analysis.

## Additional source-byte and native evidence

The VST3 SHA256 remains `edeb18dcb0b7557123cc46036fd17b2d08cbd45d3bf1cb882e4b5a8a4a35f95c`. New evidence is retained outside the repo under `C:/Users/User/Documents/Codex/Investigations/sonible-2026-10-24/`:

- `scan-gate-strings.ps1`: linear source-byte scan for printable ASCII and UTF16LE runs of 5–1024 characters, keyword-filtered. ASCII and UTF16 sample assertions pass. This reads bytes without loading the target DLL.
- `smartgate-followup-string-candidates.json`: 68 selected candidates from 1,085 initial keyword matches, preserving original byte offsets, source identity and selection scope. These are candidates, not executed call paths.
- `smartgate-profile-section-comparison.json`: all ten profile identities, export names/addresses and hashes of complete source-backed `.text` and `.rdata` sections, using the retained PE parser.
- `smartgate-speech-followup-evidence.json`: nine retained REA records, 450,054 bytes. Includes four new native function dossiers.
- `smartgate-speech-followup-snapshot.json`: saved native analysis for those four queries, avoiding an unnecessary repeat of analysis.

**Observed type information:** the VST3 contains MSVC type-name strings for an `OnnxInferenceInterface` specialization involving `SpectralBlock`, `RealImagInterleaved` and vectors. Example source offsets are 52,686,256 and 52,993,776. This supports a spectral-interface hypothesis; it does not establish FFT/window/hop sizes, feature normalization, mel features, the complete model graph, or an executed inference route. ONNX error strings also describe input/output dimension checks; they do not reveal the actual tensor dimensions.

**Observed wrapper linkage candidates:** source offsets 48,706,296 onward contain the same export names found in Speech.nn, adjacent to dynamic-library loading diagnostics. A nearby learning-controller diagnostic refers to asynchronous learning. Together with the first-pass native exports this supports dynamic profile-loading as a candidate architecture. No code xrefs in the large VST3 were recovered, so execution, thread scheduling and training semantics are not established. Generic runtime strings such as LowLatencyMode or ONNX session options are not evidence that a product mode or configuration is enabled.

**Observed profile relationship:** Speech.nn and Vocals.nn have identical complete `.text` section SHA256 `61dc14ea0e11f3deb1b1044251a5b3533d80024fe7801777ad24d0cc690deb12`, with different complete `.rdata` hashes. All ten profiles have distinct `.rdata` hashes and six distinct `.text` hashes across the set. This establishes shared native code bytes for Speech/Vocals, not equivalent models, behavior or graph topology. Different profile data could include weights, metadata or other encoded content; no proprietary model was decoded or executed.

**Focused native trace:** the Speech wrapper's previously observed `stopHammerTime` route reaches `FUN_1800062f0`, using Windows once-only initialization. Its two helper calls resolve to an identity wrapper and a call to `FUN_1800038a0`, which iterates native containers. No neural graph or signal-processing equations were recovered from that route. New dossier IDs:

- `ev_acc4ffb1c3b905e6ec26ae47df25f76b702685b8637d6d3bfd8e9dbcd2059af7` — once-only helper.
- `ev_ac26cb6f9e3df2afc46f75ce2baf69c8309847be083b3ada3a1607fbfd1ec315` — identity wrapper.
- `ev_4318d174f5f0d3ca210d7222fb0f09e7c3be770a842ed4c91da49732c5b49d30` — forwarding helper.
- `ev_068c49086cfdd38a9b734694e81888093b2a540aca6a7264a151fd9ede8c1e92` — container iteration.

A bounded ASCII keyword scan of Speech.nn found runtime/export strings but no useful network dimension/operator labels. This is a negative result for that search only. The large VST3 was not auto-analyzed; only the 1.6 MB Speech library was. Native evidence was exported and the session closed.

## What to carry into our plugin

Retain the accepted two-mode design. Session analyzes captured synchronized microphones and preserves source-time history. Live Mic detects speech and applies an independent conservative gate controller. Sonible's source-category profiles provide no evidence that single-channel speech detection solves podcast speaker ownership.

For Live Mic, the most useful references are explicit detector readiness, separate activity/gain displays, quiet-speech protection, distinct hold/release behavior, finite attenuation and removed-audio audition. These are recommendations for later scoped implementation; this investigation does not add product controls. The existing embedded Silero path remains a candidate that needs a bounded streaming adapter and measurements, not a proven sonible equivalent. Do not copy proprietary payloads into CleanSlate.

## Next evidence: controlled audio behavior

The next useful boundary needs a valid runnable smart:gate installation/trial. Common VST3 locations checked in this pass do not contain smart:gate. Owner action: install smart:gate's 64-bit VST3 and complete the normal vendor trial/license activation if needed. Other sonible products are unnecessary for these questions. No activation, installer or DAW was started by this investigation.

After installation, establish whether a small host can safely expose its parameters, processing and initialization. The existing CleanSlate host harness is not yet claimed to support an arbitrary third-party plugin. If activation or initialization requires its GUI, ask for that one interaction; avoid routine full-DAW testing. Keep a processing/latency receipt with version, sample rate, block size, parameter state, initialization state and source identity.

| Fixture / comparison | What it answers |
|---|---|
| Same speech at several gains; quiet sentence endings | Content sensitivity versus level bias; missed speech |
| Noise-only keyboard/fan/room tone and noise mixed with speech | False openings versus speech preservation |
| Word onsets, brief pauses and sustained word endings | Onset loss, hold behavior, closing artifacts |
| Other person's speech and overlapping speech | Category detection versus speaker identity; no assumed personal recognition |
| Repeat renders with fixed state, varied block sizes and sample rates | Determinism, timing alignment, reported versus effective latency |
| Save/reopen with initialized and reset profile states | What initialization actually persists |
| Original, processed and difference renders | Audible damage and suppression amount, alongside objective timings |

Label speech truth independently before comparing detectors. Align for measured latency and match attenuation/timing before comparing sound. Human listening remains necessary for acceptable syllable loss and artifacts; output renders and basic timing/CPU/memory checks can be automated if the runnable plugin permits a small host.

## Completion boundary

Answered: documented controls/readiness; shared Speech/Vocals code relationship; spectral interface type evidence; bounded wrapper trace. Partially answered: profile loading and detection/control architecture. Unresolved: model topology, actual inference cadence, initialization semantics, latency, quality, host persistence and resource costs.

No sonible target was executed and no CleanSlate code/binary/link was changed. Source inventory hashes and new evidence files are checked; the native analysis process is closed. This is completion of the static follow-up, not completion of the plugin build or behavioral qualification.
