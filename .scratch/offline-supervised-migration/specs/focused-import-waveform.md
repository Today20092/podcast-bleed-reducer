# Centered import and quieter waveform display

Status: accepted specification, 2026-10-11
Ideas: I09 interim scope, I10
Build ticket: [33](../../offline-supervised-editor/issues/33-focused-import-waveform.md)
Decision record: [owner interview](../../../docs/roadmap-ideas.md#interview-decisions-2026-10-11)

## Problem Statement

Import choices occupy a full-window view with controls at the upper left. A lower green gain line duplicates the purple waveform gain curve during normal editing.

## Solution

Center the existing complete import setup in a compact in-app dialog like Export. Hide the lower gain-editor strip normally, retaining it when Edit gain points is enabled until the separately designed overlay editor replaces it.

## User Stories

1. As an editor, I want import choices centered so I can focus on the task.
2. As an editor, I want channel choices, names, count and errors together before confirming.
3. As an editor, I want large imports and small windows to remain usable through sizing and scrolling.
4. As an editor, I want the existing workspace visible behind the dialog without accidental edits.
5. As an editor, I want one effective gain display during normal waveform use.
6. As an editor, I want the existing gain-point editor available when explicitly enabled.

## Implementation Decisions

- Reuse the Export dialog's existing modal presentation and focus conventions, adapting only the import contents. Center all existing in-app setup/validation stages; native file-picker presentation remains platform-owned.
- Preserve channel selection, names, selected count, eight-track limit, validation feedback, progress/cancellation and import confirmation. Support file selection and drag/drop entry. Use bounded dialog dimensions and internal scrolling for overflow; keep primary/Cancel actions reachable. Show errors in the dialog.
- Retain the workspace behind the modal; prevent background timeline/transport shortcuts or edits from receiving modal input. Successful import uses the existing source/project transaction. Cancel/Escape preserves the pre-import project and invalidates pending results using existing cancellation rules. Keep the already centered empty-project introduction.
- Hide the separate lower gain-editor strip whenever Edit gain points is off. Retain the purple effective-gain curve, attenuation shading, waveform and selections. Reclaim the hidden strip's layout/hit-testing area; no invisible editing target remains.
- When Edit gain points is on, retain the existing lower editor with all point operations, automatic/effective distinctions, protected edits and undo behavior. Turning the toggle off hides controls without removing edits or changing gain. Both curves still use the shared effective-envelope evaluator.
- Coordinate shared GUI presentation with ongoing polish; do not expand this task into a whole-app restyle. Waveform-overlay editing is a separate design task.

## Testing Decisions

Use existing app import transaction and gain-toggle tests. Verify channel/name/count/limit validation, Cancel/Escape and late results, import from file/drop, hidden-strip hit testing and unchanged edits on toggling. Review narrow/wide dialogs, long names, overflow, keyboard focus, validation errors, loading/cancel states and gain editing on/off in the development GUI. Confirm import writes no split source files and leaves source validation/export behavior unchanged.

## Out of Scope

Synchronization, new input formats, custom themes, changes to speaker-popup location or manual automation controls.

## Further Notes

The lower teal line supports real editing; this is a conditional visibility change, not deletion of the editing capability.
