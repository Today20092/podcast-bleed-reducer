# Desktop releases

The first release is `v0.1.0-alpha.1`: an unsigned, experimental Windows 11 x64 ZIP.
Acoustic analysis runs offline with the embedded Silero model. Nemotron is removed.
Tickets 14 and 15 still govern listening, preservation and long-session qualification.
An alpha package does not establish that those gates passed. Future version tags
also build macOS Apple Silicon and Intel DMGs and a Linux x64 AppImage. Native
package checks are separate from clean desktop/audio testing and listening approval.

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

## Prepare a candidate

1. Finish the scoped ticket and review the diff. Exclude recordings, projects,
   caches, raw screenshots and build artifacts. Update `Cargo.toml`, `Cargo.lock`
   to the same version. Use `0.1.0-alpha.2` next. The packaged README substitutes
   version and platform from the build; do not hard-code a version in that template.
2. Run `cargo fmt --all -- --check`, `cargo clippy --all-targets --locked -- -D warnings`
   and `cargo test --all-targets --locked`. Ignored qualification tests must be
   accounted for separately; a green test command does not imply listening approval.
3. Commit the reviewed source and docs. Build from that commit:

   ```powershell
   pwsh -File scripts/package.ps1 -Version 0.1.0-alpha.1
   pwsh -File scripts/check-package.ps1 -Package dist/CleanSlate-0.1.0-alpha.1-windows-x64
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

After local checks and review, push the candidate branch and the version tag:

```powershell
git push origin codex/offline-supervised
git tag -a v0.1.0-alpha.1 -m "CleanSlate 0.1.0-alpha.1"
git push origin v0.1.0-alpha.1
```

`.github/workflows/portable-release.yml` runs on version tags. It installs the pinned
compiler, runs checks/tests and packages on Windows x64, both Mac architectures and
Linux x64. A single upload job runs only after all four succeed, then prepares a
**draft prerelease** with all four downloads and checksums. It never publishes
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
assets, checksum and limitations, keep **Set as a pre-release** selected, then publish.
Do not move a published tag or replace published assets; issue the next numbered alpha.

Test each Mac DMG on its matching architecture without build tools or caches. Open
the app through Finder, import test audio, Analyze, save/reopen, audition and export.
Test the Linux AppImage on a clean Ubuntu 22.04 desktop and a newer desktop, including
the normal executable launch and `--appimage-extract-and-run` fallback. Headless
CI self-checks verify analysis and delivery, not real sound-device playback.

Release notes must state the attached platforms, signing status, offline acoustic
workflow and any pending desktop/listening/performance checks. Before a stable `v0.1.0`, complete tickets
14/15 and update the workflow's prerelease policy deliberately.

## Keep README downloads current

`.github/workflows/release-downloads.yml` updates only the README's marked download
table when a release is published, edited or deleted. It queries actual published
assets, including prereleases, and selects the newest available package and checksum
for each platform. Drafts and missing assets never produce download links. This
avoids GitHub's `latest/download` shortcut, which does not select alpha prereleases.
The workflow commits that table to the default branch. It does not publish releases.

Use `pwsh -File scripts/update-downloads.ps1` to refresh the same table locally,
or dispatch **Update published download links** after changing release assets.
