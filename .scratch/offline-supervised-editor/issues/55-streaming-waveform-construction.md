# 55: Share streaming waveform-cache construction

Status: done
Type: task
Assignee: Codex
Blocked by: none

## What to build

Progressive import and waveform-cache rebuilding produce identical detailed waveforms for identical source samples, regardless of decoder chunk boundaries. The waveform module owns streaming bucket accounting, completed-prefix refinement and final cache construction. Preserve early full-recording previews, per-channel identity, validation before acceptance, cancellation and persisted-cache compatibility.

## Acceptance criteria

- [x] The waveform module owns streaming bucket size, extrema, sample count and final partial-bucket handling; import and rebuilding no longer duplicate these rules.
- [x] Chunked construction matches independently known peak values for empty input, exact bucket boundaries, irregular chunks, multiple buckets and a final partial bucket.
- [x] Progressive prefix refinement includes completed detail without presenting unfinished source validation as accepted audio.
- [x] Multichannel source separation and existing cache identity/version behavior are retained.
- [x] Existing import cancellation, corrupt/truncated/changed-source rejection and persisted-cache regressions pass.
- [x] Shared local checks and GUI build pass; source commit and executable path are recorded. Test compilation used the temporary override documented below.
- [x] Two-axis code review is complete; the tested batch is merged into development and temporary worktrees are retired safely.

## Accepted testing seams

1. Waveform construction: samples/chunks to resulting cache and observable peaks. Catches bucket/tail/chunk discrepancies; does not prove decoder correctness.
2. Existing import events and waveform loading: source fixtures to previews, accepted cache and persisted reload. Catches wiring, per-channel separation and validation regressions; does not establish listening quality or physical removable-media behavior.

## Comments

2026-10-20: owner approved the single-ticket breakdown, no blocking edges and both testing seams after the architecture report. This is the only suggestion selected from that review and the only implementation ticket authorized in this batch. It follows the accepted migration architecture and ADR-0006. No new dependency, source mutation, cache-format change, GUI interaction or audio-policy change. Existing unrelated tickets remain outside this batch; main/release promotion remains separate.

Review report: `C:/Users/User/AppData/Local/Temp/architecture-review-20261020-cleanslate.html`. Exploration notes: `C:/Users/User/AppData/Local/Temp/cleanslate-architecture-scan-20261020.md`.

## Implementation checks — 2026-10-20

- Implementation worktree started at integration commit `9b1e5fb`, on `codex/waveform-55`.
- A crate-private waveform builder owns sample ingestion, completed-prefix refinement, sample count and final partial-bucket completion. Both import and waveform loading use it; raw peak construction/refinement remain private to the waveform module. The test-only in-memory constructor also uses it, removing the third bucket implementation. Existing pyramid combination and persisted cache format/version remain unchanged.
- TDD red: `cargo test --lib <shared arguments> streaming_waveform_preserves_chunk_boundaries_and_only_refines_complete_buckets -- --exact --nocapture` failed compilation with E0433 for the absent `WaveformBuilder`; green: rerunning the filter without `--exact` selected and passed that regression after implementation. Explicit expected base peaks above unity avoid the old test helper's clamped extrema as an oracle.
- Scoped checks used `scripts/local-build-settings.ps1`, `CARGO_INCREMENTAL=1` and its locked/offline one-job Cargo arguments: `cargo test --lib <shared arguments> waveform:: -- --nocapture` passed 12 tests, with the existing owner-recording visual test ignored; `cargo test --lib <shared arguments> import:: -- --nocapture` passed 6 tests.
- Coverage includes empty/exact/irregular/partial buckets, completed-prefix preview behavior, channel separation, persisted-cache invalidation/rebuild, import-to-reopen cache equality, cancellation and invalid/truncated/changed-source rejection. Formatting and `git diff --check` pass.
- Combined primary-checkout checks, GUI build, two-axis review, development integration and worktree retirement remain pending; this ticket stays claimed until those finish. No physical-media or listening check was run for this refactor.

## Integration checks — 2026-10-20

- Integration branch merged implementation at `f9022e7`; development merge is `102fee921f09278c9e05ea09697c304350f0103d`. Both independent review axes compared with baseline `835fef1`: Standards 0 findings, Spec 0 findings. The import/reopen equality check exercises persisted reload; existing channel/cache tests separately exercise cold rebuilding with known peaks.
- On primary development, shared local build settings, native PR/workflow fixtures, packaging/download fixtures, release-status fixtures, formatting and strict all-target/all-feature Clippy passed through `scripts/check.ps1 -Local`.
- The standard full-test build and a library-only retry stopped in rustc with memory-allocation failures before test execution. No application assertion failed. Available virtual memory was about 3 GiB after the initial failure. Other running applications were left intact.
- A command-only override, `--config profile.test.package.rust-app.codegen-units=4`, retains the pinned toolchain, shared target, locked/offline dependencies, one Cargo job and incremental compilation while reducing parallel code generation. The unchanged library suite then passed: 128 passed, 0 failed, 10 ignored. The project profile is unchanged; the initial standard-profile run is not claimed as passing.
- Cargo documentation used for the temporary override: <https://doc.rust-lang.org/cargo/reference/profiles.html#codegen-units> and <https://doc.rust-lang.org/cargo/reference/config.html#command-line-overrides>.
- Full all-target testing with that command-only override passed across 33 targets: **280 passed, 0 failed, 18 ignored**. Existing ignored owner-recording/device, independent-evaluation and measurement checks remain unperformed. Test compilation completed in 5m36s with the lower-parallelism override; no performance claim is made.
- `scripts/gui-dev.ps1 -BuildOnly` passed with the unchanged development profile (7.37 seconds observed). Native `--package-self-check` passed for both the canonical and preserved executable: offline model inference, analysis, project save/reopen, render and cancellation.
- Tested source commit: `102fee921f09278c9e05ea09697c304350f0103d`. Canonical executable: `C:/Users/User/AppData/Local/CleanSlate/BuildCache/debug/rust-app.exe`. Preserved executable and receipt: `C:/Users/User/AppData/Local/CleanSlate/VerifiedBuilds/ticket-55-102fee9/`. Executable SHA-256: `D32F659680D0BD5E33C28EF9F4E665BEA2D4D5B5EDA1C3A3BF34165A03496F8B`.
- Source changes are limited to the waveform module and import caller; cache format/version, decoder, GUI interactions, audio policy and dependencies are unchanged. Physical-media, listening and other-platform checks were not run.
- Completion: all ticket work is merged into development; main remains unchanged. The app archive request returned queued without removing the checkout, so the verified clean worktree (including no ignored/untracked files) was removed with `git worktree remove` after confirming its commit was an ancestor of development. Both temporary merged branches were deleted with `git branch -d`. The frozen baseline and shared build cache remain intact; the tested executable/receipt were preserved first. This is the only ticket implemented from the architecture review.
