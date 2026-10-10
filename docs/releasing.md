# Portable Windows releases

The first release is `v0.1.0-alpha.1`: an unsigned, experimental Windows 11 x64 ZIP.
Acoustic analysis runs offline with the embedded Silero model. Nemotron is removed.
Tickets 14 and 15 still govern listening, preservation and long-session qualification.
An alpha package does not establish that those gates passed.

## Prepare a candidate

1. Finish the scoped ticket and review the diff. Exclude recordings, projects,
   caches, raw screenshots and build artifacts. Update `Cargo.toml`, `Cargo.lock`
   and `docs/portable-readme.txt` to the same version. Use `0.1.0-alpha.2` next.
2. Run `cargo fmt --all -- --check`, `cargo clippy --all-targets --locked -- -D warnings`
   and `cargo test --all-targets --locked`. Ignored qualification tests must be
   accounted for separately; a green test command does not imply listening approval.
3. Commit the reviewed source and docs. Build from that commit:

   ```powershell
   pwsh -File scripts/package.ps1 -Version 0.1.0-alpha.1
   pwsh -File scripts/check-package.ps1 -Package dist/CleanSlate-0.1.0-alpha.1-windows-x64
   ```

The script builds with `scripts/release.toml` for Windows x64 and bundles Microsoft's
app-local C++ runtime DLLs from the licensed Visual Studio Build Tools installation,
collects dependency/model license texts, checks the exact file inventory and hashes,
and runs the shipped executable's offline analysis/save/reopen/render/cancellation
self-check. The ZIP, SHA-256 file and internal build manifest identify the candidate.
The manifest's dirty flag covers application, build and documentation inputs;
local `.scratch` test reports and ignored recordings are outside that scope.
Inspect PE imports after dependency changes: every non-system DLL must be included.
If a dependency adds a redistributable DLL, package and license it before publishing.

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
compiler, runs checks/tests, builds the same portable package and prepares a **draft
prerelease** with ZIP and checksum. It never publishes automatically. A rerun uploads
replacement assets to the existing draft; a published release is never overwritten.

Manual rebuilds accept an existing draft tag as `release_tag`. Select that tag's
ref when dispatching; branch dispatch is for diagnosing an unpublished candidate.
CI caches Rust dependencies and build outputs, including after a failed check,
and reports Windows application fault events if a test process crashes.
Playback preparation unit tests use a fixed output rate without sound hardware;
real device enumeration and listening still require a Windows GUI check.

Download and clean-test the actual GitHub ZIP before publication: its binary can
differ from the local build. In GitHub's Releases page, review the draft's version,
assets, checksum and limitations, keep **Set as a pre-release** selected, then publish.
Do not move a published tag or replace published assets; issue the next numbered alpha.

Release notes must state Windows 11 x64, unsigned ZIP, offline acoustic workflow and
any pending listening/performance checks. Before a stable `v0.1.0`, complete tickets
14/15 and update the workflow's prerelease policy deliberately.
