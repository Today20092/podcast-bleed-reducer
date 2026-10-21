# 06: Avoid optional visual work in a minimized editor

Status: ready-for-agent
Type: task
Blocked by: 01 — Measure application frame pacing and input feedback
Source proposal: P5

**What to build:** Measure visible versus minimized playback and background work, then suspend optional visuals with no visible consumer where needed and resume truthfully without pausing useful audio or jobs.

## Acceptance criteria

- [ ] Confirm the installed GUI stack's real visibility/minimization behavior; unfocused but visible windows retain normal updates.
- [ ] Compare visible/minimized/resumed observations; if the framework already suppresses the work, record evidence instead of adding duplicate polling or state.
- [ ] Where needed, gate optional drawing/meter updates and restore the current playhead/meters on resume without stale screens.
- [ ] Playback, accumulating measurements, analysis/export progress, cancellation and result acceptance remain correct through visibility transitions; verify through confirmed application/playback seams and native observation where possible.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. TDD test-boundary confirmation is pending before tests; this is a shared execution prerequisite rather than a sibling ticket. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

