# Center import setup and hide duplicate gain display

Status: done
Type: task
Assignee: Codex focused_import_33
Blocked by: 16, 21, 28
Spec: ../../offline-supervised-migration/specs/focused-import-waveform.md

## Scope

Implement the [accepted focused-layout specification](../../offline-supervised-migration/specs/focused-import-waveform.md): show complete in-app import setup in a compact centered dialog using the existing Export modal conventions. Preserve channel/name choices, validation, limits, errors and cancellation. Hide the lower gain-editor strip during normal use and retain it when Edit gain points is enabled. Existing waveform gain display and all manual edits retain their meaning.

Dependencies are done. Coordinate presentation/shared GUI changes with ticket 23 without duplicating its broad polish scope. Ticket 35 investigates the later overlay editor; this ticket does not depend on that design and must keep today's editor usable.

## Acceptance checks

- [x] File/drop import opens centered setup; channel choices, names, selected count, eight-track limit, validation/errors, progress, confirmation and Cancel remain available.
- [x] Dialog fits narrow/wide windows, supports long names/overflow and leaves actions reachable. Background workspace stays visible and cannot receive modal edits/shortcuts.
- [x] Cancel/Escape and late import results preserve the prior project under existing transaction/cancellation rules. Native picker and centered empty-state behavior remain compatible.
- [x] With Edit gain points off, the lower strip and its hit area disappear and the purple curve remains. Enabling restores all existing gain-point operations and automatic/effective distinctions.
- [x] Toggling visibility never modifies manual points, protections, undo history, effective gain or output samples. Source files remain unchanged; import creates no split WAV sources.
- [x] Existing import/gain-toggle regressions and scoped modal/hit-testing tests pass. Use scripts/gui-dev.ps1, formatting, appropriate Rust checks and strict Clippy.
- [x] Review dialog focus, error/loading/cancel and overflow states and waveform editing on/off in the development GUI; record unavailable checks.

## Results

2026-10-12: Implemented in `3d262a7` on the integration sequence `codex/editor-workflow-31-37`. All in-app import stages use a centered bounded dialog, scrolling choices/progress/errors and fixed title/actions. The committed workspace stays visible behind it; background edits and transport input are suppressed. Captured Escape from channel-name input cancels before publication. The lower gain canvas and its hit area exist only when Edit gain points is enabled; visibility changes preserve existing edits and evaluated output.

Import preparation now defers persistence on the worker clone. A prepared result requires GUI acceptance before the worker performs normal validation and atomic saving. Cancel before acceptance discards staged work and leaves live/saved prior projects unchanged. During final atomic publication, Cancel is explicitly unavailable and the dialog shows Finishing. Save failure retains the prior live session and saved bytes with an actionable modal error. No rollback, split WAVs, new formats or dependencies were added.

TDD regressions reproduced and fixed early worker persistence, modal Escape/background input and hidden-strip input. Checks cover canceled readiness/late delivery, successful later publication, real saved-project failure, zero channel selection, oversized names, eight-track limit and gain toggle/drag/undo preservation. Library/project/editor-workflow/effective-envelope checks passed 124 tests with 8 intentional opt-in checks ignored. Strict all-target/all-feature Clippy, formatting and whitespace checks passed. `scripts/gui-dev.ps1 -BuildOnly` passed in 32.98 seconds.

GUI inspection confirmed the strip fully hides in a maximized 2560×1392 window, returns with existing pink points and editing controls, then hides again. Native file selection opened the existing multichannel WAV. At 1026×932 the centered dialog showed Tr5/Tr6/Tr7 names, source basenames, 3 selected/3 existing/maximum 8 and reachable Import/Cancel actions. Escape from a focused name closed the dialog and retained three tracks and 36 examples. No import was confirmed; source files and original project were untouched.

Unavailable visual checks: separate loading/error/overflow/final-publication states and wide-window import presentation. Automated tests cover those state transitions and validation. Listening and release qualification remain outside this ticket; no overlay editor is delivered.

## Comments

2026-10-11: Owner accepted interim hide-unless-editing behavior while the automation editor receives separate design work; authorized specification/ticket creation.
