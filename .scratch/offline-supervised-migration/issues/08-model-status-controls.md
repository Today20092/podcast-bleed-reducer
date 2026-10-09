# Simplify model controls after installation

Type: grilling
Labels: wayfinder:grilling
Status: resolved
Parent: ../map.md
Blocked by: none

## Answer

2026-10-09: Owner confirmed the [consolidated design](../gui-design-summary.md). Earlier questions and recommendations below are interview history; the consolidated design is the final decision. Implementation is consolidated into [build ticket 16](../../offline-supervised-editor/issues/16-gui-editing-workflow.md); do not create a separate build ticket for this decision. No implementation has been performed for this decision.

## Question

What should the editor show when the required offline model and runtime are ready?

## Owner feedback

The model panel contains too much technical text and prominent Download / verify Nemotron and Verify installed model offline buttons even when ready. Their prominence suggests work remains. See [feedback](../gui-feedback.md).

## Recommendation and pending decisions

2026-10-09 owner answer: accepts compact Model ready status with details available on demand. Specific failure/recovery states remain to be settled.

Show a compact Model ready status during normal editing, with setup/repair actions when needed and details available on demand. Confirm this presentation, then define missing, verifying, downloading, corrupt, offline and recipe-mismatch states. Keep required attribution/license notices accessible. Do not infer runtime readiness solely from the presence of a file.

## Proposed acceptance questions

Does a verified installation show no competing setup call to action? Does a real failure show a specific recovery action? Can the owner inspect provenance and license notices? Does model checking avoid delaying unrelated Original playback unnecessarily?
