# Loop audition while labeling

Status: accepted specification, 2026-10-11
Ideas: I05, I06, labeling portion of I14
Build ticket: [31](../../offline-supervised-editor/issues/31-label-loop-audition.md)
Decision record: [owner interview](../../../docs/roadmap-ideas.md#interview-decisions-2026-10-11)

## Problem Statement

After selecting an example, the user repeatedly starts playback to identify everyone speaking. The popup's shortcuts need clearer presentation.

## Solution

Opening the speaker-label popup starts a repeating audition of its selected range across all tracks in Original mode. The popup remains beside the selection. Saving or dismissing stops the audition and leaves the range selected.

## User Stories

1. As an editor, I want to hear a selected example immediately so I can identify everyone speaking.
2. As an editor, I want the example to repeat while the popup is open so I can check uncertain speech.
3. As an editor, I want every track audible in Original so attenuation and Solo cannot hide speakers.
4. As an editor, I want Space to pause/resume the example without closing the popup.
5. As an editor, I want visible speaker key badges and room-tone guidance so labeling is understandable.
6. As an editor, I want Save or Cancel to stop audition without restarting earlier playback.
7. As an editor, I want my listening mode and Solo choices restored after labeling.
8. As an editor, I want preparing audio to start only for the popup I still have open.

## Implementation Decisions

- Reuse selection audition and prepared playback. Loop the bounded half-open selected range using the existing source-aware timebase; keep preparation outside the audio callback.
- Snapshot listening mode and Solo on popup entry. Use Original and all tracks temporarily; retain monitoring gain/headroom. Restore the snapshot exactly once on closure and leave playback stopped. These temporary changes do not affect projects, analysis or export.
- Space toggles loop pause/resume. Speaker keys 1–8, R, Enter and Escape retain their popup meanings. Show key badges, “None selected = room tone” and “1–8 Toggle speakers · R Clear · Enter Save · Esc Cancel”. Text entry and modal focus take priority.
- Save closes and stops only after successful label validation. A failed save keeps the popup and its controls available with the error. Outside dismissal, Escape and explicit Cancel stop the loop without saving.
- When preparation is pending, show “Preparing audio…”. Start only while the same popup, range, session and source generation remain current. Preparation failure shows the existing actionable error; labels remain editable. Closing/replacing the popup or changing/loading the project invalidates delayed starts and stops active loops.
- Preserve one selection on ordinary Save/dismissal. Escape consumes the popup cancellation; subsequent Escape deselection is specified in the navigation scope. Loop only nonempty valid ranges; preparation/source blockers remain enforced. Tiny ranges must remain bounded and responsive without a busy-loop restart on the UI thread.
- Ordinary transport Stop stops the audition; Space can resume it while the popup remains open. Stop during preparation also cancels the pending automatic start. Keep automatic playback confined to the labeling popup; no general repeat-playback feature is added.

## Testing Decisions

Use existing app-event and selection-playback tests as the highest behavioral seams. Verify repeated selection boundaries, pause/resume/Stop, successful and failed Save, dismissal, restored state, popup replacement, late preparation, failure and project changes. Keep Original sample selection, monitoring headroom and source validation under existing playback checks. Review actual repeat playback and popup keyboard operation in the development GUI; record unavailable device/listening checks explicitly.

## Out of Scope

New labeling semantics, detector changes, persistent loop preferences, resumed pre-popup playback and automation-editor redesign.

## Further Notes

This is accepted future behavior, not a claim of implementation or audio-quality qualification. Existing release gates remain unchanged.
