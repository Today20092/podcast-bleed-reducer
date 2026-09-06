# Bleed ownership: research and next experiment

Research date: 2026-09-06. Scope: algorithm investigation only. This report makes no production changes, loads no recording, and does not claim a listening test.

## Recommendation

Prototype a **frequency-dependent model of leakage between microphones**, then use its prediction error as evidence of independent wanted sound. Keep the original samples and existing gain renderer. Compare this candidate against the shipped classifier on the newly marked failures and preservation cases before making another application build.

The current detector already includes calibrated level, delay-aware correlation, and residual rules. Adding those ingredients again would repeat the previous work. The next experiment should test whether a richer acoustic model separates copied speech from quiet independent speech well enough to replace the fragile scalar rules. This is a design proposal, not a demonstrated improvement.

The project's [Rust target architecture](C:/Users/User/Documents/Codex/2026-08-14/podcast-automix-rust-gui/docs/research/cleanslate-rust-target-architecture.md) already lists frequency-dependent coherence or adaptive leakage estimates immediately after lag-aware correlation. Its requirement to preserve intended speech and overlap takes precedence over the older [competitive survey](C:/Users/User/Documents/Codex/2026-08-14/podcast-automix-rust-gui/docs/research/competitive-landscape.md) suggestions about gain normalization and final loudness processing. Those are separate product decisions.

## What the existing evidence establishes

The [previous quality report](C:/Users/User/.codex/worktrees/cleanslate-ownership/podcast-automix-rust-gui/.scratch/guided-ownership/evidence/quality-results.md) is direct local evidence, not a hypothesis:

- Guided classification improved combined labeled bleed attenuation from -1.660171 dB to -2.023057 dB. That modest average concealed cases with no attenuation.
- The earlier 828–831 second failure remained at unity on all microphones. The copy tests passed 58/93 frames for A01 to A02 and 61/93 for A01 to A03. Remaining ownership fragments seeded 512 ms padding on both sides and reopened the short copied runs.
- Suppressing short padding seeds improved bleed slightly but damaged a filtered 64 ms independent interjection. That experiment was discarded. It must not be revived as a simple duration threshold.
- The recorded wanted-speech tests showed no added samples below -0.1 dB, but did contain tiny additional gain changes. That does not establish perfect preservation.
- All previous real-audio results are development evidence because tuning inspected them. They cannot become an unseen test set by renaming the files.

The new screenshots show persistent preserved regions that the user identifies by listening as bleed. They are additional failure evidence. Their exact sample bounds, active source and saved calibration state need to come from the parent investigation. This report does not assume that the old padding mechanism is the sole cause of every new mark.

## Why more examples and stronger settings can still fail

