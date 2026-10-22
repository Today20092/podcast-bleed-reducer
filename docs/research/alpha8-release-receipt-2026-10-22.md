# Alpha.8 release verification, 2026-10-22

The owner accepted the integrated development application and authorized documentation, screenshots, promotion and release. [Ticket 64](../../.scratch/offline-supervised-editor/issues/64-alpha-release-preparation.md) tracks completion; [PR 17](https://github.com/Today20092/podcast-bleed-reducer/pull/17) promotes development to main with a merge commit.

## Source and documentation

Initial alpha.7 candidate source: `7312c44ab8935ef55cc256e085fe399da8e921e9`. Seven Windows GUI screenshots are actual application captures, including preservation, gain dragging, shortcut reference and grouped controls. Temporary demo edits were undone. Alpha.8 retains that interface and corrects package notices/version metadata. 106 local documentation references resolved; native images were visually checked. Browser-rendered Markdown was unavailable on this machine.

Local formatting, script fixtures and strict Clippy passed. Local test compilation could not finish after Windows reported paging-file exhaustion (OS 1455); a four-codegen-unit command-only retry also exhausted rustc allocation. No host settings or product build configuration were changed. [Fresh native PR checks](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39732866350) passed: Windows 353 tests, each Mac/Linux target 349, all with 20 ignored and zero failures, plus formatting, strict Clippy and script fixtures. Ignored diagnostics/qualification tests are not passing evidence.

## Native checks and downloaded packages

PR 17 merged as `e5c37aa29856da002f148f9475ca171e8ad98674`; annotated tag `v0.1.0-alpha.7` points to that reviewed main commit. Development was synchronized to the merge. [Alpha.7 native package workflow](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39734525594) passed source checks but all four package collectors rejected missing `zune-core 0.4.12` license texts. No draft or public release was produced. The failed tag remains unchanged.

The corrected candidate advances to alpha.8 and includes the upstream Zlib alternative for both `zune-core 0.4.12` and `zune-jpeg 0.4.21`. Their crate VCS receipts identify commits `f8fbb123d5ed04441e8324a555bfcda0cb1bd28f` and `fa2c767a01d7d9373911d0bf63e0588553d67e0e`; both contain identical LICENSE-ZLIB text. A local metadata audit of all resolved dependencies on all four desktop targets found no remaining missing license coverage. Source CI, package verification and clean offline Windows verification for alpha.8 remain pending; publication must wait for that boundary.

## Known limits

This is an experimental alpha. Owner acceptance does not supply fresh physical Mac/Linux desktop testing, whole-session independent benchmarks or an exhaustive device/gesture matrix. Known quiet-overlap and wide-canvas frame-budget limits remain documented in the [performance summary](alpha7-performance-summary-2026-10-22.md).

An existing [low-severity dependency alert](https://github.com/Today20092/podcast-bleed-reducer/security/dependabot/1) concerns `atty 0.2.14`, reached through `bwavfile 2.0.1 → clap 2.34.0`. GitHub lists no patched version. The advisory describes a potential unaligned Windows read with a custom allocator; this application's only custom allocator declaration is test-only and delegates allocation to System. The dependency remains present; this is not a claim that the advisory was fixed.
