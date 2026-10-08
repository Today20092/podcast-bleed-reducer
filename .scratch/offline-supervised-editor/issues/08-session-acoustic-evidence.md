# Fit session acoustic evidence from trusted references

Status: done
Assignee: acoustic_evidence
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

## Completion evidence — 2026-10-08

- Added public `AcousticEvidenceCache` / immutable `SessionAcoustics`: source-bound fixed 32 ms facts are retained across label edits; trusted solo/room fits are replaced atomically after successful validation. Cancellation, changed native sources and tampered prepared payloads preserve prior accepted evidence and reject reuse. Analysis requires confirmed synchronization.
- Scalar directed leakage remains the baseline. A pinned maintained Nalgebra 0.35.0 Cholesky solver qualifies bounded regularized two-tap transfer evidence on disjoint training/held-out samples inside trusted solo unions. Repeated overlapping labels cannot inflate physical held-out evidence or mix training with validation. Below-floor/insufficient evidence cannot be promoted by ideal correlation. Overlap labels never train transfers; delivered audio is never subtracted or clipped.
- Frozen recipe, finite f64 power accumulation, normalized feature ordering/masks, units, thresholds, explicit unavailable reasons, native half-open EOF behavior, actual controlled comparison and limitations: `docs/acoustic-evidence.md`.
- Controlled 64-sample delayed two-tap coloration: scalar held-out explanation 0.69067854, FIR 1.0 at printed precision, residual improvement 75.124146 dB, 2048 unique held-out samples; FIR selected. Quiet independent overlap increases residual excess rather than becoming ownership proof. Scalar remains selected for simple leakage and small gain movement (reliability 0.8791418). Zero/noisy/severely moving and below-floor fixtures remain explicitly unusable or unidentified.
- Actual public red→green regressions: same-length prepared-payload tamper; finite large float32 energy overflow; sub-floor FIR promotion; missing synchronization validation; per-source native EOF/ceil-resampling tail; duplicate physical reference validation samples (2048 falsely became 4096 before the union fix). Public feature/cache/fit behavior is exercised, rather than implementation mocks.
- Final scoped `cargo test --test session_acoustics --locked -j 1 --config 'profile.test.package.rust-app.debug=0'`: **10 passed**. Earlier broad all-target check: **145 passed, 14 ignored, zero failures**; final boundary changes were followed by focused checks. Final strict all-target Clippy, formatting and whitespace checks passed.

## Remaining migration boundaries

Ticket 09 consumes these acoustic features alongside mapped verified Nemotron/VAD evidence and must deduplicate physical label truth when building training examples. Ticket 12 wires the complete editor job pipeline. These controlled development fixtures qualify mechanism and estimator selection only; independent recording quality, performance targets and owner listening acceptance remain ticket 14. Sampled two-tap/constant-lag fitting does not claim arbitrary room reverberation, nonlinear microphones or every position change. Missing/mismatched evidence must retain conservative speech/overlap handling.