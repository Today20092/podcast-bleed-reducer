# Package future desktop releases and add direct downloads

Status: claimed
Type: task
Assignee: Codex
Blocked by: none

## Scope

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
Ticket remains claimed until native packaging acceptance checks pass; real-device
listening and long-session qualification remain separate release gates.
