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
Implementation and packaging fixes are committed and pushed. Tag
`v0.1.0-alpha.1` points to `fea493f`. The final committed package passed its
offline checker, and its ZIP (16,530,465 bytes) and SHA-256 file are uploaded to
GitHub draft prerelease 408491229. Publication remains manual as agreed.
Tag-triggered workflow 38013248811 is running its check/test step; remote CI
verification and the owner listening/interactive checks remain pending. Ticket
stays claimed until those remaining checks are recorded.

Remote workflow 38013248811 failed after formatting and Clippy passed: the first
library test process exited with Windows STATUS_ACCESS_VIOLATION (0xc0000005).
The same library tests pass locally with serial and parallel execution. A manual
workflow entry and serial, uncaptured test execution are being added to identify
the failing test on the hosted runner before considering the candidate verified.
Repository discovery topics were added; the alpha version tag already exists.

The serial hosted run 38014730596 reproduced the access violation in
`audition_stop_relink_and_error_do_not_start_output`. Local library stress with
64 test threads passed 20 attempts. Playback preparation was probing CPAL's
cached Windows device enumerator on short-lived workers, matching upstream
RustAudio/cpal issue 1302. Device-rate discovery now stays on the GUI thread;
unit tests use a fixed 48 kHz grid without hardware while exercising real
source preparation, caching and cancellation. Full local tests, strict Clippy,
formatting and the development GUI build passed after the change.
Temporary test probes are removed and CI parallel test execution is restored.
Hosted verification of the corrected code and real-device listening remain
pending. Release workflow diagnostics and build/native-runtime caching are added.

Hosted runs 38016775156 and 38017010305 passed all 82 ordinary library tests,
confirming the audio-device crash is resolved. Parallel integration tests exposed
timestamp-only fixture directory collisions: one run failed in the WAV parser
with unsigned subtraction overflow, another while saving a project with access
denied. Freezing the local fixture clock reproduced the parser failure; adding
a per-process counter made the same frozen-clock workload pass. Shared fixture
helpers now use counters and exclusive directory creation, matching the existing
reference-label fixture pattern. A regression creates parallel projects under a
frozen clock and verifies deleting one cannot invalidate the other source.
Full local validation with 64 test threads: 216 passed, 15 ignored. CI omits PDBs
from its build cache after observed cache uploads delayed diagnostics by nine
minutes. Corrected hosted integration/package verification remains pending.

Hosted run 38018772731 passed all 216 ordinary tests (15 ignored), formatting
and strict Clippy, then compiled the release executable. Packaging failed because
the windows-2025 runner now supplies Visual Studio 2026 while discovery accepted
only 2022. Runtime discovery now accepts both supported installations and locates
their VC143/VC145 CRT folder; the bundled redistribution notice covers both.
Local packaging and its offline self-check passed with the existing 2022 toolset.

Tagged run 38020644134 passed check/test, portable ZIP verification and draft
asset upload for source dbc7326a19d6540b83f000d32e884be6e41ca70d. The downloaded
GitHub ZIP also passed the offline package checker on the host, and its manifest
identifies that source commit. ZIP SHA-256:
`b2b404e54561c3f19154569f554d1e17c01e2e14ff5765649047074ed55ab41a`.
Release remains an unpublished prerelease draft. Clean Windows 11 testing of this
GitHub-built artifact and real-device listening remain pending before publication.