Silero's documented output is speech activity or speech timestamps. A voice recorded indirectly is still speech, so a correct speech detector can activate on all microphones. This ownership limitation is an inference from the detector's task, and is directly consistent with the multichannel crosstalk study below. Replacing Silero with another ordinary VAD does not add microphone ownership information. [Silero official repository](https://github.com/snakers4/silero-vad)

Research on personal microphones has encountered exactly this problem. Pfau, Ellis and Stolcke found simple detection unreliable because of crosstalk and channel variation; normalization plus cross-correlation improved their frame error rate. That supports the existing baseline, but does not establish that correlation alone reaches CleanSlate's preservation and bleed goals. [SRI paper abstract](https://www.sri.com/publication/speech-natural-language-pubs/multispeaker-speech-activity-detection-for-the-icsi-meeting-recorder/)

Our design inference is that room tone estimates background noise, while solo examples constrain direct speech and its leakage. Neither label automatically changes how much of a frequency-dependent room response the model can represent. More examples can improve parameter estimates, but cannot remove that representational limit. Examples spanning changed microphone positions may even describe several different acoustic relationships. The experiment should measure this instead of assuming one session-wide average is sufficient.

The attenuation amount determines the depth after the detector chooses attenuation. It cannot correct an ownership decision that remains open. Padding and release also affect the final gain. The next comparison must report both raw ownership and rendered gain, so an improvement in one is not mistaken for an improvement in the other.

## Alternatives and their practical fit

| Approach | Evidence and likely value | Limit for this project |
|---|---|---|
| Ordinary VAD, noise gate, louder-channel rule | Cheap activity and level evidence. Useful baseline features. | Speech-like bleed activates VAD. A louder-channel rule can suppress a quiet participant. It does not solve the new failure class. |
| Dugan gain sharing | Manufacturer documentation describes continuously sharing system gain and reducing inactive channels. | It shares gain when several people speak, which changes wanted-stem levels. This is a different output contract from keeping every real owner at unity. [Dugan manual](https://www.dandugan.com/Assets/manuals/Waves-Dugan-Automixer-Manual.pdf) |
| Banded coherence and learned transfer response | Tests whether another microphone explains the target across frequency, including coloration and phase. The mathematical proposal is below. | Estimates need averaging and a valid acoustic relationship. Common noise, movement, overlapping sources and nonlinear processing can confuse them. Best bounded next experiment. |
| Target-speaker VAD | TS-VAD predicts a separate activity probability for each enrolled speaker and supports simultaneous activity. [Original TS-VAD paper](https://arxiv.org/abs/2005.07272) | Inference: ordinary speaker identity is also present in that person's bleed. Use the enrolled owner of each microphone, not just generic identity matching. Quiet short words, laughter, cross-mic enrollment contamination and this recording domain still need evaluation. |
| Learned multichannel ownership detector | Han and Kumar jointly classify personal-microphone activity from spectral features and outperform their single-channel and energy baselines. [Paper](https://arxiv.org/html/2402.09797v1) | Their model uses one-second windows and labels for ASR filtering. This is useful evidence for learning joint spectral cues, not a ready-made syllable-accurate gain controller. Training data, usable weights and redistribution terms need separate verification. |
| Learned multichannel separation or target extraction | Channel-decorrelation target extraction and FaSNet with TAC show that learned systems can exploit inter-channel differences and variable microphone arrangements. [Target extraction paper](https://arxiv.org/abs/2010.09191), [TAC paper](https://arxiv.org/abs/1910.14104) | These reconstruct audio. They require stronger artifact and identity tests than a gain controller, plus model/runtime work. They are an escalation if ownership-only attenuation cannot meet the quality requirement, not the first replacement. |
| Explicit user-labeled bleed | A negative example says that the target contains no wanted participant during that interval, which is information room tone cannot provide. | A training label and an enforced gain edit have different meanings. Keep them separate. One label does not justify attenuating every superficially similar quiet interval. Start with offline labels; no UI change is needed for the experiment. |

The joint-VAD paper is particularly relevant because it uses one directional microphone per person rather than assuming a calibrated array. Its authors train with playback and re-recording to generate overlap labels. Their evaluation is not evidence of performance on this podcast, laughter preservation, or short interruptions. A neural alternative is credible if the deterministic experiment fails, but it must be trained and assessed for this task. [Han and Kumar](https://arxiv.org/html/2402.09797v1)

## What commercial products document

Auphonic describes joint track analysis and removal of the same or correlated signal from inactive microphones. This confirms that the requested outcome is an established product behavior. The documentation does not disclose a reproducible ownership classifier or prove that its internal method is a neural network, adaptive filter, or a particular coherence rule. Treat it as a possible behavioral comparison, not an algorithm specification. No recording was uploaded. [Auphonic multitrack documentation](https://auphonic.com/help/algorithms/multitrack.html)

iZotope RX 6 De-bleed documents learning a relationship between a bleed-source track and an active track, with aligned audio and a Learn selection. It also documents possible wanted-audio removal at higher strength and FFT-related artifacts. That is a useful precedent for guided pairwise learning, but its proprietary implementation is not available from the manual. Its processing goal also differs from CleanSlate's unchanged samples plus gain envelope. [RX 6 De-bleed manual](https://downloads.izotope.com/docs/rx6/20-de-bleed/index.html)

## Bounded offline prototype

### 1. Freeze the failure and preservation cases

Use exact source identities, channel order and half-open sample ranges. Preserve the user's saved examples unchanged for a faithful baseline. Mark the new bleed intervals as development labels. Include quiet direct speech, independent overlap, breaths and laughter as wanted labels, plus unknown regions that are not silently scored as bleed.

Build only one candidate initially. Compare it with the shipped settings and a controlled baseline that holds the envelope parameters constant. Test calibration from a disjoint time segment. Reserve a new contiguous region or another recording before tuning starts. Adjacent frames from the same utterance are not an independent test set.

### 2. Fit a frequency-dependent prediction from solo examples

For a declared solo owner A and another microphone B, estimate how A predicts B in short-time frequency bins. A regularized estimate is:

```text
S_ab = average(conjugate(X_a) * X_b) over the solo calibration frames
H_ab = S_ab / (S_aa + regularization)
predicted_B = H_ab * X_a
residual_B = X_b - predicted_B
```

This is a proposed least-squares analysis model. Unlike a scalar level ratio and a single waveform correlation, it can represent different gain and phase relationships across bands. Cross-spectral estimation and coherence are established calculations; SciPy documents Welch averaging and coherence as `abs(S_ab)^2 / (S_aa * S_bb)`. This does not require adopting SciPy in the Rust application. [Cross spectral density documentation](https://docs.scipy.org/doc/scipy/reference/generated/scipy.signal.csd.html), [coherence documentation](https://docs.scipy.org/doc/scipy/reference/generated/scipy.signal.coherence.html)

Do not compute coherence from a single unsmoothed complex FFT pair. Algebraically it collapses to one wherever both bins are nonzero, providing no useful evidence. Average multiple observations. Use a short decision hop for onsets and a longer context for stable estimates; choose the contexts in a small preregistered comparison rather than unlimited tuning.

Keep the first model fixed from verified solo examples. Compare a small number of time-local models only if measured residuals show a changed path. Do not adapt through unknown overlap, where fitting can absorb the wanted participant into the leakage model. Do not subtract this prediction from output audio. The residual is only a diagnostic feature for deciding whether B contains additional wanted sound.

AEC is architectural inspiration, not a drop-in replacement. Speex's echo canceller expects the known played signal as reference and documents delay, clock and nonlinear-distortion limitations. Podcast reference microphones are themselves mixtures. A filter fitted on a contaminated reference can explain away some wanted sound, so low residual is evidence rather than proof of absence. [Speex official manual](https://www.speex.org/docs/manual/speex-manual/node7.html)

### 3. Decide ownership jointly, then apply time smoothing

Compare hypotheses that A explains B and C with hypotheses that include independent activity on B or C. Permit several owners. Do not let one unverified pairwise copy vote disqualify a source globally, or let every microphone containing bleed independently veto every other copy decision. The previous experiments already exposed this ambiguity.

Measure the target residual against its room-tone spectrum and the calibration model's own prediction error. Preserve unexplained energetic human sound even if the ordinary speech VAD misses it. Require useful model fit on disjoint solo audio before trusting its residual to remove ownership.

Only after this evidence is evaluated should padding and release form the gain envelope. The experiment should distinguish confirmed direct-speech onsets from uncertain fragments. It must prove that a newly attenuated microphone reopens for a 32 or 64 ms quiet independent word, including filtered copies, and that real laughter remains audible. Removing padding around every short event is already a known bad shortcut.

### 4. Keep preparation and interaction separate

Compute expensive spectral features once from original synchronized audio. Cache compact band powers, cross-spectral summaries, model fit and residual evidence using the existing prepared-analysis ownership boundary. Slider changes should rescore cached evidence and reuse the renderer. Changing calibration may refit a small model from cached sufficient statistics; it must not require rerunning VAD or decoding the entire session.

Record retained bytes and cold preparation time before choosing the cached representation. Keeping every complex FFT bin for all channels and all frames can be much larger than the current cache. Compare only the representation needed by this candidate, with bounded processing chunks and cancellation. Measure warm full-session p95 after implementation instead of assuming that a frequency model is free.

## Acceptance and escalation

The prototype report must show per-interval bleed energy reduction, time below meaningful attenuation levels, longest missed bleed run, raw ownership errors, wanted gain loss, onset recovery, overlap/laughter results, transition artifacts, preparation time, retained memory and warm p95. An average bleed score cannot hide that the user's marked interval remains fully open. Existing gain-tail deviations must be reported separately from newly introduced wanted damage.

Include deterministic tests for gain imbalance, polarity reversal, filtering and room tails, drifting or changing delay, overlapping sources, and a short wanted event beginning while its microphone is attenuated. Require no new material wanted-speech damage and no regression of explicit protection. An experiment that improves only the old labels while damaging a short word is rejected.

If spectral prediction cannot separate the new bleed from independent wanted events at the required preservation level, stop adding scalar exceptions. The next research implementation is a small multichannel activity model trained to predict each microphone owner's activity from synchronized spectral features and representative overlap examples. Target-speaker conditioning is a candidate additional feature, not a replacement for cross-channel acoustics. Full waveform separation remains a later, explicitly different processing mode.

This report recommends that experiment. It does not claim that the current failure is solved, that more user examples guarantee success, or that a published benchmark supplies product-quality weights for this recording.
