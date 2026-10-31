# sonible smart:gate controls and our podcast-vocal priorities

Research date: 2026-10-31. Public documentation only; no plugin installation,
runtime comparison, proprietary extraction, user-audio upload or product edits.
Vendor facts below are separate from our proposed controls and implementation.

## Product/version scope

The current official page identifies **smart:gate 1.0.1**, approximately **80 ms**
latency, 44.1–192 kHz support, Windows VST/VST3/AAX and Mac VST/VST3/AU/AAX.
It presents one smart:gate product and a fully functional 30-day demo; the reviewed
page does not establish a separate Pro/Lite feature tier. These are vendor-listed
specifications, not a checked installed binary. [Official product/specifications](https://www.sonible.com/smartgate/)

The current [official manual index](https://help.sonible.com/hc/en-us/articles/360017065019-Where-do-I-find-the-latest-manual-for-my-plug-in)
(updated September 21, 2026) links the 14-page
[English manual](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf).
The dated URL is not evidence of a newer executable or an independent tier.

## Detection, profiles and learning: documented distinctions

- Target selection conditions processing on a source class rather than overall
  input level. Official examples include vocals, electric guitar and piano.
  [Product](https://www.sonible.com/smartgate/)
- The official marketing menu image visibly includes **Vocals, Speech, Guitar,
  Bass, Keys, Kick**, with **Snare** selected. It is cropped, not a verified
  exhaustive current menu. [Official profile-menu image](https://www.sonible.com/wp-content/uploads/2023/07/feature-1_smgate.jpg)
- New instances use **No Target**. For conventional level gating, sonible advises
  No Target plus Level Bias **100**. [Conventional-gate FAQ](https://help.sonible.com/hc/en-us/articles/9636773988636-How-can-I-use-smart-gate-as-a-conventional-level-driven-gate)
- Profile initialization uses representative playback; detection initializes the
  profile, and an adjacent reset icon reinitializes it. [Manual p.8](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf#page=8)
- There is **no Learn button and no automatic parameter setting** after profile
  selection. Gate settings remain the user's decisions. Initialization therefore
  must not be described as automatic threshold/timing calibration or speaker
  training. [No-learning FAQ](https://help.sonible.com/hc/en-us/articles/9636750686364-Why-is-there-no-learning-button-and-no-auto-parametrization-in-smart-gate)
- Level Bias blends emphasis toward target level: high follows level; low reacts
  strongly even to quiet detected target content. [Level Bias FAQ](https://help.sonible.com/hc/en-us/articles/9636746490140-Is-smart-gate-s-processing-level-independent-and-what-does-the-Level-Bias-do)
- sonible explicitly describes a realtime neural network requiring temporal
  context; its approximately 80 ms latency is why sonible advises against live
  recording use. Realtime analysis is not the same as low-latency live monitoring.
  [Live-use FAQ](https://help.sonible.com/hc/en-us/articles/9636748686620-Can-I-use-smart-gate-while-recording-or-in-live-situations)

The reviewed official material does **not establish** neural architecture,
training corpus, weights, calibration, individualized voiceprints, speaker
ownership, or guaranteed preservation of whispers, breaths, laughter or overlapping
speakers. Do not infer those internals from source-class selection. The vendor's
vocal tutorial demonstrates rumble/guitar-bleed handling and recommends adjusting
hold/release; that is an illustrated vendor workflow, not a podcast preservation
benchmark. [Official vocal tutorial](https://www.sonible.com/blog/smartgate-vocal-track/)

## Documented control inventory

The terse manual-derived descriptions below describe semantics, not undocumented
factory presets or measured performance.

| Control / feedback | Documented role | Primary source |
| --- | --- | --- |
| Activity threshold | Opens above activity threshold. | [p.7](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf#page=7) |
| Attack / Release | Opening / closing times. | [p.9](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf#page=9) |
| Release Hold | Delay before closing begins. | [p.9](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf#page=9) |
| Tolerance | Relative transition zone; softer switching. | [p.9](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf#page=9) |
| Impact | 0=open; 100=closed; intermediate=partial closure. | [p.10](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf#page=10) |
| Band suppression | Low/mid/high; movable crossovers; enable/solo. Positive increases suppression; maximum stays suppressed; negative reduces it. | [p.10](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf#page=10) |
| Gating / Ducking | Close below / above threshold. | [p.11](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf#page=11) |
| External sidechain | External signal drives detection/ducking. | [p.11](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf#page=11) |
| Detection | Mono mixdown or Side-only analysis/processing. | [p.11](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf#page=11) |
| Diff. Signal | Audition input-minus-output; highlight delta history. | [p.11](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf#page=11) |
| Displays/meters | Activity, gate, output RMS/peak; input/output history. | [p.6](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf#page=6) |
| States | Eight comparison slots; copy/clear. | [pp.6,12](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf#page=12) |
| Presets | Save/load/share `.spr` settings. | [p.12](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf#page=12) |
| Settings | Tooltips, OpenGL, multithreading, data sharing, tour, license/update information. | [p.13](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf#page=13) |
| Instant Replay | Advertised alongside activity/gating visualization; implementation and buffer duration unspecified here. | [Product](https://www.sonible.com/smartgate/) |

No separate detector high-pass/low-pass, output gain, wet/dry, lookahead control,
complete automatable-parameter list or automatable state-slot switching was
established by these official sources. Their absence from this inventory is not
a claim that no executable version supports them. Band suppression must not be
renamed a detector-frequency filter: those are different operations.

Numeric threshold/time defaults and full ranges were not established. Values
shown in tutorial/manual screenshots are examples, not guaranteed startup values.
The manual's preset paragraph mentions reference tracks without documenting a
matching reference-track workflow elsewhere; do not infer a custom-learning feature.

For CPU overloads sonible advises larger DAW blocks and enabled multithreading;
its separate threading FAQ explains that multiple analysis threads can be disabled
if problematic. This documents a vendor setting, not the appropriate architecture
or fix for our callback/worker. [Overload FAQ](https://help.sonible.com/hc/en-us/articles/9636822418972-What-can-I-do-if-I-receive-CPU-overload-errors-with-smart-gate),
[Threading FAQ](https://help.sonible.com/hc/en-us/articles/9636795131292-What-does-the-option-Enable-multi-threaded-processing-on-the-settings-page-do)

## What our Gate actually has

Three snapshots must not be conflated. This report branch is frozen from
`2212416`; the delivered preview binary is from `2e3adbf` with the original fixed
−6 dB engineering floor. Newer committed GUI-actor source **`4afa18a`** already
has a real host parameter **Range 0–6 dB, default 6 dB**, plus last-block gain and
history feedback. The newer source was inspected through `git show`, not edited
or assumed to be the installed binary. Its range setting survives validated state
restoration, which still forces bypass and never restores temporary arming.
Sources: `git show 4afa18a:plugins/smart-gate/src/lib.rs`, `gain.rs`, `engine.rs`
and `history.rs`. No claim about that actor's later uncommitted GUI is made.

The actual detector is fixed embedded **Silero VAD v5**, not a per-person model.
It consumes 512 new 16 kHz samples per inference (32 ms), with past recurrent/FIR
context and an asynchronous worker. Stereo takes the greater score from two
independent detectors and applies one linked gain. It has no peer microphone
ownership, session training, external sidechain or user detector EQ. The fixed
anti-alias FIR is not an adjustable detection filter. Zero added audio delay and
zero reported host latency do not mean zero detection/decision delay.
[Detector source](../../plugins/gate-detector/src/lib.rs),
[worker](../../plugins/smart-gate/src/worker.rs),
[gain policy](../../plugins/smart-gate/src/gain.rs).

Other engineering constants remain opening score **0.10**, closing score
**0.01**, **300 ms** last-strong-activity hold, approximately **250 ms** closing
ramp, **5 ms** reopening/unity-return ramp, and **100 ms** evidence freshness.
These are existing values, not calibrated product defaults or preservation
acceptance. There is no existing Learn/profile selection, adjustable sensitivity
or timing control. The fixed-floor tree and newer Range source use the same
basic detector/gain logic. The stale-data deadline is a safety rule, not a
user-facing gate Release control.

The [existing detector research](smart-gate-detector-92.md#2026-10-26-measured-follow-up-and-bounded-decision)
records low whole-clip scores for whisper and laughter smoke cases. Those means
are not error rates, calibrated confidence or safe thresholds. They are enough
to reject a guarantee that a Speech VAD plus more aggressive gating will preserve
breaths, chuckles, whispers and quiet word starts. The original sustained
realtime mono128 fault also remains causally unresolved; controls cannot resolve
unqualified worker/overload behavior.

## Recommended controls for podcast vocals

**Recommendation, not implemented parity:** start with **Sensitivity, Range,
Attack, Hold, Release and Removed-audio audition**, with truthful detection/gain
feedback. Keep source type fixed to Speech until distinct profiles really exist.
Do not add a fake Learn button or promise speaker isolation. Defaults below are
conservative proposals/reused engineering settings, pending matched listening.

| Priority / control | Proposed default semantics | Podcast rationale and DSP implication | Actual status |
| --- | --- | --- | --- |
| Must retain: Bypass and explicit experimental Arm | Bypass on; arming temporary, never recalled/automated | Allows safe loading and clear original-audio recovery. Keep experimental consent separate from saved sound settings. | Exists |
| Must have: Sensitivity | Normal initially maps to current 0.10/0.01 opening/closing scores; higher sensitivity retains weaker detected activity | Needed for quiet speech; a low score is not proof of noise. One monotonic user control can drive an ordered hysteresis pair. More sensitivity can also admit another speaker's bleed. | New DSP/parameter mapping needed |
| Must retain: Range, in dB | 6 dB reduction ceiling; 0 dB leaves unity. Retain current 0–6 dB limits first | Finite attenuation retains room and reduces the cost of a VAD miss. “6 dB Range” means maximum cut, not added gain or a percent wet/dry blend. | Real in `4afa18a`; fixed 6 dB in delivered preview |
| Must have: Attack / opening | Initially retain approximately 5 ms after a valid opening decision | Smooth reopening without claiming it precedes the 32 ms detector evidence. Bound/smooth automation; test first consonants separately. | Fixed internally; expose only with verified semantics |
| Must have: Hold / pause protection | Initially retain 300 ms after last accepted strong voice activity | Prevents rapid attenuation between words and many breaths. Define this timing anchor explicitly; do not silently change it to a different post-threshold hold. | Fixed internally |
| Must have: Release / closing | Initially retain 250 ms toward the chosen finite floor | Natural tails/room decay; fast closing can damage quiet syllables and ambience. Rate- and block-independent gain policy, not delayed evidence reuse. | Fixed internally |
| Must have: Original / Processed / Removed audition | Processed as normal mode; Removed is explicit temporary monitoring | Listening to what is cut exposes breaths, chuckles and syllables being lost. Difference must use time-aligned original-minus-output, not a separately inferred stem. | Not exposed in Gate |
| Must have: activity, gain reduction and status | Show score/activity separately from actual applied reduction; mark Waiting/Failed/Bypass | “Voice detected” is not “this microphone owns the voice.” Show last-block/current-source validity, not a fictitious continuous playback or quality assertion. | Readiness exists; newer gain/history exists; complete activity feedback needs real data |
| Next: editable hysteresis / softness | Stable linked opening/closing behavior; retain existing gap until evaluated | Reduces chatter, but should not compete with Sensitivity on the main surface. Current 0.01/0.10 hysteresis is not the vendor's illustrated 10% Tolerance. | New control/validation |
| Next: optional onset-preservation lookahead | Off for the current zero-delay path; explicit higher-latency editing mode only after measurement | Deeper gating needs a way to preserve starts before delayed VAD evidence arrives. Requires an audio delay, correct sample alignment, reported latency/PDC and reset tests. Do not copy an unmeasured 80 ms value. | No delay path today |
| Next: deeper Range and A/B settings | Consider bounded deeper reduction only after onset/tail tests; A/B recalls sound settings, never Arm | Useful cleanup strength without promising silence or false-negative immunity. Extend the present safety cap only with listening evidence; two comparison slots suffice initially. | Proposed |
| Next/advanced: analysis-only detector HP/LP | Off by default; listenable detector path if added | May reduce rumble/plosive-driven triggering, but filtering can alter neural input distribution and lose low/soft voice. Does not EQ the audible output and is not vendor Band Suppression. | New filter/state/quality work |
| Optional: external sidechain and Ducking | Internal detection and Gate remain defaults | Useful for a clean guide mic or voice-over/music workflow; requires real adapter buses, channel mapping and host routing tests. It is not session ownership learning. | Not supported |
| Optional: multiband reduction or M/S | Full-band, linked mono/stereo remain defaults | More control over rumble/high noise costs crossovers, band envelopes and possible tonal/transient artifacts. Validate against the simple linked envelope before adding complexity. | Not supported |

For now, **no independent “Preserve breaths”, “Preserve laughter”, “Learn my voice”
or “Remove other speakers” switch**. Those labels imply evidence/classes the
current detector does not supply. Keep breaths/laughter through conservative
range/hold/release, manual bypass or a future validated detector/context policy;
none is a categorical preservation guarantee. A broad-band gate also cannot
remove background noise underneath wanted speech while leaving that speech
unchanged. Denoising/separation is a distinct algorithm and acceptance task.

## Proposed original interface groups

This is a control/workflow specification for the GUI owner, not a visual clone
or instruction to edit the current GUI branch.

1. **Signal workspace:** original/output history, a separately labelled speech
   activity lane, actual gain-reduction envelope and a clear last-block status.
   Missing/stale evidence makes gaps explicit. If an old passage is recomputed
   after a control change, label it as an estimate, never already-applied audio.
2. **Keep voice:** Sensitivity and Range, with numeric values and short help.
   Avoid a profile menu full of unsupported instruments or a percentage claiming
   calibrated speech certainty. “Speech” may be an honest fixed mode label.
3. **Transitions:** Attack, Hold and Release; put advanced hysteresis/softness
   behind disclosure. Show units. A small finite curve can explain the timing
   without turning reference screenshot numbers into our defaults.
4. **Check the result:** Original/Processed/Removed, A/B settings when implemented,
   Bypass and separate experimental consent. Removed audition should be visually
   unmistakable and should not silently persist as a restored monitoring mode.
5. **Advanced routing:** detector filtering and external sidechain/ducking only
   when backed by working DSP and actual host buses. Show latency as a measured
   contract; do not offer an unimplemented low-latency/quality mode selector.

## DSP, automation and validation implications

Parameter IDs/ranges need stable state migration, validation and bounded
callback-safe reads. Preserve valid sound settings while forcing restored bypass
and retiring consent; new parameters must not be erased by the old blanket
`filter_state` implementation. The `4afa18a` Range migration is the existing
example. Parameter interpolation, threshold retargeting, rate changes, bypass
and resets need sample-level tests; no GUI locks, allocation or model reload in
the callback. Changing a control must not relabel stale evidence as fresh.

Keep the detector's signal/sample-time identity distinct from gain settings.
Changing smoothing/range should not require pretending to retrain the VAD.
Audio lookahead, if accepted, must delay the original and processed references
consistently and report the real latency. Delayed processing is not a GUI-only
feature. Greater reduction should follow, not precede, onset-preservation work.

Before choosing new defaults, use independent labelled podcast excerpts with
quiet voiced speech/whispers, inhalations, natural laughter, short acknowledgments,
speaker overlap, intermittent room noise and another speaker's mic bleed. Measure
wanted-sound attenuation and onset/tail damage separately from unwanted reduction;
listen to original, processed and removed signals at matched levels. Exercise
44.1/48/96 kHz, mono/stereo, varied blocks, noise changes, resets and actual host
automation. A whole-clip probability mean or a synthetic silence case cannot
select a universal threshold. Native realtime/overload and speech-quality
acceptance remain independent of a control looking correct.

## Desktop versus our Gate versus sonible: where an advantage could exist

The companion source audit is commit `ee544520652383ddd9d09720ebe3e6409fcce542`,
`docs/research/main-engine-vs-smartgate-2026-10-31.md`. It inspects Desktop
`development a2606de` (same `src`/Cargo as `lusca e2a5f8d`) and Gate `4afa18a`.
The following is a concise architecture comparison, not a sound-quality ranking.

| Dimension | Main Desktop engine | Our actual current Gate | sonible public documentation |
| --- | --- | --- | --- |
| Decision/evidence | Synchronized assigned microphones; fixed VAD plus room/relative-level/dominance and directed peer-bleed features | One input's generic Silero speech presence; stereo max of two scores | Selected source-class activity; external sidechain is also available. Internal features/model remain proprietary/unknown here. |
| What learns | Per-session per-track binary logistic ownership models and trusted-solo acoustic transfer fits from explicit solo/overlap/room labels | Fixed neural weights; no session trainer, ownership labels or mic-transfer fitting | Profile initialization is documented; no Learn/automatic gate parameter setting. Training data, architecture and personal adaptation are not established. |
| Time/correction | Whole-source offline Analyze, exact human constraints, manual protections/effective envelopes | Causal fixed-frame worker and gain policy; temporary arming; no source-project correction workflow | Approximate 80 ms latency and temporal context; gate controls, comparisons and visual feedback documented. |
| Potential advantage | More task-specific information and editable human correction for synchronized dedicated-mic podcast bleed | Simple bounded single-input path; no claimed inherited ownership advantage | Broader mature control/routing surface is documented; measured comparative preservation/performance is unknown. |

Desktop sources: `src/session_ownership.rs::features/feature_names`,
`crates/ownership-model/src/lib.rs::OwnershipModel::train`,
`src/acoustics.rs::AcousticEvidenceCache::analyze`,
`src/acoustic_transfer.rs::qualify`,
`src/effective_envelope.rs::EffectiveTrackEnvelope::gain_at_time`, and
`src/editor_workflow.rs::EditorAnalysisWorker` at the audited snapshot.
The feature count is `3 + 5*(microphones-1)`. The qualified scalar/two-tap FIR
fits are decision features, not destructive bleed subtraction. Independent
ownership outputs allow overlap. Session fitting does not fine-tune Silero;
there is no current Nemotron/ASR/voiceprint path.

**Yes, we have a potential information/workflow advantage in Desktop's dedicated
multi-mic supervised problem. We have not demonstrated that we sound better than
sonible, and that advantage is not present in the current single-input Gate.**
The vendor's sidechain option means it should not simply be described as always
single-signal, either. Its undocumented internals cannot be ranked against ours.
Our fixed Silero reuse is not evidence of equivalent or superior source-class
detection. More features or linear-versus-neural architecture alone do not rank
quality. Desktop still has an explicitly ignored quiet-overlap qualification
case; extra information is not a guarantee of preservation.

Do not port whole-session training, future context or peer-dependent ownership
into a live callback. If session-aware mic bleed is the desired differentiator,
keep it in the supervised Desktop/Podcast path or first accept a new host input,
training, latency and state contract for a separate feature. For this Gate,
finish truthful controls, natural transitions, onset protection and holdout
listening before claiming a competitive result.
