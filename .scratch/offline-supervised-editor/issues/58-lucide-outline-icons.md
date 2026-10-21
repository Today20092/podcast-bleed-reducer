# Adopt Lucide outlines in the native GUI

Status: done
Type: task
Assignee: Codex
Blocked by: 57
Related: 23

## Owner request

The owner selected Lucide outlines after reviewing the GUI screenshots. Continue on `codex/fabfilter-controls`; preserve development and the prior tested builds. The owner clarified that shadcn was only a compatibility question and is not requested.

## Scope and acceptance

- Replace the custom Lock/Center/Meters artwork with bundled Lucide outline assets and add matching Previous/Play/Pause/Stop transport artwork. Keep labels, tooltips, focus/selection, meter placement feedback and all command behavior.
- Use the existing Iced native SVG widget with only the required SVG feature; pin the upstream asset revision and retain complete license/provenance in source, the application and packaged notices. No web UI, shadcn, network icon loading or processing changes.
- Verify Dark/Light and compact/wide rendering, including disabled/primary icon contrast, run appropriate standard local checks and the GUI build, and preserve a verified executable and mobile-readable screenshots.

## Results

Implemented on `codex/fabfilter-controls` at executable source `8935e1422ee42147da7d32ead4346b02d36c12b7`. The eight unchanged Lucide SVGs and complete license are pinned to upstream `20aef31db93fe4d9f398dbb15b120d99c48eef95` under `assets/lucide/`. Iced's native SVG feature adds 28 supporting lockfile packages without upgrading existing versions. Icons are embedded, theme-tinted and available offline; the custom meter-placement marker remains separate. Pause is used only for the existing reference-loop pause command. About and release packaging retain upstream notices. No shadcn integration or audio-command changes were made.

`scripts/check.ps1 -Local` passed: **326 tests, zero failures, 18 existing opt-in checks ignored across 33 targets**, plus local-build/workflow/packaging/release-status fixtures, formatting and strict Clippy. A final strict all-target/all-feature Clippy rerun also passed after the foreground-tint simplification. `scripts/gui-dev.ps1` passed (1.33 seconds after test compilation, not a benchmark). No profile overrides. Canonical and preserved executables passed `--package-self-check` for offline inference, analysis, save/reopen, render and cancellation.

Native Windows checks covered wide 2560×1392 and compact 1028×930 rendering, Dark/Light tint, primary Play/disabled Stop contrast, open/closed Lock with persistent selected state, and both meter-placement cues. Dark was restored. The active reference-loop Pause state was not exercised with hardware playback; cross-platform, enlarged OS text and physical middle-drag acceptance were not repeated for an artwork-only change. No claim of release-package acceptance is made.

Executable, license, review-project input copy, self-check reports, logs, receipt and native screenshots are preserved in `C:/Users/User/Documents/Codex/Builds/CleanSlate-lucide-8935e14/`. Canonical/preserved executable SHA-256: `AE041BA5CF1027F5702325870C83C7C21C1DCDB5074D4ABDCA1807C3CC0892CC`. The preceding tested build remains in `CleanSlate-fabfilter-controls-a77a60b/`. Development stays at `91fa021`; main stays at `0b9622a`; no merge or publication occurred.

Code graph transport remains unavailable; focused source searches were the documented fallback. Context7 fetched Lucide static-asset conventions, Iced 0.14 drawing/SVG documentation and shadcn requirements before implementation.
