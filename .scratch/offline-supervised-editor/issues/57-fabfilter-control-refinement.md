# Refine native controls from Pro-Q 4 / Pro-R 2 references

Status: done
Type: task
Assignee: Codex
Blocked by: 16, 21, 34
Related: 23, 42, 56

## Owner authorization — 2026-10-21

The owner supplied four screenshots, accepted development as the stable comparison, and requested an autonomous second GUI pass using REA, the installed Pro-Q 4 / Pro-R 2 and the prior report. The owner delegates design decisions and wants every recommendation in this scoped pass implemented before reviewing. Work only on `codex/fabfilter-controls`, created from development `91fa021`; do not merge or advance development/main. Dependencies 16/21/34 are implemented. Retain the approved build and original projects.

## Accepted implementation decisions

- [x] Use a coherent rotary family for continuous sound parameters: retain a featured attenuation dial and replace the three transition/gap sliders with smaller timing dials and always-readable editable values. Commands, toggles and disclosures remain distinct controls. Keep existing full typed ranges/defaults, quick-adjustment ranges and audio update paths.
- [x] Give all parameter dials original layered vector faces, clear value arcs/pointers, quiet endpoint markers and explicit hover/drag/disabled/Mute states. Preserve semantic red reduction, purple gain and blue selection; use accent color purposefully rather than reproducing FabFilter artwork.
- [x] Implement precise native dial gestures: vertical drag, Shift fine adjustment, hover wheel adjustment, keyboard adjustment with text/dialog ownership, Ctrl/Command-click default reset and double-click numeric editing. Single click must not change a value or Mute; extended typed values must survive until a real intentional adjustment. Complete changed drags once on release outside/unfocus. Numeric validation and commit/cancel/reset remain authoritative.
- [x] Replace long Lock playhead / Center playhead / Meters labels with a compact, consistent toolbar using original vector icons and short visible labels. Keep effective-binding/state tooltips, visible keyboard focus, selected lock state, existing commands and independent meter placement.
- [x] Remove the nested box treatment from Original / Processed / Removed. Use stable-width flat segments with one clear persistent selected treatment, no redundant checkmark/outer border and keyboard-only focus indication. Preserve readiness, shortcuts and live audition semantics.
- [x] Implement the specifically requested locked middle-drag suspension/reanchor behavior from ticket 42: retain lock intent during the gesture, pan freely while audio continues, reanchor from the actual release-time playhead position, preserve a no-drag click, handle offscreen/bounds/release-outside/cancel/unfocus, and let explicit mode changes win. Do not implement the separate animated edge-catch proposal or change seeking/audio/selection/project data.
- [x] Refine sidebar grouping/spacing and compact layout around these controls, preserving pinned status/export/errors, source blockers, selected-editing context, Dark/Light contrast and existing keyboard/shortcut ownership. No hidden processing or new DSP features.

## Reference evidence

Prior report and decoded original resource sheet: `C:/Users/User/Documents/Codex/Investigations/fabfilter-gui-2026-10-21/`. Fresh REA resource inventories: Pro-Q 4 `ev_2c0d9b91b999c1afec34c34b603c38966df97549b45a011aa094b0648ddbfb45` (361 resources, artifact SHA-256 `8f4f09d08cb2b495dadf386af90c83f47f44d36841ba66800098da809223fe15`); Pro-R 2 `ev_b55835f11621bcdbd1b1421610d86d3b35670d945a0526f468398c3a6e9879be` (178 resources, SHA-256 `de7ce568fb7acd12abf02cfaf73120d8bba2630f1f280cf77b87435148cf1fee`). Static resources establish packaged artwork/state layers, not runtime behavior or exact easing.

Primary documented knob behavior: https://www.fabfilter.com/help/pro-q/using/knobs and https://www.fabfilter.com/help/pro-r/using/knobs (vertical/wheel/text entry, fine adjustment and modifier reset). The existing report remains the renderer investigation; no new renderer, copied asset/code or library version is needed.

## Checks and handoff

