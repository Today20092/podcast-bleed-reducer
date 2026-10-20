# Publish the current desktop application

Status: done
Type: task
Assignee: Codex
Blocked by: none

Owner requested on 2026-10-20 that the public GitHub download include the approved logo and the newer keyboard-shortcut/editor workflow. Published alpha.2 predates both. Draft alpha.5 includes the editor workflow but predates the application icons merged in PR 12.

## Scope

- Prepare alpha.6 from current main, retaining the implemented editor workflow and application icons.
- Build and check all four native packages through the existing release workflow.
- Verify the downloaded Windows executable's embedded icon, package integrity and offline self-check.
- Publish the experimental prerelease and update README downloads through the protected PR workflow.
- Identify the existing local development executable for the owner; preserve unrelated checkout changes.

## Acceptance

- Cargo manifest/lock version and release notes identify alpha.6.
- Four platform checks and package jobs pass; all four downloads have checksums.
- Downloaded Windows executable has the approved icon and passes the package self-check.
- Published release and README Windows link resolve to alpha.6.
- Record unchecked native desktop/audio/listening qualification honestly.

## Results

- Existing local GUI executable: `C:/Users/User/.codex/worktrees/offline-timebase/podcast-automix-rust-gui/target/owner-ui/debug/rust-app.exe`, built 2026-10-17. Windows extracted its 32px icon with 274 mint pixels.
- Current main contains the editor workflow through alpha.5 and PR 12 application branding. Configurable shortcut remapping remains planned in ticket 41.
- Local native-workflow, protected-branch download automation, package integrity/download, and release-status fixture checks passed; Cargo formatting and diff whitespace checks passed.
- [PR 14](https://github.com/Today20092/podcast-bleed-reducer/pull/14) passed all four required native checks and merged into main. Its Windows check initially failed on an ONNX Runtime CDN connection reset; the unchanged retry passed.
- [The tagged release workflow](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39447930397) passed all four native test/Clippy/formatting and package checks. Immutable tag `v0.1.0-alpha.6` identifies `9b2fa9dd8886aed07cc57d4d105ca9fb1bf2fd65`, which is an ancestor of main; package manifests are clean.
- [Published alpha.6](https://github.com/Today20092/podcast-bleed-reducer/releases/tag/v0.1.0-alpha.6) includes all four packages and checksums. All eight public asset URLs returned HTTP 200.
- Downloaded the Windows workflow artifact and the actual release ZIP; both SHA-256 values are `09661f0eb78ad52f1d9e868590124da81bf22e9c39c06dd47a119d7fddec5940`. Extracted package inventory and offline self-check passed on the owner machine. Its 32px executable icon exactly matches the approved ICO (0/1024 differing pixels); embedded window RGBA, shortcut-reference text and compiled alpha.6 version are present.
- New local executable: `C:/Users/User/Downloads/CleanSlate-0.1.0-alpha.6-windows-x64/CleanSlate.exe`. The old alpha.2 download and running development build remain intact.
- [PR 15](https://github.com/Today20092/podcast-bleed-reducer/pull/15) refreshes every README platform link from the published assets and records verification. The duplicate release-event download PR 16 was closed and its branch removed.
- Removed merged completed local/remote branches, including the squash-merged application-icon branch after confirming its tree exactly matched PR 12. Detached clean completed checkouts retain their files. Preserved dirty checkouts, the active release-evaluation branch, frozen baseline, development executable/cache and all unrelated primary-checkout changes. Updated local main to the merged release source.
- Native Mac/Linux desktop/audio inspection, physical device recovery, independent listening and long-session qualification remain pending. This remains an experimental prerelease; customizable shortcut bindings remain ticket 41.
