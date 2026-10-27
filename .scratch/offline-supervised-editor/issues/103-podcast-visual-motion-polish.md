# Podcast visual, motion and performance refinement

Status: claimed
Assignee: Codex (Podcast UI refinement)
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

Claimed 2026-10-27 after 96 completed and integrated at `13e6dfd`. Scoped `codex/podcast-ui-polish-103` starts at that exact permanent Podcast tip in the retained Podcast worktree. One initial unrun public egui/Session Catch up regression is prepared for the accepted 250 ms progression, selection preservation and unchanged host transport. No production implementation yet; 93 owns the compiler slot until explicit handoff. 102 supplies the original design target, not measured vendor-equivalent performance or native animation qualification.
