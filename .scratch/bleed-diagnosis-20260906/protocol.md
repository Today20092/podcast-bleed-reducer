# Algorithm investigation, 6 September 2026

## Scope and decisions

The current request is to explain persistent bleed and recommend the next algorithm step. UI work and a new application build are outside this investigation. The user previously authorized answering routine Matt workflow questions from evidence. This is the research and design stage, not a claim that a replacement has passed evaluation.

- What is wrong? Human-identified bleed remains preserved after guided calibration and extreme controls. Reproduce the saved session before changing a threshold.
- Is the goal ordinary speech detection? No. The needed decision is whether each microphone contains its intended participant, allowing several simultaneous participants. A voice on the wrong microphone is still speech.
- What must survive? Quiet direct speech, short interruptions, laughter, overlap, and explicit protection. Keep the existing original-samples-plus-gain output contract.
- Should we request more calibration first? No. Establish what the existing 22 examples do and whether the model can represent their acoustic relationship.
- Can lowering padding be the fix? Treat it as a diagnostic ablation only. Previous short-event gating damaged a 64 ms independent interjection.
- Should we switch neural models now? No. First determine whether ownership representation and decision logic, rather than generic speech recognition, explain the failure. Compare one bounded alternative before another production build.

## Reproduction

Base application source: `546e61413ee53731106b7b7f34247f2ce3c0800b`; integrated branch documentation HEAD at investigation start: `ccbabf6`.

Read `C:/Users/User/Downloads/CleanSlate-10.cleanslate` directly with Rust deserialization. It matches the screenshot's ordered A01/A02/A03 tracks, enabled joint ownership, 22 calibration examples and zero protections. Stored VAD threshold 0.9 is UI speech sensitivity 0.10. Other settings: discrimination 18 dB, attenuation -12 dB, padding 768 ms, open 50 ms, close 500 ms.

The evaluator embeds the deserialized session, source stamps, case definitions and every variant's parameters in its report. It checks source stamps and input bytes before/after. It never writes audio or modifies the saved session. Output uses create-new semantics. Integer source timestamps are not round-tripped through JavaScript numbers.

`cases.json` uses conservative interiors inferred from the screenshot rulers: 4387–4406 seconds and 3248–3262 seconds. These are provisional development ranges, not exact sample boundaries selected by the user. The likely-direct A01 comparison ranges are explicitly not new human speech labels. The older 828–831 second failure is included separately.

Five runs share one prepared cache:

1. Saved settings and all saved examples.
2. Same settings and examples, zero padding.
3. Same examples and envelope parameters, threshold 0.5 and discrimination 6 dB.
4. Variant 3 with zero padding.
5. Saved settings with automatic ownership and no examples.

Report decision duration separately for target-owned, another owner, global uncertainty and no speech. Report actual gain duration below -1 dB, exact unity duration, minimum gain and longest interval above -0.1 dB. The mean gain metric is time weighted, not audio-energy weighted; it must not be described as perceptual bleed reduction.

This ablation cannot establish wanted-speech safety or validate a replacement classifier. All inspected intervals are development evidence. No listening was performed by the agent. Production classifier and GUI are unchanged.

## Reproduce command

From this worktree:

```powershell
cargo test --release --bin ownership_diagnose
cargo run --release --bin ownership_diagnose -- 'C:/Users/User/Downloads/CleanSlate-10.cleanslate' '.scratch/bleed-diagnosis-20260906/cases.json' '.scratch/bleed-diagnosis-20260906/control-ablation.json'
```

Choose a new output filename when rerunning. Algorithm alternatives and primary references are in `research.md`; measurements and conclusions are in `findings.md`.
