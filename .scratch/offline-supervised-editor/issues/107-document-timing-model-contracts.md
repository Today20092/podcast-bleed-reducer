# 107: Document timing, gain and ownership-model API contracts

Status: ready-for-agent
Assignee: unclaimed
Readiness: ready-for-agent
Type: task
Blocked by: None (can start immediately)

## What to build

Developers using exported timing/gain and ownership inference interfaces can determine the required coordinates, bounds and feature semantics from the API documentation, reducing mistakes that compile with primitive types.

## Specification

[Accepted Google Rust audit corrections](../../offline-supervised-migration/rust-guidelines-corrections.md).

## Acceptance criteria

- [ ] Project time units, half-open interval semantics, source-sample coordinates, conversion rounding and gain accessor boundary behavior are documented at the non-obvious exported seams.
- [ ] Ownership inference documentation explains feature order/dimension, finite-value and normalization responsibilities, and the meaning of unavailable results and errors.
- [ ] Representative examples or links explain actual usage without duplicating whole domain manuals or narrating self-evident names.
- [ ] Source/contract consistency and links are checked; public signatures, serde representation, timing behavior and model recipes are unchanged.
- [ ] Any new executable documentation example is verified when resource policy permits; otherwise use clear non-executable examples and record the scoped validation.

## Results

Not started. Record source, scoped verification and limitations here before closure.
