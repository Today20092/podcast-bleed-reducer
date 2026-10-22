# CleanSlate 1.0.0 stable release receipt

[v1.0.0](https://github.com/Today20092/podcast-bleed-reducer/releases/tag/v1.0.0) was published on explicit owner authorization at 2026-10-22 17:28:36 UTC. It is public, marked stable (not prerelease), and selected as GitHub's latest release. Four native packages and four adjacent SHA-256 files are attached.

## Source and checks

Annotated tag `v1.0.0` identifies main merge `68198b3703a6fb4ec0aef53b9712acf44a87cacd`, promoted through [PR 21](https://github.com/Today20092/podcast-bleed-reducer/pull/21). Development was synchronized with that merge. The tested development parent `866134114cd1fe75b8ded62fb3e0fae32a6abf7b` and the main merge have identical tracked source tree `3f5dc8a999a4b33f997ab30e06a19128e19e5575`.

[Required source CI](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39816464249) passed all four platforms: Windows 356 tests; each Mac and Linux target 352 tests. Each had zero failures, 20 ignored tests and 38 targets, with formatting, strict Clippy and script fixtures. The local shared check also passed 356 tests with 20 ignored. The first local attempt failed from Windows memory exhaustion before Rust validation; the unchanged retry completed successfully.

[Tagged package workflow](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39819315222) passed all four native package/check jobs and draft preparation. The normal release-readiness report confirmed `packagesReady` and `releaseReady`. Stable draft metadata used the 1.0.0 notes and cleared the prerelease flag.

## Artifact verification

All eight downloaded draft files matched their checked Actions artifacts byte-for-byte by SHA-256. All four archive checksums matched their adjacent checksum files. The extracted final Windows ZIP passed `scripts/check-package.ps1`, including its file inventory/hashes and offline inference, analysis, project save/reopen, rendering and cancellation self-check. Publication did not replace any older tag or asset.

| Package | SHA-256 |
| --- | --- |
| CleanSlate-1.0.0-windows-x64.zip | `C42C99ED2DF88F35EF698E2EA7D89BB0F824373DF8B3B1BA0AE20673F130F748` |
| CleanSlate-1.0.0-macos-arm64.dmg | `8AE2596B6459E1590DC5D418AF9C0E434934F0E4A677BA323E99413D8F82CBAA` |
| CleanSlate-1.0.0-macos-x64.dmg | `B3344A5A66882B9BEFA670C9FE3E23B99D8FA4FD67DCC9FD322B8B2222A213DB` |
| CleanSlate-1.0.0-linux-x64.AppImage | `511BB501BE97D8F8347AA4522EF356729CF8989389C42255B1371E1F02E8524E` |

Local logs, downloaded artifacts and extracted Windows package are preserved in `C:/Users/User/Documents/Codex/Builds/CleanSlate-1.0.0-verification/`. The separately preserved development executable `CleanSlate-1.0.0.exe` has product version 1.0.0 and hash `C4783F6AA9379E397B961016BD8C736DEACE21247A62E8B6C2FC17E69ED51B9E`; its final standard GUI build and self-check passed. It is not the packaged release executable.

## Standards

No hard violations were found. One minor draft-step naming inconsistency was corrected, including fixture extraction, current documentation and historical report compatibility; follow-up review confirmed resolution.

## Spec

One P2 outdated alpha-only publication instruction was corrected. Stable tagging, draft flags, latest selection and subsequent SemVer guidance now agree. Follow-up review found no remaining issue in the reviewed scope.

## Acceptance limits

The owner accepted the development appearance, explicitly requested no further computer control, then authorized stable publication and the checked README download merge. No new native screenshot matrix, clean-desktop GUI session, physical Mac/Linux audio qualification, subjective listening study or independent whole-session benchmark is claimed. Existing signing and analysis limits remain in the release notes. Ignored tests are not passing evidence. Research 70 and proposed features 72/73 are excluded from this release.

The publication-triggered README workflow derives downloads from actual public assets. [PR 22](https://github.com/Today20092/podcast-bleed-reducer/pull/22) combines its generated table with the stable publication record. Its merge is gated by required checks; release publication and README promotion are separate operations.
