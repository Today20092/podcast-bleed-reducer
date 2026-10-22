# CleanSlate roadmap

CleanSlate focuses on offline bleed reduction for podcasters with a dedicated microphone per speaker. Tracks must already be synchronized. The goal is a useful supervised first pass, followed by listening, label corrections and manual gain edits before delivery.

This page is the public overview, last reviewed on 2026-10-22. The [active map](.scratch/offline-supervised-migration/map.md), [accepted specification](.scratch/offline-supervised-migration/spec.md) and [implementation tracker](.scratch/offline-supervised-migration/implementation.md) remain authoritative for scope, dependencies and acceptance. Dates and ticket numbers do not imply delivery deadlines.

## Current priorities

- The owner accepts current sound and performance; [evaluation ticket 14](.scratch/offline-supervised-editor/issues/14-independent-release-evaluation.md) is closed. Formal benchmarking is optional future work, with no new ticket requested. This does not assert that independent comparisons or long-session measurements passed.
- Finish verification of the integrated editor's appearance, import flow and playback. [Ticket 23](.scratch/offline-supervised-editor/issues/23-gui-frontend-polish.md) consolidates the remaining checks from implemented tickets 38/39/40. Bug reports and UI/UX feedback help identify what still gets in the way.
- Ship the owner-accepted development batch as alpha.7 under [release ticket 64](.scratch/offline-supervised-editor/issues/64-alpha-release-preparation.md), with refreshed documentation, screenshots and checked native packages. The [per-release procedure](docs/branch-workflow.md#release-checklist) retains package and physical-check requirements. Alpha wording and known limits remain explicit.
- Export settings for encoding and separate or multichannel WAV delivery are implemented under [ticket 28](.scratch/offline-supervised-editor/issues/28-export-settings-multichannel-wav.md). See the [usage guide](docs/editor-workflow.md#export-settings) for available choices and compatibility limits.
- Refresh native documentation screenshots for the accepted interface under [ticket 20](.scratch/offline-supervised-editor/issues/20-editor-first-documentation-visuals.md). The eleven teaching diagrams and their local browser review are already complete.

## Delivered in this batch

The integrated editor has customizable shortcuts, rotary sound controls, waveform Original/Processed/Both views, persistent edge Catch up, playhead lock/reanchoring, compact icon transport, themes and collapsible analysis timing details. Drawing reuse, background-work handling and playback resource lifetime have also improved. See the [release notes](docs/alpha-release-notes.md) for behavior, measurements and remaining limits.

The [implementation tracker](.scratch/offline-supervised-migration/implementation.md) records remaining release, cleanup and verification work. Adjustable shortcuts (41) and Catch up (42) are implemented; their open records retain specific unperformed native checks. Waveform-overlay editing (35) remains deferred as unnecessary for current use.

Check each ticket for its latest status. This list describes agreed work, not a claim that every item is finished or available in a published build.

## Future possibilities

These ideas need investigation and an agreed specification before implementation. They have no delivery date and are not release requirements.

The [feature idea table](docs/roadmap-ideas.md) captures the owner's proposals and interview decisions. I05–I14 and initial settings/help/themes now have accepted scoped specifications and build tickets listed above, with overlay editing separated for design. Independent-source synchronization and drift correction are deferred by owner decision. Plugin exploration and later customization still require decisions before implementation commitments.

### Synchronization and drift correction: deferred

On 2026-10-11, the owner chose to leave alignment and drift correction to the user's audio or video editor. CleanSlate continues to require already-synchronized tracks. Automatic synchronization, guided alignment and time stretching are deferred; no implementation or drift-checking feature is planned from this discussion. The [research findings](docs/research/synchronization-and-drift.md) remain available if a concrete need justifies revisiting the idea.

### Guided tutorial and usage video

Plan an optional first-launch tutorial, also available later from Help, once the basic editor workflow is settled. A workflow chart should explain the order: import synchronized tracks, listen and label speaker examples, analyze, review by listening, correct examples and analyze again, then export.

Use a practice recording with four or five microphone tracks to teach waveform selection and labeling. Include solo speech, overlapping speakers and room tone. Guide users to supply a solo example for every speaker and a room-tone example, then additional examples that cover different speaking conditions. After analysis, teach users to compare Original, Processed and Removed audio across the recording. Where ownership is wrong, select the passage, label everyone actually speaking, and analyze again. For example, if speakers one and three are talking together, label both. Corrective examples can improve predictions elsewhere, but users must listen again rather than assume the rest is correct. Distinguish speaker-example corrections from manual gain edits.

Create a dedicated step-by-step usage video after the basic features are settled, covering the same workflow through export. Tutorial delivery, practice-audio rights and packaging, feedback on exercises, skip/replay behavior and completion criteria need an agreed specification before implementation. See the [tutorial proposal](docs/roadmap-ideas.md#guided-tutorial-and-usage-video-2026-10-12).

### Reporting problems from the app

Explore a Help action that opens the GitHub issue forms, followed by an optional local diagnostic report users can preview, copy or export. The report should provide useful version, platform and failure information even when analysis is broken. Audio, transcripts, personal file paths and other private content should be excluded by default. Users would decide what to share and submit the issue themselves.

### Other proposals

- [Menu research/redesign](.scratch/offline-supervised-editor/issues/60-rea-plugin-menu-design.md) using the requested plugin interface references.
- [Registered actions and shortcuts](.scratch/offline-supervised-editor/issues/63-registered-actions-shortcut-contract.md), so new actions become customizable without separate requests. This infrastructure is planned, not part of alpha.7.

An [advanced training panel](.scratch/offline-supervised-editor/issues/17-advanced-training-panel.md) is parked as a deferred proposal outside the active queue; implementation remains unapproved. Further ideas are welcome, including problems we have missed in the recording or editing workflow. The current focus remains bleed reduction and editable attenuation; use your usual audio editor for other production tasks.

## Help decide what comes next

Use [GitHub Issues](https://github.com/Today20092/podcast-bleed-reducer/issues/new/choose) to report bugs, confusing flows, listening results or unmet recording needs. Describe the problem and desired result before prescribing a solution. See [contributing](CONTRIBUTING.md) for the proposal and implementation workflow.
