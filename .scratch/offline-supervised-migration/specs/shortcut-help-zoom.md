# Shortcut reference and selection navigation

Status: accepted specification, 2026-10-11
Ideas: I11, I12, I13, I14
Build ticket: [34](../../offline-supervised-editor/issues/34-shortcut-help-zoom.md)
Decision record: [owner interview](../../../docs/roadmap-ideas.md#interview-decisions-2026-10-11)

## Problem Statement

Users lack a complete shortcut reference and fast ways to fit the recording or inspect a selected region. Escape does not generally clear waveform selection.

## Solution

Add Help → Keyboard shortcuts and Ctrl+/. F fits the recording, Shift+F fits selection, and Alt+drag draws a zoom region without labeling. Escape clears selection after existing modal/edit cancellations have taken precedence.

## User Stories

1. As an editor, I want to open a reference listing supported shortcuts and their contexts.
2. As an editor, I want F to show the complete recording quickly.
3. As an editor, I want Shift+F to inspect selection with surrounding context.
4. As an editor, I want Alt+drag to zoom directly without creating a speaker label.
5. As an editor, I want Escape to clear selection without discarding labels or edits.
6. As an editor, I want text fields and popup actions to take priority over workspace shortcuts.

## Implementation Decisions

- Provide a keyboard-accessible Help entry and shortcut-reference dialog, opened by Ctrl+/. List every supported user-facing binding and mouse modifier, including contexts and disabled/unavailable cases. Populate only implemented actions, adding new sound-reset actions when ticket 32 lands. Keep bindings and reference consistent using existing patterns without inventing a general remapping system.
- F fits the full valid session timeline; an empty project is a harmless no-op. Shift+F fits the current nonempty selection; no selection is a harmless no-op. Neither changes playhead, source timing, labels or gain.
- Selection fit uses approximately 10% of selected duration as context on each side and at least one second visible, centered on the range. Clip/reposition the viewport to valid recording bounds; if the recording is shorter, show it entirely. Keep selection fully visible. Reuse current viewport/pan/zoom math.
- Alt+primary-button drag draws a temporary zoom range, then fits it using the same margins/minimum duration. Give this gesture precedence over labeling, reference handles and gain-point edits in either edit mode. A click without a valid range does not open labeling or change zoom. Escape cancels an active zoom drag. This temporary range does not replace an existing saved label or waveform selection.
- F, Shift+F, Ctrl+/ and Alt-drag are currently unassigned; preserve Ctrl-wheel zoom, middle-drag pan and other existing bindings. Suppress workspace actions in text fields and unrelated modal contexts. Track platform modifier naming through existing app conventions; no unsupported host/platform behavior is claimed.
- Escape first cancels the active popup/dialog, text edit or drag using existing behavior. Only when no cancellation owns the event does it clear the waveform range. It never deletes saved labels, manual edits or protections. A speaker popup closes on first Escape and its retained selection clears on a subsequent Escape.
- Explicit fit/zoom gestures stop playback following just as manual viewport navigation does, so following cannot immediately undo the user's requested view. Playback itself remains unchanged.

## Testing Decisions

Use existing app keyboard-event and waveform viewport tests. Cover empty/full/edge/tiny selections, recordings shorter than one second, margin clipping, no-selection behavior, unchanged playhead/data, modifier precedence, Escape cancellation/deselection and text/modal suppression. Verify the reference against supported bindings; add context entries as sibling tickets introduce controls. Review keyboard-only reference navigation and Alt-drag with labeling/gain editing on/off in the development GUI.

## Out of Scope

Shortcut remapping, arbitrary zoom history, new transport actions, application settings/About/reporting menus beyond the shortcut reference, and waveform-overlay automation editing.

## Further Notes

Loop audition's popup lifecycle lands first so Escape behavior is verified against the accepted selection-retention contract. No stable release qualification is implied.
