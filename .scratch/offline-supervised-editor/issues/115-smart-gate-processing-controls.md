# Implement the complete vocal/podcast gate processing controls

Status: open
Readiness: needs-triage
Type: task
Blocked by: 93, 114
Related: 85, 94, 116

## Scope

Implement the original processing contract selected in 114. Ticket 93 owns the
existing detector/worker/native-processing foundation; this ticket expands that
working foundation into the selected complete vocal/podcast control set. Final
GUI styling is separate. Research is evidence, not a vendor-equivalence spec.

Keep one shared DSP/parameter/state implementation behind VST3 and CLAP. Retain
Silero v6.2 unless 114 records a justified change. Extracted sonible assets stay
outside the product. Neither generic VAD nor a preset is a personal voice model.

## Implementation sequence and acceptance

- [ ] Core controls: real threshold/sensitivity, Tolerance/softness, level
  influence, reduction strength/depth and Attack/Hold/Release. Every control
  changes actual processing; interpolation/automation stays finite and smooth.
- [ ] Onset preservation and latency: implement the chosen bounded delay and
  envelope policy, sample alignment, host latency reporting and reset behavior.
  A faster Attack alone does not recover audio preceding delayed detection.
- [ ] Advanced processing: implement the contracted three-band shaping and
  crossovers, band enable/solo, Gate/Duck and detector/spatial routing. Neutral
  settings reconstruct the expected signal; missing sidechain never silently
  substitutes an unintended source. Failure fallback remains safe in both modes.
- [ ] Workflow/state: provide time-aligned Original/Processed/Removed audition,
  versioned parameters, presets/A-B and a stable GUI-facing status/meter/history
  interface. Sound recall must not revive temporary activation or stale evidence.
- [ ] Realtime: prepare storage outside callbacks; no callback model inference,
  unbounded wait, file work or new allocation. Keep source epochs, finite-output
  checks, cancellation/reset and bounded stale-result behavior intact.
- [ ] Use focused behavioral checks for new transfer endpoints, parameter
  changes, timing/alignment, routing and neutral/failure cases. Reuse existing
  fixtures and checks; do not create a broad benchmark campaign or duplicate
  already-valid acceptance merely to add test volume.
- [ ] Hand 116 an exact source checkpoint, supported parameter inventory,
  state schema and expected processing behavior. Document remaining quality
  limitations without claiming voice ownership or superiority over sonible.

## Comments

2026-10-31: Draft only. Dependencies must be completed and 114 must select exact
behavior before this ticket is claimed. No backend or GUI edits were started.
