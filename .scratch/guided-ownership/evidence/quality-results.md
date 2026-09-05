# Guided ownership quality evaluation

Evaluation is complete with known quality limitations. The circled screenshot failure is **not fixed**. Production classification remains the reviewed `1dc1f2a` algorithm; the later protection correction forces exact unity only in explicit protection ranges.

## Evidence and provenance

All results here are **development evidence**. The early/late split was initially frozen before late scoring, as recorded in quality-freeze.md, but later tuning inspected late and screenshot results. No unseen holdout or listening acceptance is claimed. Calibration candidates also overlap early wanted labels, so those labels are not an independent calibration test.

The original three 48 kHz excerpt WAVs and three full-session WAVs were read without saving audio copies. Source stamps are retained in the raw JSON reports; the evaluator checks them before and after analysis. Existing reference-labels.csv supplies 36 labels: 53.453 wanted track-seconds and 142.591 bleed track-seconds. Screenshot bounds 828–831 seconds are inferred from the supplied screenshot, with A01 direct and A02/A03 bleed; they are provisional, not precise new human annotations.

Selected calibration and its limitations are in quality-freeze.md. full-calibration.json maps those candidates to the original full stems using the PCM-verified offset **8,973,765 samples**, not the one-sample-earlier BEXT timestamp. See quality-alignment.md.

The retained reports are quality-development-reviewed.json (27 early labels), quality-holdout-reviewed.json (9 late labels, historical filename only), and quality-screenshot.json (full-session analysis). Combined figures below sum per-label input/output energies, not averages of decibels. Classifier production code was restored exactly to the measured version after rejected experiments. These unprotected metrics are unaffected by the later protection-only engine correction.

## Actual Rust comparison

Each run prepares one PreviewCache and calls its real analyze_with_ownership path four times. All modes use attenuation −18 dB, opening 50 ms, closing 500 ms, padding 512 ms and discrimination 6 dB. Only legacy 0.20 changes sensitivity from 0.50 to 0.20. The evaluator reads original source energy and evaluates gain envelopes without rendering WAV copies.

| Mode | Bleed energy attenuation | Bleed time below −0.1 / −1 dB | Wanted samples exactly unity | Wanted samples below unity |
|---|---:|---:|---:|---:|
| Legacy 0.50 | −1.660171 dB | 96.826542 / 95.126333 s | 2,465,051 | 100,693 |
| Legacy 0.20 | −0.660257 dB | 76.266333 / 74.173688 s | 2,507,903 | 57,841 |
| Automatic joint | −1.653737 dB | 96.559208 / 95.027208 s | 2,465,059 | 100,685 |
| Guided joint | −2.023057 dB | 99.087625 / 97.344042 s | 2,464,855 | 100,889 |

Every mode has **zero wanted samples below −0.1 dB or −1 dB**. Worst wanted gain is −0.024432892 dB at A01 228.511–229.135 seconds, identical across modes. Guided has 15,524 samples (0.323417 s) with any additional attenuation versus legacy, worst additional depth −0.000122700 dB at A01 7.386–9.199. There are zero additional wanted samples deeper than −0.1 or −1 dB. Improvements elsewhere explain why the net below-unity count grows by only 196. These small differences do not satisfy or waive an exact-unity requirement.

All 2.737 seconds of labeled simultaneous wanted speech stay exactly unity in every mode. Guided adds 7.112688 bleed track-seconds deeper than −0.1 dB and 5.064042 deeper than −1 dB versus legacy; maximum bleed attenuation is −18 dB.

## Per-label bleed cases

Seconds below are excerpt-relative. Full precision and all wanted/ignore metrics are in the raw reports.

| Track | Seconds | Legacy dB | Guided dB |
|---|---|---:|---:|
| A01 | 4.651–6.865 | −0.19069 | −0.28079 |
| A01 | 11.507–13.849 | −2.31093 | −2.31093 |
| A01 | 14.844–17.368 | −3.82052 | −3.50189 |
| A02 | 52.098–55.298 | −0.00214 | −0.01162 |
| A03 | 36.127–40.240 | −0.25918 | −2.46456 |
| A03 | 41.040–44.613 | ≈0 | −0.00023 |
| A03 | 121.207–123.661 | −17.99660 | −17.99991 |
| A03 | 124.776–130.044 | −5.21473 | −6.15812 |
| A01 | 185.882–228.511 | −6.60449 | −7.74606 |
| A01 | 229.135–252.006 | −13.88062 | −13.88062 |
| A02 | 254.768–270.755 | −0.00214 | −0.00205 |
| A02 | 288.773–303.136 | 0 | 0 |
| A03 | 231.181–252.234 | −17.97668 | −17.97340 |

