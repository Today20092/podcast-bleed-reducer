# Desktop releases

The current stable release is [v1.0.0](https://github.com/Today20092/podcast-bleed-reducer/releases/tag/v1.0.0), promoted through [PR 21](https://github.com/Today20092/podcast-bleed-reducer/pull/21) and published on owner authorization. The [1.0.0 notes](release-notes.md) describe the workflow and compatibility promise. Follow [Versioning and stability](#versioning-and-stability) for subsequent versions.

The preceding published prerelease was [v0.1.0-alpha.8](https://github.com/Today20092/podcast-bleed-reducer/releases/tag/v0.1.0-alpha.8), following alpha.6.
Alpha.7 stopped at the dependency-license packaging gate and was not published.
Alpha.8 tag source `24361a2aa201fb7e22258f44a2a1be4626d58ce1` was promoted through
[PR 18](https://github.com/Today20092/podcast-bleed-reducer/pull/18).
[Source CI](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39741751147)
and [all four native packages plus draft preparation](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39742979614)
passed. The draft's four packages and four checksum files were downloaded and
matched against the tested Actions artifacts before publication. Windows Sandbox
interactive export produced three 600-second, 48 kHz mono 32-bit float WAVs with
independently inspected RIFF headers. Classification save/reopen retained 18 references;
the actual host package analyzed and advanced in all three listening modes without
device errors. Ticket 64 is complete. See the
[release receipt](research/alpha8-release-receipt-2026-10-22.md) for identities and
native acceptance limits. These functional checks are not a new subjective sound-quality
claim or fresh Mac/Linux hardware qualification.
The owner authorized promotion and release in ticket 64. Use the current
[branch workflow and per-release checklist](branch-workflow.md#release-checklist).
Ticket 14 is owner-closed; retired 15/25 now route to that repeatable checklist.
Acoustic analysis runs offline with the embedded Silero model. All four native
package targets are built by the release workflow. Native package checks remain
separate from clean desktop/audio testing and physical listening approval.

| Build system | Package | Target |
| --- | --- | --- |
| Windows x64 | `CleanSlate-VERSION-windows-x64.zip` | Windows 11 x64 |
| Mac Apple Silicon | `CleanSlate-VERSION-macos-arm64.dmg` | macOS 13+ |
| Mac Intel | `CleanSlate-VERSION-macos-x64.dmg` | macOS 13+ |
| Ubuntu 22.04 x64 | `CleanSlate-VERSION-linux-x64.AppImage` | glibc 2.35+, desktop and audio system |

Each download has an adjacent `.sha256` file. Replace `VERSION` with the Cargo
version without `v`. Windows needs the bundled DLLs; Mac bundles its support files
inside `CleanSlate.app`; Linux bundles them inside the AppImage. Analysis needs no
runtime model download. Windows is unsigned; Mac is ad-hoc signed, not notarized.

## Versioning and stability

On 2026-10-22 the owner accepted **1.0.0 as the next release target and first stable release** for the existing offline podcast workflow. UI refinement can continue after 1.0; another feature milestone or beta period is not required. This decision supersedes earlier instructions to keep future releases experimental. The published alpha.8 packages retain their original version and status until a new release is prepared, verified and published. This policy decision authorizes documentation; promotion, tagging and publication require release-specific owner authorization.

Stable means we recommend and maintain the documented workflow: import synchronized microphone tracks, provide speaker examples, analyze, audition and edit gain, save the project, and export processed audio. It does not promise perfect detection on every recording. Keep listening review, aligned-input requirements, known sound limitations, signing status and platform-specific testing evidence visible. Package CI success and hands-on desktop/audio qualification are separate evidence. Formal independent comparisons and whole-session benchmarks remain unestablished; the owner-closed evaluation project is not reopened by choosing 1.0.

### Choose a version

Use [Semantic Versioning 2.0.0](https://semver.org/). For this desktop application, the public compatibility contract is the documented workflow and supported inputs/exports, existing saved projects, speaker examples, manual gain edits and persisted preferences. Preserve these across compatible updates; migrate stored data safely when its representation changes. Internal format or implementation changes alone do not require a major version when compatibility is preserved. Compatibility does not promise identical reanalysis results after a documented algorithm fix or improvement; preserve saved human edits and explain consequential sound changes.

| Change | Version after 1.0.0 | Selection rule |
| --- | --- | --- |
| Compatible bug fix or small UI refinement | `1.0.1` | Existing functionality and saved data remain usable. |
| Compatible new functionality | `1.1.0` | New capabilities preserve the compatibility contract. Reset patch to zero. |
| Incompatible change | `2.0.0` | A deliberate break to the contract requires a major version, migration/recovery guidance and release notes. Reset minor and patch to zero. |

Choose the highest required increment in the batch. Documentation-only changes can accompany the next release without creating an application release. Stable versions have no suffix. Use `-alpha.N` for an incomplete experimental candidate, `-beta.N` for a workflow ready for broader testing, and `-rc.N` for a stable-release candidate awaiting final verification. These labels describe the candidate's readiness; they are not mandatory stages. Stable does not mean development stops or that future polish requires a major version.

### Prepare the stable transition

Complete these steps under the release-specific ticket before publishing 1.0:

1. Select the reviewed source and set Cargo.toml, Cargo.lock and candidate packaging inputs to the same unused version. Keep existing published tags and assets intact. Completion: source metadata and package identities agree.
2. Update current README/status/help/package wording and release notes to describe the stable workflow and concrete limitations. Preserve historical alpha receipts and notes. Completion: current release copy agrees on version/status and platform claims match recorded evidence.
3. Review the release automation against the intended status. `portable-release.yml` defaults to v1.0.0 and keeps every release a draft: tags with a prerelease suffix use the prerelease flag and historical alpha notes; plain stable versions clear that flag and use `docs/release-notes.md`. A draft rerun refreshes its metadata before uploading checked assets; a published release cannot be replaced. Completion: the draft has the intended assets and release notes, its prerelease flag matches the selected status, and a stable release is selected as the latest release when published. See [GitHub release settings](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository).
4. Complete the [per-release checklist](branch-workflow.md#release-checklist), record actual results and omissions, and obtain the owner's authorization for promotion/tagging/publication. Completion: the ticket identifies the tested source and final packages, records support limits and authorization, and verifies public downloads after publication.

The existing development → main merge promotion and four-platform draft/package verification process remains in place. A version-policy decision alone does not change the published downloads or establish new platform qualification.

## Prepare a candidate

1. Finish the scoped ticket and review the diff. Exclude recordings, projects,
   caches, private captures and build artifacts. Keep approved documentation screenshots.
   Update `Cargo.toml` and `Cargo.lock` to the same unused version selected using
   the policy above. The packaged README substitutes
   version and platform from the build; do not hard-code a version in that template.
2. Run `./scripts/check.ps1 -Local` for the shared Windows cache/profile, or
   `./scripts/check.ps1` in hosted/native packaging environments. Ignored tests must be
   accounted for separately; a green test command does not imply listening approval.
3. Commit the reviewed source and docs. Build from that commit:

   ```powershell
   pwsh -File scripts/package.ps1 -Version 1.0.0
   pwsh -File scripts/check-package.ps1 -Package dist/CleanSlate-1.0.0-windows-x64
   ```

Run the same `pwsh -File scripts/package.ps1 -Version VERSION` command natively on
each target system. On Mac, set `MACOSX_DEPLOYMENT_TARGET=13.0` for all builds/tests.
On Ubuntu 22.04 install `libasound2-dev libdbus-1-dev libssl-dev pkg-config patchelf
desktop-file-utils libfuse2`. Install PowerShell 7 and the pinned Rust 1.99.0 compiler
on every build system. Cross-compiling is not a substitute for the native checks.

The script builds with `scripts/release.toml`. Windows packaging bundles Microsoft's
app-local C++ runtime DLLs from the licensed Visual Studio Build Tools installation,
collects dependency/model license texts, checks the exact file inventory and hashes,
and runs the shipped executable's offline analysis/save/reopen/render/cancellation
self-check. The download, SHA-256 file and internal build manifest identify the candidate.
The manifest's dirty flag covers application, build and documentation inputs;
local `.scratch` test reports and ignored recordings are outside that scope.
Inspect PE imports after dependency changes: every non-system DLL must be included.
If a dependency adds a redistributable DLL, package and license it before publishing.

Mac packaging writes `Info.plist`, rejects unresolved non-system dylibs, ad-hoc
signs the `.app`, builds a DMG, then mounts it read-only and checks that exact package.
It does not use an Apple Developer ID or notarization. Linux packaging downloads a
pinned, SHA-256-verified linuxdeploy tool, bundles linked libraries and their Ubuntu
copyright records, and creates an AppImage. Its extracted file inventory and the
actual AppImage offline self-check must both pass. The desktop, audio server and
graphics drivers remain system requirements. Revisit the Ubuntu 22.04 runner before
its announced April 2027 retirement.

`pwsh -File scripts/test-packaging.ps1` exercises package corruption, missing and
unexpected files, traversal, duplicate paths and download-link generation.
`check-package.ps1 -IntegrityOnly` checks files/hashes without starting an executable,
including packages for another OS. It does not replace the default native self-check.

## Test the extracted ZIP

Check the published ZIP's SHA-256, extract it into a fresh writable folder, and run
`scripts/check-package.ps1` against that extracted folder.

Use Windows Sandbox with networking disabled and only the candidate folder mapped
read-only. Map a separate empty results folder writable for the self-check report.
`pwsh -File scripts/sandbox-smoke.ps1 -Package <extracted-folder>` creates this
configuration and launches the self-check followed by the GUI. Inspect the report
in the newly created `.scratch/release-checks/sandbox/<run>/smoke.txt`; a launched
window alone is not a pass. Check `failure.txt` if the executable fails to start.
Run `CleanSlate.exe --package-self-check --report <results-path>` and require PASS.
Open the GUI, import synthetic/public test audio, label room tone and solo speech,
Analyze, save/reopen, audition and export. No Rust, Python, developer caches or model
downloads should be needed. If Sandbox is unavailable, record that check as pending
and use another clean Windows 11 x64 machine before publishing.

Then audition representative original/processed/removed audio on the host, including
quiet wanted speech and overlap. Record the candidate hash, OS, observations and
remaining limitations in the release ticket. The automated synthetic smoke test
verifies packaging mechanics; it does not establish real-recording audio quality.

## Build drafts on GitHub

After local checks and review, push development, open a development-to-main PR,
wait for all four required native CI checks and merge with a merge commit. Fetch
the reviewed main commit and tag that commit explicitly. Check that the new tag
does not already exist before creating it:

```powershell
git fetch origin
git tag -a v1.0.0 origin/main -m "CleanSlate 1.0.0"
git push origin v1.0.0
```

`.github/workflows/portable-release.yml` runs on version tags. It installs the pinned
compiler, runs checks/tests and packages on Windows x64, both Mac architectures and
Linux x64. A single upload job runs only after all four succeed, then prepares a
**draft release** with all four downloads and checksums. Plain stable versions use
stable release notes and clear the prerelease flag; suffixed versions remain
prereleases with alpha notes. It never publishes
automatically. A rerun uploads
replacement assets to the existing draft; a published release is never overwritten.

Manual dispatch accepts `release_tag`. With **build_only** checked, it builds the
selected branch and saves workflow artifacts without creating/updating a release.
This is the default diagnostic mode. Uncheck it to rebuild an existing draft tag;
the workflow checks out that tag, so its build identity matches the release.
CI caches Rust dependencies and build outputs, including after a failed check,
and reports Windows application fault events if a test process crashes.
Playback preparation unit tests use a fixed output rate without sound hardware;
real device enumeration and listening still require a Windows GUI check.

Download and clean-test the actual GitHub packages before publication: their binaries can
differ from the local build. In GitHub's Releases page, review the draft's version,
assets, checksum and limitations. For stable 1.0.0, clear **Set as a pre-release**
and select it as the latest release; retain the prerelease flag for a suffixed
candidate. Publish only after package verification and owner authorization.
Do not move a published tag or replace published assets; select the next version
under the semantic-versioning policy.

Test each Mac DMG on its matching architecture without build tools or caches. Open
the app through Finder, import test audio, Analyze, save/reopen, audition and export.
Test the Linux AppImage on a clean Ubuntu 22.04 desktop and a newer desktop, including
the normal executable launch and `--appimage-extract-and-run` fallback. Headless
CI self-checks verify analysis and delivery, not real sound-device playback.

Release notes must state the attached platforms, signing status, offline acoustic
workflow and any pending desktop/listening/performance checks. For stable 1.0.0,
complete the versioning policy's transition and per-release checks, retain known
limits and identify actual package/hardware evidence without treating omitted
qualification as passing.
Owner-closed 14 and retired 15/25 are historical evidence, not tickets to reopen.

## Keep README downloads current

`.github/workflows/release-downloads.yml` updates only the README's marked download
table when a release is published, edited or deleted. It queries actual published
assets, including prereleases, and selects the newest available package and checksum
for each platform. Drafts and missing assets never produce download links. This
avoids GitHub's `latest/download` shortcut, which does not select alpha prereleases.
The workflow proposes that table through a PR to the default branch and explicitly
dispatches native checks. Merge the checked PR and synchronize main back to
development. It does not publish releases.

Use `pwsh -File scripts/update-downloads.ps1` to refresh the same table locally,
or dispatch **Update published download links** after changing release assets.
