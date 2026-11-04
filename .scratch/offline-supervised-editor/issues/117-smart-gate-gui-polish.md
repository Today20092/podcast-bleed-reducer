# Smart Gate visual polish and responsive motion

Status: claimed
Assignee: Orca supervised GUI track (single implementation owner)
Readiness: ready-for-agent
Type: task
Blocked by: 90
Related: 85, 93, 94, 114, 115, 116

## Owner-authorized parallel scope

2026-10-31 owner rejects the first pass after opening it. Required revision:
actual native resizing; internal title/toolbar like the reference hierarchy;
custom high-quality Range and buttons; smooth responsive drawing. Supplied
FabFilter CLAP targets: Pro-G, Pro-L2, Pro-Q4 under Program Files/FabFilter.
Use a separate REA worker and supplied screenshots; original CleanSlate artwork
and honest supported controls remain required. The title/toolbar explicitly
supersedes the previous removal preference. First-pass review acceptance below
is historical and is not owner visual acceptance. Continue in the same GUI
workspace; provide a fast live-signal preview for meaningful motion iteration.

Revision Run `run_0b716777320d` started successfully in the same GUI workspace:
builder `task_1a34670fad19` / `ctx_15b57d6dfc49`; FabFilter REA
`task_9b98000e547b` / `ctx_118ee4ff5afc`; source mechanics
`task_26e0c176dfc3` / `ctx_4cf772ad4d8d`. Builder is sole production writer and
compiler; investigation workers own only their separate research reports.

Mechanics task succeeded; report retained and worker released. Structured-chat
reuse through `worker-start --terminal` was refused in preflight
(`terminal_unsupported_for_agent_session`); request/state inspection confirmed
no extra Task or Dispatch. A fresh independent reviewer started successfully:
`task_612e8758294e` / `ctx_65a9b636e074`.

Revision acceptance (supersedes first-pass visual acceptance):
- [ ] Real native resize from640x440 through larger sizes, correct host/editor
  agreement and same-instance reopen, without merely stretching parent chrome.
- [ ] Crafted internal title/toolbar, precise Range dial and custom controls;
  materially improved rendered compact/full states and independent visual review.
- [ ] Correct Windows delayed repaint scheduling, active motion pacing and
  measured bounded idle behavior, without changing DSP timing or claiming FPS
  from requested intervals.
- [ ] Safe labeled live-signal preview and quick local build/run workflow so
  waveform/control motion can be evaluated without a DAW or audio device.
- [ ] Focused FabFilter Pro-G/Pro-L2/Pro-Q4 CLAP report with exact identities,
  evidence-backed conclusions/unknowns; preserved revised candidate and reports.

The coordinator authorizes a minimal reproducible patch of pinned baseview0.4.1
inside the plugin workspace if needed to honor the Windows repaint-message
delay. Preserve exact upstream/license provenance, add focused scheduling
verification and avoid registry edits, upgrades or desktop dependency changes.

2026-10-31 real-audio preview steering: owner supplies
`C:/Users/User/Documents/podcast-automix-rust-gui/test-audio/quick-test-10min/261005-channel-1-10min.wav`.
Header verified: RIFF float32 (tag3), mono48kHz,600s,115200000 data bytes.
Owner explicitly selects **waveform/meters only**, not audible playback. Extend
the preview host with read-only real-WAV input at realtime cadence, paused
startup, host playback/position controls and clear source labeling. Do not open
an audio output device or copy the media into Git. Retain synthetic mode only
as a separately labeled optional fixture. This extends the same active GUI task.

Owner iteration window opened by coordinator: PID21820, preserved
`Gui117/revision-final-7140295/smart_gate_vst3_check.exe`, paused preview.
Owner explicitly asks to keep the UI open for feedback. It is user-owned and
must remain separate from worker cleanup; coordinator will switch it to the
WAV-enabled candidate when ready. This does not retain any agent terminal.

Revision checkpoint: UI commit1ba729a, focused event-pump/gesture7140295 and
host assertion phase27e67da are preserved. Final foreground UI-entry sample:
median15.829ms, p9531.056ms, max32.934ms; UI-body median0.186ms. This is UI-call
timing, not physical FPS or a controlled comparison with earlier occlusion.
Foreground idle tail median269.133ms and processCPU234.375/5004.605ms were
observed.38 tests/lint/build and both format checks passed for that checkpoint;
new WAV-mode checks and exact final source/host identity remain pending.
REA and mechanics Tasks succeeded and their workers were released; reports are
retained uncommitted in the GUI workspace for coordinator preservation after
builder commits. Fresh Pro-G CLAP report cites29 verified EvidenceIDs from a
74-record bundle; detailed L2/Q4 CLAP internals and vendor runtime/FPS remain
explicit gaps. Reviewer and builder remain active for real-WAV completion.

Build coordination note: an external completed compiler root (exit0) left only
VCTIP31452 in its owned job, holding the global wrapper. Coordinator checked
`20261031-144746-e5913e0f0a154735a8de3786d3b25256.diagnostics.log`, exact MSVC
14.44.35207 helper path/creation time, and absence of compiler6832/rustc28756/
parent26868 before stopping only that orphan helper. No active build, memory
limit or coordination bypass was changed; the queued GUI rebuild then completed.

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
