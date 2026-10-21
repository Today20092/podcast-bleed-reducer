# Complete the FabFilter-inspired GUI review branch

Status: claimed
Type: task
Assignee: Codex
Blocked by: 16, 21
Related: 23

## Owner authorization — 2026-10-21

After selecting inspector B, the owner requested implementing the entire earlier fourteen-row design proposal and applicable lessons from the Pro-Q 4 / Pro-R 2 investigation, using subagents, on `codex/gui-inspector` for morning review. This supersedes the earlier first-pass-only sequencing. Keep development/main integration and publication separate. Dependencies 16 and 21 are done. Initial B source is `cf580e5`, from development `e668e60`; disposable sketches remain on `codex/gui-inspector-sketches` (`b28c841`).

Source proposal: this chat's “Here's the full set I'd consider, in priority order” table. Evidence: `C:/Users/User/Documents/Codex/Investigations/fabfilter-gui-2026-10-21/report.html`, especially “Why the interfaces work well” and “What is useful for CleanSlate.” The decoded artwork is reference evidence, not assets to ship or running-plugin screenshots.

## Full implementation checklist

- [ ] Distinguish command actions, persistent states, editable numbers and disclosures through their visual treatments.
- [ ] Shared coherent button heights/padding/corners/alignment; normal, hover, pressed, selected, disabled and visible keyboard-focus states.
- [ ] One primary next action: Analyze when examples need applying; Export when the completed result is current. Stable control positions.
- [ ] Compact grouped inspector: status, examples, featured sound, supporting transitions/gaps, export and collapsed sources.
- [ ] Left labels, aligned inset editable values with stable digit widths and sliders beneath. Retain all typed validation/commit/cancel/reset behavior.
- [ ] Status/export remain reachable outside the body scroll, including expanded examples, long messages, blockers and export errors. Compact windows pair status and Export above the body instead of increasing panel height.
- [ ] Attenuation and Mute are the main sound controls, with timing settings grouped beneath.
- [ ] Concise state-specific next-action guidance; keep requirements/errors visible and longer instructions discoverable in help.
- [ ] Concise examples count/readiness, compact Undo/Redo/Analyze row and a manageable expandable examples list.
- [ ] Original / Processed / Removed become one segmented listening selector, with persistent selection distinct from command emphasis.
- [ ] Clearly relate selected track/range/editing target and applicable controls. Preserve red reduction, purple gain and blue selection.
- [ ] Quiet occasional project-management actions after opening a project; preserve saving, keyboard access and discoverability.
- [ ] Brief purposeful hover/selection/disclosure transitions, immediate action delivery and no continuous idle frame requests.
- [ ] Implement selected attenuation dial B, numeric editing, explicit mute/reset and bounded adjustment; keep timing sliders.

## Report guardrails

Use original native vector controls and shared themed components with dynamic data geometry. Refine existing direct waveform/example/gain manipulation and active editing/listening cues. Keep window resizing, OS/UI scaling and timeline zoom independent; the report does not specify a new scale setting. Audit existing caches/invalidation/frame scheduling before renderer changes. Exact FabFilter easing, sensitivity, FPS and runtime behavior were not established by static analysis.

Do not add EQ/reverb DSP, copy FabFilter assets/code, replace Iced/rendering, implement deferred ticket 35 or catch-up scrolling ticket 42, change accepted audio/defaults/ranges, or weaken source validation, cancellation, persistence and safe export. Ticket 41 retains its separate owner acceptance. All changes stay on this GUI branch.

## Verification and handoff

- Focused regressions for new input/focus/animation/selection logic and existing sound, shortcut ownership, analysis/export guards.
- Dark/Light text contrast, formatting, strict Clippy, `scripts/check.ps1 -Local`, GUI build through `scripts/gui-dev.ps1` and executable self-check.
- Record actual native visual checks separately from browser sketches: narrow/wide, both themes, enlarged text, focus/keyboard, empty/loading/error/stale/selected/long-content states.
- Preserve source commit, executable and receipt; open the separate development project copy for owner review when build checks finish. Do not modify original owner recordings/projects.
- Record any unavailable native/hardware checks honestly. Implementation completion does not imply owner acceptance, integration or release.

## Results

In progress. Parallel ownership: shared controls/focus/transitions, waveform/gain selection polish, and main-agent inspector/listening/toolbar/integration. Main agent coordinates Cargo and final evidence; no parallel Cargo runs.
