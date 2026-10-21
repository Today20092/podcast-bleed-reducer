# 07: Measure and improve continuous-follow drawing reuse

Status: ready-for-agent
Type: task
Blocked by: 01 — Measure application frame pacing and input feedback
Source proposal: P6

**What to build:** Use complete-editor and geometry measurements to decide whether continuously following the locked playhead needs additional retained geometry reuse, and implement the smallest justified improvement.

## Acceptance criteria

- [ ] Measure waveform plus gain drawing together, separating CPU preparation and application frame observations from actual presented-frame metrics.
- [ ] If follow geometry is a demonstrated bottleneck, add bounded reuse within the existing renderer; otherwise record the measured decision and preserve the current implementation.
- [ ] Verify clipping, source time, edits, zoom/fit, resize, themes/scales, recording bounds and middle-pan release reanchoring; bound additional memory.
- [ ] Coordinate with the existing edge-catch-up proposal without implementing its separate animation behavior or changing playback/selection semantics.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. TDD test-boundary confirmation is pending before tests; this is a shared execution prerequisite rather than a sibling ticket. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

