# Podcast visual, motion and performance refinement

Status: claimed
Assignee: Codex
Readiness: ready-for-agent
Type: task
Blocked by: 96, 102
Related: 80, 84, 90, 101

## Scope and acceptance

Implement the owner's FabFilter/sonible quality target in the existing Podcast editor using the shared contract and 102's original visual study. Preserve the complete capture/label/Analyze/audition/recovery workflow; the waveform remains the main work surface.

- [ ] Replace default-looking control density with deliberate type/spacing, restrained sections, readable numeric controls, source/gain layering, a stable playhead and visible selection/focus. Keep actions reachable at 720×560 and 1000×680, 100/150/200% DPI, eight tracks and all readiness/error states.
- [ ] Implement 90's interruption-safe Catch up and visual value smoothing, useful meter/playhead feedback and reduced motion. Immediate audio/automation changes remain independent of visual interpolation. Do not smooth away captured audio transients or human-label boundaries.
- [ ] No ongoing animation when the editor is closed/hidden or when settled; schedule only current visible work, bounded by display width and visible lanes. Preserve keyboard, focus, hover help, selection and scroll semantics.
- [ ] Compare the same pre/post fixtures and build profile: event-to-state, peak/geometry/tessellation work, allocations, per-instance memory/CPU, idle/playback/resize and simultaneous instances. Treat 60 Hz as a presentation target, not a passed result. Report p50/p95/p99 and misses; measure native presentation separately from CPU geometry.
- [ ] Verify unchanged effective-envelope/format/state/automation behavior with relevant existing checks. Keep native listening/PDC/export acceptance separate; no 80 closure by screenshots or synthetic timing.
- [ ] Preserve tested source/artifacts and compact/full/DPI captures; independent Standards/Spec review before integration. Use existing guarded product scripts, one compiler job globally, no new GUI dependency without a demonstrated blocker and a separately measured integration proof.

## Results

Claimed 2026-10-27 from permanent Podcast tip 13e6dfd after 96 integration and 102 completion. Scope uses the existing egui/Session seams and shared visual contract; no framework migration. Gate 93 owns the sole heavy slot: only document/source inspection and one first RED test may be prepared until the coordinator grants execution. No production implementation or renderer/compiler execution has started. Native ticket 80 and dependencies 81–83 remain unchanged.

Coordinator subsequently granted the sole heavy slot. First public-egui/Session Catch up test produced semantic RED (instant jump) then GREEN for a finite 250 ms eased slide with selection/host transport unchanged. Comparable eight-track generated native-PCM baseline records cold and 40 warm query/draw/tessellation samples, p50/p95/p99, mesh counts/capacity at compact/full sizes. No visual refinement, reduced-motion implementation, gain/processing change or native/FPS qualification has landed. Context7 quota was exhausted; pinned egui 0.37 source supplied API documentation.

The next reduced-motion/stop regression could not execute: unchanged guard `20261027-102017-498887ecffda405da9a8ede99fe30316.json` stopped compilation on low system commit headroom (7.74 GiB, below 8 GiB running reserve), with cleanup verified. No semantic RED or GREEN is claimed for that test, and no reduced-motion implementation followed. No retry or threshold relaxation occurred. Keep all acceptance unchecked; finite-motion checkpoint still needs interruption/seek/hidden/reduced-motion completion, formatting, full relevant checks, visual slices and comparisons, final artifacts/captures and independent review. [Checkpoint and raw baseline](../../../docs/research/podcast-ui-polish-2026-10-27.md).
