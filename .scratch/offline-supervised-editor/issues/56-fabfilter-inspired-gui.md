# Complete the FabFilter-inspired GUI review branch

Status: done
Type: task
Assignee: Codex
Blocked by: 16, 21
Related: 23

## Owner authorization — 2026-10-21

After selecting inspector B, the owner requested implementing the entire earlier fourteen-row design proposal and applicable lessons from the Pro-Q 4 / Pro-R 2 investigation, using subagents, on `codex/gui-inspector` for morning review. This supersedes the earlier first-pass-only sequencing. Keep development/main integration and publication separate. Dependencies 16 and 21 are done. Initial B source is `cf580e5`, from development `e668e60`; disposable sketches remain on `codex/gui-inspector-sketches` (`b28c841`).

Source proposal: this chat's “Here's the full set I'd consider, in priority order” table. Evidence: `C:/Users/User/Documents/Codex/Investigations/fabfilter-gui-2026-10-21/report.html`, especially “Why the interfaces work well” and “What is useful for CleanSlate.” The decoded artwork is reference evidence, not assets to ship or running-plugin screenshots.

## Full implementation checklist

- [x] Distinguish command actions, persistent states, editable numbers and disclosures through their visual treatments.
- [x] Shared coherent button heights/padding/corners/alignment; normal, hover, pressed, selected, disabled and visible keyboard-focus states.
- [x] One primary next action: Analyze when examples need applying; Export when the completed result is current. Stable control positions.
- [x] Compact grouped inspector: status, examples, featured sound, supporting transitions/gaps, export and collapsed sources.
- [x] Left labels, aligned inset editable values with stable digit widths and sliders beneath. Retain all typed validation/commit/cancel/reset behavior.
- [x] Status/export remain reachable outside the body scroll, including expanded examples, long messages, blockers and export errors. Compact windows pair status and Export above the body instead of increasing panel height.
- [x] Attenuation and Mute are the main sound controls, with timing settings grouped beneath.
- [x] Concise state-specific next-action guidance; keep requirements/errors visible and longer instructions discoverable in help.
- [x] Concise examples count/readiness, compact Undo/Redo/Analyze row and a manageable expandable examples list.
- [x] Original / Processed / Removed become one segmented listening selector, with persistent selection distinct from command emphasis.
- [x] Clearly relate selected track/range/editing target and applicable controls. Preserve red reduction, purple gain and blue selection.
- [x] Quiet occasional project-management actions after opening a project; preserve saving, keyboard access and discoverability.
- [x] Brief purposeful hover/selection/disclosure transitions, immediate action delivery and no continuous idle frame requests.
- [x] Implement selected attenuation dial B, numeric editing, explicit mute/reset and bounded adjustment; keep timing sliders.

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

Implementation complete on `codex/gui-inspector`; owner appearance acceptance remains in 23. Scoped subagents implemented shared controls/focus/transitions and existing waveform/gain/example selection polish; the main agent integrated inspector/listening/toolbar changes and coordinated every Cargo run. Initial B is `cf580e5`, the full pass `5e716bc`, review corrections `ece546b`, and final executable source **`24b812a640c5f8e83110035cb053835773e5a2b4`**. No integration into development/main or publication was performed.

The inspector keeps status, source review, Export and bounded general feedback reachable outside the body scroll in desktop and compact layouts. Numeric values retain their validation and full typed ranges; the attenuation dial keeps the former slider's −30…−3 dB quick range, four pixels per dB, with click-without-drag preserving extended typed values and Mute. Timing settings and all audio defaults/update paths remain unchanged. Buttons have visible keyboard focus and finite hover/selected accents; pointer use leaves the existing playback Space command available. Tab navigation reveals controls in nested scroll areas. Shortcut tooltip dismissal resets only its timer/open state, retaining the focused/armed button beneath it.

Review corrected captured numeric Tab/Escape routing, competing editing-target highlights and unrelated range context, and feedback hidden inside Sources. Standards review initially reported 1 finding and Spec review 3; all were resolved. Both follow-ups, including the final tooltip-state fix, found no further actionable issues. Regressions cover dial click/drag/keyboard ownership, finite and offscreen transitions, clipped disclosure input, selected editing target, nested focus reveal, pointer/keyboard button ownership and hint-reset survival.

Final `scripts/check.ps1 -Local` passed build/workflow/packaging/release-status fixtures, formatting, strict all-target/all-feature Clippy and all-target tests: **311 passed, 0 failed, 18 existing opt-in checks ignored, across 33 targets**, including 159 passing library tests. No profile override was used. Existing Dark/Light contrast checks passed; measured minimum regular-text ratios were 4.5263:1 Dark and 4.5971:1 Light, action-text 4.6827:1 and 7.0909:1, status-on-panel 5.3234:1 and 6.8481:1. These palette checks do not establish physical font legibility.

`scripts/gui-dev.ps1 -BuildOnly` passed. Canonical and preserved executables both passed `--package-self-check` (offline inference, analysis, project save/reopen, render and cancellation). [Build receipt](C:/Users/User/AppData/Local/CleanSlate/VerifiedBuilds/ticket-56-24b812a/receipt.md) records commands, source, hashes and limitations. Preserved executable: `C:/Users/User/AppData/Local/CleanSlate/VerifiedBuilds/ticket-56-24b812a/rust-app.exe`; canonical executable: `C:/Users/User/AppData/Local/CleanSlate/BuildCache/debug/rust-app.exe`. Both SHA-256: `6943F1D73EFDCED4D1D107714A4153E13C215EC24A67D0B8EF917E721A999516`.

`scripts/gui-dev.ps1` launched the separate `.gui-dev.cleanslate` copy at 02:17:56 EDT; process 46184 remained running when checked. Original quick-test project SHA-256 remained `20BB912825964A2BA9B717525477F6AF365ABB3E434084CF7836E6B5725CBD98`. Native UI automation is unavailable: rendered narrow/wide, both themes, enlarged OS text, physical keyboard/mouse and listening/device acceptance were **not** performed. Browser sketches, regression tests and process launch are not native appearance acceptance. Those owner checks remain in 23/41; other-platform packages are untested. Ticket 42 retains SHA-256 `CD9E048A9CF97D0048271F793BD556FD24A0E00CF925741436C2739D2AA4BF31` unchanged; deferred 35 was not implemented.
