# Offline supervised editor implementation sequence

## Current queue — reconciled 2026-10-20

Owner follow-up accepts current sound/performance, closes 14, retires standing release ticket 15 into the per-release procedure and defers unnecessary waveform-overlay editor 35. There are now **5 active tickets and 2 deferred proposals**. The owner's full GUI implementation pass in 56 is done, its reviewed appearance is owner-accepted, and all completed application work is integrated into development at `6b99a9a`, with a tested/preserved executable. Broader workflow/device checks remain in 23. Owner-approved architecture ticket 55 is done and integrated. The earlier three-agent audit consolidated duplicate implementation records while retaining unmet checks; owner-waived formal benchmarks are now optional future work, with no new benchmark ticket created. Original specifications and results remain in place. This section governs current routing; dated entries below preserve history.

| Ticket | Current work | State |
| --- | --- | --- |
| [20](../offline-supervised-editor/issues/20-editor-first-documentation-visuals.md) | Gain composition and eleven-diagram browser pass completed; only screenshot refresh remains | Claimed; remaining captures wait for 23 |
| [22](../offline-supervised-editor/issues/22-local-storage-cleanup.md) | Git temporary files removed and integrity verified; only retired recovery-target cleanup remains | Claimed; separate old build target still exists |
| [23](../offline-supervised-editor/issues/23-gui-frontend-polish.md) | Integrated GUI acceptance plus B and full FabFilter-inspired pass in development; absorbs 38/39/40 | Claimed; reviewed appearance owner-accepted, integration verified; broader workflow/device checks remain |
| [41](../offline-supervised-editor/issues/41-adjustable-keyboard-shortcuts.md) | Adjustable shortcuts and associated playhead commands/tooltips | Claimed; implementation/review/full checks/build passed, owner GUI acceptance pending |
| [42](../offline-supervised-editor/issues/42-playhead-catch-up-scroll.md) | Catch-up scrolling | Unstarted; ticket unchanged by owner instruction |
| [56](../offline-supervised-editor/issues/56-fabfilter-inspired-gui.md) | Owner-selected B and all fourteen FabFilter-inspired GUI proposals in development | Done; owner-approved and merged; 311 tests/build/self-checks passed; temporary branches retired |

