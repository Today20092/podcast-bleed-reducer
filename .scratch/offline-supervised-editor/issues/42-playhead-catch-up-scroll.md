# Add animated catch-up scrolling at the timeline edge

Status: ready-for-agent
Type: task
Assignee: unassigned
Blocked by: 34
Spec: #accepted-design

## Scope

Implement the owner's requested playback-follow behavior in the GUI: let the playhead travel across a stationary timeline, then quickly slide the timeline left when the playhead reaches the right edge. Stop moving the timeline once the playhead reaches the left side, and repeat on the next crossing. The reference is FabFilter Pro-L 2's Slow Down display behavior, as described in the owner's supplied excerpt.

This refines the roadmap's Page-at-edge behavior into an animated catch-up rather than an abrupt page jump. Ticket 34 is done. Coordinate shared viewport and control edits with ticket 41's playhead commands and zoom preference; its unrelated shortcut work is not a prerequisite. This ticket records future GUI work; it does not claim implementation or authorize a merge.

Also refine locked navigation: middle-mouse dragging temporarily suspends locked following and reanchors the lock at the playhead's current screen position on release. This explicitly supersedes permanent unlock-on-middle-pan for an already locked playhead, including ticket 41's instruction to preserve the old manual-pan behavior. Other manual navigation retains its existing policy.

## Accepted design

1. **Stationary between catches.** While playback follow uses catch-up scrolling, keep the viewport fixed and let the playhead advance normally. Trigger one catch when it reaches the usable right edge of the waveform area, excluding track headers and scrollbar gutters. Do not start scrolling early or keep the playhead continuously centered.
2. **Quick leftward slide.** Animate the viewport forward in source time, so waveform, ruler, labels, selection and gain overlays slide left together. Keep the playhead tied to the real playback position throughout. Finish with the advancing playhead near the left edge: initially target a 5% inset of usable timeline width and approximately 250 ms duration. Use a short eased transition with a stable endpoint that accounts for playback advancing during the catch, then leave the viewport stationary again. These are initial implementation constants, not new Settings controls; record owner-requested tuning in this ticket.
3. **Repeat without jitter.** Permit only one catch at a time. After it finishes, wait for the playhead to cross the remaining visible width before another catch. A delayed frame must use elapsed time rather than replaying missed animation steps. Preserve zoom and all track alignment.
4. **Existing follow controls.** Use the existing Page-at-edge option if it is still exposed; otherwise expose one clearly named `Catch up` choice beside the current manual and locked follow behavior. Keep Lock / Unlock and Center playhead semantics intact, with only one follow behavior active at a time. Manual pan or explicit Unlock cancels an active catch and leaves the viewport at its current position. Switching to Lock cancels the catch before applying the existing lock behavior. Center remains a one-shot viewport command. Follow selection and animation state are transient, not project edits.
5. **Interruption and navigation.** Pause/stop cancels the animation without moving playback or completing a pending slide afterward. On resume, follow the current position. Seeking or loop wrap cancels the old target; if follow is active and the new playhead is outside the viewport, reveal it promptly near the left inset, clipped to valid bounds, then resume normal edge-triggered behavior. User zoom/fit cancels the old animation target and respects the applicable follow/zoom policy, including ticket 41's preference when integrated. Recompute against the new viewport instead of applying a stale target. Resize must likewise use the new usable width without changing playback position.
6. **Bounds and safety.** Clamp viewport movement to the recording's valid bounds. Near the recording end, accept that the playhead cannot return to the left inset; stop at the final valid viewport without repeated catches. A recording that fits entirely in the view does not scroll. Empty timelines are safe no-ops. Animation affects only the viewport: it must not seek, restart audio, change selection/labels/gain, mark the project dirty, or affect analysis, preview/export evaluation or saved source timing.
7. **Middle-drag repositions an existing lock.** When middle-mouse panning starts with the playhead locked, suspend automatic viewport following for the gesture while playback continues normally. Retain the user's lock intent; do not switch the control to an unlocked state. On release, capture the actual playhead position in the resulting viewport as the new horizontal lock anchor and resume locked following there, without snapping back to the old anchor. Capture the playhead position at release, not the pointer position or the playhead position when dragging began. Store the anchor as a fraction of usable timeline width, consistent with existing resize behavior. A middle click without a drag preserves the original anchor. Starting from unlocked/manual or Catch up mode must not enable lock; middle-pan in Catch up mode still cancels following as specified above. Explicit Unlock or a follow-mode change during the gesture overrides automatic relocking. If the playhead is offscreen at release, reveal it at the nearest usable edge, clamped to recording bounds, and anchor there without seeking. Release outside the timeline must still complete the gesture; on focus loss or gesture cancellation, clear the suspended state and restore the prior lock intent/anchor unless explicitly changed, so following cannot remain stuck. Paused playback supports the same anchor repositioning for the next resume. Recording-bound limits continue to take precedence over maintaining an anchor.

