# 01: Measure application frame pacing and input feedback

Status: ready-for-agent
Type: task
Blocked by: None (can start immediately)
Source proposal: P2

**What to build:** Provide an opt-in, bounded performance capture that exercises the real editor and reports reproducible frame/interaction observations alongside existing processing and playback measurements.

## Acceptance criteria

- [ ] Reuse the current preview/editor/pan/device probes and add only missing application-level observations; normal operation has no unbounded event history or per-frame file I/O.
- [ ] Capture idle, playback, locked-follow, pan, zoom and sound edits with/without background work using reproducible commands and scenario metadata.
- [ ] Report sample counts and median/p95/p99 where meaningful, input-to-visible/update timing, rebuild counts and resource observations; name the exact observed boundary. CPU geometry time and redraw requests must never be labeled presented GPU frames or physical display latency.
- [ ] Record source/build identity and available display/device information; unavailable hardware metrics are explicit rather than fabricated.
- [ ] Retain a baseline for dependent tickets and add public-report or application-boundary checks through the confirmed test seams; preserve source files and existing projects.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. TDD test-boundary confirmation is pending before tests; this is a shared execution prerequisite rather than a sibling ticket. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

