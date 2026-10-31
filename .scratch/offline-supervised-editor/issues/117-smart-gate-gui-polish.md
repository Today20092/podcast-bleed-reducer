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

## Live orchestration

- Run: `run_84f8a85fc7e0` (coordinator: this lusca session).
- Workspace: `../smart-gate-gui-polish`, branch `codex/smart-gate-gui-polish`.
- Builder: task `task_66769da7a67c`, dispatch `ctx_243a06c24d7b`.
- Independent reference review: task `task_9b4f7480779b`, dispatch `ctx_03b0684c0728`.
- Both structured workers started successfully, 2026-10-31 18:01 UTC.
- Baseline review succeeded and was released; report retained at
  `../smart-gate-gui-polish/docs/reviews/smart-gate-gui-baseline.md`.
- After-change review: task `task_d627e3107339`, dispatch `ctx_f295a7d23ed4`.
- Production GUI edits and builds belong only to the builder. Reviewer owns its
  report; coordinator owns this ticket and cross-workspace integration decisions.

## Delivered checkpoint — 2026-10-31 18:19 UTC

All three Orca Tasks settled `succeeded` and all three workers were released;
no reclaimable terminal remains. Accept the bounded first visual iteration,
not complete native/performance acceptance. Keep this ticket claimed for the
remaining qualification below; 94 and DSP93/114/115/116 are not cleared.

Implementation: `08530970b9693c7b0a3b77d6649af79585cf2ed6`.
Evidence/ticket: `bc1c717` on `codex/smart-gate-gui-polish`.
Result: `../smart-gate-gui-polish/docs/research/smart-gate-gui-polish-result.md`.
Independent baseline/after reports: `../smart-gate-gui-polish/docs/reviews/`.

Implemented a reserved Range rail, aligned level meters, independent downward
GR scale, readable selected controls, precision value entry and host gesture
cleanup, truthful Last/Held labels, immediate meter attack/180 ms visual decay,
unchanged-history caching and 250 ms inactive polling. The owner-requested
minimal chrome remains; no permanent title/banner/footer or fake DSP controls.

34 tests, formatting, strict all-target Clippy and actual CLAP/VST3 coexistence
checks pass. 28 actual-widget CPU captures cover seven states, compact/full and
100/150% scale. Independent review closed both concrete visual findings and a
stale stopped-image derivative; no blocking source/visual defect was found in
the bounded pass. Full native open/close/reopen/close succeeded programmatically.

Preserved preview launcher:
`C:/Users/User/AppData/Local/CleanSlate/Gui117/preview-0853097/Open Smart Gate Preview.cmd`.
Preserved binary SHA256: `fe330aa81ecc21a936be9b4bcebd5138525944ff2cfb127ec56df8d855d7c6df`.
Inventory: `C:/Users/User/AppData/Local/CleanSlate/Gui117/evidence-manifest.json`.
Preview opens safely bypassed/unarmed, without audio-device/demo processing.
The workspace/artifacts remain for owner iteration; primary development and
installed links remain unchanged because this source includes unfinished93.

Remaining qualification: native interaction/keyboard/automation forwarding and
DPI could not be observed through the occluded Orca provider/unavailable CUA
pipe. Native compact negotiation is rejected by the incumbent fixed-size
adapter; compact captures qualify CPU layout only. The small CPU drawing fixture
has sub-1 ms medians, but whole native idle process used ~0.65 core in one 5 s
sample without attribution/baseline. Do not claim low native idle cost, hidden
scheduling, physical FPS, listening acceptance, full GUI117 completion or release.
Carry these concrete checks into this ticket/94 before final GUI acceptance.

## Acceptance checklist (full native acceptance remains open)

- [x] Establish rendered baseline and a concise design/motion brief from the
  actual editor and the existing reference research. Preserve product truth.
- [x] Improve the complete current surface: graph hierarchy, input/processed
  history and applied reduction, meters, typography, spacing, semantic colors,
  real controls, readiness/empty/error/bypass/frozen states and useful help.
- [x] Give supported controls precise, predictable pointer/keyboard/value-entry
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
- [x] Run focused existing UI checks, formatting and lint appropriate to edited
  files. Only the implementation owner coordinates builds. Preserve the exact
  tested candidate/host/artifact hashes before shared caches change.
- [x] Inspect actual native open/close and interactions through supported tools
  when available. Distinguish requested refresh rate and CPU-render work from
  measured presentation FPS; document unperformed native/performance checks.
- [x] Independent visual/interaction review is processed, critical defects fixed,
  and a reviewable local preview plus screenshots and limitations are delivered.
  Do not replace installed links or a user-owned active preview unexpectedly.

Final integration of future DSP controls remains 94 after the 114/115/116
backend contract and real candidate are ready. Public release is not authorized.
