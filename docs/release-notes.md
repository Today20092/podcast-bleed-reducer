# CleanSlate 1.0.0

CleanSlate's first stable source delivers the accepted offline podcast workflow: import synchronized microphone tracks, label speaker examples, analyze, audition and edit gain, save the project, then export processed WAVs. The owner approved promoting the current development branch to main on 2026-10-22.

This page describes 1.0.0 source and release preparation. Public downloads remain [v0.1.0-alpha.8](https://github.com/Today20092/podcast-bleed-reducer/releases/tag/v0.1.0-alpha.8) until new native packages are checked and published. Historical [alpha release notes](alpha-release-notes.md), screenshots and package receipts retain their original identities.

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
