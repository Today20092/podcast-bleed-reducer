# Candidate frozen before holdout

No labels starting at 185 seconds or later and no screenshot interval have been scored while choosing this candidate. Automatic estimates use the entire imported session, as the app does; this is not an inductive machine-learning train/test split. Human calibration and scored holdout ranges remain disjoint.

## Rules

- Retain the existing band filter, delay search, 0.25 explained-power copy threshold, level/residual margins of 3 dB, two-frame copy stability and unchanged output gain formulas.
- Require each candidate track's own VAD evidence and band level above its floor. Another microphone's VAD alone cannot activate room noise.
- Preserve quiet independent residual despite weak own VAD when a correlated source is active and residual exceeds the target floor. Automatic calibration cannot self-authorize suppression of that residual. Guided fallback additionally requires a well-fit learned copy model, residual at least 10 dB below copy level, and observed residual more than 3 dB above the learned residual.
- Remove copied candidates before applying that track's speech padding. Do not override the resulting own-track padding with a second copy veto. This retains boundaries and short gaps between independent speech bursts, at the cost of leaving ambiguous fragmented bleed open.
- Reject a declared solo example when at least three audible target frames have median explained power below 0.1. A weak correlation does not prove the identity of another sound, so the error asks for a clearer solo interval or removal. Silent target tracks do not require a copy relationship.

## Calibration selected from development audio

| Track | Kind | Excerpt seconds | Selection evidence |
|---|---|---|---|
| A01 | Solo candidate | 7.386–7.770 | Minimum median absolute cross-correlation to the other two tracks 0.786 |
| A03 | Solo candidate | 5.802–6.186 | Minimum median absolute cross-correlation 0.501 |
| A02 | Solo candidate | 66.099–66.611 | Minimum median absolute cross-correlation 0.595 |
| A03 | Human room noise | 3.345–3.878 | Original room-noise label |

These narrower intervals were chosen with the existing independent NumPy probe from early human wanted-speech labels. Search windows were 384 ms with 96 ms steps inside the first proposed A01/A03/A02 ranges, then 512 ms with 128 ms steps inside the existing A02 55.475–70.476 label. They are audio-selected solo candidates, not newly verified human solo annotations. The first A02 47.868–51.810 candidate had nearly equal A03 energy and median absolute correlation 0.193; it was replaced. No other microphones receive invented room-tone labels.

Full-session example ranges add the PCM-verified 8,973,765-sample offset. See quality-alignment.md. The screenshot 828–831-second interval remains provisional human interpretation of inferred bounds and is never calibration.

## Development evidence before freeze

Across 27 early/development labels, 48.365 wanted track-seconds and 25.688 bleed track-seconds:

| Candidate | Bleed energy attenuation | Wanted extra time deeper than 1 dB | Worst extra wanted depth |
|---|---:|---:|---:|
| Integrated initial guided, original broad examples | -2.742 dB | 2.15025 s | -6.4164 dB |
| Own-track padding, original examples | -0.386 dB | 0 | 0 |
| Padding, coherent examples, global VAD activation | -0.000188 dB | 0 | 0 |
| Own VAD activation, coherent examples | -1.53794 dB | 0 | -0.0001227 dB |
| Broad excess-residual fallback | -0.16647 dB | 0 | 0 |
| Frozen candidate, reliable guided residual fallback | -1.53794 dB | 0 | -0.0001227 dB |
| Legacy defaults | -0.96715 dB | baseline | baseline |

Frozen guided candidate has 73,399 wanted samples below exact unity; legacy has 73,203. There are 15,524 samples with any additional attenuation versus legacy, because improvements and losses occur at different samples. Neither has wanted samples below -0.1 dB or -1 dB. Worst absolute wanted depth is -0.002471 dB versus legacy -0.002480 dB. All 2.737 seconds of known overlap stay at exact unity. These diagnostics do not waive the existing exact-unity gate or establish listening acceptance.

Red/green regressions cover own-track padding gaps, cross-track VAD activating room noise, weak own-VAD overlap under reversed microphone gain, and unsupported solo calibration. Additional synthetic checks cover 1/3.25/10 ms delay, both polarities, gain ratios 0.15/0.4/1.6, low room noise, silence and a pulsing harmonic quiet voice/laugh substitute with target VAD 0.01. The substitute is synthetic and is not a listening evaluation.
