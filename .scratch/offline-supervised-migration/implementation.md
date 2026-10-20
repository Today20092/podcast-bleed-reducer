# Offline supervised editor implementation sequence

Status: accepted implementation plan, 2026-10-08

## Playhead catch-up follow-up, 2026-10-20

[42 — Animated playhead catch-up scrolling](../offline-supervised-editor/issues/42-playhead-catch-up-scroll.md#accepted-design) specifies the owner's Pro-L 2 Slow Down-style navigation: hold the viewport still until the playhead reaches the right edge, quickly slide the timeline left until the playhead is near the left edge, then hold again. Status: ready-for-agent, unclaimed; blocked by 34, which is done. Coordinate with 41's playhead commands and zoom preference without changing locked follow behavior. Timing, interruption, bounds and GUI verification are scoped in the ticket. No implementation or merge is claimed.

## Branch refinement: progressive import, 2026-10-12

2026-10-12 owner-requested playback follow-up: [40 — Reusable selection playback](../offline-supervised-editor/issues/40-reusable-selection-playback.md) covers measurement, warm prepared-audio/output-stream reuse, truthful feedback and existing monitoring/label-loop semantics. Specification-ready, unclaimed and blocked by completion of 38; 04/31 are done. [Primary-source research](../../docs/research/selection-playback-latency.md) informs implementation. No benchmark, playback change, dependency migration or merge is claimed.

2026-10-12 owner-approved UI follow-up: [39 — Tooltip removal and shortcut readability](../offline-supervised-editor/issues/39-shortcut-reference-readability.md) specifies sound-value tooltip removal and a readable action-first shortcut reference with keycap badges. Dependencies 32/34/37 are done; specification-ready and unclaimed. Tooltip removal is already a working-tree edit with formatting checked; the reference redesign and build/owner checks remain pending. Playback preparation behavior is outside this ticket.

The owner selected `codex/editor-workflow-31-37` for continued refinement before merging into `main`. [38 — Progressive import flow](../offline-supervised-editor/issues/38-progressive-import-flow.md) contains its accepted design and complete build acceptance in one ticket. Status: ready-for-agent, unclaimed. Blocked by 03, 21, 24 and 33, all done on the target branch. It covers early skeletons/coarse previews, truthful progress, reduced repeated source work, queued imports, per-recording readiness, failure/cancellation and playback continuity. Implement on this branch; this planning update does not authorize or claim a merge into `main`.

## Accepted workflow extensions, 2026-10-11

2026-10-12 synchronization reconciliation supersedes the under-interview note below: the owner deferred I01–I04 in the separate research chat. Already-synchronized input remains required; no alignment, stretching, drift-correction or drift-checking ticket is planned. The interview is closed and research retained.

2026-10-12 owner-confirmed additions: [36 — App menu, Help and About](../offline-supervised-editor/issues/36-app-menu-help-about.md), with [accepted spec](specs/app-menu-help-about.md), is done through `bb5ec90`. [37 — Semantic colors and dark/light appearance](../offline-supervised-editor/issues/37-semantic-dark-light-themes.md), with [accepted spec](specs/semantic-dark-light-themes.md), is done in `7bcaaed`. Refresh-rate controls, diagnostics, OS-theme matching and custom theme packages remain deferred. Synchronization I01–I04 remains under interview and is not a build commitment.

Owner confirmed the I05–I14 interview and authorized four specs/build tickets plus a separate design ticket. These are planning records; no implementation was performed or claimed in this discussion. Specification readiness does not clear dependencies.

| Ticket | Scope / accepted specification | Blocked by | Planning status |
| --- | --- | --- | --- |
| [31](../offline-supervised-editor/issues/31-label-loop-audition.md) | [Loop audition while labeling](specs/label-loop-audition.md) | 13, 16 (done) | done, integrated through bad87a6 |
| [32](../offline-supervised-editor/issues/32-sound-value-entry.md) | [Typed sound values, mute and parameter reset](specs/sound-value-entry.md) | 02, 11, 13, 18, 28 (done) | done, integrated in 52b61f0 |
| [33](../offline-supervised-editor/issues/33-focused-import-waveform.md) | [Centered import and gain-strip visibility](specs/focused-import-waveform.md) | 16, 21, 28 (done) | done, implemented in 3d262a7 |
| [34](../offline-supervised-editor/issues/34-shortcut-help-zoom.md) | [Shortcut help, fit zoom and Escape deselection](specs/shortcut-help-zoom.md) | 31 (done) | done, `56009c2` |
| [35](../offline-supervised-editor/issues/35-waveform-automation-editor-design.md) | Separate waveform-overlay automation design | none | needs-info; design only |

Coordinate shared GUI edits with ongoing polish ticket 23; do not duplicate its broad appearance work or treat its owner-review status as a functional blocker. Update shortcut-reference entries as new controls land. Ticket 35's later build work must reconcile 32–34, but does not block the interim cleanup in 33. Tickets 14/15 retain independent quality/listening/release gates. Remaining [roadmap ideas](../../docs/roadmap-ideas.md) need clarification before joining this sequence.

## Existing implementation plan

2026-10-11 owner extension: [ticket 30](../offline-supervised-editor/issues/30-algorithm-documentation-coherence.md) aligns the README, reader explanation, model and temporal contracts, architecture and historical records with the current acoustic editor. This is documentation work; it changes no algorithm or release qualification.

2026-10-11 owner extension: [ticket 29](../offline-supervised-editor/issues/29-contributor-documentation-intake.md) covers the README origin story, contributor guide, public roadmap and GitHub issue forms. The [public roadmap](../../ROADMAP.md) summarizes current priorities without replacing this tracker. Automatic synchronization and in-app support reporting are exploratory possibilities, not implementation commitments or release requirements.

2026-10-11 owner extension: [ticket 28](../offline-supervised-editor/issues/28-export-settings-multichannel-wav.md) is done for the agreed Export dialog, per-project encoding preference, separate or multichannel WAV delivery, size estimates, final integer dither and overload safeguards. Its ticket records code, tests and GUI checks. This completes export scope, not independent audio-quality or stable release qualification.

2026-10-11 owner extension: [ticket 27](../offline-supervised-editor/issues/27-documentation-diagrams.md)
replaces all seven Mermaid documentation diagrams with GUI-matched SVGs and
accessible HTML sources. Static checks pass; browser review remains pending.

2026-10-10 owner extension: [ticket 26](../offline-supervised-editor/issues/26-cross-platform-release-packages.md)
is claimed for future Windows/macOS/Linux packages and a README download table.
Draft/manual publication and the existing stable qualification gates remain active.

2026-10-09 owner override: Nemotron is removed from the application and shipping
package. Tickets 05/06 and native mapping portions of 07 are historical retired
implementation; acoustic ownership and source/output safeguards remain active.
25. [Package the acoustic-only Windows portable alpha](../offline-supervised-editor/issues/25-portable-acoustic-alpha.md).
    Claimed for unsigned `v0.1.0-alpha.1`, local clean-machine checks and future
    GitHub draft releases. Explicitly unqualified alpha; independent of stable gates 14/15.

2026-10-09 completion update: ticket 16 is done following owner workflow acceptance of integrated checkpoint 6991d39. Its prior pending owner-workflow status below is historical. Owner preparation timing was not measured; audio-quality and release gates remain in tickets 14 and 15.

The architecture is [specified here](spec.md). The six wayfinder child tickets resolve architecture decisions; the fifteen tickets below are build work, not additional unresolved product decisions. Dependency fields, not this list alone, govern execution.

1. [Establish the accepted development baseline](../offline-supervised-editor/issues/01-establish-baseline.md)
2. [Introduce a source-aware timebase and versioned project contract](../offline-supervised-editor/issues/02-timebase-project-contract.md)
3. [Decode common mono formats and prepare aligned analysis copies](../offline-supervised-editor/issues/03-import-analysis-copies.md)
4. [Carry native rates through playback and WAV rendering](../offline-supervised-editor/issues/04-native-rate-playback-render.md)
5. [Install and verify the required Nemotron model and native runtime](../offline-supervised-editor/issues/05-model-runtime-acquisition.md)
6. [Retain complete Nemotron evidence for long recordings](../offline-supervised-editor/issues/06-nemotron-long-recordings.md)
7. [Edit reference labels and resolve diarization speaker identities](../offline-supervised-editor/issues/07-reference-labels-identity.md)
8. [Fit session acoustic evidence from trusted references](../offline-supervised-editor/issues/08-session-acoustic-evidence.md)
9. [Train independent supervised ownership models](../offline-supervised-editor/issues/09-session-ownership-model.md)
10. [Optimize ownership regions with conservative uncertainty](../offline-supervised-editor/issues/10-temporal-ownership-review.md)
11. [Compose editable exponential envelopes with protected overrides](../offline-supervised-editor/issues/11-effective-envelope-edits.md)
12. [Expose ownership review and envelope editing in Iced](../offline-supervised-editor/issues/12-editor-review-envelope-ui.md)
13. [Make audition and final exports use the same current envelope](../offline-supervised-editor/issues/13-current-preview-export-parity.md)
14. [Evaluate independent quality and long-session performance](../offline-supervised-editor/issues/14-independent-release-evaluation.md)
15. [Qualify the Windows offline release](../offline-supervised-editor/issues/15-windows-release-qualification.md)
16. [Make playback and reference labeling flow directly from the tracks](../offline-supervised-editor/issues/16-gui-editing-workflow.md)
18. [Use S-shaped automatic fades for transparent attenuation](../offline-supervised-editor/issues/18-s-shaped-automatic-fades.md). Authorized on 2026-10-09; dependencies 11/13 are done. Supersedes ticket 11's exponential curve while preserving its other contracts. Listening acceptance remains ticket 14.
19. [Add a podcast-editor workflow diagram to the README](../offline-supervised-editor/issues/19-readme-workflow-diagram.md). Authorized on 2026-10-09; documentation only, with no implementation dependencies.

2026-10-09 owner feedback adds only ticket 16, consolidating GUI decisions 07–11. Its dependencies 12/13 are done; it is the next owner-workflow implementation ticket when implementation is authorized. Acoustic-first evaluation from decision 12 is folded into existing ticket 14, following the usable GUI candidate. Numerical numbering is historical, not execution order. Ticket 15 remains blocked by quality/listening qualification; completing ticket 16 does not qualify a release.

21. [Import synchronized channels from a multichannel WAV](../offline-supervised-editor/issues/21-multichannel-wav-import.md). Owner confirmed and authorized on 2026-10-09; dependencies 02/03/04/13 are done. Implementation and scoped acceptance checks are done, including the owner's complete Zoom WAV. This extends the input boundary and preserves mono downstream processing; it does not qualify a release.

## Deferred backlog

24. [Start a new project from the GUI](../offline-supervised-editor/issues/24-new-project-action.md). Owner authorized a subagent implementation on 2026-10-09. Adds a New project action with explicit discard confirmation and an empty workspace while preserving saved projects and source audio. Dependencies 16/21 and scoped tests, checks and development build are done; the button was visually verified in the reopened app.

23. [Polish application text and visual hierarchy](../offline-supervised-editor/issues/23-gui-frontend-polish.md). Owner confirmed the full frontend polish brief on 2026-10-09: restrained dark styling, clearer typography and plain language throughout, contextual gesture help and useful layout changes. Dependencies 16/21 are done; claimed by Codex. No audio behavior changes or release qualification.


20. [Add editor-first documentation visuals after the GUI is settled](../offline-supervised-editor/issues/20-editor-first-documentation-visuals.md). Owner authorized screenshots from the open development build on 2026-10-09; README/workflow screenshot pass is implemented and the ticket remains claimed. Wider labeled examples, bleed, S-shaped fades and gain-composition visuals remain pending; interactive delivery needs triage. Not a new release requirement.

17. [Add an advanced training panel](../offline-supervised-editor/issues/17-advanced-training-panel.md). Requested on 2026-10-09 for future work only. Needs triage and explicit implementation authorization; it is not part of current GUI work or a new release requirement.

## Adjustable shortcut follow-up

The owner extended ticket 41 with Center playhead and Lock / Unlock keyboard access and delayed shortcut tooltips on command controls. Include these in the same binding definitions, Settings, reference and acceptance checks.

Ticket 41 also preserves lock across user zoom/fit actions by default and adds an off-by-default `Unlock playhead when zooming` local preference. Cover every distinct zoom entry point and both lock states in acceptance checks.

2026-10-12 follow-up: [ticket 41 — Adjustable keyboard shortcuts](../offline-supervised-editor/issues/41-adjustable-keyboard-shortcuts.md) is specification-ready and blocked by 31, 34, 37 and 39. Add S as the default for saving examples, local remapping with context-aware conflicts and matching shortcut hints on `codex/editor-workflow-31-37`. This ticket is not implemented; dependency completion and scoped acceptance checks still govern execution.

## Existing work reconciliation

Ownership `6a60cc2` contains the responsive editor plus later calibration/cache/safety work. Prior ticket statuses are inconsistent with implemented code: six MVP implementation tickets are marked done, while responsive and guided-ownership tickets include ready/triage states despite existing implementation/evidence. Establish-baseline work reconciles their records and carries acceptance gaps forward; do not rebuild imports, transport, waveform caching, safe output handling or protection mechanics merely because an old issue is open.

The owner requested repository cleanup on 2026-10-08. The primary checkout now contains this developed code and the new roadmap on `codex/offline-supervised`. Superseded issue folders and branches have been removed; their history/records are in the external recovery snapshot. Continue only these fifteen build specifications. Ticket 01 records completed baseline checks and editor launch; later quality and release acceptance remain pending.

The historical ownership detector is a comparison baseline, not accepted truth. Preserve rejected experiments and reports as provenance, outside the shipping implementation. No build ticket is complete until its scoped checks pass.