Add focused gesture/state regressions at real widget/app seams, run formatting, strict Clippy and appropriate complete local checks, build through `scripts/gui-dev.ps1`, self-check and preserve the executable/source receipt. Inspect native rendering where supported; identify unavailable physical/native checks honestly. Keep the baseline GUI/project intact, and launch the review build on a separate development project copy. Record all results here before closing implementation; owner acceptance and later merge remain separate.

## Results

Implemented all seven scoped decisions on `codex/fabfilter-controls`. Source commit `a77a60bf625534063924326465a162728f7eff58` contains the implementation and native-discovered scrollbar fix; later documentation records this receipt. Owner appearance acceptance and integration remain separate.

- Replaced the attenuation-only widget with one shared parameter-dial family: featured attenuation and three smaller timing dials, layered original vector artwork, visible values and semantic states. Commands/toggles retain their distinct forms.
- Added vertical/fine drag, wheel accumulation, owned arrow adjustment, platform-modifier reset and double-click numeric entry. Ten widget regressions cover intentional adjustment, unchanged clicks/Mute/extended values, modifier changes, disabled/text ownership, precise wheel input and once-only completion.
- Added original icon/short-label Lock, Center and Meters controls, flat fixed-width listening segments, persistent selection and keyboard focus. Numeric validation, preview and audition paths remain shared with existing controls.
- Added locked middle-pan suspension/reanchor through viewport, waveform and app state, with eight regressions covering live release time, no movement, bounds/offscreen, cancel/unfocus/outside release, explicit mode precedence, stale gestures and selected-example/project preservation. Ctrl-wheel/manual Pan keeps its existing policy. No seeking or animated edge catch was added.
- Updated in-app help and both GUI/workflow guides. Reserved right-side scrollbar space after a native compact-layout check found a clipped saved-count edge.

Validation on the final source:

- `scripts/check.ps1 -Local`: **326 passed, 0 failed, 18 existing opt-in checks ignored across 33 targets**; formatting, strict all-target/all-feature Clippy, local-build/workflow/packaging/release-status fixtures passed. Standard pinned toolchain/shared incremental profile, no overrides.
- `scripts/gui-dev.ps1`: final build passed (0.98 seconds after test compilation; not a performance benchmark). Canonical and preserved binaries each passed `--package-self-check` for offline inference, analysis, save/reopen, render and cancellation.
- Two independent review axes cleared the full pass; the final scrollbar-only delta also has no actionable findings.
- Native Windows Computer Use checked wide 2560×1392 and compact 1028×930 layouts, the compact scrollbar fix, Dark/Light control contrast, dial wheel adjustment/restoration, double-click numeric focus, a typed draft/Escape cancellation, selected Lock state, meter placement toggle, visible keyboard focus and the flat listening row. Theme restored to Dark; review GUI left open on the isolated development project copy.
- Physical held-modifier mouse gestures and middle-button dragging are unsupported by the available native automation API; their event/state regressions pass, but physical acceptance remains unclaimed. Enlarged OS text, audio-device/listening and cross-platform checks were not performed.

Preserved build and complete receipt: `C:/Users/User/Documents/Codex/Builds/CleanSlate-fabfilter-controls-a77a60b/` (`CleanSlate.exe`, `receipt.md`, both self-check reports and check/build logs). Executable SHA-256: `593534077E56FD4D9240532F94F17C3F642ED6B74D5356D2FDAB337E76A16790`.

Fresh REA inventories exported to `C:/Users/User/Documents/Codex/Investigations/fabfilter-gui-2026-10-21/gui-controls-followup-evidence.json`. Static resource findings and primary documented knob behavior informed the original implementation; exact proprietary runtime/easing behavior is not claimed.

Development remains `91fa021efe84450e332f598bc8358ff4bfbda204`; main remains `0b9622a88643093fc4496cd4729be1d441b3a27c`. Ticket 42 remains byte-identical (SHA-256 `CD9E048A9CF97D0048271F793BD556FD24A0E00CF925741436C2739D2AA4BF31`); only its specifically requested middle-drag rule was implemented here. Both the original quick-test project and previous accepted GUI development project remain at SHA-256 `20BB912825964A2BA9B717525477F6AF365ABB3E434084CF7836E6B5725CBD98`. No merge, release or publication occurred.

Discovery limitation: code graph transport failed twice; focused source searches were used as the documented graph-gap fallback.
