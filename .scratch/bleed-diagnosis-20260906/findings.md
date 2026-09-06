# Why the marked bleed remains open

I reproduced the failure using `CleanSlate-10.cleanslate`, your three original recordings, and all 22 saved examples. The current ownership algorithm needs revision. More calibration examples or stronger slider settings alone will not reliably fix it.

The clearest measured problem is that **small false speech detections become long preserved passages through padding**. The acoustic model and the ownership decision need to distinguish credible direct speech from ambiguous copied speech before deciding what deserves that padding.

## What happened in your marked sections

These ranges are conservative interiors estimated from the screenshot rulers, not exact boundaries selected by you. A01 is the top track; A02 is the middle; A03 is the bottom.

| Marked bleed | Duration | Target-owned before padding | Target-owned with your 768 ms padding | Actual output at exactly unchanged gain |
|---|---:|---:|---:|---:|
| A03, 3248–3262 s | 14.000 s | 1.312 s | 14.000 s | 14.000 s |
| A03, 4387–4406 s | 19.000 s | 2.240 s | 16.536 s | 15.684 s |
| A02, 3248–3262 s | 14.000 s | 0.416 s | 3.552 s | 1.848 s |

For the first row, the app effectively recognizes that A03 should not own most of the interval, but the surviving fragments reopen the entire 14 seconds. In the second row, actual output stays within 0.1 dB of unchanged for one continuous 11.696-second run. That matches the broad preserved passages in your screenshots.

“Before padding” above is a controlled rerun with padding zero. The code applies padding only after its raw ownership tests, so this isolates the effect on the target's ownership flag. It does not identify which raw predicate created each surviving fragment.

Your examples do have an effect. With the same aggressive settings but no examples, A02 remains completely unchanged for the entire 14-second interval. With your examples, 10.484 seconds fall below -1 dB. A03 remains completely unchanged in both runs. The calibration is useful but insufficient.

Global uncertainty is **not** what keeps these new marked intervals open with your saved settings: measured uncertainty duration is zero there. They are classified as owned after padding. An uncertainty fallback is a separate design problem that appears when padding is removed and is relevant to the redesign.

## Why the controls and examples reach a limit

1. **Ordinary speech detection recognizes bleed as speech.** The indirect voice is still a voice. The app needs to decide which microphones contain their own participant, including simultaneous participants. A different generic speech detector would not add that missing distinction by itself.
2. **Calibration currently describes only a small, stationary relationship.** Fourteen solo-speech examples and eight room-tone examples collapse into one noise-floor estimate per microphone and one median level ratio plus one median residual value per directed microphone pair. There is no learned speaker identity or frequency-dependent room response. Adding examples improves those summaries; it does not increase what the model can represent.
3. **Padding spreads any remaining ownership evidence.** The current 768 ms setting extends a surviving ownership frame in both directions. Small fragments close enough together bridge a long passage. This is the dominant measured amplification in the new A03 failures.
4. **The “bleed discrimination” control is a margin above each microphone's own noise floor.** It is not directly a stronger test of similarity to the real speaker's microphone. Some copy-fit thresholds remain fixed. The same sensitivity and level gates also determine whether a source can explain another track as bleed. Stronger settings can remove useful source evidence; with no owner and unresolved sound, the conservative fallback preserves all microphones.

These are limitations in the design, not evidence that you supplied too few examples. Your stored threshold of 0.9 correctly corresponds to the displayed sensitivity of 0.10; that conversion is not the bug.

## What I recommend next

Build **one offline replacement experiment for ownership and its time smoothing**, compare it with this exact saved session, and make another application build only after the comparison passes. Keep the current gain renderer and fast prepared-analysis cache.

The experiment should do three connected things:

