# Establish the accepted development baseline

Status: done
Assignee: baseline_implementation
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

2026-10-08 baseline qualification: `codex/offline-baseline` started at integration commit `f75080c`. Seven Iced slider step literals now explicitly use `f32`; this removes compiler numeric fallback diagnostics without changing slider values or behavior. No new tests were added for literal type annotations.

Fresh checks on the changed code passed: `cargo fmt --all -- --check`, `cargo test --all-targets --locked -j 2` (87 passed, six ignored), `cargo clippy --all-targets --all-features --locked -j 2 -- -D warnings`, and `cargo build --locked -j 2`. The built `target/debug/rust-app.exe` launched with a nonzero native window handle, title `CleanSlate`, a responding process, and empty startup stderr. The launched process was closed afterward. This establishes launch acceptance, not a visual interaction, playback-device, or listening qualification.

Reusable infrastructure, with its current limits:

- `audio_io::Stem` and engine source validation: cancellable WAV/BWF decode, mono/48 kHz constraints, sample validation and timing metadata. Tickets 02-04 extend these existing paths; they do not rebuild import from scratch.
- `waveform`, `timeline`, and `app`: source-stamped waveform caching, incremental import, shared viewport/selection/transport and background jobs. Retain these while introducing source-aware time coordinates and editing.
- `audition`: shared source-checked PCM preparation, cancellation/generation handling, monitor-only gain and Original/Processed/Removed modes. It currently assumes 48 kHz and retains one full PCM session; native-rate and long-session acceptance remain later work.
- `prepared_analysis::PreviewCache`: shared immutable fixed facts separated from ownership derivation, source invalidation and diagnostics. The current VAD/joint detector is only a frozen comparison baseline; Nemotron and supervised ownership are unimplemented.
- `engine::GainEnvelope`: common envelope evaluation and protected regions, already used by preview/audition/render. Editable manual constraints and finite exponential endpoints remain tickets 11-13.
- `project`, `review`, `export`, and `audio_io::AtomicOutput`: persistence/review plumbing, current-render checks, collision rejection, cancellation and staged completion. Reuse safety behavior while replacing the project contract; earlier persisted formats need no migration.
- `detector_quality` and `release_gate`: evaluation/reporting scaffolding exists. Historical developmental evidence does not satisfy independent quality, long-session performance, or owner listening approval in tickets 14-15.

Prior roadmap states were reconciled by the implementation index's Existing work reconciliation section. Historical ADRs/records remain in the verified recovery snapshot; ADR-0006 records target supersession without claiming the old detector is accepted. No later implementation ticket is completed by this baseline qualification.
