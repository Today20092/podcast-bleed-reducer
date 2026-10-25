# Usable supervised plugin calibration and gain prototype

Status: claimed
Assignee: Codex
Type: prototype
Blocked by: none
Related: 67, 76, 77

## Owner authorization — 2026-10-24

Finish as far as possible autonomously until a working plugin needs owner host/listening help. Native VST3 sharing/export in 76 and autonomous checks in 77 establish this bounded prerequisite; 76's CLAP export failure stays open. No routine DAW launch or UI control. Preserve prior builds, user projects and audio. No release publication.

## Scoped first listening build

- Reuse the maintained desktop logistic trainer/portable model in a small shared crate, preserving the desktop numerical recipe and validation. Add allocation-free compiled inference for audio callbacks.
- Give the plugin its own explicitly versioned fixed-window streaming feature recipe. This first two-microphone prototype is not desktop detector parity: full-recording VAD/transfer features are not silently claimed. Record the feature/time-context limitation beside code and here; replace after equivalent streaming measurements and preservation tests qualify.
- Capture trusted room tone, solo member 1, solo member 2 and optional overlap while using host playback. Bound and deduplicate training data; fit outside callbacks; reject missing classes, invalid data, source/rate/layout conflicts, obsolete results and inadequate training. Preserve originals until valid calibration is deliberately enabled.
- Provide an actual editor in both formats: group/member assignment, capture/learn/clear, saved-calibration confirmation, processing/reduction/fades and Original/Processed/Removed monitoring. Keep diagnostics secondary and no duplicate DAW waveform UI.
- Separate calibration from transport generations. Persist validated calibration through host state with explicit confirmation on reload, and invalidate changes that cannot safely reuse it. No audio recording or automatic identity claim.
- Deliver bounded delayed attenuation with host-reported latency, finite S-shaped fades, exact unity for explicit speech truth, conservative missing/invalid/late-evidence fallback and Original/Processed/Removed parity. No boosting/source separation. Explicitly keep native CLAP export unsupported for attenuation until its timing issue qualifies; VST3 is the listening target.
- Verify independently generated wanted/bleed/room/overlap cases, state round-trip, resets, cancellations and actual format processing headlessly. Preserve candidate first; publish only checked files through existing links. Record software, memory and remaining physical/host/listening limits.

## Acceptance

- [x] Shared model extraction preserves root tests/numerics and validates compiled prediction parity.
- [x] Capture, training, persistence and invalidation have meaningful synthetic tests.
- [x] Delayed gain and monitoring parity pass preservation, missing-data, seek and tail checks.
- [x] CLAP/VST3 actual hosts and editor construction pass automated checks with bounded memory.
- [ ] Native editor has observable controls/status and documented owner test; no host automation required.
- [x] Required local checks pass; source/artifact identity and old-build preservation recorded.
- [ ] Owner accepts real setup, sound and export; remain open if not yet performed.

## Implementation and verification notes

The desktop model implementation now lives in `crates/ownership-model`, with a thin error-converting root shim. Numerical training, recipe and truth-row validation are preserved. Fixed-size compiled prediction performs no per-frame allocation. Root local checks passed: 370 tests, 20 intentionally ignored; strict all-feature/all-target Clippy, formatting and repository script fixtures passed. The shared crate's compiled predictor parity/invalid-frame test passed separately; `scripts/check.ps1` now includes that package test in future checks.

The plugin has an embedded egui editor with bundled fonts, shared group calibration and four explicit labels. It stores at most 2,048 native 32 ms feature rows and 128 merged truth ranges. Capture requires playback; seeking during capture cancels it. Learning runs outside callbacks and validates both independent binary models. Saved fields include a callback-visible revision: host preset/state replacement immediately makes old runtime evidence ineligible and requests reactivation. Failed binding unregisters the offending member and prevents it from publishing eligible PCM. Source identity is deliberately manual: confirm unchanged sources/setup on reload, and recapture after source/microphone/gain changes.

Processing uses a reported delay of 416 ms plus the configured maximum host block (up to 8,192 samples), bounded histories and finite quintic fades. This budget covers the 96 ms evidence lag, maximum fade lookahead, frame rounding and a complete callback. The plugin declares its delayed tail. Disabled is zero added latency and bit-exact; enabled but missing/stale/invalid evidence preserves the delayed original. Original/Processed/Removed share that delay. Explicit positive human truth is exact unity; both microphones can remain owned during overlap.

The streaming feature recipe uses native 32 ms AC energy, relative energy and squared zero-lag coherence. It shares the desktop trainer, **not** the full-recording VAD/transfer feature pipeline. This is a measured synthetic prototype pending real recording/listening acceptance, not desktop detector parity. The code records the replacement condition beside the recipe. CLAP attenuation is disabled through that adapter until native export scheduling and reliable render transitions qualify in ticket 76; the editor/calibration/diagnostics remain available. This is stronger than inspecting an offline-at-activation flag, which cannot reliably identify every render transition. Manual export labeling is diagnostic context only, not reliable host mode detection.

