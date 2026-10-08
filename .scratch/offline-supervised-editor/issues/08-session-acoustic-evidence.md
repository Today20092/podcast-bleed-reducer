# Fit session acoustic evidence from trusted references

Status: ready-for-agent
Blocked by: 03, 07
Spec: ../spec.md

## Scope

Reuse source-bound noise floors, directed scalar leakage and existing pairwise evidence as the initial acoustic baseline. Build explicit finite feature values for local noise-relative energy, competing level, bleed explanation/residual and fit reliability. Fit only from trusted appropriate references, including examples across small microphone movements. Poor/contradictory fits become missing/unreliable evidence rather than ownership proof.

Qualify regularized offline FIR/transfer evidence against the scalar baseline on development fixtures only. Retain the simplest estimator that improves relevant evidence without wanted-speech damage; if FIR adds no value, retain scalar evidence and record that result. Freeze estimator/feature recipes before independent acceptance tests. Do not subtract predicted bleed from delivered audio or update calibration during overlapping speech merely to explain it away.

## Reach

src/joint_detector.rs JointFacts::prepare/regions and existing leakage facts, src/calibration.rs fitting inputs, src/prepared_analysis.rs reference-derived cache, src/vad.rs fixed speech evidence. Add spectral/math dependencies only where the selected nontrivial estimator needs them.

## Acceptance

- Known synthetic leakage, delayed/colored leakage, zero energy, noisy references, simultaneous speakers and small movements exercise fit validation.
- Quiet independent speech cannot be classified as bleed solely because an acoustic model fits badly.
- Label/source changes invalidate derived fits while compatible fixed evidence remains cached.
- No nonfinite feature reaches training; feature units/normalization and failure reasons are documented.
- Development comparison records the selected estimator and its limitations without claiming unseen acceptance.