[17 — Advanced training panel](../offline-supervised-editor/issues/17-advanced-training-panel.md) and [35 — Waveform-overlay editing](../offline-supervised-editor/issues/35-waveform-automation-editor-design.md) are deferred outside the active queue. [14](../offline-supervised-editor/issues/14-independent-release-evaluation.md) is done on explicit owner acceptance; unperformed formal tests are not claimed as passing. **15/25 → [per-release checklist](../../docs/branch-workflow.md#release-checklist), 27 → 20, 38/39/40 → 23**. Ticket 41's former blocker 39 is removed because its prerequisite software is integrated; its remaining manual review is in 23. No release or main promotion is authorized. Automated Git-temp deletion was rejected, but subsequent owner-manual cleanup is verified: zero temporary objects remain and full Git integrity passed. Ticket 22 stays open only for the separate retired recovery target and final preservation checks.

Earlier audit validation: three read-only agent audits covered all 14 formerly open records; a separate review found no actionable issues. That earlier eight-ticket snapshot passed dependency/link/whitespace assertions; the owner decisions above supersede its count. Ticket 42's unchanged SHA-256 is `cd9e048a9cf97d0048271f793bd556fd24a0e00cf925741436c2739d2aa4bf31`. Application tests/builds and hardware acceptance were not rerun for these tracker decisions. Separate documentation implementation/browser checks are recorded in 20; cleanup integrity checks and rejections are recorded in 22.

## Historical implementation record

2026-10-20: [55 — Shared streaming waveform construction](../offline-supervised-editor/issues/55-streaming-waveform-construction.md) is done and merged into development at `102fee9`. The waveform module owns chunk ingestion, prefix refinement and cache completion for import and reopening. Two-axis review found no actionable issues. Workflow/packaging fixtures, formatting, strict Clippy, 280 tests (18 ignored), GUI build and native self-check passed. Test compilation required a command-only four-codegen-unit override after rustc memory failures; no project configuration changed. The tested executable/receipt are preserved outside the checkout, and temporary branches/worktree are removed. No unrelated ticket or release/main promotion is included.

2026-10-20: [54 — Shared local builds and development cleanup](../offline-supervised-editor/issues/54-local-build-consolidation.md) is done after shared-settings/workflow checks, strict Clippy, 278 passing tests (18 ignored), GUI build and native self-checks. Primary development is consolidated, old feature branches retired and the frozen baseline preserved. Use scripts/check.ps1 -Local and scripts/gui-dev.ps1 for the shared cache/profile. Local integration and batch PRs to main supersede the earlier mandatory per-feature-PR policy; one unused recovery cache remains a documented manual-cleanup exception.

2026-10-20: [51 — Removable media and export](../offline-supervised-editor/issues/51-removable-media-export.md) is done for the owner-approved direct SD-card workflow: local working storage, read-only originals, independent destination folders, contextual I/O errors and bounded read recovery. Full and final publication checks, strict Clippy, GUI build and native self-check passed. Physical SD interruption/write-lock and listening checks remain unclaimed; independent release qualification is unchanged.

Status: accepted implementation plan, 2026-10-08

2026-10-20 backlog reconciliation: [41 — Adjustable shortcuts](../offline-supervised-editor/issues/41-adjustable-keyboard-shortcuts.md) and [42 — Catch-up scrolling](../offline-supervised-editor/issues/42-playhead-catch-up-scroll.md) are accepted, unclaimed build specifications retained during development integration. Check each dependency before claiming; coordinate shared viewport changes, with ticket 42 taking precedence for locked middle-pan behavior. No implementation is claimed.

2026-10-20 owner follow-up: [53 — Development integration](../offline-supervised-editor/issues/53-development-integration.md) combines completed feature branches and records the shared development/release policy. Feature work starts from development; batch promotion to main and release publication remain separate steps.

2026-10-20 owner follow-up: [52 — Analysis progress feedback](../offline-supervised-editor/issues/52-analysis-progress-feedback.md) is done after the separate GUI build, 144 relevant tests, formatting/diff checks and strict all-target/all-feature Clippy passed. The supervised job reports preparation/speech-detection work and completed-stage timings. Full-recording speed and native visual/listening checks remain unmeasured; performance qualification in tickets 14/15 remains open.

2026-10-17 owner-requested documentation: [49 — README walkthrough video](../offline-supervised-editor/issues/49-readme-walkthrough-video.md) adds the newer transcript-reviewed video as the primary demo while retaining the alpha demo link. Documentation only; no application or release change.

2026-10-17 owner-approved identity: [48 — Application icon and GitHub branding](../offline-supervised-editor/issues/48-application-icon-branding.md) is done, with Windows GUI/resource checks, the embedded-icon test and packaging regressions passing. [PR 12](https://github.com/Today20092/podcast-bleed-reducer/pull/12) carries native application icons and the README logo; hosted checks gate integration. Audio behavior and release qualification remain unchanged.

2026-10-11 owner extension: [ticket 30](../offline-supervised-editor/issues/30-algorithm-documentation-coherence.md) aligns the README, reader explanation, model and temporal contracts, architecture and historical records with the current acoustic editor. This is documentation work; it changes no algorithm or release qualification.

2026-10-11 owner extension: [ticket 28](../offline-supervised-editor/issues/28-export-settings-multichannel-wav.md) is done for the agreed Export dialog, per-project encoding preference, separate or multichannel WAV delivery, size estimates, final integer dither and overload safeguards. Its ticket records code, tests and GUI checks. This completes export scope, not independent audio-quality or stable release qualification.

2026-10-11 owner extension: [ticket 29](../offline-supervised-editor/issues/29-contributor-documentation-intake.md) covers the README origin story, contributor guide, public roadmap and GitHub issue forms. The [public roadmap](../../ROADMAP.md) summarizes current priorities without replacing this tracker. Automatic synchronization and in-app support reporting are exploratory possibilities, not implementation commitments or release requirements.

2026-10-11 owner extension: [ticket 27](../offline-supervised-editor/issues/27-documentation-diagrams.md)
replaces all seven Mermaid documentation diagrams with GUI-matched SVGs and
accessible HTML sources. Static checks pass; browser review remains pending.

2026-10-10 owner extension: [ticket 26](../offline-supervised-editor/issues/26-cross-platform-release-packages.md)
is done: Windows, both Mac variants and Linux alpha 2 packages are public, with
verified direct README downloads (2026-10-11).
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


20. [Add editor-first documentation visuals after the GUI is settled](../offline-supervised-editor/issues/20-editor-first-documentation-visuals.md). README/workflow screenshots and the solo/overlap/room, bleed and S-shaped fade teaching figures are implemented. Remaining gain composition, gain-edit/preservation captures, interactive-delivery triage and the common viewer review (including former 27) are tracked together here. Not a new release requirement.

17. [Add an advanced training panel](../offline-supervised-editor/issues/17-advanced-training-panel.md). Requested on 2026-10-09 for future work only. Needs triage and explicit implementation authorization; it is not part of current GUI work or a new release requirement.

## Adjustable shortcut follow-up

The owner extended ticket 41 with Center playhead and Lock / Unlock keyboard access and delayed shortcut tooltips on command controls. Include these in the same binding definitions, Settings, reference and acceptance checks.

Ticket 41 also preserves lock across user zoom/fit actions by default and adds an off-by-default `Unlock playhead when zooming` local preference. Cover every distinct zoom entry point and both lock states in acceptance checks.

2026-10-20 dependency reconciliation: [ticket 41 — Adjustable keyboard shortcuts](../offline-supervised-editor/issues/41-adjustable-keyboard-shortcuts.md) remains specification-ready and unimplemented on development. Dependencies 31/34/37 are done; 39's prerequisite reference software is integrated and its manual checks now live in 23. Add S as the default for saving examples, local remapping with context-aware conflicts and matching shortcut hints when implementation is requested; scoped acceptance still governs completion.

## Existing work reconciliation

Ownership `6a60cc2` contains the responsive editor plus later calibration/cache/safety work. Prior ticket statuses are inconsistent with implemented code: six MVP implementation tickets are marked done, while responsive and guided-ownership tickets include ready/triage states despite existing implementation/evidence. Establish-baseline work reconciles their records and carries acceptance gaps forward; do not rebuild imports, transport, waveform caching, safe output handling or protection mechanics merely because an old issue is open.

The owner requested repository cleanup on 2026-10-08. The primary checkout now contains this developed code and the new roadmap on `codex/offline-supervised`. Superseded issue folders and branches have been removed; their history/records are in the external recovery snapshot. Continue only these fifteen build specifications. Ticket 01 records completed baseline checks and editor launch; later quality and release acceptance remain pending.

The historical ownership detector is a comparison baseline, not accepted truth. Preserve rejected experiments and reports as provenance, outside the shipping implementation. No build ticket is complete until its scoped checks pass.
