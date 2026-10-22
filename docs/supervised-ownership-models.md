# Supervised ownership estimates

This is the current acoustic editor contract. Start with [How CleanSlate reduces microphone bleed](ownership-explained.md) for the reader explanation. `EditorAnalysisWorker` calls `SessionOwnershipCache::analyze_acoustic` with human references and verified 16 kHz analysis copies. It trains independent binary logistic regression models for the assigned tracks and evaluates the whole recording. Embedded Silero supplies fixed speech-presence evidence. There is no Nemotron activity input, anonymous speaker mapping, model download or neural fine-tuning.

## Feature recipe and operation order

The feature recipe is `acoustic-raw-silero-acoustic08-uniontruth-centers-fulltime-classcap2048-trackbound-v3`. It uses a 32 ms source-time grid. Acoustic preparation measures the recording and fits noise floors and directed bleed relationships from trusted references. Fixed Silero supplies one raw speech score per track/frame. Training rows combine these measurements inside labeled ranges; the fitted models then estimate ownership outside those ranges too.

Each track's ordered inputs are:

1. `raw_vad`: raw fixed Silero probability.
2. `local_noise_relative_db/60`: speech-band level minus the calibrated room floor.
3. `relative_dominance_db/60`: local level minus the strongest other microphone's level.
4. For each other track in project order: `explanation`, `expected_db/60`, `excess_db/60`, `reliability`, `identified_mask`.

There are `3 + 5 * (track_count - 1)` inputs before constant-feature removal. See [acoustic evidence](acoustic-evidence.md) for units, fitting and missing-value encoding. The positive label means that this track's assigned person is speaking locally. Separate binary models permit simultaneous owners; their outputs need not sum to one.

## Human examples and sampling

Only complete frames inside authoritative human labels enter training. Same-truth overlapping intervals are united before sampling; a nested duplicate contributes neither another physical frame nor another reference center. Contradictory truth is rejected. Each union contributes at most 2048 uniformly spaced candidate frames. Before feature materialization, each track/class retains at most 2048 unique frames: one united-reference center where the budget permits, then deterministic samples across the remaining session times. If more than 2048 regions compete in one class, their centers are sampled across time. This caps each class without forcing equal class counts.

Solo and overlap membership supply positives independently for each assigned speaker. Room-tone and other-speaker labels supply negatives. Unlabeled frames are not negative training examples. The temporal stage still enforces every reference over its exact interval, including frames omitted from training.

## Weights and numerical checks

Training-only means and population standard deviations are stored explicitly. Features with standard deviation at most `1e-8` are removed. Prediction standardizes each retained feature, multiplies it by its learned coefficient, adds the fitted intercept and applies the sigmoid:

```text
scaled_input = (input - training_mean) / training_standard_deviation
score = intercept + sum(coefficient * scaled_input)
ownership_estimate = 1 / (1 + exp(-score))
```

The maintained trainer is pinned `linfa-logistic =0.8.1`, with `linfa =0.8.1` and ndarray 0.16. It uses Argmin LBFGS in Rust. Defaults are mean-loss L2 coefficient `0.01`, 300 maximum iterations and trainer gradient tolerance `1e-7`. The intercept is fitted without regularization. Linfa sums data loss, so its alpha is the mean-loss coefficient times the selected row count.

Application checks recompute mean logistic loss plus the L2 penalty and its gradient. Gradient norm must be at most `1e-5`; penalized loss must improve the actual class-prior entropy by at least `1e-4`. Training requires at least three usable positive and three negative rows and a varying feature. Invalid, nonfinite or inconsistently shaped inputs fail explicitly. The production builder materializes at most 4096 reference rows per track; it does not retain a full-session feature matrix or PCM.

The adapter inspects positive-class metadata, reverses coefficients/intercept where necessary, and stores `true` as local ownership. Its predictions must match correctly oriented library probabilities within `1e-12`. Finite rows are sorted deterministically before fitting. These checks establish numerical consistency, not calibrated probabilities or recording accuracy.

## Persistence, cache and failure boundaries

JSON metadata stores schema/recipe, ordered feature names, means/scales, retained indices, coefficients/intercept, positive orientation, settings and diagnostics. Session metadata binds ordered source identities, track/speaker assignments, reference identity, acoustic identity and fixed VAD identity. Fixed VAD identity includes the embedded Silero payload SHA-256, runtime build recipe and ordered analysis-copy identities. The acoustic path's mapping key records human reference identity; it does not represent native diarization.

Label changes invalidate reference-derived fits and estimates while compatible fixed acoustic facts and VAD can be reused. Gain settings and manual edits do not train the model. Compatible restored models rebuild estimates without fitting; current truth rows, normalization, retained dimensions and the useful penalized objective are checked before reuse. Old stored diagnostics alone cannot validate changed coefficients. Whole-session f32 output uses a `1e-6` roundtrip tolerance; the f64 model boundary is checked at `1e-12`.

Cancelled or failed jobs, changed sources and corrupt payloads do not replace the last completed result. Incomplete terminal acoustic frames carry a preservation warning. An unavailable bleed fit supplies explicit missing-value features; it does not automatically force every corresponding ownership estimate to uncertainty. The [temporal stage](temporal-ownership.md) handles frame protection and human constraints, and the [effective envelope](effective-envelopes.md) applies gain.

## Validation and limits

Public tests cover rejected one-class/constant/nonfinite input, insufficient iteration budgets, class-prior-only fitting, positive orientation, persistence tampering, roundtrip prediction, bounded sampling, fixed-evidence reuse and cancellation. Acoustic editor integration tests exercise training every track and prediction beyond reference ranges. Synthetic fixtures establish mechanics, not unseen recording quality. The owner accepted current sound/performance and closed 14 on 2026-10-20; remaining formal independent/long-session measurements are unperformed optional future work. Ticket 15's package checks continue in the [per-release checklist](branch-workflow.md#release-checklist), without implying those missing measurements passed.

The earlier recipe that used mapped Nemotron activity is retired. Its implementation and measurements remain in Git history and the [research note](research/nemotron.md); it is not the current Analyze path.

## Source references

- [Editor analysis order](../src/editor_workflow.rs)
- [Feature assembly, labels and sampling](../src/session_ownership.rs)
- [Training, prediction and model validation](../src/ownership_model.rs)
- [Released trainer source and objective](https://docs.rs/linfa-logistic/0.8.1/src/linfa_logistic/lib.rs.html)
