# CleanSlate 1.1.0

This compatible update adds manual update checking and optional purchases. Existing projects, preferences, source media and exports retain their formats and behavior.

## Changes in 1.1.0

- Check for updates from the application. Stable releases are selected by default; prereleases can be included for an individual check.
- Clean, unmodified Windows x64 portable installations on supported NTFS storage can install a verified whole-package update. Signed metadata, archive and file hashes are checked before replacement; backup, rollback and recovery protect interrupted updates. Unsaved work must be saved or explicitly discarded, and background workers exit before replacement.
- Modified or mixed folders, unsupported storage and Mac/Linux installations receive manual download guidance. Update checks and downloads require an explicit action; audio processing stays offline.
- Optional purchases support the project without changing access to the editor or its audio features.

The Windows update descriptor is signed with the pinned release Minisign key. This authenticates updater metadata; Windows executables remain without Authenticode signing. macOS packages remain ad-hoc signed and not notarized. Native CI and package self-checks do not establish fresh physical audio, listening or Mac/Linux desktop qualification.

**Updating from 1.0.0:** download and extract the complete 1.1.0 package into a new folder, then launch it. The 1.0.0 application predates the updater. Keep personal files outside the application folder so future supported Windows updates can replace that folder safely.

The [updater delivery receipt](research/portable-updater-delivery-2026-10-23.md) records 370 passing tests, strict Clippy and nine disposable Windows installer cases, including recovery, locked files and permission denial. The [signing guide](update-signing.md) describes release verification and owner-held keys.

## First stable release: 1.0.0

CleanSlate's first stable release delivers the accepted offline podcast workflow: import synchronized microphone tracks, label speaker examples, analyze, audition and edit gain, save the project, then export processed WAVs. The owner approved promotion and publication on 2026-10-22.

Download [v1.0.0](https://github.com/Today20092/podcast-bleed-reducer/releases/tag/v1.0.0) for Windows x64, Mac Apple Silicon, Mac Intel or Linux x64, with a SHA-256 checksum for each package. Historical [alpha release notes](alpha-release-notes.md), screenshots and package receipts retain their original identities.

## Changes since alpha.8

- The menu uses aligned full-width command rows, grouped sections and a separate effective-shortcut column. Its panel fits its contents with bounded scrolling. Tab, Shift+Tab, Up/Down, Enter and Escape retain keyboard navigation; an outside click dismisses the menu without acting on the waveform.
- Registered actions share command identity, button/keyboard activation, availability and effective hints. Settings and the shortcut reference use the same command declaration. Existing saved bindings remain compatible. Meters demonstrates a customizable action with no default binding.
- New action buttons must use the registered command path or have an explicit documented exception. Individually listed legacy buttons remain exceptions; this does not make every existing button remappable. See the [action contract](agents/actions-and-shortcuts.md).
- Catch up has a fixed-size arrow-to-edge icon with the existing selected state, tooltip and customizable shortcut. The three waveform icons no longer need a Waveform text label.

The [menu/action integration receipt](research/menu-actions-integration-2026-10-22.md) records source identities, owner acceptance, software verification and omitted native checks.

## Included offline editor

The stable workflow retains the improvements delivered during the alpha releases:

- Original, Processed and Removed audition, with separate Original/Processed/Both waveform visualization.
- Persistent right-edge Catch up, playhead Lock and one-shot Center. Manual timeline navigation keeps Catch up selected.
- Rotary attenuation and timing controls with smooth indicators, typed values, fine adjustment and reset gestures; centered fixed-size transport icons and collapsible timing details.
- Configurable context-aware shortcuts, Dark/Light appearance, speaker-example editing, protected ranges and manual gain edits that survive reanalysis.
- Read-only source media, local working projects, validated save/reopen and exports into new destination folders. Separate mono and compatible multichannel WAV delivery remain available.
- Offline analysis with embedded Silero VAD and ownership models learned from human examples. No Python, runtime model download or internet connection is needed for analysis.

See the [usage guide](editor-workflow.md) for the complete workflow and export restrictions.

## Performance evidence

These are previously recorded component measurements, not new 1.0.0 benchmarks:

- In the dense synthetic two-hour fixture, gain-draft p95 fell from 43.496 ms to 22.396 ms by reusing unaffected tracks' prepared envelopes.
- Two interleaved 2176-column runs reduced complete waveform-plus-gain preparation medians by about 1.35% and 1.9% after omitting redundant flat-span vertices. Tails varied; the wider 4096-column workload still missed its diagnostic budget.
- Unchanged drawing reuses caches; minimized windows suspend optional drawing while audio/background work continue. Ready playback reuses prepared audio and retires replaced buffers outside device-sample iteration.

The [performance summary](research/alpha7-performance-summary-2026-10-22.md) identifies workloads and source revisions. These results do not measure whole-session analysis, import, export, physical audio latency or a universal frame rate.

## Stability and remaining limits

Stable means the documented offline workflow is recommended and maintained. The compatibility promise covers supported inputs/exports, saved projects, speaker examples, manual gain edits and persisted preferences. Compatible fixes use patch versions, compatible capabilities use minor versions, and deliberate compatibility breaks require a major version and migration guidance. See the [versioning policy](releasing.md#versioning-and-stability).

Tracks must already be synchronized. Quiet overlap without representative examples can receive low ownership estimates. Listen to wanted speech and Removed audio, add examples, protect speech or edit gain before export. Stable does not mean perfect detection on every recording. Formal independent comparisons and long-session qualification remain unestablished.

Windows packages are unsigned; macOS packages use ad-hoc signing and are not notarized. Mac/Linux physical desktop/audio qualification remains separate from native CI. Prior alpha.8 package checks do not verify 1.0.0 artifacts. No new hardware qualification, subjective listening study or native screenshot matrix is claimed by this version change.

The owner accepted the final menu and toolbar appearance and waived further computer control. Automated native compact/wide Light/Dark menu and remap/restart checks were not completed; software routing, persistence and action-contract checks are recorded in the integration receipt. Version-specific package/source checks belong to the [release checklist](branch-workflow.md#release-checklist).
