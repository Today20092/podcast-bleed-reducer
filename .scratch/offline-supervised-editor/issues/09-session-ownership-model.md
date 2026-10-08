# Train independent supervised ownership models

Status: ready-for-agent
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
