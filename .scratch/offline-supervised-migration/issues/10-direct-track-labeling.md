# Label selected ranges directly on tracks

Type: grilling
Labels: wayfinder:grilling
Status: resolved
Parent: ../map.md
Blocked by: none

## Answer

2026-10-09: Owner confirmed the [consolidated design](../gui-design-summary.md). Earlier questions and recommendations below are interview history; the consolidated design is the final decision. Implementation is consolidated into [build ticket 16](../../offline-supervised-editor/issues/16-gui-editing-workflow.md); do not create a separate build ticket for this decision. No implementation has been performed for this decision.

## Question

Should selection always be available, with a right-click label menu and a visible keyboard-accessible action for the selected range?

## Owner feedback

Select a reference creates a mode where seeking requires Finish selecting first. The owner wants normal DAW interaction: click to seek, drag to select, immediately label solo speakers, arbitrary overlap sets or no talking/room noise, without repeated inspector trips.

## Recommendation and pending decisions

2026-10-09 owner answer: show labeling actions immediately after dragging. This supersedes right-click as the primary action. Preserve ordinary click-to-seek. Decide pointer-adjacent versus fixed-position actions and keyboard behavior next; existing unverified menu work remains preserved.

Recommend plain click to seek and drag to select, then right-click to label, plus a compact visible Label selection alternative. Confirm the interaction, then decide menu defaults, solo versus overlap presentation, short-range errors, keyboard operations, replacing conflicting whole labels, undo/redo and range persistence. The clicked track must own the selection; another track's old range cannot be applied. Menu input must not seek or edit behind it; Escape/outside dismissal and edge clamping are required.

## Existing implementation state

2026-10-09 second round: pointer-adjacent immediate label actions accepted. Labels describe the active speakers across the whole session: solo Speaker 1 excludes other assigned speakers in that interval, overlap names all active speakers, and room tone names none. The dragged track binds the new selection; it does not limit the meaning of the label to that microphone. Explicit human speech/overlap remains preserved unless intentionally manually edited.

Uncommitted app/controller and waveform/canvas work exists in the offline-timebase worktree, with new controller coverage added in this chat. It has not passed tests or strict checks, been built into a replacement GUI, or been integrated. Preserve and inspect it after the planning interview rather than rebuilding it from scratch.

## Proposed acceptance questions

Can the owner seek while selecting references without toggling modes? Can every configured solo speaker and arbitrary overlap be labeled? Do validation, explicit conflict replacement, undo, save/reopen and stale-analysis behavior survive? Does the visible alternative provide equivalent keyboard access?
