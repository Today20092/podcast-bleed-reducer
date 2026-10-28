# 107: Document timing, gain and ownership-model API contracts

Status: done
Assignee: Codex rust_ticket107
Readiness: ready-for-agent
Type: task
Blocked by: None (can start immediately)

## What to build

Developers using exported timing/gain and ownership inference interfaces can determine the required coordinates, bounds and feature semantics from the API documentation, reducing mistakes that compile with primitive types.

## Specification

[Accepted Google Rust audit corrections](../../offline-supervised-migration/rust-guidelines-corrections.md).

## Acceptance criteria

- [x] Project time units, half-open interval semantics, source-sample coordinates, conversion rounding and gain accessor boundary behavior are documented at the non-obvious exported seams.
- [x] Ownership inference documentation explains feature order/dimension, finite-value and normalization responsibilities, and the meaning of unavailable results and errors.
- [x] Representative examples or links explain actual usage without duplicating whole domain manuals or narrating self-evident names.
- [x] Source/contract consistency and links are checked; public signatures, serde representation, timing behavior and model recipes are unchanged.
- [x] Any new executable documentation example is verified when resource policy permits; otherwise use clear non-executable examples and record the scoped validation.

## Results

Completed 2026-10-27 on `codex/rust-audit-107`, based on integration commit `88887932e7877891225c8639b36b4b1354703b17`.

Added targeted rustdoc for project ticks, native-frame conversions, outward half-open range rounding, and envelope sample/time access. The contracts retain two non-obvious existing distinctions: effective envelopes return unity beyond the source while legacy point access extrapolates, and time-bound queries test emptiness after outward sample-grid conversion. Added recipe-space feature order/dimension/value limits, internally applied training normalization, metadata/current-truth validation responsibilities, and the difference between an unavailable predictor result and zero ownership. Linked the existing effective-envelope and supervised feature manuals rather than duplicating those recipes.

Verification: graph-first symbol/caller inspection followed by exact editing-source reads, including the underlying effective evaluator and production feature assembly. Contracts were cross-checked against the existing timebase, effective-envelope and ownership-model regressions, including arbitrary-rate preservation edges, the source endpoint, retained constant-feature inputs, and rejected invalid inference inputs. The numeric 96 kHz range example matches an existing literal regression. Normalized before/after comparisons excluding full-line comments confirmed identical non-comment contents for all three changed Rust files; signatures, representation and model recipes remain unchanged. `git diff --check` passed. The ticket's relative specification link and linked domain-manual paths resolve; intra-doc targets were checked against their definitions.

Examples are prose/inline notation only; no executable documentation fences or new tests were added. No compiler, rustdoc build, formatter, test suite or native host was launched. Existing tests were inspected, not rerun; full-batch execution and review remain with the integration coordinator under the existing resource policy. This documentation-only scope claims no newly tested audio or inference behavior.
