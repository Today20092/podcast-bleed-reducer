# Smart Gate visual polish and responsive motion

Status: claimed
Assignee: Orca supervised GUI track (single implementation owner)
Readiness: ready-for-agent
Type: task
Blocked by: 90
Related: 85, 93, 94, 114, 115, 116

## Owner-authorized parallel scope

2026-10-31: Owner explicitly requests supervised GUI implementation now, using
the FabFilter and sonible reference research to produce a clean, professional,
well-animated and responsive vocal/podcast plugin. This authorizes this bounded
visual track in parallel with unfinished backend work. It does not clear 93 or
claim that the expanded processing contract in 114/115 is implemented.

Source base: preserved latest committed GUI checkpoint `a6ca1f6`, on a dedicated
`codex/smart-gate-gui-polish` workspace. Retain the existing Rust/nice-plug/egui
editor and original CleanSlate identity. Do not make a framework migration,
replace vendor artwork, change the detector/gain algorithm or alter identities.
One worker owns production UI files; an independent worker reviews evidence.

## Acceptance

- [ ] Establish rendered baseline and a concise design/motion brief from the
  actual editor and the existing reference research. Preserve product truth.
- [ ] Improve the complete current surface: graph hierarchy, input/processed
  history and applied reduction, meters, typography, spacing, semantic colors,
  real controls, readiness/empty/error/bypass/frozen states and useful help.
- [ ] Give supported controls precise, predictable pointer/keyboard/value-entry
  behavior. Show genuine parameter and detector state; no working-looking fake
  Threshold/Tolerance/AHR/sidechain controls where the backend lacks them.
  Future full-control composition may be documented as a clearly labeled design
  artifact, separate from the functional plugin editor.
- [ ] Add purposeful, interruptible UI-only motion and readable meter ballistics.
  Bound work/history geometry; avoid perpetual redraw when idle/hidden. Respect
  reduced motion where supported. Audio callback behavior remains untouched.
- [ ] Verify compact 640x440 and full 1000x620 layouts and representative DPI,
  current/waiting/failed/bypass/stopped/frozen states, with rendered evidence.
  Preserve normal host automation gestures, state/arming and close/reopen safety.
- [ ] Run focused existing UI checks, formatting and lint appropriate to edited
  files. Only the implementation owner coordinates builds. Preserve the exact
  tested candidate/host/artifact hashes before shared caches change.
- [ ] Inspect actual native open/close and interactions through supported tools
  when available. Distinguish requested refresh rate and CPU-render work from
  measured presentation FPS; document unperformed native/performance checks.
- [ ] Independent visual/interaction review is processed, critical defects fixed,
  and a reviewable local preview plus screenshots and limitations are delivered.
  Do not replace installed links or a user-owned active preview unexpectedly.

Final integration of future DSP controls remains 94 after the 114/115/116
backend contract and real candidate are ready. Public release is not authorized.
