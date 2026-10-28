# 106: Document Windows updater unsafe contracts

Status: ready-for-agent
Assignee: unclaimed
Readiness: ready-for-agent
Type: task
Blocked by: None (can start immediately)

## What to build

Maintainers can review each updater Windows FFI boundary using an explicit safety contract and technical justification, without changing updater behavior.

## Specification

[Accepted Google Rust audit corrections](../../offline-supervised-migration/rust-guidelines-corrections.md).

## Acceptance criteria

- [ ] Every production updater unsafe boundary has a concise SAFETY contract describing its actual pointer/buffer initialization, capacity/lifetime, NUL termination and returned length/result obligations.
- [ ] Parent-process identity ordering, handle validity/lifetime and exactly-once closure are documented against the actual control flow and pinned Windows API contracts.
- [ ] The technical reason for the unsafe platform operation is clear; no undefined behavior is claimed or behavioral rewrite introduced without evidence.
- [ ] Documentation/source consistency, links and the scoped diff are verified; no API, dependency, runtime or serialized behavior changes are included.

## Results

Not started. Record source, scoped verification and limitations here before closure.
