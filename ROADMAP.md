# CleanSlate roadmap

CleanSlate focuses on offline bleed reduction for podcasters with a dedicated microphone per speaker. Tracks must already be synchronized. The goal is a useful supervised first pass, followed by listening, label corrections and manual gain edits before delivery.

This page is the public overview, last reviewed on 2026-10-11. The [active map](.scratch/offline-supervised-migration/map.md), [accepted specification](.scratch/offline-supervised-migration/spec.md) and [implementation tracker](.scratch/offline-supervised-migration/implementation.md) remain authoritative for implementation scope, dependencies and acceptance. Dates and ticket numbers do not imply execution order or delivery deadlines.

## Current priorities

- Planned application support and appearance work: [ticket 36](.scratch/offline-supervised-editor/issues/36-app-menu-help-about.md) adds the app menu, documentation/issue links and About; [ticket 37](.scratch/offline-supervised-editor/issues/37-semantic-dark-light-themes.md) establishes semantic colors and a saved Dark/Light choice. Custom theme packages, diagnostic collection and refresh-rate settings remain deferred. These features are specified, not implemented.

- Accepted editor-workflow improvements now have specifications and tickets: [loop audition while labeling](.scratch/offline-supervised-editor/issues/31-label-loop-audition.md), [custom sound values and mute](.scratch/offline-supervised-editor/issues/32-sound-value-entry.md), [centered import and quieter waveform display](.scratch/offline-supervised-editor/issues/33-focused-import-waveform.md), and [shortcut help and zoom](.scratch/offline-supervised-editor/issues/34-shortcut-help-zoom.md). These are planned work, not available features. [Waveform-overlay automation editing](.scratch/offline-supervised-editor/issues/35-waveform-automation-editor-design.md) has its own design ticket.

- Evaluate bleed reduction, preserved speech, natural overlap and fades on independent recordings. [Ticket 14](.scratch/offline-supervised-editor/issues/14-independent-release-evaluation.md) tracks quality, listening and long-session evidence; stable Windows qualification remains gated by [ticket 15](.scratch/offline-supervised-editor/issues/15-windows-release-qualification.md).
- Improve the editor's wording, visual hierarchy and user flow. [Ticket 23](.scratch/offline-supervised-editor/issues/23-gui-frontend-polish.md) tracks the agreed GUI work. Bug reports and UI/UX feedback help identify what still gets in the way.
- Maintain experimental desktop packages and clear download instructions. [Ticket 25](.scratch/offline-supervised-editor/issues/25-portable-acoustic-alpha.md) and [ticket 26](.scratch/offline-supervised-editor/issues/26-cross-platform-release-packages.md) cover alpha and platform packaging. The [README](README.md#download) links published alpha packages; packaging does not establish listening quality or stable-release qualification.
- Export settings for encoding and separate or multichannel WAV delivery are implemented under [ticket 28](.scratch/offline-supervised-editor/issues/28-export-settings-multichannel-wav.md). See the [usage guide](docs/editor-workflow.md#export-settings) for available choices and compatibility limits.
- Make user and contributor documentation easier to follow, including workflow explanations and consistent public issue reporting.

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

An [advanced training panel](.scratch/offline-supervised-editor/issues/17-advanced-training-panel.md) remains a triage item. Further ideas are welcome, including problems we have missed in the recording or editing workflow. The current focus remains bleed reduction and editable attenuation; use your usual audio editor for other production tasks.

## Help decide what comes next

Use [GitHub Issues](https://github.com/Today20092/podcast-bleed-reducer/issues/new/choose) to report bugs, confusing flows, listening results or unmet recording needs. Describe the problem and desired result before prescribing a solution. See [contributing](CONTRIBUTING.md) for the proposal and implementation workflow.
