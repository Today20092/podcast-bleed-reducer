# Synchronization and recorder clock drift

Research date: 2026-10-11. Scope: roadmap ideas I01–I04, automatic and guided alignment of independently recorded podcast audio, and voice quality during timing correction. This is research and a proposed evaluation direction, not an accepted specification, dependency selection or quality claim. No recordings were processed or listened to.

Owner decision, 2026-10-11: defer synchronization and drift correction. Users align their audio in an external editor; CleanSlate retains its already-synchronized input requirement. The recommendations and experiments below are retained research, not active work.

## Roadmap fit

The [idea inventory](../roadmap-ideas.md) asks for independent microphone/camera/recorder inputs, automatic starting alignment, passage or transient anchors, and shared drift correction across four or five tracks. Its −30/+30 ms example asks whether distributing correction is preferable to moving one extreme by 60 ms. The [public roadmap](../../ROADMAP.md) calls for uncertainty review and audible previews. The [active map](../../.scratch/offline-supervised-migration/map.md) and [accepted specification](../../.scratch/offline-supervised-migration/spec.md) still require supplied synchronization and explicitly prohibit silently stretching, trimming or padding sources. This research does not amend that contract.

## Recommended direction

First evaluate a conservative offline pipeline: find shared events, estimate offsets at many times, robustly fit each recorder's clock mapping, reject unsupported fits, then preview a reversible correction. Start with constant offset plus constant rate error; promote to changing-rate correction only if held-out residuals demonstrate the need. Use guided anchors as the same estimator's input, rather than building an unrelated manual synchronization engine. These are engineering recommendations, not claims that a specific estimator is best on CleanSlate recordings.

For genuine clock error, benchmark high-quality asynchronous resampling first. For substantial intentional duration changes, benchmark pitch-preserving time stretching separately. A file being longer does not establish clock drift: it might have started earlier, ended later, contain an inserted pause, or be a different edit.

## What must be estimated

Use a conceptual mapping from source time to session time, `T = a_i + b_i t_i`, where `a_i` is starting offset and `b_i` is the relative clock-rate correction. Fit in physical time, retaining source rates and sample counts. A varying clock needs a monotonic, smoothly varying map; missing or inserted samples introduce discontinuities that a smooth drift model cannot repair correctly.

Sampling time offset and sampling rate offset are distinct. Temperature changes can produce time-varying rate errors, and changing speakers or positions complicate estimation. The measured acoustic arrival delay also includes propagation from speaker to microphone: it is not simply the recorder's clock offset. The cited work explicitly distinguishes synchronization of clocks from removing physical time-of-flight differences. [Gburrek et al., 2021](https://arxiv.org/html/2110.12820v1)

That paper's experimental method assumes at most one active source at a time and uses two microphones per node for its distance-related processing. Its results are evidence for the problem and candidate techniques, not an off-the-shelf guarantee for CleanSlate's mono-per-speaker tracks and overlapping conversation.

Consequences for this product, derived from that model:

- One clap establishes one correspondence, not a rate estimate. Two well-separated correct correspondences determine an affine map, but provide no independent check. Obtain additional evidence near the middle and across the recording.
- A clap closer to one microphone reaches it earlier. Aligning its peaks exactly may remove a real acoustic delay. Decide whether the target is common physical time, perceptual dialogue alignment, or phase matching for a particular source; these are different requirements.
- A speaker change can change the correlation peak without a clock change. Use repeated evidence from the same speaker/location when possible, or fit source-specific lag intercepts with a common clock slope. Do not interpret every local lag change as drift.
- Dedicated remote microphones with no shared audible signal cannot reveal their relative start time through waveform matching. A guide mix, shared event, trustworthy timing evidence or human correspondence is needed. A disconnected set of tracks must remain unresolved.

## Automatic and guided estimation

