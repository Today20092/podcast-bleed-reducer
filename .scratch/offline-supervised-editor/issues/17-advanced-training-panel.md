# Add an advanced training panel

Status: needs-triage
Type: task
Deferred: yes; not authorized for implementation
Blocked by: 14, 16
Spec: ../../offline-supervised-migration/spec.md

## Problem

Ownership training uses internal regularization and optimizer settings. Users cannot inspect convergence or distinguish training settings from ownership thresholds and gain controls. The owner requested one future ticket for an advanced panel, with no implementation now.

## Proposed scope

Add a collapsed Advanced training panel with concise explanations and reset-to-default actions. Keep the ordinary labeling, Analyze and gain workflow unchanged.

- Expose mean-loss L2 regularization, currently 0.01, and maximum optimizer iterations, currently 300. Explain that regularization limits large weights and more iterations help only when optimization needs further steps. Neither directly controls attenuation aggressiveness.
- Show available per-track training diagnostics, usable positive/negative example counts and fit/convergence status. Report actual diagnostics; do not fabricate accuracy, calibrated confidence or optimizer steps unavailable from the trainer.
- Show learned weights, bias, feature names and normalization read-only. Weight meaning depends on normalization and values can change after retraining. Raw coefficient editing is outside this ticket.
- Consider ownership start/stop thresholds as a separate decision section during triage. Acoustic defaults are 0.8/0.2. Explain hysteresis separately from existing gap and fade controls. Per-track sensitivity/bias remains a design question, not an accepted requirement.

Persist accepted settings with explicit defaults and validated bounds. Choose numerical bounds during triage using development evidence; do not invent release-qualified tuning ranges. Clearly indicate which changes require Analyze and which can reuse cached probabilities.

## Reach

Iced settings UI; project settings/schema defaults; OwnershipTrainingSettings; model/cache identity; temporal settings if approved; analysis invalidation; envelope/export freshness; documentation and focused persistence/invalidation tests. Reuse the maintained trainer and shared envelope evaluator.

## Acceptance for later implementation

- The panel is collapsed by default, explains each control and has reset actions.
- Inputs are finite/bounded and validated. Older projects load with current defaults; save/reload retains new settings.
- Training-setting changes mark dependent analysis stale and require explicit Analyze. Reuse compatible fixed measurements, but rebuild models/probabilities with settings included in their cache/result identity.
- Approved threshold controls reuse probabilities and regenerate dependent decisions/envelopes with a new identity. Human labels, manual edits and protection retain their authority.
- Failed, cancelled and obsolete jobs retain the last completed preview and prevent stale final export. Display, audition and export use the same effective envelope.
- Diagnostics distinguish convergence, training fit and independent quality. More iterations after convergence are not presented as an accuracy improvement.
- Focused checks cover defaults, persistence, invalid inputs, reset, cache reuse/invalidation, failed-fit retention and preview/export parity. Tune on development data before independent qualification.

## Non-goals

Implementation now; new classifiers; neural fine-tuning; raw coefficient editing; automatic tuning against held-out release data; new uncertainty review workflows.

## Comments

2026-10-09: Created at the owner's request as deferred work. Do not claim or implement until explicitly authorized and triage settles bounds and decision-control scope. See [the ownership explanation](../../../docs/ownership-explained.md) for labels, bleed calibration, inputs, weights and gain generation.
