# Clarify the order of editing actions

Type: grilling
Labels: wayfinder:grilling
Status: resolved
Parent: ../map.md
Blocked by: none

## Answer

2026-10-09: Owner confirmed the [consolidated design](../gui-design-summary.md). Earlier questions and recommendations below are interview history; the consolidated design is the final decision. Implementation is consolidated into [build ticket 16](../../offline-supervised-editor/issues/16-gui-editing-workflow.md); do not create a separate build ticket for this decision. No implementation has been performed for this decision.

## Question

Should the workspace show a compact readiness summary and one next useful action, with advanced controls collapsed?

## Owner feedback

The right inspector has too much text and too many equally prominent actions. The owner cannot easily infer the order of import, timing checks, listening, references, Analyze, comparison, editing, saving and export. See [feedback and diagrams](../gui-feedback.md).

## Recommendation and pending decisions

2026-10-09 owner answer: concise guidance for now. Recommend one persistent workspace with readiness and the next useful action; no staged wizard. Exact presentation remains to be settled.

Recommend one persistent workspace with a compact progress/readiness summary and state-dependent emphasis. Keep transport and tracks visible, move technical/recovery information into details, and show actions relevant to the current state. Confirm this approach before deciding exact groups, names, empty/loading/error states and how to distinguish training labels from final gain edits. The accepted specification excludes a staged wizard; reopen that decision explicitly if the owner prefers one.

## Proposed acceptance questions

Can a new project show the next useful step? Can an existing configured project skip repeated setup? Are missing references named concisely? Can the owner always find transport, save and comparison? Does stale analysis clearly identify reanalysis versus gain-only changes?
