# 08: Protect foreground work during background processing

Status: ready-for-agent
Type: task
Blocked by: 01 — Measure application frame pacing and input feedback
Source proposal: P8

**What to build:** Measure playback/edit responsiveness during representative imports and analysis, then tune only demonstrated contention using bounded independent work while retaining processing correctness and useful throughput.

## Acceptance criteria

- [ ] Record foreground latency, preparation/analysis throughput, memory and available underrun observations for representative track counts, durations and rates with heavy work active.
- [ ] Inspect actual worker/critical-path behavior before changing concurrency; record a measured no-change decision when more scheduling machinery is unjustified.
- [ ] Any tuning preserves source validation, cancellation, stale-generation rejection, authoritative envelope/export results and bounded memory.
- [ ] Add no speculative parallel DSP/SIMD rewrite or user worker-count setting; retain reproducible comparison evidence and verify the affected public job/playback/report boundaries.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. TDD test-boundary confirmation is pending before tests; this is a shared execution prerequisite rather than a sibling ticket. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

