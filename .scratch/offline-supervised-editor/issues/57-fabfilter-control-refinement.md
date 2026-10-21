# Refine native controls from Pro-Q 4 / Pro-R 2 references

Status: claimed
Type: task
Assignee: Codex
Blocked by: 16, 21, 34
Related: 23, 42, 56

## Owner authorization — 2026-10-21

The owner supplied four screenshots, accepted development as the stable comparison, and requested an autonomous second GUI pass using REA, the installed Pro-Q 4 / Pro-R 2 and the prior report. The owner delegates design decisions and wants every recommendation in this scoped pass implemented before reviewing. Work only on `codex/fabfilter-controls`, created from development `91fa021`; do not merge or advance development/main. Dependencies 16/21/34 are implemented. Retain the approved build and original projects.

## Accepted implementation decisions

- [ ] Use a coherent rotary family for continuous sound parameters: retain a featured attenuation dial and replace the three transition/gap sliders with smaller timing dials and always-readable editable values. Commands, toggles and disclosures remain distinct controls. Keep existing full typed ranges/defaults, quick-adjustment ranges and audio update paths.
- [ ] Give all parameter dials original layered vector faces, clear value arcs/pointers, quiet endpoint markers and explicit hover/drag/disabled/Mute states. Preserve semantic red reduction, purple gain and blue selection; use accent color purposefully rather than reproducing FabFilter artwork.
- [ ] Implement precise native dial gestures: vertical drag, Shift fine adjustment, hover wheel adjustment, keyboard adjustment with text/dialog ownership, Ctrl/Command-click default reset and double-click numeric editing. Single click must not change a value or Mute; extended typed values must survive until a real intentional adjustment. Complete changed drags once on release outside/unfocus. Numeric validation and commit/cancel/reset remain authoritative.
- [ ] Replace long Lock playhead / Center playhead / Meters labels with a compact, consistent toolbar using original vector icons and short visible labels. Keep effective-binding/state tooltips, visible keyboard focus, selected lock state, existing commands and independent meter placement.
- [ ] Remove the nested box treatment from Original / Processed / Removed. Use stable-width flat segments with one clear persistent selected treatment, no redundant checkmark/outer border and keyboard-only focus indication. Preserve readiness, shortcuts and live audition semantics.
- [ ] Implement the specifically requested locked middle-drag suspension/reanchor behavior from ticket 42: retain lock intent during the gesture, pan freely while audio continues, reanchor from the actual release-time playhead position, preserve a no-drag click, handle offscreen/bounds/release-outside/cancel/unfocus, and let explicit mode changes win. Do not implement the separate animated edge-catch proposal or change seeking/audio/selection/project data.
- [ ] Refine sidebar grouping/spacing and compact layout around these controls, preserving pinned status/export/errors, source blockers, selected-editing context, Dark/Light contrast and existing keyboard/shortcut ownership. No hidden processing or new DSP features.

## Reference evidence

Prior report and decoded original resource sheet: `C:/Users/User/Documents/Codex/Investigations/fabfilter-gui-2026-10-21/`. Fresh REA resource inventories: Pro-Q 4 `ev_2c0d9b91b999c1afec34c34b603c38966df97549b45a011aa094b0648ddbfb45` (361 resources, artifact SHA-256 `8f4f09d08cb2b495dadf386af90c83f47f44d36841ba66800098da809223fe15`); Pro-R 2 `ev_b55835f11621bcdbd1b1421610d86d3b35670d945a0526f468398c3a6e9879be` (178 resources, SHA-256 `de7ce568fb7acd12abf02cfaf73120d8bba2630f1f280cf77b87435148cf1fee`). Static resources establish packaged artwork/state layers, not runtime behavior or exact easing.

Primary documented knob behavior: https://www.fabfilter.com/help/pro-q/using/knobs and https://www.fabfilter.com/help/pro-r/using/knobs (vertical/wheel/text entry, fine adjustment and modifier reset). The existing report remains the renderer investigation; no new renderer, copied asset/code or library version is needed.

## Checks and handoff

Add focused gesture/state regressions at real widget/app seams, run formatting, strict Clippy and appropriate complete local checks, build through `scripts/gui-dev.ps1`, self-check and preserve the executable/source receipt. Inspect native rendering where supported; identify unavailable physical/native checks honestly. Keep the baseline GUI/project intact, and launch the review build on a separate development project copy. Record all results here before closing implementation; owner acceptance and later merge remain separate.

## Results

In progress. The code graph transport is unavailable after two attempts; use focused source searches as the documented graph-gap fallback. REA resource tools are available. Ticket 42's accepted middle-drag rule exists as specification; its separate edge-catch work remains outside this pass.
