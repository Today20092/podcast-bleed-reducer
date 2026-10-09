# Import synchronized channels from a multichannel WAV

Status: done
Type: task
Assignee: Codex
Blocked by: 02, 03, 04, 13

## Scope

Owner confirmed on 2026-10-09 after grill-me: select WAV channels at import, initially all selected; create editable project tracks referencing the original file. Preserve native grid and timing. Use iXML recorder names with Channel N fallback, allow renaming, and mix selected channels with mono files under existing synchronization checks. Keep the eight-speaker limit. No split source WAV files; processed mono WAVs are created only by export.

## Acceptance checks

- Bounded decoding selects the correct interleaved channel for supported WAV encodings and rejects invalid selections, invalid samples and cancellation.
- File-plus-channel identities isolate waveform, analysis and playback caches; saved projects reopen and relink without losing channel/name bindings. Existing mono projects remain readable.
- Import picker supports channel selection and names, preserves valid sessions on failure and enforces one to eight selected tracks.
- Analysis, original/processed/removed audition and native mono exports use the selected channel, native rate and BEXT timing.
- Inspect the owner's 261005-T001.WAV: 3 channels, Tr5/Tr6/Tr7, 48 kHz float32, 53:31.008; exercise full-file import without changing its bytes.
- Run scoped regressions, formatting, strict Clippy and scripts/gui-dev.ps1 build.

## Results

Completed on 2026-10-09 on codex/offline-supervised, preserving the existing uncommitted GUI work.

- Added file-plus-channel source identities with backward-compatible optional channel/name fields, per-channel fingerprints/stamps, bounded channel decoding, source-aware waveform/analysis/playback preparation and native mono rendering. Existing mono entry points remain compatible. Selected channel order, names and relinks persist in projects.
- File picker and drops inspect sources asynchronously. Multichannel imports show channel selection and editable recorder labels before committing. Inspection retains placeholder paths and queues later drops. Cancellation, invalid names, invalid track counts and source validation failure preserve the committed session. Track headers expose Rename; exports use safe track names with ordinal prefixes.
- 124 scoped checks passed across library, editor_workflow, import_audio, multichannel_delivery, native_playback, project and supervised_delivery. These include all five supported WAV encodings across decode-block boundaries, BEXT round-trip, invalid channel/sample/truncation/cancellation rejection, distinct channel caches, mixed mono import, saved names/relink bindings, waveform isolation and full acoustic analysis/native-export/playback sample parity. The first run exposed an import-queue inspection gap; retaining inspection paths fixed it and the complete rerun passed.
- Explicit full-file check passed for test-audio/261005-T001.WAV: Tr5/Tr6/Tr7, 48 kHz float32, 154,128,376 frames per channel. Waveforms, validation and reopened-project source checks passed. Before/after SHA-256 matched. Import/check phase took 152.23 seconds, whole check 158.48 seconds while compiling in parallel. No split WAVs were written; the temporary verification project/executable were removed.
- Strict Clippy passed for all targets/features with -D warnings; formatting and git diff --check passed. Final scripts/gui-dev.ps1 -BuildOnly passed in 14.03 seconds without warnings. Dependencies and release profile were unchanged.
- Physical GUI interaction/listening and release-quality acceptance were not measured here. They remain owner/release checks in tickets 16/14/15. Saved channel projects require the original multichannel WAV to remain available or be relinked.