1. **Learn how a solo voice reaches the other microphones across frequency.** Estimate the gain, coloration and delay relationship from solo examples. Check the fit on separate solo audio. Use the predicted leakage and what remains unexplained as evidence; keep original audio unchanged except for the chosen gain envelope. Start with a fixed calibrated model. Add time-local models only if measured changes in the acoustic path require them.
2. **Choose owners jointly and give uncertainty a useful meaning.** A microphone should not simultaneously act as an unquestioned source and be explained as another microphone's copy without resolving that contradiction. Separate source evidence from suppression aggressiveness. Preserve independent overlap and laughter. Low prediction error alone must not authorize suppression, because reference microphones themselves contain mixtures.
3. **Pad credible direct speech without letting ambiguous fragments reopen entire bleed passages.** Speech onsets and quiet short interruptions need explicit recovery checks. Stronger evidence of a copied voice should inform temporal decisions, instead of being discarded before unconditional dilation. This must be tested as a replacement, not implemented as a blanket “short sounds are bleed” rule.

A richer acoustic model is a hypothesis, not a demonstrated fix. The measurement proves the padding amplification; it does not yet prove that frequency-dependent prediction will distinguish every surviving fragment. Keep the experiment small enough to reject if it cannot do so safely.

## Why not just turn padding down?

Zero padding reduces the marked bleed, but it also introduces attenuation within the likely-direct A01 comparison ranges. Those ranges include pauses, so this run alone does not establish how much speech is damaged. The stronger warning comes from the earlier controlled test: suppressing short padding seeds damaged a 64 ms independent interruption. That candidate was rejected.

We should preserve the protection that padding provides while changing the evidence that triggers it. There is no safe justification yet for a global duration cutoff or a post-padding bleed veto.

## How to decide whether the experiment is better

- Score each marked bleed interval separately: meaningful attenuation, longest missed-bleed run, and actual audio-energy reduction. An improved average must not hide an unchanged 14-second failure.
- Reuse the labeled quiet-speech cases and short 32/64 ms interjection tests. Include independent overlap, laughter, gain imbalance, filtering, room tails and changing delay. Preserve explicit protections exactly.
- Treat all screenshots and previously inspected recordings as development evidence. Reserve a new interval or recording before tuning, with direct speech and bleed labels separated from calibration examples.
- Use explicit bleed labels in the offline evaluation. Room tone means nobody is speaking; bleed means someone else is speaking and this microphone contains no wanted participant. They supply different information. A future manual edit and a training example should also remain distinct.
- Report initial analysis cost, cache size and warm slider latency. Expensive evidence belongs in preparation; controls should rescore cached evidence.

If this deterministic experiment cannot separate copied speech from short independent speech, move to a learned multichannel owner-activity model using representative labeled data. Target-speaker information may help, but identity alone is also present in that speaker's bleed. Full audio separation is a larger processing change and is not the first step.

## Evidence and status

- [Control-ablation results](C:/Users/User/.codex/worktrees/cleanslate-ownership/podcast-automix-rust-gui/.scratch/bleed-diagnosis-20260906/control-ablation.json): five configurations, eight probe ranges, one preparation of the full original recordings. Saved configuration preparation took 75.714 s; subsequent configurations took 43–47 ms. These are evaluator timings, not whole-application responsiveness measurements.
- [Protocol and answered design questions](C:/Users/User/.codex/worktrees/cleanslate-ownership/podcast-automix-rust-gui/.scratch/bleed-diagnosis-20260906/protocol.md): assumptions, source identities, reproduction command and scope.
- [Algorithm research](C:/Users/User/.codex/worktrees/cleanslate-ownership/podcast-automix-rust-gui/.scratch/bleed-diagnosis-20260906/research.md): primary sources, alternatives and the proposed experiment. The existing local Rust architecture already identifies frequency-dependent leakage/coherence as the next escalation beyond scalar lag-aware correlation.
- [Read-only diagnostic](C:/Users/User/.codex/worktrees/cleanslate-ownership/podcast-automix-rust-gui/src/bin/ownership_diagnose.rs): saved-session reproduction; no UI or classifier changes. Its decision-counting regression check and strict release Clippy check pass. All 40 decision partitions exactly cover their probe intervals.

No source audio or saved examples were changed, no application was launched, and no new application build is being presented as a fix. The next production change needs the experiment and preservation results above.
