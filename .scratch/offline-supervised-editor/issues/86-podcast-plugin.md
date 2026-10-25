# CleanSlate Podcast: supervised multitrack post-production plugin

Status: open
Readiness: ready-for-agent
Type: task
Blocked by: 87
Related: 67, 75, 76, 77, 78, 80, 81, 82, 83, 84, 85

## Accepted product

A distinct post-production plugin bringing the desktop application's supervised bleed-reduction workflow into a DAW or compatible video-editor audio host. Work with already-synchronized dedicated microphone tracks for up to eight people. Capture host audio, inspect persistent source-time waveforms, select room/solo/overlap examples, explicitly analyze with the maintained Rust engine, review and adjust attenuation, and render through the host. Keep original sources intact. The desktop app does not need to run beside it.

Development belongs to `codex/plugin-podcast` in its own worktree. The existing 0.0.6 Session implementation is the starting baseline; the Live Mic mode is no longer a second mode of this product. The detailed Session contract in `../../offline-supervised-migration/plugin-modes-interview.md` continues unless superseded here. Live multitrack podcast recording/monitoring is not a requirement.

## Existing evidence and dependencies

Ticket 84 completed scoped navigation checks; 77 supplies actual-format headless hosts. These remain valid evidence, not whole-product acceptance. Ticket 80 is still claimed with sound/export/PDC/save/reopen acceptance outstanding. Tickets 81–83 remain blocked by 80 until it is done; this product ticket does not clear those dependencies. Finish the existing Session candidate's checks before continuing that implementation sequence.

## Implementation order

1. Preserve and qualify 80's current Session build, then introduce a separate Podcast package/name/stable format IDs/state, with explicit legacy probe coexistence rather than silently changing saved projects.
2. Complete 81 shared defaults/per-track sound edits, host automation/dirty-state notifications and envelope reuse.
3. Complete 82 source-bound analysis reuse and coverage/invalidation; complete 83 metadata, duplicate/project isolation and recoverable identities.
4. Close workflow gaps against the desktop application: label/edit/analyze/review, waveform navigation, Original/Processed/Removed audition and effective-envelope editing, with explicit parity checks rather than a promise that every desktop control is already present.
5. Qualify FL Studio first and DaVinci Resolve/Fairlight as a separate intended host, including actual supported plugin formats/routing, scheduling, latency compensation, seeks/loops, offline export and save/reopen.

## Acceptance

- [ ] Unique product identity, state and artifacts coexist with Smart Gate and the old probe; existing FL projects remain recoverable.
- [ ] Persistent source-time waveforms survive loops/seeks; changed replayed audio invalidates affected evidence, unchanged audio preserves it, and unplayed edits remain explicitly unverified.
- [ ] Every participating instance exposes shared session tracks, labels and permitted settings. Use host track name/color/identity when available with explicit manual fallback; never infer ownership from a display name alone.
- [ ] Synchronized participants agree on source identity and envelopes; independent sessions, duplicate instances and unsupported process isolation cannot silently share decisions.
- [ ] Source validation, explicit/cancellable analysis, stale-result rejection, human-label protections, shared envelope evaluation and original fallback remain intact.
- [ ] Native playback and offline export match the checked envelopes with correct timing/PDC, preserving wanted quiet speech and overlap under recorded listening acceptance.
- [ ] No export-to-desktop/reimport step is required. Explain capture/coverage readiness and the need to feed host audio before analysis; a plugin cannot assume access to unplayed host media.
- [ ] VST3/CLAP first and VST2 desired legacy coverage after adapter/licensing review. Windows first; macOS/Linux require native package and host evidence. AAX/AU remain deferred.
- [ ] Product build/check scripts and links publish only this product's artifacts after checks, with explicit old-build preservation and no accidental overwrite of the other plugin.

## Results

Independent product direction accepted. Existing probe software/native navigation evidence is preserved; no finished Podcast-branded binary or new host qualification is claimed.
