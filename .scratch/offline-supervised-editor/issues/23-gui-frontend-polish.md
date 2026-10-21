# Finish integrated editor verification and GUI polish

Status: claimed
Type: task
Assignee: Codex
Blocked by: 16, 21

## Selected inspector B — 2026-10-21

Owner subsequently authorized all fourteen earlier FabFilter-inspired suggestions and applicable report lessons, with subagents on the same GUI branch. [Ticket 56](56-fabfilter-inspired-gui.md) records that expanded implementation and review checklist. This ticket retains the integrated owner-acceptance queue.

Owner selected B after comparing the interactive A/B sketches. Codex claims this native implementation on `codex/gui-inspector`, from development `e668e60`; the throwaway sketches remain on `codex/gui-inspector-sketches` at `b28c841` and will not be merged into development. Dependencies 16 and 21 are done.

Scope: a compact status header, existing speaker-example/Undo/Redo/Analyze controls, featured attenuation dial with numeric editing/reset/mute, a separate transitions-and-gaps group, existing gain-point controls and Sources disclosure, and an export footer outside the desktop body scroll. Preserve Dark/Light, narrow layouts, typed validation, control ranges/defaults, commit boundaries, source checks, analysis cancellation and safe export. No waveform-editor or audio redesign; tickets 35/41/42 retain their scope/status.

Checks: dial interaction/range/release regression, existing sound and input-ownership tests, theme contrast, formatting, strict Clippy and local checks; build using `scripts/gui-dev.ps1`, preserve a source/build receipt. Record native visual checks that are unavailable separately from browser prototype checks. This selected pass does not close the broader acceptance queue below.

Native implementation: 340 px desktop inspector with bounded status header, scrolling controls and fixed Export WAV files footer; compact windows retain a single scrolling inspector. Speaker examples retain the existing Undo/Redo/Analyze row and disclosure. Attenuation now has a vector dial, numeric editor, explicit Mute state and Reset; transitions/gaps form a separate group. Existing gain controls and automatically expanded source blockers remain. Export errors have a bounded footer area; general job messages are in the body. Cancel is at the top of the analysis header. No audio/domain, dependency or waveform-editor changes.

The dial retains −30…−3 dB quick adjustment (four pixels per dB, or Up/Down while hovered); typed attenuation remains −120…0 dB. A click does not unmute or change extended typed values; a real adjustment uses the existing attenuation update. Release outside the control or loss of window focus ends a changed drag once. Active sound/name drafts keep arrow input. All existing numeric commit/cancel/reset validation and timing ranges remain.

Native rendered appearance, enlarged OS text, physical keyboard/mouse interaction, listening and cross-platform checks are pending. No native UI automation is available in this session; browser-sketch checks are not native acceptance. Keep this ticket claimed.

## Current remaining work — 2026-10-20

Owner follow-up, 2026-10-20: confirmed that GUI work is continuing. Keep this consolidated editor-verification/polish ticket open; this is not a request to close the remaining GUI work or implement additional unrelated features.

Subsequent owner acceptance: current audio quality/performance is satisfactory and 14 is closed. The formal timing/memory/instrumentation follow-ups inherited from 38/40 below are retained as optional future benchmarking notes, not outstanding acceptance blockers in this GUI ticket. No benchmark ticket is created now. Continue visible workflow checks, device/error recovery and demonstrated responsiveness bugs; these remain useful GUI work. Final downloaded-package checks move from retired 15 to docs/branch-workflow.md#release-checklist.

This is the single acceptance queue for implemented GUI polish, progressive import (38), shortcut-reference presentation (39) and reusable playback (40). Those three implementation records are superseded into this ticket, with their specifications and results retained. Software integration is complete; closing duplicates does not establish manual, device or measurement acceptance. The 2026-10-20 combined verification in ticket 54 records 278 passing tests, 18 ignored, strict Clippy, GUI compilation and native self-checks. This audit changed tracker records only and ran no application tests.

- [ ] Finish original polish review: measure text contrast; inspect Dark/Light, narrow/wide, enlarged OS text, focus/keyboard navigation, loading/empty/error/export/selected/long-content states. Preserve semantic timeline colors and all source/audio/export safeguards.
- [ ] From 38: visually verify channel-choice counts, immediate correctly ordered skeletons, whole-recording preview then refinement, truthful per-track/overall feedback, cache-hit presentation and keyboard access in both themes and window sizes. Verify independent recording readiness, stable queued admission/eight-track limits, Retry/Remove, failure isolation and edits retained through readiness changes. Exercise individual/all/queued cancellation, New project/replacement and late results; pending/invalid sources must not enter unsafe save/analysis/export paths. Exercise imports during playback and label looping: current mix/selection/playhead remain stable and newly ready sources join on the next start.
- [ ] From 39: owner verifies absence of redundant sound-value hover instructions while typing/reset/mute/validation/context menus work; readable one-action rows, themed keycaps, long-content wrapping and scrolling at narrow/wide widths in both themes; Close/Enter/Escape and text/dialog input ownership remain correct.
- [ ] From 40: owner exercises repeated selection play/seek/stop/pause/resume, Original/Solo/Processed/Removed, monitor gain, label-loop changes and close/restore, and imports becoming ready during playback. Record perceived delay, clicks/dropouts, device unplug/error and explicit retry. Software tests and simulated device errors do not establish physical listening acceptance.

The original acceptance and the preserved checks in 38/39/40 remain the detailed contracts if a scenario fails. Fix demonstrated failures here; no new GUI feature is authorized by consolidation. Adjustable shortcuts remain separate in 41. Ticket 42 is unchanged and unstarted. Finish GUI acceptance before refreshing settled-GUI captures under 20; this ticket does not qualify stable audio quality or publication.

### Optional future benchmarking notes — no current acceptance action

From 38: retain recorded headless cold/warm, traversal, worker-cancellation and memory evidence with cache/profile limits. GUI scheduling/readiness/cancellation latency and complete-app memory remain unmeasured. Worker timing is not visible readiness. Keep owner recordings local. The owner waived formal full-session qualification when closing 14; these notes do not reopen it.

From 40: quiet Release warm p95 was 21.57 ms versus 102.57 ms baseline at the first-valid-PCM callback marker, with bounded retained stream/worker counts and sampled underrun limits. Controlled empty-cache conversion/cache-lookup comparisons, GUI input/result-delivery stages and process/GUI-GPU memory were not completed. That marker is distinct from a whole output buffer and hardware audible onset, so it does not establish the proposed 100 ms whole-buffer target. See [measurement limits](../../../docs/research/selection-playback-40-measurements.md). Revisit only if a separately requested benchmark or concrete performance issue warrants it.

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
