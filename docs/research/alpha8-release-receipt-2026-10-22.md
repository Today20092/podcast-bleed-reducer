# Alpha.8 release verification, 2026-10-22

The owner accepted the integrated development application and authorized documentation, screenshots, promotion and release. [Ticket 64](../../.scratch/offline-supervised-editor/issues/64-alpha-release-preparation.md) tracks completion; [PR 17](https://github.com/Today20092/podcast-bleed-reducer/pull/17) promotes development to main with a merge commit.

## Source and documentation

Initial alpha.7 candidate source: `7312c44ab8935ef55cc256e085fe399da8e921e9`. Seven Windows GUI screenshots are actual application captures, including preservation, gain dragging, shortcut reference and grouped controls. Temporary demo edits were undone. Alpha.8 retains that interface and corrects package notices/version metadata. 106 local documentation references resolved; native images were visually checked. Browser-rendered Markdown was unavailable on this machine.

Local formatting, script fixtures and strict Clippy passed. Local test compilation could not finish after Windows reported paging-file exhaustion (OS 1455); a four-codegen-unit command-only retry also exhausted rustc allocation. No host settings or product build configuration were changed. [Fresh native PR checks](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39732866350) passed: Windows 353 tests, each Mac/Linux target 349, all with 20 ignored and zero failures, plus formatting, strict Clippy and script fixtures. Ignored diagnostics/qualification tests are not passing evidence.

## Native checks and downloaded packages

PR 17 merged as `e5c37aa29856da002f148f9475ca171e8ad98674`; annotated tag `v0.1.0-alpha.7` points to that reviewed main commit. Development was synchronized to the merge. [Alpha.7 native package workflow](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39734525594) passed source checks but all four package collectors rejected missing `zune-core 0.4.12` license texts. No draft or public release was produced. The failed tag remains unchanged.

Alpha.8 includes the upstream Zlib alternative for both `zune-core 0.4.12` and `zune-jpeg 0.4.21`. Their crate VCS receipts identify commits `f8fbb123d5ed04441e8324a555bfcda0cb1bd28f` and `fa2c767a01d7d9373911d0bf63e0588553d67e0e`; both contain identical LICENSE-ZLIB text. A local metadata audit of all resolved dependencies on all four desktop targets found no remaining missing license coverage.

[Alpha.8 PR 18](https://github.com/Today20092/podcast-bleed-reducer/pull/18) passed [all four source checks](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39741751147): Windows 353 tests, each Mac/Linux target 349, zero failures and 20 ignored on each. Formatting, strict Clippy and packaging fixtures passed. Merge commit `24361a2aa201fb7e22258f44a2a1be4626d58ce1` is the immutable `v0.1.0-alpha.8` source; development is synchronized. [Package run 39742979614](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39742979614) passed all four native package jobs and created one draft with eight assets. The actual release downloads passed their checksum files and match the checked Actions artifacts byte for byte.

| Package | SHA-256 |
| --- | --- |
| Windows x64 ZIP | `0da2697dc6c59c6f6b490440331fbfa4841f803c6d1dd4fe277b238099ba67a0` |
| Linux x64 AppImage | `527d2da04b67e8c0ef86c3d93809117d757ad41769d5f4632c951ffc2cc6d82d` |
| macOS arm64 DMG | `b3bad4afa0dba620073b2573f059b3b826017fad87d7657d26e03717ac72c611` |
| macOS x64 DMG | `b04d815bc5b54f2e9ccab2029460048f88ecfeaf7219e5443120ca9d85f04175` |

## Clean Windows acceptance

The downloaded Windows package passed extracted-file integrity checks; its manifest reports alpha.8, the source commit above and `dirty: false`. Its executable and supporting files are preserved outside the checkout in `C:/Users/User/Documents/Codex/Builds/CleanSlate-alpha8-verification/package/`.

A fresh Windows 11 Sandbox had networking, clipboard redirection and vGPU disabled. Only the package and owner-authorized fixture copies were mapped read-only, with a separate writable results folder. No developer runtime or model cache was supplied. The packaged self-check reported `PASS: offline model inference, analysis, project save/reopen, render and cancellation.`

Interactive checks imported three synchronized ten-minute microphone WAVs; reopened a copy of the owner's project; relinked its three sources with all 18 human examples retained; resaved existing solo and room-tone classifications; analyzed offline; advanced Original, Processed and Removed playback; saved and reopened the managed project; and exported three separate WAVs. The GUI confirmed successful export. Independent RIFF inspection confirmed each output is mono, 48 kHz, 32-bit float, 28,800,000 frames and 600 seconds. Sources were read-only throughout. Reopening correctly requires Analyze before processed output is available. Catch up remained selected through example navigation. These checks establish package operation, not a new independent listening-quality result.

The same downloaded binary also opened the representative project on the Windows 11 host, analyzed its ten-minute sources and advanced Original, Processed and Removed playback through the host audio device, including the labeled Speaker 1 + Speaker 2 overlap region. Stop and mode switching completed without device errors. This is an operational device check; no fresh subjective listening or independent sound-quality score is claimed. The owner's earlier development listening acceptance remains the quality evidence.

## Publication

[CleanSlate v0.1.0-alpha.8](https://github.com/Today20092/podcast-bleed-reducer/releases/tag/v0.1.0-alpha.8) was published as an experimental prerelease on 2026-10-22 with four native packages and their four SHA-256 files. All eight public asset URLs returned HTTP 200 without authentication after publication. The reviewed tag and package assets are immutable; final documentation and download links are reconciled through [PR 19](https://github.com/Today20092/podcast-bleed-reducer/pull/19) and synchronized back to development.

## Known limits

This is an experimental alpha. Owner acceptance does not supply fresh physical Mac/Linux desktop testing, whole-session independent benchmarks or an exhaustive device/gesture matrix. Known quiet-overlap and wide-canvas frame-budget limits remain documented in the [performance summary](alpha7-performance-summary-2026-10-22.md).

An existing [low-severity dependency alert](https://github.com/Today20092/podcast-bleed-reducer/security/dependabot/1) concerns `atty 0.2.14`, reached through `bwavfile 2.0.1 → clap 2.34.0`. GitHub lists no patched version. The advisory describes a potential unaligned Windows read with a custom allocator; this application's only custom allocator declaration is test-only and delegates allocation to System. The dependency remains present; this is not a claim that the advisory was fixed.