Generalized cross-correlation with phase weighting (GCC-PHAT) is a candidate for local delay evidence, alongside normalized correlation. PHAT emphasizes phase agreement; it is not a guarantee of correctness in silence, weak bleed, reverberation or repeated speech. Correlation-based rate estimation and coherence-drift methods address asynchronous sampling explicitly; DXCP and a controlled asynchronous resampler are further candidates if the simpler offline baseline fails. [Correlation-maximization research](https://doi.org/10.1109/TASLP.2016.2517326), [DXCP/ASRC research](https://arxiv.org/abs/2105.13743)

Proposed baseline to test:

1. Group channels known to share a recorder clock. Preserve their relative sample positions; estimate one clock correction per group.
2. Find a coarse shared region using inexpensive analysis copies, then refine candidate delays around it. Keep the full-resolution source for final refinement/rendering. Matching only low-rate analysis frames cannot establish native-sample precision.
3. Collect local delay observations in several separated, informative windows throughout the overlap. Record signal usability, competing peaks, peak width and forward/reverse agreement. Reject silence and ambiguous matches.
4. Fit offset and slope with robust residual rejection (for example RANSAC followed by weighted regression), and reserve windows for validation. Require evidence spanning the supported interval; report poorly supported extrapolation separately.
5. Use good pairwise links across all tracks when a single reference has weak shared audio. Check graph connectivity and cycle consistency; contradictory paths are a review signal. Joint all-signal SRO estimation is an active research direction, but its measured results do not validate this proposed implementation. [Joint estimation paper](https://arxiv.org/abs/2206.13014)
6. Inspect residuals over time. Curvature may support changing-rate correction; a jump may indicate a dropout/edit or a changed acoustic path. Never hide either by silently adding increasingly flexible warp points.
7. Render a preview, remeasure independent events in the actual rendered output, and obtain listening approval before committing a mapping.

For guided operation, ask the user to choose corresponding passages or approximate transient positions; search locally to refine each proposed anchor. Show the source and target waveforms and the refined match, allow disabling refinement, and preserve explicit manual anchors. Request a second distant anchor when rate is unknown and a third check when drift is being corrected. Report residual error in milliseconds and temporal coverage, not an uncalibrated “99% confidence.” Contradictory anchors should be identified, not silently averaged away. These are proposed interactions requiring the owner's agreement.

## Sharing correction across tracks

Pairwise audio evidence establishes relative timing. Without an external clock, adding a common offset and scaling the session clock preserves pairwise consistency: the absolute timeline remains a choice. This is an identifiability consequence of the mapping above, not evidence that averaging clocks improves sound.

For a common elapsed duration `L`, write measured end drift as `d_i` and choose target drift `c`. Corrections are `c - d_i`. Minimizing the largest absolute end correction gives `c = (min(d_i) + max(d_i)) / 2`. Thus −30 and +30 ms produce a 0 ms target and 30 ms corrections at the extremes. Minimizing weighted squared correction instead gives a weighted mean; minimizing absolute correction gives a median. These objectives are different and do not optimize perceptual artifacts. For unequal overlaps, changing rates, or different start times, this endpoint calculation alone is insufficient.

Recommendation: use a fixed reference whenever delivery must follow a video, camera, master recorder or externally specified timeline. An audio-only virtual reference can be offered later if the owner values spreading changes, with explicit weights/locks and deterministic persistence. An uncertain outlier must not pull a midpoint target; reject unreliable estimates before choosing the target. Several channels on one recorder must not acquire extra voting weight merely because it recorded more channels.

Spreading correction reduces a selected mathematical maximum but processes more audio, may move a trusted track unnecessarily and has no established audible benefit for tiny rate errors. It cannot improve the accuracy of the underlying measurements. Do not promise that all tracks will sound better because each moves less.

## Clock correction versus pitch-preserving stretching

For a recorder whose actual sampling clock differs from its declared rate, its nominal playback already contains a corresponding speed/pitch error. Resampling onto a trustworthy target clock corrects both timing and that error. Preserving the nominal playback pitch exactly would preserve the tiny pitch error too. With only relative audio evidence, the physically correct absolute pitch/clock is unknown.

The size is small in the roadmap example. A 60 ms relative error over one hour is approximately 16.67 ppm, or 0.0289 cents of rate-linked pitch change (`1200 log2(1 + 0.060/3600)`). Splitting it gives about 8.33 ppm and 0.0144 cents at each extreme. These are calculations, not listening results or a proposed acceptance threshold. A large percentage stretch is a different problem.

Pitch-preserving time-scale modification changes duration while retaining the intended fundamental-frequency contour. Preserving pitch alone does not guarantee natural timbre, intact consonants, breaths, transients or interchannel phase. WSOLA finds similar overlapping waveforms and is useful for monophonic material; complex mixtures and transients can produce problems. Phase-vocoder methods need phase-coherence/transient handling to reduce characteristic artifacts. No one algorithm is established here as best for speech plus bleed at both ppm-scale and larger changes. [Driedger and Müller review](https://www.audiolabs-erlangen.de/content/resources/MIR/00_PCD_AudioLabs/2016_DriedgerMueller_TSMOverview_AppliedSciences_ePrint.pdf)

There is an especially important timing caveat: a stretcher can deliver the right total duration while moving features locally. Rubber Band explicitly documents a locally varying effective ratio. Consequently, matching output frame count or endpoint anchors does not prove that every word remains synchronized; test interior events and simultaneous bleed. [Rubber Band integration notes](https://breakfastquay.com/rubberband/integration.html)

## Candidates and dependency constraints

The checkout declares MIT licensing and pins `rubato = "=0.16.2"` in [Cargo.toml](../../Cargo.toml). Current Context7 documentation was fetched for Rubato, Rubber Band and Signalsmith Stretch. Context7's Rubato examples describe a newer API; they must not be copied into the pinned version. No dependency upgrade is recommended by this note.

| Candidate | Research-backed capability | Proposed use and remaining check |
| --- | --- | --- |
| Existing Rubato | Pure Rust asynchronous sinc resampling supports adjustable ratios; synchronous FFT resampling does not serve arbitrary varying ratios. Delay and tail handling are explicit. MIT license. [Pinned 0.16.2 docs](https://docs.rs/rubato/0.16.2/rubato/), [upstream](https://github.com/HEnquist/rubato) | First clock-correction benchmark; verify which resampler the app currently uses before reuse. It is not a pitch-preserving duration stretcher. |
| Rubber Band R3 | Offline two-pass processing, independent time/pitch factors, and source-to-output key-frame maps. GPL v2-or-later or commercial license. [Official repository](https://github.com/breakfastquay/rubberband), [API](https://breakfastquay.com/rubberband/code-doc/classRubberBand_1_1RubberBandStretcher.html) | Strong comparison candidate for pitch-preserving stretching; C++ integration and distribution terms need explicit selection. Do not assume an MIT wrapper changes underlying terms. |
| Signalsmith Stretch | C++ pitch/time processing under MIT; documented input/output latency, seeking and flushing; block sizes must average to the desired ratio. [Official repository](https://github.com/Signalsmith-Audio/signalsmith-stretch) | Permissive candidate to compare with Rubber Band. Validate frame accounting, transient timing, long-session memory and Rust binding coverage before adoption. |
| SoundTouch | WSOLA-like tempo changes and separate rate/pitch controls; LGPL 2.1, with a commercial alternative stated by its author. [Official README](https://www.surina.net/soundtouch/README.html) | Useful time-domain comparison, not an assumed quality winner. Packaging and exact timing behavior require evaluation. |

Rubber Band's author recommends the offline Finer engine for quality-oriented time stretching. Pitch factor 1 requests unchanged pitch; its formant-preservation option is discussed for pitch shifting, and should not be treated as a magic repair switch for ordinary time-only correction. The R2 technical notes describe a phase vocoder with transient resets and phase-coherence handling; they explicitly do not fully describe R3. [Integration](https://breakfastquay.com/rubberband/integration.html), [technical notes](https://breakfastquay.com/rubberband/technical.html)

Rubber Band key-frame maps do not define overall output duration: the overall time ratio must also be supplied, and maps are offline-only. Signalsmith's current Context7 docs describe a whole-buffer `exact()` operation, but a two-hour, multitrack application should evaluate chunked processing and bounded memory, not adopt whole-session buffering by default. [Rubber Band API](https://breakfastquay.com/rubberband/code-doc/classRubberBand_1_1RubberBandStretcher.html), [Signalsmith processing documentation](https://github.com/signalsmith-audio/signalsmith-stretch/blob/main/_autodocs/api-reference/signalsmith-stretch-processing.md)

Commercial Rubber Band licenses permit proprietary distribution; evaluate before purchasing and verify the chosen license/build dependencies at adoption. No purchase or legal compatibility determination was made here. [Publisher licensing terms](https://breakfastquay.com/technology/license.html)

## Architecture decisions before implementation

These are proposed requirements, not code findings. No code graph or call-chain audit was performed because this task changes no domain behavior.

- Persist originals, recorder groups, source identity, anchors, mapping version, chosen reference and evidence. Corrections should be reversible and recomputed from the originals, never from previously stretched exports.
- Decide source-time versus session-time attachment for labels, gain points and protections. Changing synchronization must invalidate dependent analysis, waveforms, playback and exports consistently, and must not silently move a human decision to different speech.
- Use one accepted mapping for analysis, display, audition and rendering. A preview processed with a different timing algorithm cannot qualify the export.
- Define unequal duration and partial-overlap behavior separately from drift: earliest/latest session coverage, leading/trailing silence, clip boundaries, missing material and output bounds. Padding belongs to explicit delivery policy, not evidence that sources match.
- Decide whether camera input means audio extracted elsewhere or direct video-container decoding. Audio alignment alone does not qualify video demuxing, timestamps, variable frame rates or picture retiming.
- Keep discontinuity repair and multiple clips per speaker explicit scope choices. A dropped sample block is missing information; stretching surrounding speech does not recover it.
- Revise native frame-count and BWF timing promises deliberately if corrected exports are introduced. Preserve metadata provenance; original timestamps are not automatically valid for a newly mapped timeline.

## Validation plan

This is a proposed experiment plan. Numerical pass limits must be agreed and frozen before independent evaluation.

1. Generate truth-known fixtures with positive/negative offsets, fractional-sample offsets, constant drift, smooth changing drift and explicit sample insertions/deletions. Include shared-clock multichannel groups, 44.1/48/96 kHz, unequal lengths and partial overlaps.
2. Test clock error separately from acoustic delay: different microphone distances, speaker changes, movement, reverberation, polarity inversion, low shared bleed, noise, repeated phrases, silence, clipping and lossy decode boundaries. Include no-shared-audio and disconnected cases that must fail safely.
3. Measure held-out correspondence residuals through the entire overlap: median, high percentile, maximum, slope error in ppm, error by speaker, unsupported coverage and false confident acceptance. Report onset/endpoint error in output samples separately from perceptual synchronization.
4. Compare normalized correlation/GCC-PHAT plus robust affine fitting against a coherence-drift method only if the baseline's failures justify the added complexity. Freeze estimator parameters before the independent set.
5. Compare identity/no correction, sinc resampling, Rubber Band and Signalsmith (optionally SoundTouch) at realistic ppm corrections and separately at deliberate larger stretches. Check pitch contour, transient displacement, spectral coloration and mix/bleed combing. Pitch estimates or waveform nulls alone are not perceptual quality verdicts.
6. Conduct blinded listening with the owner: individual voices and simultaneous tracks; quiet words, consonants, breaths, laughs, overlap and room tone; start, middle, end and map changes. Count audible artifacts and incorrect matches, retaining rejected examples.
7. Verify exact planned frame count, startup delay compensation, tail flushing, no missing first/last speech, monotonic mapping, stable seeks, linked-channel coherence and parity between preview and final export. An exact final duration is necessary but insufficient.
8. Measure four/five-track and eight-track two-hour cases on the owner's machine: CPU time, peak memory, cache growth and cancellation. Check undo/save/reload/relink, stale jobs and failed/cancelled exports before claiming readiness.

## Owner discussion, first round

The owner described the intended case as microphones recorded on separate recorders, commonly with a clap or another shared alignment event at the start. Support for remote isolated recordings, video-container import, edited clips and restarts was not confirmed.

The owner questioned whether synchronization belongs in CleanSlate at all: users may align multicamera recordings and audio in their nonlinear editor, then export synchronized audio for CleanSlate. This is an unsettled product-scope decision, not approval of a fixed or shared reference clock and not a decision to implement alignment.

If synchronization is pursued, the owner selected natural voices, with processing chosen through measurement and listening, rather than exact preservation of uncorrected playback pitch. Every proposed synchronization should receive preview and explicit acceptance before application.

In the second round, the owner selected: "Defer synchronization; users align in their editor." This closes the current interview. No synchronization, stretching or drift-checking implementation is authorized. Keep the findings and conditional preferences for a future concrete need.

## Archived questions if scope is reopened

These questions are not pending while synchronization is deferred. If the scope is reopened, start with cases and timing policy before selecting algorithms:

1. Which real recordings must work first: colocated microphones with bleed, camera audio, remote isolated tracks plus a guide mix, or edited/restarted files? Will camera audio arrive as audio files or video containers?
2. Must the finished audio keep sync with an unchanged video/master clock, or may an audio-only session choose a shared virtual clock? If both, which should be the default?
3. Is the first version for continuous recorder drift, or must it also repair pauses, missing blocks and multiple clips? How should unsupported recordings be handed back for manual work?
4. Does “keep pitch correct” mean faithful correction of a recorder's clock error, or exact preservation of how each uncorrected file currently sounds? The distinction is tiny for the example but determines the processing objective.
5. Should automatic alignment always propose a preview for approval, or can well-supported cases apply reversibly? How much guided work is acceptable when the signal is weak: a passage, two distant anchors, or a third validation anchor?
6. Is perceptual dialogue alignment sufficient, or is there a downstream requirement for phase-sensitive mixing? What error or audible change would make a result unacceptable?
7. Is spreading small corrections a preference that can yield to measured quality and video constraints, or a mandatory behavior? Should a trusted recorder be lockable?
8. How should corrected exports cover partial recordings, and what happens to labels and gain edits when alignment changes after editing?

No universal “best” algorithm or transparent quality guarantee is established by this literature review. If this work is reopened, agree a supported-case contract before a measured, listening-reviewed comparison on representative recordings.
