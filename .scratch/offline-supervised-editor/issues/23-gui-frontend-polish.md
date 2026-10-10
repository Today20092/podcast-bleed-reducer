# Polish application text and visual hierarchy

Status: claimed
Type: task
Assignee: Codex
Blocked by: 16, 21

## Scope

Owner confirmed on 2026-10-09: polish the whole existing GUI using Refactoring UI principles, with a restrained dark audio-editor appearance informed by FabFilter and RX. Improve wording, readable typography, spacing, colors and action hierarchy across the workspace, setup, dialogs, empty/loading/error/export states. Permit useful layout adjustments. Keep current audio behavior and semantic timeline colors. Keep status and next actions visible; move longer gesture instructions to contextual help. Judge the first pass in the development application.

## Acceptance checks

- Shared dark palette and readable type scale apply throughout; primary actions stand apart from secondary controls and selected modes.
- Sentence-case, consistent podcast-creator wording; timeline instructions remain discoverable through contextual help.
- Preserve timeline meaning, source validation, cancellation, explicit analysis, editable gain and safe exports.
- Build with scripts/gui-dev.ps1, run app regressions, formatting and strict Clippy. Measure theme text contrast.
- Inspect rendered narrow/wide, enlarged text, focus, loading/empty/error/selected/long-content states where available; record unavailable checks without claiming them.

## Results

First polish pass implemented on codex/offline-supervised: charcoal surfaces, restrained blue actions, explicit readable button states, Segoe UI, larger supporting text, semibold section headings, quieter secondary controls, consistent wording, contextual timeline instructions, a wider inspector and stacked transport below 1250 px. Original waveforms are clearer; ruler/example text is larger and ruler labels stay within the canvas. Semantic red/blue/purple timeline roles and all audio/export validation remain intact.

First-pass checks: 79 library tests passed, seven opt-in diagnostics/benchmarks ignored; strict all-target/all-feature Clippy, formatting and diff checks passed. Native compact and 2560 px workspace views were inspected, including stale/selected/disabled states, long source names, tooltips and the rename form. Initial inspection access failed, then recovered after relaunch. Enlarged OS text, all loading/error/export states and exhaustive keyboard navigation were not visually checked; release/listening acceptance is unchanged. Owner refinement below supersedes the dedicated rename form. Broader appearance review remains active.

Inline refinement checks: 33 app tests passed, two opt-in full-recording checks ignored. New regression covers invalid-name retention, Escape and explicit cancel, trimmed names persisted on reopen, and suppression of audition shortcuts while editing. Strict all-target/all-feature Clippy, formatting and diff checks passed. Uses the existing validated, transactional rename path; source loading clears obsolete name drafts.

## Owner refinement, 2026-10-09

Follow-up polish: constrain the rename hit area to the title text, with separate noninteractive header space; emphasize stale analysis in amber; group Undo, Redo and Analyze into one row with consistent sizing and rounded corners. Claimed under the same ticket. This is presentation only, with no new audio or persistence behavior.

Replace the Rename button and dedicated rename screen with direct track-header editing. Click the speaker/title text to edit in place. Enter saves, Escape cancels; preserve name validation and persistence, keep the timeline visible, focus/select the field immediately and display invalid-name feedback beside it. Claimed as part of this ticket.

Completed this refinement: scripts/gui-dev.ps1 built and reopened the development copy in 92.75 seconds. Native compact-view inspection confirmed the title opens a focused, selected inline name field with the timeline retained and no Rename button. Physical Escape through the automation API restored the unchanged title. Save/validation/persistence are covered by the app regression; no owner name was changed during the visual check. The development app remains open. Wider appearance, enlarged OS text and exhaustive visual state coverage remain part of the ongoing polish review.
