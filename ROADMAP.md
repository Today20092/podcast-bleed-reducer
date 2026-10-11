# CleanSlate roadmap

CleanSlate focuses on offline bleed reduction for podcasters with a dedicated microphone per speaker. Tracks must already be synchronized. The goal is a useful supervised first pass, followed by listening, label corrections and manual gain edits before delivery.

This page is the public overview, last reviewed on 2026-10-11. The [active map](.scratch/offline-supervised-migration/map.md), [accepted specification](.scratch/offline-supervised-migration/spec.md) and [implementation tracker](.scratch/offline-supervised-migration/implementation.md) remain authoritative for implementation scope, dependencies and acceptance. Dates and ticket numbers do not imply execution order or delivery deadlines.

## Current priorities

- Evaluate bleed reduction, preserved speech, natural overlap and fades on independent recordings. [Ticket 14](.scratch/offline-supervised-editor/issues/14-independent-release-evaluation.md) tracks quality, listening and long-session evidence; stable Windows qualification remains gated by [ticket 15](.scratch/offline-supervised-editor/issues/15-windows-release-qualification.md).
- Improve the editor's wording, visual hierarchy and user flow. [Ticket 23](.scratch/offline-supervised-editor/issues/23-gui-frontend-polish.md) tracks the agreed GUI work. Bug reports and UI/UX feedback help identify what still gets in the way.
- Prepare experimental desktop packages and clear download instructions. [Ticket 25](.scratch/offline-supervised-editor/issues/25-portable-acoustic-alpha.md) and [ticket 26](.scratch/offline-supervised-editor/issues/26-cross-platform-release-packages.md) cover alpha and platform packaging. Packaging does not establish listening quality or stable-release qualification.
- Add export settings for encoding and separate or multichannel WAV delivery under [ticket 28](.scratch/offline-supervised-editor/issues/28-export-settings-multichannel-wav.md).
- Make user and contributor documentation easier to follow, including workflow explanations and consistent public issue reporting.

Check each ticket for its latest status. This list describes agreed work, not a claim that every item is finished or available in a published build.

## Future possibilities

These ideas need investigation and an agreed specification before implementation. They have no delivery date and are not release requirements.

### Reliable automatic synchronization

Explore aligning recordings when it can be done reliably. Investigate both starting offsets and recording drift, including cases with weak shared audio. Preserve source files, provide an audible preview and flag uncertain alignment for manual checking. Until this is implemented and validated, supply synchronized tracks.

### Reporting problems from the app

Explore a Help action that opens the GitHub issue forms, followed by an optional local diagnostic report users can preview, copy or export. The report should provide useful version, platform and failure information even when analysis is broken. Audio, transcripts, personal file paths and other private content should be excluded by default. Users would decide what to share and submit the issue themselves.

### Other proposals

An [advanced training panel](.scratch/offline-supervised-editor/issues/17-advanced-training-panel.md) remains a triage item. Further ideas are welcome, including problems we have missed in the recording or editing workflow. The current focus remains bleed reduction and editable attenuation; use your usual audio editor for other production tasks.

## Help decide what comes next

Use [GitHub Issues](https://github.com/Today20092/podcast-bleed-reducer/issues/new/choose) to report bugs, confusing flows, listening results or unmet recording needs. Describe the problem and desired result before prescribing a solution. See [contributing](CONTRIBUTING.md) for the proposal and implementation workflow.
