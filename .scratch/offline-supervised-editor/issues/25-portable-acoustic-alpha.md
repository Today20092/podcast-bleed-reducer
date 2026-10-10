# Package the acoustic-only Windows portable alpha

Status: claimed
Type: task
Assignee: Codex
Blocked by: none

## Scope

Owner confirmed the release interview on 2026-10-09 and instructed continuation: unsigned public `v0.1.0-alpha.1`, Windows 11 x64 portable ZIP, MIT application license, current reviewed GUI/documentation, local first build, clean offline Sandbox smoke check and host listening. Future version tags build GitHub draft prereleases, published manually. This is an explicitly unqualified alpha; tickets 14/15 remain open and their stable-release gates are unchanged.

Remove Nemotron from shipping code, GUI, package and required dependencies. Retain only a research note and Git history. Existing projects preserve sources, references, manual gain and protections; former Nemotron results become stale and require explicit acoustic reanalysis. Preserve acoustic source/cache/result validation, cancellation and safe exports.

## Acceptance

- Acoustic workflow, old-project compatibility, cancellation and export checks pass; no native runtime/model download or experimental UI remains.
- ZIP contains the release executable, actual necessary DLLs, MIT license, dependency/model notices, quick-start instructions and build identity/checksums. No recordings, project copies, caches or research/runtime binaries.
- Package smoke test detects missing/corrupt files; offline clean Windows launch/analysis/export verified, with limitations recorded honestly.
- Document release versions, local commands, candidate checks, clean test, GitHub tag/draft/manual publication and stable-release qualification.
- Review and commit intended source/docs only before tagging. Future workflow builds the same package and checks; no automatic publication.

## Results

Acoustic-only source, GUI and tests are implemented. Nemotron inference, acquisition,
helper executable, native build script and OpenMDW package notice are removed.
Legacy projects retain source bindings, labels, gain edits and protections; prior
results become stale, and new saves omit the retired flag. Existing acoustic
recipes also require one reanalysis after the pipeline identity change.

Checks on 2026-10-09: all-target tests 215 passed, 15 ignored; strict all-target
Clippy and formatting passed. scripts/gui-dev.ps1 -BuildOnly passed in 0.92s.
The optimized x64 build passed in 6m17s. Static CRT linking was rejected by the
prebuilt ONNX libraries; the portable package instead includes the licensed x64
Microsoft app-local runtime DLLs. PE imports were inspected and non-system CRT
imports are covered. Windows 11 system DirectML/D3D libraries remain system inputs.

The package contains executable, 8 CRT DLLs, MIT license, quick start, real Rust
dependency texts plus full Silero v5/ONNX1.22 notices and an inventory/identity
manifest. 292 resolved dependency records have license coverage, including 38
crate archives supplemented from recorded upstream license sources. ZIP extraction
and checker passed. Deliberately changed README bytes were rejected; restoring
them passed. Invalid version traversal was rejected before staging.

Clean Windows Sandbox launched with networking, vGPU and clipboard disabled,
candidate mapped read-only and a unique writable results folder. Its shipped
executable self-check reported PASS for model inference, acoustic analysis,
project save/reopen, render and cancellation; GUI startup was visually verified,
including New project and absence of native controls. Report:
`.scratch/release-checks/sandbox/f240e9ef3e114d93a70192170e712f3e/smoke.txt`.
Only synthetic test audio was used; interactive import/audition/export and host
listening have not been separately verified for this portable candidate.

Known qualification gap: synthetic unlabeled quiet overlap gives track-2
probability about 0.142. Preserved as an ignored ticket-14 gate and documented
in alpha instructions. Explicit overlap labels remain authoritative and pass.
Ordinary tests do not claim independent audio quality or 2-hour qualification.

Release process and tag-triggered GitHub draft-prerelease workflow are written.
Commit/tag/draft upload and remote CI verification remain pending. Ticket stays
claimed until candidate delivery and its remaining owner checks are recorded.
