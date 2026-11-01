# Smart Gate fixture rescore — 2026-10-31

Ticket 01, claimed and completed for measurement only by task_27c975a146f7.
Base: `23cf53403e6f56086cf7cfbc13f560646e194a2b` on
`Today20092/finish-implement-spec`; branch `codex/smart-gate-fixture-rescore`.
The ticket/spec/research inputs and detector source exist in parent `lusca` as
uncommitted inputs, absent from this base; they were read without modifying them.

**Do the current lines still match their original fixture rationale? Yes.**
Whisper's mean remains above open 0.10; laughter's mean remains below the
derived close `min(open / 10, 0.01)` = 0.01. These reproduce the historical
measurements; the historical v5 label was stale, not a different model.
**Do they establish preservation of whisper/laughter or rejection of all noise?
No.** Laughter remains a strong preservation risk, and 96 kHz noise has one
frame above open. No retuning is justified as a measured preservation fix here.

## Fresh results

Arithmetic means of all complete 32 ms frames, including unknown nonspeech
spans; not calibrated confidence or false-negative rates. Counts are probability
comparisons, not actual gate transitions after hold/attack/release.

| Fixture | Rate Hz | Frames | Mean probability | Frames >= 0.10 | Frames <= 0.01 |
|---|---:|---:|---:|---:|---:|
| Recorded whisper | 44100 | 80 | 0.1402594283 | 16 | 46 |
| Recorded whisper | 48000 | 80 | 0.1398464460 | 16 | 47 |
| Recorded whisper | 96000 | 80 | 0.1397991918 | 14 | 46 |
| Recorded laughter | 44100 | 468 | 0.0090858889 | 10 | 411 |
| Recorded laughter | 48000 | 468 | 0.0091493867 | 11 | 409 |
| Recorded laughter | 96000 | 468 | 0.0072817412 | 7 | 414 |
| Synthetic white noise | 44100 | 31 | 0.0084880023 | 0 | 28 |
| Synthetic white noise | 48000 | 31 | 0.0103721417 | 0 | 26 |
| Synthetic white noise | 96000 | 31 | 0.0128591599 | 1 | 25 |

Noise peaks: 0.0645932853 / 0.0884140432 / 0.1060531735 at the three rates.
The noise mean straddles close, so the lines do not separate every noise frame.
Only 14–16 whisper frames clear open; no per-frame speech masks or listening
evidence establish preservation. Whole-clip means cannot settle that question.

## Model and fixture identity

Bundled `assets/silero_vad.onnx`, 2,327,524 bytes, verified SHA-256:
`1a153a22f4509e292a94e67d6f9b85e8deb25b4988682b7e174c65279d8788e3`.
This is the specified Silero v6.2.3 streaming model. The preserved harness embeds
that digest/model; its recipe string still says `silero-v5`. No model was added.

Original 48 kHz mono f32le fixture SHA-256 identities, verified by the harness:

- `Whispering-example.f32`, 492636 bytes: `a6dea42b04d2c1143627c189fc2df27c372a8bd03a0365103efcdb0a05bf96d5`.
- `Laughter.f32`, 2879772 bytes: `69b42ce2ea35222395ca11fb16d8a5a1311c4a66397dba3b7679d520a8aeb9b7`.
- Synthetic noise, 48000 samples: `5dd32ad75073f84e1f3202580c8b709fde1f14e002e6348f260ba88647856c2c`.

Whisper is Mx. Granger's Commons CC0 recording; laughter is ezwa's Commons
public-domain-dedicated recording. See the existing
[fixture provenance](smart-gate-detector-fixtures.md). First decoded channel,
at most 15 seconds, no normalization or probability-driven trim. Noise uses the
existing seed-1 wrapping u32 LCG (1664525, 1013904223), mapped to [-1,1] then
float32-scaled by 0.01. No fixture audio is committed.

## Execution and reproducibility

Reused preserved category harness rather than changing or rebuilding DSP.
All 225 runs (15 cases × 3 rates × blocks 1/17/64/257/1024) passed its exact
partition/reset assertions. Table uses block 257; all blocks have identical
probability arrays. Model input is 512 new 16 kHz samples plus 64 carried
context, state [2,1,128], causal 127-tap Blackman 7 kHz FIR, delay 64 native
samples. State resets per case/block; no incomplete EOF frame is scored.
Fixture host-rate conversion uses the existing linear interpolation.

Paths below are local evidence, with no source recordings distributed:

```powershell
$store = 'C:/Users/User/AppData/Local/Packages/OpenAI.Codex_2p2nqsd0c76g0/LocalCache/Local/CleanSlate'
$env:ORT_DYLIB_PATH = 'C:/Users/User/Documents/speech-transcriber/.venv/Lib/site-packages/onnxruntime/capi/onnxruntime.dll'
& "$store/VerifiedBuilds/gate-detector/categories-9cbfc2ab/feasibility.exe" `
  'C:/Users/User/Documents/podcast-automix-rust-gui/test-audio' `
  'NEW_REPORT_PATH.json' `
  "$store/GateDetector92/commons-category-fixtures-550d9196d0c24223b3309600bdc89bda"
```

Harness executable SHA-256:
`9cbfc2ab82c10cdd7a957a5df4cd2414833992ab326a80ddc8657137489f3541`.
ORT DLL file version `1.24.20260304.7.3a728b7`; CPU single-thread inference.
Fresh full report retained at this worktree's `rescore-private.json`, SHA-256
`bdc1446ad706b1b68a35eeeebf6994e71a7efc26119babc7da563d5f70b14b19`.
It includes source bindings and every probability, and is intentionally untracked.

## Scope and remaining checks

All three ticket measurement checks are satisfied. No threshold, hold, attack,
release, range default, gain policy, GUI, or other DSP behavior changed.
Level Bias above zero is deferred until activity exists; this is the requested
raw-model baseline, equivalent to the specified Level Bias 0 behavior. No native
plugin rendering, listening approval, independent accuracy, or gain-envelope
preservation check was performed. The retained laughter and quiet-speech risks
remain open; this note does not qualify production attenuation.
