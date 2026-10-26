# CleanSlate 1.1.0 release receipt

The owner explicitly authorized shipping the completed batch on 2026-10-23. Version 1.1.0 follows the compatible-capability rule: optional purchases (72) and signed Windows portable updates (73). Publication is pending final artifacts and the owner's local metadata signature.

## Source and validation

[PR 23](https://github.com/Today20092/podcast-bleed-reducer/pull/23) promoted tested development `79797ce46dae556843fafa4f7236135677348907` through merge `88dda154daa9f442018302e19f1c792513fd0874`. Both have source tree `8d4fee63213d29e211c53d739be2fa935accc56a`. The annotated `v1.1.0` tag identifies the merge; development was synchronized and pushed. Main remains the default branch.

[Required native source checks](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39981510311) passed all four platforms: Windows 370 tests; each Mac and Linux 365 tests; zero failures and 20 ignored per platform. Formatting, strict Clippy and script fixtures passed. Earlier attempts exposed two fixture portability issues: hidden `.DirIcon` lookup without PowerShell `-Force`, and linked macOS temporary ancestry. Corrected fixture roots preserve production symlink rejection. The final native run confirms both corrections.

The complete final local check passed 370 tests, zero failures and 20 ignored, plus formatting, strict Clippy and script fixtures. Standard GUI compilation and offline model inference, analysis, project save/reopen, rendering and cancellation self-check passed. Preserved development executable: `C:/Users/User/AppData/Local/CleanSlate/ReleaseChecks/1.1.0/CleanSlate-development-1.1.0.exe`, SHA-256 `5ACA340E79856ABB19E9D6D5F02D3A206A3E3D5B1E5A60E2BB60A69BBF7BE446`. It is a development build, separate from final release packages. Its JSON receipt and full local log are beside it.

The [feature delivery receipt](portable-updater-delivery-2026-10-23.md) records nine disposable Windows installer cases, including success/cleanup, failed self-check, startup crash/timeout, interrupted replacement/recovery, mixed folder, corrupted archive, a real file lock and real ACL denial. Those synthetic fixture packages are not release artifacts.

## Final artifact verification

[Tagged package run 39983829792](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39983829792) is the sole final artifact source. Native packaging, downloaded artifact checks, Windows offline self-check and signature verification are pending. Checked downloads will be retained under `C:/Users/User/AppData/Local/CleanSlate/ReleaseChecks/1.1.0/`.

2026-10-26 follow-up: connected GitHub now confirms all four native package jobs and draft preparation succeeded, with four unexpired containers at the recorded source. Actual final download/inner-package/descriptor verification remains pending: the Windows connector returned a file reference, but its guarded local streaming request failed HTTP 403 before any archive was created. See the [artifact access checkpoint](stable1.1-artifact-access-2026-10-26.md) for exact evidence and retained guard receipt. No final package self-check, private signature or publication was performed by that attempt.

## Signing and compatibility

The pinned release public key is `RWQMaOa5uWhvVOJCDE5I0zi1ybM7aIPZW9D8FZpNyOb7aA6ibb9VsJ0c`. The owner holds its encrypted private key outside the checkout and CI and enters the password privately. The exact checked Windows descriptor must be signed and independently verified before publication. No private-key bytes or password enter the repository or release workflow.

Existing 1.0.0 needs a manual complete-package replacement to bootstrap the updater. Supported clean Windows x64 NTFS installations can use future signed whole-package updates; modified or mixed installations, unsupported storage and Mac/Linux receive manual guidance. Source media and personal files belong outside the application folder. Project and preference formats remain compatible; audio processing stays offline.

## Acceptance limits

The owner previously accepted the desktop appearance and waived further computer control. No new interactive desktop matrix, subjective listening, physical sound-device qualification, Mac/Linux physical desktop acceptance or independent long-session benchmark is claimed. Native CI/self-check evidence is distinct from those checks; ignored tests are not passing evidence. Windows executables remain without Authenticode signing; macOS uses ad-hoc signing without notarization. Minisign authenticates updater metadata and does not change OS code-signing status. Unrelated research 67/70 remains outside the shipped batch.
