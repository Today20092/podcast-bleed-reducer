# Configure WAV encoding and separate or multichannel delivery

Status: done
Assignee: Codex
Type: task
Blocked by: 02, 04, 13, 16, 21
Spec: ../../offline-supervised-migration/spec.md

## Scope

Owner confirmed the export design after grilling on 2026-10-11 and requested this single roadmap ticket. This records future implementation, not completed application work or release qualification.

### Export dialog

- A single Export action opens a small in-app settings dialog before WAV preparation. Replace the current Prepare WAV export / Save WAV files split with this flow; changing encoding must not silently reuse an incompatible prepared render.
- Show output layout, encoding, destination folder selection, estimated audio size, and a summary of track/channel count and original sample rates. Include Cancel and Export files buttons. Export files prepares and safely publishes the selected delivery.
- Default layout is separate mono WAV files. Also offer one multichannel WAV containing separate processed channels, never a summed mix.
- Order multichannel output channels as tracks appear in the editor. Show their names and order before export.
- Default encoding is Preserve source encoding, with 24-bit PCM and 32-bit float as explicit alternatives. Explain non-WAV and float64 exceptions below.
- Remember the encoding choice per project across save/reload; choose the destination each time. Existing projects without this preference default to Preserve source encoding.
- Settings belong in the export dialog, not the editing workspace. Follow docs/gui-development.md and scripts/gui-dev.ps1; coordinate src/app.rs changes with ticket 23.

### Encoding and timing

- Separate files with Preserve source encoding retain each supported WAV source's own integer/float encoding and bit depth. A session may export one PCM16 WAV and another float32 WAV. Keep the current float64-to-float32 exception because the engine processes f32 samples; do not claim float64 precision preservation.
- Preserve source encoding keeps existing non-WAV behavior: decoded AIFF/FLAC/MP3/AAC-LC/ALAC/Vorbis sources export float32 WAV. Clearly describe this default; no compressed-format export is added.
- Explicit 24-bit PCM or 32-bit float applies to every exported track/channel.
- Multichannel WAV requires one common sample rate and encoding. When rates differ, disable this layout with an explanation and keep separate native-rate WAVs available. No export resampling is added.
- For multichannel Preserve source encoding, keep a shared encoding when all effective output encodings match. When they differ, require an explicit 24-bit PCM or 32-bit float choice; do not silently select float or a lower bit depth. Account for the documented non-WAV/float64 mappings when comparing effective encodings.
- Preserve every source's native sample rate, decoded frame count, synchronization and verified timing. Multichannel delivery requires compatible frame counts/timing; never trim, pad, stretch or silently repair disagreement.
- Show size estimates beside encoding choices, using channel count, frame count and bytes per sample. Clearly label estimates; 24-bit audio payload is 25% smaller than float32 for identical grids/channel counts.

### Quantization and safeguards

- Automatically apply appropriate final dither when quantizing processed floating-point samples to integer PCM, including preserved PCM16/24 outputs. No separate dither UI. Keep exact silent output spans silent and do not add dither to float output or apply it twice.
- Detect samples outside the target integer full-scale range before publishing any integer output, including preserve-encoding outputs. Block with an actionable offer to use float32; never silently clip, normalize or attenuate.
- Retain the shared effective-envelope evaluator, source/provenance validation, current-analysis/render requirements, stale-job rejection, cancellation, destination collision checks and atomic completion. Solo monitoring must not change exports.
- Preserve compatible WAV/BWF timing metadata; conflicting metadata cannot be silently copied into a combined file. Exported WAV channels are audio, not saved project edits or gain curves.
- Respect the current supported 32-bit RIFF size limit, accounting for all multichannel data and metadata; fail before publication with an actionable error. RF64 and compressed export are outside this ticket.

## Acceptance

- GUI review verifies the single Export dialog, layout/encoding choices, channel-order preview, destination selection, size estimates, Cancel and Export files, and clear mixed-rate/mixed-encoding blockers.
- Save/reload tests verify project-local encoding preference and defaults for older projects without it. Destination is requested on each export.
- Separate-file tests cover mixed PCM16/24/32 and float32 WAV inputs, float64 mapping, non-WAV float32 mapping, and explicit PCM24/float32 overrides. Verify headers, native rates and exact decoded frame counts.
- Multichannel tests verify channel count/order and per-channel effective-envelope samples against separate-file rendering, compatible timing metadata, shared-encoding preservation, explicit selection for mixed encodings, and rejection of mixed rates or incompatible grids/timing without resampling/trim/pad.
- Quantization checks cover PCM16/24 precision and bounded dither, exact silence, float output without dither, integer overload rejection, and float32 retry without level changes. Use deterministic test control where needed rather than flaky random assertions.
- Size estimates are checked against known payload sizes; oversized multichannel RIFF is rejected safely.
- Existing current-envelope/provenance, source-change, destination collision, cancellation and obsolete-job checks pass for both output layouts; no partial deliverable is published on failure.
- Complete appropriate Rust checks and the GUI development build. Record results and remaining limitations here before marking done. Stable audio-quality/release gates remain in tickets 14/15.

## Comments

2026-10-11: Owner accepted the dialog flow, preserve-encoding default, project-local encoding preference, separate channels in one optional WAV, disabling multichannel for mixed rates, explicit encoding selection for mixed encodings with size estimates, automatic final dither with silent spans preserved, and blocking integer overloads with a float32 alternative. No implementation performed while recording this ticket.

2026-10-11: Codex implemented this ticket in `daefded` and corrected the final-size boundary in `de2ee85`. The single Export dialog now selects layout, encoding and a fresh destination, displays native grids/channel order and payload estimates, and prepares then atomically publishes the delivery. Encoding is saved per project. Streaming final conversion applies deterministic TPDF dither once for integer output, preserves silence, rejects overload, and keeps float output unquantized. Source/provenance, generation, cancellation, collisions, timing/BWF and RIFF checks remain enforced for both layouts. Private float staging can use the dependency's RF64 support; delivered files remain within RIFF32 limits.

### Acceptance results

- Rust checks passed: `cargo check --tests`, the full `cargo test --all-targets --all-features` suite, strict all-target/all-feature Clippy (`-D warnings`), formatting and scoped diff whitespace checks. Commands used the locked offline GUI-development configuration and retained target cache.
- After the final size correction, the library and export/settings/native/supervised delivery suites passed again: 107 passed, 7 existing ignored. Ten new export-settings integration tests cover preferences/defaults, format mappings/overrides, native grids, channel order/effective samples, BWF conflicts, deterministic dither/silence, overload/float retry, size boundaries and both-layout failure cleanup.
- `scripts/gui-dev.ps1 -BuildOnly` and the normal development launch passed. Native review at the normal window size verified the single dialog, default separate/preserve choices, ordered names/native grids, all encoding estimates, destination selection and successful multichannel publication. PCM16/PCM24 sources showed the explicit common-encoding blocker; selecting PCM24 removed it. A 16/48 kHz session showed multichannel unavailable and separate native-rate output available. Cancel and Escape both closed the dialog. Automated GUI state tests also cover transient destinations, preference persistence and Solo-independent exports.
- Standards review against `50efe3e9777cc9f782575196a3b1cb13c4f1c670`: no findings. Specification review identified the private float-stage size restriction; it was corrected and re-reviewed with no remaining findings.

### Remaining limitations

No actual multi-gigabyte delivery was generated; boundary eligibility is tested without allocating large audio and private RF64 staging relies on the existing writer dependency. Existing ignored tests were not enabled. Stable listening/audio-quality and release qualification remain in tickets 14/15. RF64 delivery, resampling and compressed export remain outside scope.