Read-only review identified and fixed stale-state calibration, insufficient lookahead at large/max-fade blocks and unreported delayed tails. The paired streaming regression exercises actual registry/PCM sharing, training, independent held-out room/solo/overlap, 44.1 kHz, 512/8,192-sample serial callbacks and maximum fades; every interior output sample must reach the expected unity/floor. Other tests cover cancellation, invalid training, persistence/reconfirmation, delayed tails/seeks, unknown/stale fallback and monitoring reconstruction.

Native editor attachment, FL delay compensation, real quiet speech/overlap, state save/reopen and exported sound remain owner checks. No DAW or desktop application window was launched by this work. VST2, AAX, AU and non-Windows native packaging/qualification are not delivered by this listening ticket.

## Checked delivery — 2026-10-24

### Owner UI rejection and revised direction

### Waveform candidate delivered — 0.0.4

Owner-confirmed frozen-history selection is implemented and checked. [Delivery receipt and actual offscreen renders](../../../docs/research/plugin-history-delivery-2026-10-24.md) record bounded true min/max input history, drag selection, shared label replacement/removal, pending-example persistence, source/state guards, model/history cursor separation, application palette and persistent audition controls. The live capture form is replaced. 28 plugin tests, strict Clippy and both actual-format hosts pass. Linked binaries match tested SHA256 `a8aead9c0a1d12e54cf05e06b6212b72e991427887271248e66b1f658ecb281c`; 0.0.3 remains preserved. Native FL window behavior, feel, save/reopen and real sound/export remain owner acceptance. Keep this ticket open.

Owner confirms the primary workflow: play a section, freeze waveform history, then drag-select and label it like the full application. Codex claims the waveform history, retrospective example persistence/editing, and replacement editor implementation under this ticket. Use the application dark palette and real received-audio min/max bins. No audio playback/seek change is implied by freezing the display. Keep model/DSP validation and actual-format checks; native appearance/listening acceptance remains owner work after a checked candidate is concrete.

The owner rejects the form-based 0.0.3 interface as difficult and visually unlike the full application. They request updating waveform history inside the plugin and explicitly invoke REA on installed Pro-Q 4, Pro-L 2 and Pro-R 2. [The installed-artifact investigation](../../../docs/research/plugin-waveform-ui-2026-10-24.md) records exact versions/hashes, complete REA resource Evidence IDs, shipped help/illustration findings and a waveform-first workflow. PRODUCT.md supersedes the earlier no-waveform/live-button brief. Native UI acceptance remains unchecked. No new binary was built by this research pass; this is not acceptance of the current capture workflow. Retrospective frozen-history selection is recommended, with the user's structured preference pending.

- Version 0.0.3, stable plugin identities/name retained. Development checkout base `88dda154daa9f442018302e19f1c792513fd0874`, with the recorded uncommitted implementation; no commit/push/release publication. Rust 1.99.0, exact root/plugin lockfiles and shared local Cargo target/profile.
- `scripts/check.ps1 -Local` passes after adding the shared-package check and updating its CI fixture: 371 passing tests total (370 root + one shared predictor), 20 intentionally ignored, strict Clippy, formatting and script fixtures.
- `scripts/gui-dev.ps1 -BuildOnly` passes. Desktop `C:\Users\User\AppData\Local\CleanSlate\BuildCache\debug\rust-app.exe` SHA256 `78b1fe63ffd8fb76bfb86502b01b1f617d055518021d3ccaaffb611bf33892e4`; its device-free `--package-self-check` passes inference, analysis, save/reopen, render and cancellation.
- `scripts/build-fl-probe.ps1` passes: 18 unit tests, strict all-target Clippy, builds/examples, both actual binary hosts, candidate/publication hash checks. CLAP editor API/size are present; VST3 editor interface constructs/releases with valid dimensions. Neither window was attached/shown. Headless font rendering also passes.
- CLAP: 1,858 bit-exact disabled blocks, 1,168 matching shared windows, legacy 0.0.1 state migration/round-trip and mode restart checks; 0.41 s, 17.65 MiB observed peak working set.
- VST3: 4,048 bit-exact disabled/delayed-uncalibrated blocks, 3,392 shared windows, mono/stereo 44.1/48 kHz realtime/offline, in-place/separate buffers, missing context and seeks. Enabled instances declare the correct latency and delayed tail; 0.76 s, 19.05 MiB observed peak. Peaks sample the test child every 10 ms, excluding compiler/PowerShell. These actual-format checks do not claim a GUI-driven learned-model workflow; paired learned processing is qualified separately in the unit regression.
- Both published binaries under `dist/fl-probe` have SHA256 `d2878255443e4e522cea011a8944b4de97bd749e78c0ca8702302418025c74d2`, matching the tested candidate. Existing standard-location junction/symlink paths serve those project artifacts. Prior 0.0.2 output preserved under `C:\Users\User\AppData\Local\CleanSlate\VerifiedBuilds\fl-probe\3f20ffa26616a60069da861087bef65ba219cbae71438a064f733716c0b81ef7`.
- Owner guide: `plugins/fl-probe/README.md`, first listening test. Next boundary is owner opening the VST3 0.0.3 editor, capturing room/solo examples, learning, enabling both members, comparing modes, saving/reopening and exporting. Keep this ticket claimed/open until native controls and sound are accepted. Native CLAP attenuation and all other platform/legacy adapter qualification remain separate.
