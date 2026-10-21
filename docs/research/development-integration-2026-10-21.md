# Reviewed GUI and performance batch integrated into development

The owner authorized merging the complete reviewed batch into development for desktop testing. Main/release promotion remains a later owner decision.

- Previous development: `91fa021efe84450e332f598bc8358ff4bfbda204`, verified equal to origin/development before integration.
- Reviewed performance delivery: `c3c2ff53de1bd876a08fd10776dd71b717cac1ed`; integration claim commit `c147149` adds only ticket09.
- Development merge: `1235ca158f706f426e425421d29b4fc1d6cd3292`, a non-squash merge retaining all reviewed commits. Its tree exactly matches the reviewed branch, including the earlier rotary-control and Lucide-outline refinements.
- Main remains `0b9622a88643093fc4496cd4729be1d441b3a27c`. No version bump, tag, main merge, release PR, package-workflow dispatch or publication is part of this integration.

## Checks and build

`scripts/check.ps1 -Local` passed on development: **339 passed, 0 failed, 20 ignored**, with strict all-target/all-feature Clippy, formatting and local-build/workflow/packaging/release-status fixtures. Test compilation took 4m21s; this is a build observation, not an application performance measurement.

`scripts/gui-dev.ps1 -BuildOnly` passed using the shared pinned development settings, taking 45.79 seconds. Both canonical and preserved executables passed `--package-self-check`: offline inference, analysis, project save/reopen, render and cancellation.

- Canonical executable: `C:/Users/User/AppData/Local/CleanSlate/BuildCache/debug/rust-app.exe`.
- Preserved executable: `C:/Users/User/Documents/Codex/Builds/CleanSlate/development-1235ca1/CleanSlate.exe`.
- Matching SHA-256: `8c5dc2f673d5ff22fcba9d815409d316cfe38a94260aa4ed4877ce04cbc17862`.
- `Open-GUI.ps1` beside the preserved executable opens the owned three-track `Review.cleanslate` input, referencing original audio in place. The app creates its local working project; original recordings/projects are not replaced.

The preserved development executable was launched and its native window displayed the three tracks, 18 saved examples and the integrated outline controls. The session initially requests Analyze, as expected for the saved input. The owner stopped Computer Use with physical Escape before further GUI interaction; desktop automation stopped immediately. This records launch verification, not owner acceptance or a new full native performance run.

## Remaining review

The owner will test this development build before deciding whether to promote it to main or create a release. Existing [performance evidence and limitations](fabfilter-performance-final-review-2026-10-21.md) remain applicable: physical DPI/latency/listening quality and other platforms are not newly qualified; the widest geometry probe still exceeds its inherited budget. Development is backed up through the normal source-branch push, without a release tag or publication.
