# Train independent supervised ownership models

Status: done
Assignee: ownership_models
Blocked by: 07, 08
Spec: ../spec.md

## Scope

Train an independent binary logistic session model for each assigned track using reference-label truth, mapped Nemotron probabilities, fixed VAD and acoustic evidence. Do not use multinomial softmax over speakers. Select/pin a maintained pure Rust logistic implementation for nontrivial training; do not ship Python or fine-tune neural extractors.

Keep normalization, feature recipe, coefficients and bias explicit and versioned. Validate enough usable positive/negative evidence, degenerate features, numerical convergence and repeatability. Poor training or missing identity cannot silently yield confident owners. Cache the small model and whole-track local-speaker estimates independently from fixed neural evidence. Freeze tuning/regularization on development data before independent evaluation. Do not learn from self-generated labels.

## Reach

New focused session classifier responsibility, src/joint_detector.rs owner-stage replacement, src/calibration.rs training examples, src/prepared_analysis.rs model/probability caches, src/project.rs recipe/provenance and Cargo dependencies.

## Acceptance

- Two simultaneous direct speakers can both score highly; speech presence on a bleed track is not mislabeled as its assigned speaker by construction.
- A held-out development region tests useful generalization beyond the exact reference frames.
- Empty/one-class/constant/nonfinite features, failed convergence and ambiguous mapping fail safely and retain prior completed results.
- Label edits retrain only affected reference-derived layers; gain-only edits do not train or infer.
- Exported/reloaded model metadata reproduces its predictions within a documented deterministic tolerance. UI language distinguishes an estimate from proven confidence calibration.

## Completion evidence — 2026-10-08

- Added public maintained binary trainer `OwnershipModel` and source-bound `SessionOwnershipCache` composition. Each assigned track has an independent logistic estimate; verified mapped Nemotron history, raw fixed Silero and finite ticket 08 acoustics remain distinct evidence. Human solo/room/overlap truth is the only training source. Missing/ambiguous identity returns explicit preservation-only estimates without replacing the accepted result.
- Pinned pure Rust Linfa/Linfa-logistic 0.8.1 and matching existing ndarray 0.16; current Context7 and primary released API/source/manifest were verified before selection. No handwritten optimizer, BLAS or Python training runtime was added. The first actual diagnostic caught the released library's count/encounter-based positive orientation; explicit fitted label metadata now orients true as local ownership and prediction parity is checked within 1e-12.
- Frozen training-only normalization/masks, deterministic class-capped full-time unique-frame sampling with united-reference center coverage, mean-loss L2 default 0.01, explicit unregularized intercept, bounded trainer iterations, external full gradient/current objective validation and feature/provenance recipes: `docs/supervised-ownership-models.md`. Reference features are capped before materialization, and every human label remains authoritative downstream even if its frame is not sampled.
- Separate fixed VAD, small-model and complete probability caches. Label changes retrain derived layers and reuse compatible fixed evidence; gain/manual protection changes reuse accepted output without training/inference. Compatible saved JSON models rebuild probability planes without training. Cancellation, source mutation, corruption and failed convergence retain the previous completed result. Restored coefficients/intercept/masks are checked against current human rows rather than trusting recorded diagnostics.
- Public red→green slices: absent maintained trainer; count-oriented fit rejected by independent true-label gradient; class-prior-only imbalanced fit falsely considered useful; invalid persisted configuration; absent verified session composition/model restoration; finite restored coefficient mutation; nonempty mask dropping the informative dimension. Qualification also checks empty/one-class/constant/nonfinite inputs, one-iteration failed convergence, reversed-row deterministic prediction and JSON roundtrip.
- Controlled entire held-out episode, separate from training reference regions: direct solo estimate >0.8, bleed track <0.2 despite speech presence, two simultaneous independent direct estimates >0.7 each (sum >1.4). Quiet independent direct speech below incoming bleed carries preservation/review evidence. These fixtures use real VAD/acoustic preparation and explicit synthetic probability-history boundary data; they do not assert native diarization or real-recording quality.
- Final scoped checks: **4 ownership model tests + 5 session ownership tests passed**. Broad all-target check before final mask/qualification additions: **154 passed, 14 ignored, zero failed**. Final strict all-target Clippy, formatting and whitespace checks passed. Final focused source-mutation/failed-fit retention checks also passed; no redundant broad rerun was needed.

## Handoff and remaining boundaries

Ticket 10 must preserve every authoritative label (including unsampled brief frames), allow concurrent owners, and treat missing estimates or any preservation reason as conservative review evidence. It must not interpret raw VAD, acoustic fit reliability or model estimates as proven confidence. Ticket 12 owns complete GUI job composition; the retained GUI Analyze is not silently promoted to this pipeline. Independent real quality, native downmix/model behavior, long-session resource/timing measurements and owner listening remain ticket 14. Sampling and logistic development qualification do not prove perfect behavior on unseen audio.