The full-session screenshot interval remains **exactly unity on all three microphones in all four modes**: A01 wanted is preserved, but neither A02 nor A03 bleed is attenuated.

## Diagnosis and rejected experiments

Removing the post-padding copy veto fixed the initial guided candidate's wanted-speech damage: 2.15025 seconds with additional attenuation deeper than 1 dB, worst additional depth −6.4164 dB. Own-track VAD, reliable residual preservation, and rejecting unsupported solo examples remain in production. Ordinary unprotected gain timing/formulas remain unchanged.

In the screenshot, A01→A02 copy gates pass 58/93 frames (longest run 15 frames); A01→A03 passes 61/93 (longest 17). Fragmented remaining ownership seeds 512 ms padding on both sides, reopening the short copied runs. This is a measured conservative-padding limitation.

Experimental suppression of padding seeds shorter than eight frames improved screenshot bleed to −0.293889 dB on A02 and −0.734078 dB on A03, but failed a filtered 64 ms independent-interjection check. Positive-residual seed guards preserved the interjection but erased the corpus improvement. Raising that guard to 6 dB introduced an unverified 3–6 dB wanted-voice gap. A two-pass attempt to ignore preservation votes from copied microphones retained the conservative 3 dB margin but still left a changing-delay synthetic copy entirely open; independent review identified isolated/cyclic copy evidence as another wanted-speech risk. **All experimental seed and source-disqualification changes were discarded.**

A 0.25 ms alternating acoustic delay with filtered copies remained open in the conservative synthetic check. Abrupt larger delay changes and quiet voices missed by VAD without a reliable leakage model remain limitations. No threshold was increased solely to pass a bleed-suppression test.

## Reproduce

Run from the repository root in PowerShell. Choose new output filenames; the evaluator deliberately refuses overwrite.

```powershell
cargo run --release --bin ownership_evaluate -- .scratch/responsive-preview/evidence/reference-labels.csv .scratch/guided-ownership/evidence/calibration-coherent.json all .scratch/guided-ownership/evidence/recheck-excerpts.json C:/Users/User/Downloads/00005603_A01.wav C:/Users/User/Downloads/00005603_A02.wav C:/Users/User/Downloads/00005603_A03.wav

cargo run --release --bin ownership_evaluate -- .scratch/guided-ownership/evidence/screenshot-labels.csv .scratch/guided-ownership/evidence/full-calibration.json all .scratch/guided-ownership/evidence/recheck-screenshot.json 'C:/Users/User/Downloads/chopshow podcast 155_A01.wav' 'C:/Users/User/Downloads/chopshow podcast 155_A02.wav' 'C:/Users/User/Downloads/chopshow podcast 155_A03.wav'

$env:OWNERSHIP_DIAGNOSTIC_FULL='1'
cargo test --lib reference_calibration_diagnostics -- --ignored --nocapture
Remove-Item Env:OWNERSHIP_DIAGNOSTIC_FULL
```

The diagnostic prints copy-gate statistics; it is not a substitute for the evaluator's actual gain metrics. `full-calibration.json` is also suitable for the parent's preview benchmark ownership argument.

## Verification

Independent final Standards and Spec delta reviews reported no code findings. The final delta adds a 32/64 ms independent-interjection regression with filtered/unfiltered delayed copies: ordinary recovery must exceed 0.999 at onset; explicit range protection must equal 1.0 throughout. This exposed the shared protection-envelope bug fixed by the parent.

Final all-target automated tests: **82 passed, five explicitly ignored, zero failed**. cargo fmt --check passed. Strict all-target Clippy stops at the existing nested-if warning in src/app.rs:1451 on this branch, which predates the parent's editor followups; integrated Clippy is left to the parent. Packaging, integrated performance, and user listening are handled by the parent task. Quality evaluation completion does not mean the screenshot suppression requirement or exact-unity unprotected wanted-speech requirement has been achieved.
