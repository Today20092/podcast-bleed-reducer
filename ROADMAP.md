# CleanSlate roadmap

CleanSlate focuses on offline bleed reduction for podcasters with a dedicated microphone per speaker. Tracks must already be synchronized. The goal is a useful supervised first pass, followed by listening, label corrections and manual gain edits before delivery.

This page is the public overview, last reviewed on 2026-10-25. The [active map](.scratch/offline-supervised-migration/map.md), [accepted specification](.scratch/offline-supervised-migration/spec.md) and [implementation tracker](.scratch/offline-supervised-migration/implementation.md) remain authoritative for scope, dependencies and acceptance. Dates and ticket numbers do not imply delivery deadlines.

## Three separate products

| Product | Purpose | Development |
| --- | --- | --- |
| **CleanSlate Desktop** | Existing standalone Rust GUI for supervised offline bleed reduction | `development` |
| **CleanSlate Smart Gate** | Single microphone speech gating for live production; OBS is an intended host | `codex/plugin-smart-gate`, [ticket 85](.scratch/offline-supervised-editor/issues/85-smart-gate-plugin.md) |
| **CleanSlate Podcast** | Desktop-style multitrack labeling, analysis and bleed reduction inside a DAW; FL Studio and Resolve/Fairlight are intended hosts | `codex/plugin-podcast`, [ticket 86](.scratch/offline-supervised-editor/issues/86-podcast-plugin.md) |

Each plugin has its own workspace, product interface and future binary identity. They share maintained Rust processing where appropriate. Smart Gate initially means gating, not automatic volume leveling; Podcast prioritizes post-production, not live multitrack recording. See the [product boundaries and workspace plan](docs/plugin-products.md).

The checked Windows 0.0.6 probe is a development baseline, not either finished product. VST3/CLAP are primary targets; VST2 remains desired legacy coverage pending adapter/licensing review. macOS/Linux and each intended host require their own checks. AAX/AU remain deferred.

## Current priorities

- **Prepared overnight work (88–99):** reference investigation, separate plugin identities/UI/DSP, host coverage and a desktop/updater audit have scoped dependency tickets. [The overnight plan](docs/plugin-overnight-plan.md) requires a tested global memory guard and resumable coordinator first: one heavy task at a time, no routine DAW, bounded REA and durable checkpoints. It is prepared, not scheduled or started; completion in one night is not guaranteed.

- **Optional purchase and project support (72):** an owner-accepted development change adds Ko-fi/GitHub Sponsors, first-launch dismissal and Menu/About access while keeping the full app free. Purchases help sustain maintenance, usability, testing and reliable packages. These changes are not in the published 1.0.0 downloads; savings feedback remains future work.

- **Optional purchase and project support (72):** the complete app remains free. Ko-fi is the primary optional one-time purchase route, with GitHub Sponsors alongside it. Menu/About access, a first-launch message and roadmap links are in development; these changes are not part of the published 1.0.0 packages. Purchases help sustain maintenance, usability, testing and reliable packaging. No measured time-savings or revenue figures are claimed.
- **Windows package updating (73):** implemented and tested in development for clean portable installations, with signed whole-package verification, safe restart, rollback and explicit cleanup. Mixed/modified folders and macOS/Linux use manual downloads. Published 1.0.0 is unchanged and will need a trusted manual bootstrap replacement; see the [delivery receipt](docs/research/portable-updater-delivery-2026-10-23.md).
- **Two independent plugin products (85/86):** Smart Gate and Podcast now continue on separate branches/workspaces. Podcast inherits persistent source-time waveform capture, labels and maintained offline analysis from the probe; its outstanding sound/export acceptance and shared-control/cache/recovery work remain open. Smart Gate needs a separate causal single-input detector/gain path and live-host qualification. The [split plan](docs/plugin-products.md) supersedes the combined-mode direction in discovery 67.

