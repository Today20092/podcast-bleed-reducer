# Establish the accepted development baseline

Status: ready-for-agent
Blocked by: none
Spec: ../spec.md

## Scope

The implementation branch is now `codex/offline-supervised`, established from `6a60cc2bfe187ea889fd2d8bcc3344311cfd706b`. The owner requested removal of superseded branches/worktrees and roadmap folders on 2026-10-08. The new map, tickets, source code and current domain guidance are consolidated in the primary checkout. Use this existing branch rather than recreating the baseline.

Historical roadmap records and rejected experiments were removed from the active workspace after a verified external recovery snapshot. Their implemented-code/acceptance distinctions are summarized in the new implementation index. Finish fresh baseline checks and identify reusable infrastructure. Retain commit `6a60cc2` as the frozen detector comparison baseline; it remains in Git ancestry and is not quality-accepted. Use the existing library/application structure instead of splitting many crates.

## Reach

Current Git baseline, Cargo manifest/lockfile, root domain docs/ADR-0006 and the new implementation docs. The primary checkout now contains the developed editor; it is no longer the scaffold. Older records are recovery material, not an active roadmap.

## Acceptance

- The chosen commit is reproducible and all required records remain recoverable.
- Fresh baseline formatting, existing tests and strict Clippy pass or are reported as concrete blockers; historical reports do not substitute for these checks.
- Existing editor launches and its import/audition/export infrastructure is identified for reuse.
- Prior issue states distinguish implemented code from still-pending quality acceptance. No duplicates pretend implemented infrastructure is new work.
- Planning records and supersession notes coexist with historical ADRs without silently rewriting past decisions.

## Comments

2026-10-08 cleanup: branch consolidation and roadmap migration completed. Twenty-two old branches and three clean worktrees were removed. Historical commits and untracked records were saved outside the project before cleanup; the new map and all fifteen build specifications are preserved. GUI launch and the full scoped baseline acceptance remain to be recorded before this ticket is done.

Fresh checks: `cargo fmt --all -- --check` passed; `cargo test --all-targets --locked -j 2` passed (87 passed, six ignored). Strict Clippy (`cargo clippy --all-targets --all-features --locked -j 2 -- -D warnings`) is blocked by seven existing `f32: From<f64>` fallback diagnostics in `src/app.rs` at lines 2909, 2921, 2930, 2941, 2949, 2956 and 3019. Resolve these during baseline qualification. GUI launch has not been checked. The code graph was refreshed successfully for this checkout.

Recovery snapshot: `C:\Users\User\.codex\backups\podcast-automix-rust-gui\cleanup-2026-10-08-tprrpkwx` (verified Git bundle and original workspace records).
