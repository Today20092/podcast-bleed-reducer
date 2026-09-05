# 01: Teach room tone and solo speech in the editor

**Working branch:** codex/ownership-calibration
**What to build:** Select a track/range, save room-tone or solo-speech examples, inspect/remove examples, persist mode/calibration, and invalidate stale examples. Keep protection gestures intact.
**Blocked by:** None
**Status:** ready-for-agent
**Implementation:** in progress

- [ ] Optional, understandable calibration interaction for each track, with usable selection controls and visible examples.
- [ ] New projects enable joint mode; old projects retain legacy until explicitly enabled.
- [ ] Settings save/reopen, invalidate on source changes, and mark renders stale.
- [ ] Bounds/tracks/minimum usable ranges and source identities validated; mistaken examples removable.
- [ ] Editor requests preview refresh without UI audio reads; preserve existing play/protect controls.
- [ ] Project/editor behavior tests, relevant full checks and two-axis review pass.

Read the spec, decisions and shared interface contract. Parent wires the new detector API after ticket02 lands. Do not modify detector/engine internals in this task.

## Implementation decisions

The delegated project lifecycle and headless editor seams are approved. Test mode defaults and round trips, example validation/removal, source invalidation, and editor messages that save examples without protections and refresh preview. Source checks read metadata and WAV headers, never decode PCM during calibration edits. Examples require at least 12,000 samples at 48 kHz, or 250 ms. Precise seconds fields, playhead capture, and selection capture provide keyboard-accessible range entry.
