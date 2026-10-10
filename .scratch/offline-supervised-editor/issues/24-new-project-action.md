# Start a new project from the GUI

Status: done
Type: task
Assignee: Codex subagent new_project_gui
Blocked by: 16, 21

## Scope

Owner authorized on 2026-10-09: add a New project action beside project opening/import. Confirm before discarding the current workspace, clear session state and return to fresh import. Keep source audio, saved projects and installed model state intact. Prevent reset while source loading, WAV preparation or export could still write session files.

## Acceptance checks

- New project is discoverable beside Open project and uses the existing secondary button style.
- An explicit confirmation warns to save first; cancellation preserves the workspace.
- Confirmed reset stops transport, cancels preview/analysis and clears tracks, references, gain state, selections, import state and results.
- Existing project/source bytes remain unchanged; a subsequent import chooses an available project path.
- Focused app tests, formatting, strict Clippy and scripts/gui-dev.ps1 build pass. Do not close the owner's running development window without coordination.

## Results

- Implemented New project beside Open project using the existing secondary button. Native Yes/No confirmation warns to save first and that unsaved changes are discarded. No file deletion or automatic save is performed.
- Reset stops playback/preparation, invalidates the old render generation, cancels preview analysis on drop and replaces the workspace with defaults. Installed-model acquisition state is retained. Imports/source loading, WAV preparation and export disable the action.
- Two focused reset regressions passed. The full applicable app suite passed 35 tests with two opt-in full WAV benchmarks ignored. Tests cover cancellation/refusal, preview cancellation, empty session/edit/transport state, unchanged saved project/source bytes and fresh import choosing a different project path.
- Strict all-target/all-feature Clippy with -D warnings, cargo fmt --check and git diff --check passed.
- scripts/gui-dev.ps1 -BuildOnly correctly refused to rebuild because the owner's development window is still running. GUI binary build and physical GUI interaction remain pending owner closure/relaunch; the running window was left intact. Ticket remains claimed until the scoped build check completes.

- Owner saved and closed the app. `scripts/gui-dev.ps1` then built successfully in 90.55 seconds and reopened the separate development project. Visually verified New project beside Open project in the actual native window, including the compact layout. Automated tests cover confirmation cancellation/reset; a native-dialog end-to-end click was not verified because screenshot coordinate targeting did not hit the button. No reset was performed on the owner's loaded project. Scoped build acceptance is complete.
