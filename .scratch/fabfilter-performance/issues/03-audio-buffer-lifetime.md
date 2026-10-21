# 03: Audit and harden audio-device buffer reclamation

Status: ready-for-agent
Type: task
Blocked by: None (can start immediately)
Source proposal: P3

**What to build:** Keep the playback device path predictable by measuring heap-owning block/envelope lifetime transitions and moving demonstrated reclamation work off it without compromising bounded, nonblocking streaming.

## Acceptance criteria

- [ ] Trace and instrument allocation/deallocation at the playback output boundary for steady playback, seeks, envelope/mode changes, stale blocks and starvation; retain baseline evidence.
- [ ] Remove demonstrated avoidable heap reclamation from device iteration using established worker/queue ownership patterns, or retain a measured justification where no change is needed.
- [ ] Keep bounded memory and nonblocking device iteration; preserve generation rejection, exact output/mode/envelope behavior, silence on starvation, underrun counts and safe teardown.
- [ ] Exercise playback output and resource lifetime at confirmed seams; distinguish deterministic output/allocation probes from unperformed hardware listening or deadline guarantees.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. TDD test-boundary confirmation is pending before tests; this is a shared execution prerequisite rather than a sibling ticket. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

