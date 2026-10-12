# Loop selected audio while labeling speakers

Status: done
Type: task
Assignee: Codex label_loop_31
Blocked by: 13, 16
Spec: ../../offline-supervised-migration/specs/label-loop-audition.md

## Scope

Implement the [accepted label-loop specification](../../offline-supervised-migration/specs/label-loop-audition.md) end to end: popup-bound repeating Original audition across all tracks, Space pause/resume, clear key badges, safe late-preparation handling and exact restoration of temporary listening/Solo state on closure. Leave the selection visible and playback stopped on Save/dismissal. Preserve validated labels and source/playback safeguards.

Dependencies 13/16 are done. Coordinate shared GUI edits with ongoing ticket 23; its broader appearance review is not a behavior dependency. Ticket 34 owns general Escape deselection and shortcut-reference/zoom work, building on this popup lifecycle.

## Acceptance checks

- [x] Selection release opens the speaker popup and repeats only its valid range in Original with all tracks, retaining monitor headroom.
- [x] Space pauses/resumes and Stop stops/cancels a pending start. Speaker keys, R, Enter and Escape retain their intended popup actions.
- [x] Successful Save, Cancel/outside dismissal and Escape stop playback, restore prior listening mode/Solo and retain selection; failed Save displays its error without losing the popup.
- [x] Pending preparation shows status; dismissed/replaced popups, changed sources/ranges/projects and failed jobs cannot start obsolete playback.
- [x] Key badges, room-tone wording and complete shortcut footer are visible and keyboard accessible.
- [x] Behavioral app/playback regressions cover boundaries, repeated playback, restoration, pause/stop, validation failure and late-job rejection. Tiny ranges remain responsive; no preparation occurs in the audio callback.
- [x] Use the GUI development guide and scripts/gui-dev.ps1; run appropriate Rust checks, formatting and strict Clippy. Review audible loop behavior and keyboard/focus in the development GUI, recording any unavailable checks.

## Results

2026-10-12: Integrated on `codex/editor-workflow-31-37` through `bad87a60f8e722fd6200880dd35257e521a69028`, following `2a37bd2` and `e207a96`. Changes are limited to app and audition code. Popup entry snapshots listening/Solo, uses Original/all tracks with monitor headroom, and loops the bounded half-open range in the worker-fed source. Space pauses/resumes; popup Stop cancels pending starts without dismissal. Closure restores the snapshot once and retains selection. Failed Save displays its full error and keeps the popup. Replacement, changed ranges/source generations and project/source loading reject obsolete starts.

TDD red/green checks cover automatic popup audition, repeated sample boundaries, a one-frame loop, monitoring gain, pause/resume, pending Stop, successful/failed Save, replacement, stale completion and exact restoration. Full library suite passed 85 tests with 7 existing opt-in tests ignored. Formatting, diff whitespace checks and strict all-target/all-feature Clippy passed. GUI builds used `scripts/gui-dev.ps1 -BuildOnly`; final build passed in 17.11 seconds.

Development GUI inspection used the separate `261005-quick-test.gui-dev.cleanslate` copy. With Solo 1 initially active, selection opened the popup, cleared Solo temporarily and showed badges, room-tone guidance, shortcuts and Preparing status. Playback repeatedly stayed within approximately 88.5–92.7 seconds. Space visibly paused; Escape closed/stopped, restored Solo 1 and retained selection. No labels were saved. Inspection found and prompted the paused transport wording fix. The final build adds popup Stop and `Paused · Resume` transport wording.

Follow-up GUI review during ticket 34 confirmed popup Stop leaves the popup open with playback paused and displays `Paused · Resume`; Escape then closes the popup and retains the selection.

Unavailable checks: physical audible boundaries/sound quality and a complete eight-track keyboard/focus pass. Stable listening/release gates remain tickets 14/15.

## Comments

2026-10-11: Owner confirmed the interview and authorized specification/ticket creation, not implementation in this turn.