- **Stable 1.0.0 is published** for all four desktop targets after owner approval and package verification. The [release notes](docs/release-notes.md) describe the workflow; the [versioning and stability policy](docs/releasing.md#versioning-and-stability) defines compatibility and patch/minor/major selection. Historical alpha releases retain their original identities.
- The owner accepts current sound and performance; [evaluation ticket 14](.scratch/offline-supervised-editor/issues/14-independent-release-evaluation.md) is closed. Formal benchmarking is optional future work, with no new ticket requested. This does not assert that independent comparisons or long-session measurements passed.
- Use the [recurring editor verification procedure](docs/branch-workflow.md#editor-regression-coverage) for change-appropriate appearance, import, shortcut and playback checks. Implemented feature tickets are closed; bug reports and UI/UX feedback can identify new actionable work.
- [Alpha.8 is published](https://github.com/Today20092/podcast-bleed-reducer/releases/tag/v0.1.0-alpha.8) under completed [release ticket 64](.scratch/offline-supervised-editor/issues/64-alpha-release-preparation.md), with refreshed documentation, seven actual Windows screenshots and checked native packages. Alpha.7 stopped on missing dependency license texts; alpha.8 supplies their upstream notices. The [per-release procedure](docs/branch-workflow.md#release-checklist) retains package and physical-check requirements for 1.0.0. Historical alpha identities and current known limits remain explicit.
- Export settings for encoding and separate or multichannel WAV delivery are implemented under [ticket 28](.scratch/offline-supervised-editor/issues/28-export-settings-multichannel-wav.md). See the [usage guide](docs/editor-workflow.md#export-settings) for available choices and compatibility limits.
- Native documentation screenshots for the accepted interface are delivered under [ticket 20](.scratch/offline-supervised-editor/issues/20-editor-first-documentation-visuals.md). Seven actual Windows captures complement the eleven teaching diagrams and their completed local browser review.

## Delivered in this batch

The stable source has customizable shortcuts, rotary sound controls, waveform Original/Processed/Both views, persistent edge Catch up, playhead lock/reanchoring, compact icon transport, themes and collapsible analysis timing details. Drawing reuse, background-work handling and playback resource lifetime have also improved. See the [release notes](docs/release-notes.md) for behavior, measurements and remaining limits.

Menu redesign (60), shared action/shortcut registration (63) and the toolbar icon cleanup (69) are complete and included in the 1.0.0 source promotion. The [integration receipt](docs/research/menu-actions-integration-2026-10-22.md) records checks and acceptance limits; these additions are absent from the older alpha.8 download. The [implementation tracker](.scratch/offline-supervised-migration/implementation.md) retains retired-cache cleanup (22) in the build queue, with its unresolved deletion restriction. GUI polish (23), adjustable shortcuts (41) and Catch up (42) are released and owner-closed. Advanced training (17) and waveform-overlay editing (35) remain deferred. Plugin implementation now has its own product workspaces and scoped queue above.

Check each ticket for its latest status. This list describes agreed work, not a claim that every item is finished or available in a published build.

## Future possibilities

These ideas need investigation and an agreed specification before implementation. They have no delivery date and are not release requirements. [Plugin discovery 67](.scratch/offline-supervised-editor/issues/67-live-clap-vst3-discovery.md) is retained as history; its continuing work now belongs to the accepted Smart Gate and Podcast product tickets above. The [product opportunities research](docs/research/podcast-bleed-product-opportunities-2026-10-22.md) records additional proposals for discussion.

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

An [advanced training panel](.scratch/offline-supervised-editor/issues/17-advanced-training-panel.md) is parked as a deferred proposal outside the active queue; implementation remains unapproved. Further ideas are welcome, including problems we have missed in the recording or editing workflow. The current focus remains bleed reduction and editable attenuation; use your usual audio editor for other production tasks.

## Help decide what comes next

Use [GitHub Issues](https://github.com/Today20092/podcast-bleed-reducer/issues/new/choose) to report bugs, confusing flows, listening results or unmet recording needs. Describe the problem and desired result before prescribing a solution. See [contributing](CONTRIBUTING.md) for the proposal and implementation workflow.
