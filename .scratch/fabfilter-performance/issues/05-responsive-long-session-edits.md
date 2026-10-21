# 05: Keep long-session sound and gain edits responsive

Status: ready-for-agent
Type: task
Blocked by: 01 — Measure application frame pacing and input feedback
Source proposal: P4

**What to build:** Measure warm dial/gain edits on long recordings and dense effective envelopes, then eliminate demonstrated synchronous rebuild/copy bottlenecks while preserving immediate, authoritative editing and preview/export parity.

## Acceptance criteria

- [ ] Record repeated warm edit latency and preparation reuse on representative long/dense sessions using ticket01 and existing editor/preview probes.
- [ ] Apply a targeted recomputation, coalescing or preparation change only where measurements show a bottleneck; record a supported no-change decision if the path is already within its measured budget.
- [ ] Verify last-edit wins, one undoable drag, source/result generation validation and cancellation. Shared audition/render envelope evaluation stays numerically equivalent.
- [ ] No inference rerun for a warm sound edit, silent quality reduction or new user quality mode. Record before/after results and any hardware feedback limits.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. TDD test-boundary confirmation is pending before tests; this is a shared execution prerequisite rather than a sibling ticket. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

