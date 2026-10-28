# Ship stable 1.1.0

Status: in progress
Type: release
Assignee: Codex
Blocked by: none (72 and 73 done)
Related: 68, 71, 72, 73

## Owner request — 2026-10-23

“Okay, go ahead and push that as an update. ship” authorizes preparation, checked development-to-main promotion, tagging and publication of the completed optional-purchase and signed-updater batch. Select 1.1.0 for compatible new functionality. Preserve unrelated research and existing published assets.

## Acceptance

- [x] Align version metadata, release notes and package defaults; run local checks.
- [x] Promote through a passing four-platform PR with a merge commit and tag reviewed main.
- [ ] Verify final native package artifacts, checksums and Windows package self-check; disclose unavailable physical checks.
- [ ] Have the owner sign the exact checked Windows descriptor privately; verify the pinned key and publish all verified assets.
- [ ] Merge the checked generated download update, synchronize development and record source/package identities.

The encrypted private key and its password stay outside the repository and CI. The owner enters the password only in their own terminal. No audio algorithm or unrelated research implementation is included.

## Native CI correction

The first promotion run exposed a fixture-only hidden-file lookup: the Linux `.DirIcon` inventory used `Get-Item` without `-Force`, which fails on Unix hosts. Add `-Force` to match the production inventory/checker; the existing four-platform fixture directly exercises the correction. No package content or editor behavior changes.

The next Mac run reached Rust tests and rejected transaction fixtures beneath the host's linked temporary-directory ancestry. Canonicalize fixture temporary roots before exercising package validation, preparation and cleanup; keep production rejection of linked installation ancestors unchanged. Assert the intact cleanup fixture validates before mutation, and surface validation errors directly. Native reruns must confirm this diagnosis.

## Promotion results

Local checks after both fixture corrections: 370 passed, zero failed, 20 ignored; formatting, strict Clippy and script fixtures pass. Standard GUI build and offline self-check pass. Preserved development binary and receipt: `%LOCALAPPDATA%/CleanSlate/ReleaseChecks/1.1.0/`.

[PR 23](https://github.com/Today20092/podcast-bleed-reducer/pull/23) passed [all four native checks](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39981510311): Windows 370 tests; each Mac and Linux 365, with zero failures and 20 ignored per platform. This confirms the Mac fixture correction. Merge `88dda154daa9f442018302e19f1c792513fd0874` and tested development `79797ce46dae556843fafa4f7236135677348907` have the same source tree `8d4fee63213d29e211c53d739be2fa935accc56a`. Annotated `v1.1.0` identifies that reviewed merge. Development is synchronized and pushed. [Tagged package run 39983829792](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39983829792) is the final artifact source; publication remains pending package verification and the owner's private local signing step.

## Bounded artifact-access follow-up — 2026-10-26

[Access checkpoint](../../../docs/research/stable1.1-artifact-access-2026-10-26.md): refreshed connected GitHub metadata confirms the same four unexpired artifact containers/source/digests. The Windows download connector returned a reusable file reference, but one locally guarded streaming request received HTTP 403 before any container file was created. Stopped transfer attempts at that access barrier, without retries/credential changes or other-platform downloads. No container/inner checksum, archive, manifest, descriptor or final Windows self-check acceptance passed through this attempt; all incomplete checkboxes remain unchanged and this ticket stays in progress.

Guard receipt: `%LOCALAPPDATA%/CleanSlate/ReleaseChecks/1.1.0/run39983829792-audit74/windows-transfer-guard.json`; `command-failed`, exit 1, `cleanup_verified: true`, 1.15 seconds, kernel peak job commit 35,172,352 bytes under an explicit 0.5 GiB cap / 2 GiB start / 1 GiB reserve / 300-second timeout. No signing, private material, publication, tag/push, link changes or cleanup. Once official artifact delivery is available and provenance/integrity pass, the Windows CLI self-check uses 88's separate bounded lightweight budget; builds/native GUI/listening retain their existing gates.