## Acceptance checks

- [ ] Play a recording spanning at least three view widths. Between catches the viewport is stationary. Each right-edge crossing produces one quick, visibly animated leftward slide, ending near the left inset before normal playhead travel resumes. Record a short capture and obtain owner verification of the feel against the supplied Pro-L 2 reference.
- [ ] Verify at narrow and wide window sizes and multiple zoom levels; all timeline layers stay aligned, the playhead reflects uninterrupted audio time, and a delayed frame does not cause oscillation or duplicate catches.
- [ ] Exercise pause/resume, stop, seek forward/backward, loop wrap, manual pan, Center, Lock / Unlock, zoom/fit and resize during a catch. No stale animation continues after interruption; existing control semantics and ticket 41's integrated zoom preference remain consistent.
- [ ] While locked, middle-drag in both directions during playback and while paused. The viewport follows the drag without fighting it; release reanchors at the playhead's then-current visible position with no snap to the old anchor, and resumed playback holds the new anchor. Confirm a click without dragging preserves the old anchor, resizing preserves its fraction, and playback time/selection/labels/gain/dirty state remain unchanged by the gesture.
- [ ] Cover unlocked and Catch up starting states, an offscreen playhead at release, recording boundaries, release outside the timeline, focus loss/cancellation, and explicit Unlock/mode change during the gesture. No unintended lock activation, stale reanchor or permanently suspended following occurs. Add focused state/gesture regression tests for suspend, release/reanchor and cancellation.
- [ ] Verify empty/short recordings, full-recording fit and the final partial viewport. No overscroll, invalid time range or repeated end-of-recording catch occurs.
- [ ] Add focused viewport-state tests using controlled elapsed time for edge triggering, animation progress/completion, rearming, interruption and boundary clamping. Confirm navigation leaves playback/source time, selection, labels, gain and project dirty state unchanged.
- [ ] Follow docs/gui-development.md and use scripts/gui-dev.ps1 for the development build/launch. Run formatting, applicable existing GUI/navigation tests and strict all-target/all-feature Clippy. Record results and unavailable visual/listening checks before marking done.

## Out of scope

FabFilter's Fast, Slow, Infinite and Off metering modes; new audio meters; continuous waveform compression; configurable animation speed/inset; playback-engine changes; dependency changes; shortcut remapping; and locked-playhead changes beyond the specified middle-drag suspension/reanchoring.

## Results

Not implemented. No GUI, playback, timing or visual acceptance checks have been run for this ticket.

## Comments

2026-10-20: Owner requested a GUI ticket for a screen that catches up with the playhead, citing the supplied FabFilter Pro-L 2 Slow Down description. The defining behavior is a fixed viewport, a quick catch at the right edge, then another fixed interval once the playhead reaches the left side. The other display modes in the excerpt are context, not requested features.

2026-10-20 follow-up: Owner requested that middle-mouse timeline scrolling temporarily suspend an existing playhead lock, then lock at its current screen position on release, replacing the old permanent unlock-on-pan behavior. Added the interaction, its precedence over ticket 41's old manual-pan policy, and acceptance checks. Ticket only; no GUI behavior changed.
