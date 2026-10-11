# Package future desktop releases and add direct downloads

Status: done
Type: task
Assignee: Codex
Blocked by: none

## Scope

2026-10-11 owner extension: publish the latest committed application as
`v0.1.0-alpha.2` with all four desktop packages and direct README downloads.
Release source starts at main commit `6c97145`; unrelated local edits are excluded.
The earlier four-platform build run 38028963031 succeeded after the Linux launcher
fix and an Intel DMG runner retry. This ticket remains claimed for publication.

Owner authorized macOS and Linux packages for future releases and a README download
table on 2026-10-10. Extend the existing acoustic-only portable packaging to Windows
x64, macOS Apple Silicon and Intel, and Linux x64. Keep draft/manual publication,
source validation, offline inference, license coverage and package integrity checks.
This ticket extends the existing packaging infrastructure independently of ticket
25's Windows listening qualification and stable quality gates 14/15.

## Acceptance

- Version tags build the four platform packages and checksums. A single release job
  uploads them only after every package job succeeds; published releases are never
  overwritten. Manual diagnostics can validate without creating a tag or release.
- macOS packages contain a launchable application bundle in a DMG. Linux packages
  contain an AppImage. Include application/dependency/model licenses and build
  identity. Run the shipped offline self-check and validate extracted packages.
- Existing Windows portable packaging retains its runtime DLLs and integrity checks.
- README lists each platform, installation steps, direct asset links for published
  packages and an honest availability status while releases remain drafts.
- Packaging regression checks cover invalid identity, missing/corrupt files and
  workflow/download naming. Record native-build and GUI limitations in the results.

## Results

Implemented the four native package targets, shared license/hash checks, extracted
archive checks, a single draft upload job, and a published-release README table
updater. Mac bundles are ad-hoc signed and explicitly not notarized. Linux uses a
pinned, checksum-verified linuxdeploy tool and includes system-library copyright
records. Existing stable qualification gates and manual publication are preserved.

Local checks on 2026-10-10: PowerShell syntax, package-integrity/download regression
checks, actionlint 1.7.12 and `git diff --check` passed. A rebuilt Windows release
ZIP passed staged and extracted offline analysis/save/reopen/render/cancellation
self-checks. Existing Windows package checks also passed. All 26 README local links
resolve; the updated platform badge returns SVG successfully. Live release discovery
correctly reports no public downloads while the existing alpha remains a draft.

Cross-target metadata found 373 resolved Linux dependencies with no missing license
texts and 317/318 Apple Silicon/Intel dependencies with 30 missing archive texts.
Their upstream MIT license/attribution records were added to the existing bundle.
Repeated metadata checks now report zero missing license texts for all three targets.

Native Mac/Linux hosted builds, extracted package checks and real desktop/audio
checks remain pending. Owner authorized commit, push and the hosted native checks
on 2026-10-10 after reviewing the local change. No version tag or release was
published or overwritten.
The pending note above is superseded by the following completion record.

2026-10-11: `v0.1.0-alpha.2` was built from tagged commit
`20f3b4d366df00be87f92ec1c485ec962b64ad82` and published with Windows x64 ZIP,
Apple Silicon and Intel Mac DMGs, Linux x64 AppImage, and four SHA-256 files.
[Native release run 38154565499](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/38154565499)
passed all four platform formatting, strict Clippy, test suites, staged and extracted
offline package self-checks, Mac signatures and the single draft assembly job.
All four uploaded apps were downloaded and independently matched to their checksums.
The owner explicitly authorized current-version public downloads in this session.
README release discovery is refreshed after publication. No older published release
was replaced. Real-device GUI/audio, listening and long-session qualification remain
separate gates; this is an explicitly experimental prerelease.
