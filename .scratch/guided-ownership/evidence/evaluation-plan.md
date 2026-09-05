# Quality evaluation protocol

Keep training/calibration, development validation and final holdout distinct. The existing36 human labels belong to excerpt-relative timelines; the screenshot interval belongs to the full-session timeline. Do not add an offset to excerpt labels when reading excerpt files.

## Guided calibration candidates

Use early excerpt examples, not scored holdout audio:

- A01 solo:7.386–9.199s.
- A03 solo:4.746–6.919s; A01 is explicitly bleed-labeled alongside it.
- A02 solo:47.868–51.810s.
- A03 room tone:3.345–3.878s, optionally44.774–45.301s.

The direct-speech labels establish wanted speech on their assigned microphone. They are proposed solo examples because no other direct-speech label overlaps those ranges; the existing sparse labels do not prove complete absence of unannotated overlap. Record this limitation and check whether the example resembles the assumed acoustic relationship. Do not invent human room-tone labels for other microphones. They can use automatic estimates until the user marks examples.

Early labels below80s are calibration/development material. Labels100–160s can guide threshold selection. Freeze rules before reporting final heldout results for labels185–303.136s and the provisional screenshot828–831s. Report whole-corpus totals separately, visibly identifying reused development/calibration spans. If thresholds are adjusted after seeing the final holdout, rename it development evidence and retain independent synthetic/generalization cases; never claim an untouched test set.

## Required comparisons

1. Legacy mode at defaults and stored threshold0.20.
2. Joint automatic estimates with no examples.
3. Joint guided examples above, identical gain parameters.

For each, report bleed energy attenuation, wanted speech belowunity, added attenuation relative to legacy, gain-depth/duration below-0.1dB and-1dB, overlap preservation, and per-label worst cases. The0.1dB diagnostic is not permission to waive exact-unity or replace listening. Preserve the pre-existing strict gate report.

Include synthetic independent quieter double-talk, different microphone gain, inverted polarity, varying small delay, room noise, silence, and a voice-shaped interfering signal. Change features/settings, not output gain formulas, when revising the classifier. Exercise protections and invalid/stale calibration through public preview/render interfaces.

## Performance baseline

The completed135fbb1 release benchmark on all three full239660221-sample sources measured86.185s initial preparation, one preparation/468087 inference frames. Warm20-run median/p95 in milliseconds: attenuation1.576/1.777; Open1.524/1.684; Close1.489/1.715; sensitivity24.982/26.524; bleed24.877/27.039; padding24.468/25.971; protection1.921/2.410. Restoring settings exactly restored the original preview. These are calculation timings, not GUI/device latency. Joint mode should retain compact features and avoid repeating preparation on changes; measure its actual extra cold cost and warm response.
