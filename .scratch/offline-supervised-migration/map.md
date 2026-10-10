# Plan the offline supervised CleanSlate migration

Labels: wayfinder:map
Status: complete

2026-10-09 owner override: acoustic-only analysis supersedes the required/optional
Nemotron decisions below. Nemotron is removed, with a research note retained.
Ticket 25 prepares the explicitly unqualified public Windows portable alpha;
tickets 14/15 remain the stable quality and long-session qualification gates.

2026-10-09 completion update: build ticket 16 is done after owner workflow acceptance, with implementation integrated in 6991d39. Preparation timing remains unmeasured in this owner pass; no numerical performance qualification is claimed. Historical claimed/pending notes below are superseded by the completion record in ticket 16. Continue ticket 14 quality evaluation before ticket 15 release qualification.

## Destination

Settle the architecture and produce ordered, implementation-ready tickets to migrate CleanSlate into a standalone offline, human-supervised podcast editor with editable gain envelopes.

## Notes

- Planning only. Do not implement or integrate branches during charting.
- The owner explicitly requested completing all six planning tickets in this discussion and confirmed the final architecture/build plan on 2026-10-08. Implementation remains a separate effort.
- Use wayfinder, grilling, and domain-modeling. Fetch current primary documentation through Context7 before selecting libraries or models.
- Source: [Automix plugin options](https://chatgpt.com/share/6ac7afc8-6b38-83e9-b638-a82652687295). An account chat with the same title was read in the ChatGPT project Automix plugin, conversation `6ac6a09c-637c-83ea-8719-7bf750c9c3d7`. The share endpoint exposed its title only; exact share-to-account identity was not independently verified.
- Human-confirmed scope on 2026-10-08: architecture and migration tickets; standalone offline editor; synchronized inputs with validation; individual processed WAVs and an editable project.
- Human-confirmed behavior: speaker labels train the session model; manual gain edits remain protected across reanalysis; probabilities inside the engine, understandable uncertainty in the UI, and editable envelopes replace the existing categorical/read-only constraints.
- Continue in this repository. The owner selected the newer ownership development baseline and retained Iced; resolution detail lives in the linked decision tickets.
- No saved CleanSlate projects need migration; use a fresh format. Baseline, toolkit and release scope are settled in the decision resolutions.
- Repository cleanup on 2026-10-08 consolidated the developed editor and this map into the primary checkout on `codex/offline-supervised`. `main` shares the cleaned starting point. Twenty-two old branches, three old worktrees, superseded roadmap folders and packaged builds were removed from the active workspace after an external recovery snapshot.
- Only this map and `.scratch/offline-supervised-editor/` are active. Historical implementation/acceptance distinctions are summarized in the implementation index; older records are recoverable outside the project, not competing plans.
- Reuse candidates include audio_io, waveform/timeline, audition, prepared_analysis, project, review, render/export, and quality evaluation. Existing ownership-quality evidence is developmental, not accepted release evidence.
- This tracker uses one child file per decision, Status and Type fields, and Blocked by references. Open children are the frontier; the map does not list them.

## Decisions so far

- [Select and preserve the migration baseline](issues/01-select-migration-baseline.md): reuse the ownership development branch, start fresh project files, and keep rejected experiments separate.
- [Decide whether to retain Iced or migrate to egui](issues/02-select-editor-toolkit.md): keep Iced and the existing editor; validate new interactions before reconsidering the toolkit.
- [Define labels and the offline session-analysis contract](issues/03-define-session-analysis-contract.md): required Nemotron and human references feed independent local-ownership models; source rates remain intact.
- [Define protected edits and reproducible project persistence](issues/04-define-edit-persistence-contract.md): edits persist above generated gain; finite fades and explicit Analyze use a fresh project format. The 2026-10-09 owner refinement replaces exponential automatic fades with quintic S-shaped fades in [ticket 18](../offline-supervised-editor/issues/18-s-shaped-automatic-fades.md), preserving exact endpoints and speech unity; transparency requires listening approval.
- [Define evidence required to accept supervised ownership](issues/05-define-release-evidence.md): independent preservation/bleed checks, Windows long-session evidence and owner listening/timing approval gate release.
- [Turn the settled architecture into migration tickets](issues/06-plan-implementation-sequence.md): fifteen scoped build specifications with dependencies and acceptance checks are ready.
- GUI decisions [07](issues/07-audio-waveform-cache.md), [08](issues/08-model-status-controls.md), [09](issues/09-timing-playback-readiness.md), [10](issues/10-direct-track-labeling.md) and [11](issues/11-editor-workflow-guidance.md) are resolved: shared waveform reuse, early playback, immediate track labeling and concise controls feed one [build ticket 16](../offline-supervised-editor/issues/16-gui-editing-workflow.md).
- [Attenuation policy decision 12](issues/12-attenuation-policy.md) is resolved as an evaluation direction: acoustic-first supervised ownership with optional Nemotron, representative labels and adjustable attenuation extends existing ticket 14. See [confirmed design and diagrams](gui-design-summary.md); shipping model/quality requirements remain until qualified adoption.

## Not yet specified

Owner GUI feedback on 2026-10-09 is documented in [the interview record](gui-feedback.md). Decisions 07–12 are resolved and consolidated into one new GUI build ticket 16 plus existing evaluation ticket 14. The owner subsequently authorized implement-spec, a fast GUI candidate and synchronized 10-minute source clips. Ticket 16 is claimed; release/listening acceptance remains pending.

The owner confirmed [the GUI and attenuation design](gui-design-summary.md) and authorized implementation. The accepted specification now uses acoustic-first analysis with optional Nemotron defaulting off. This contract change does not establish recording quality or release approval; those checks remain in ticket 14.

None blocking implementation planning. Numerical tuning, acoustic estimator/downmix qualification, measured speed and final audio quality have explicit development/release checks in the build tickets; they have not been measured or accepted by this map.

## Out of scope

- Live processing and DAW plugins in the first release.
- Automatic alignment in this migration.
- Mixed-master and DAW-automation exports in the first release.
- Fade-shape selector, other classifier families, blind source separation and neural fine-tuning in the first release.
- Shared microphones, speaker track switching, more than eight speakers and macOS/Linux release qualification in the first release.

## Handoff

Owner confirmed and authorized [full GUI frontend polish](../offline-supervised-editor/issues/23-gui-frontend-polish.md) on 2026-10-09 after the grilling interview. Use Refactoring UI principles for a clean dark audio editor, improve text and action hierarchy across all screens, preserve audio behavior and semantic timeline colors, and review the first pass in the development application. Ticket 23 is claimed; documentation screenshots remain deferred until the owner settles the appearance.

Owner confirmed and authorized [multichannel WAV import](../offline-supervised-editor/issues/21-multichannel-wav-import.md) on 2026-10-09 after a grill-me interview. Import selected recorder channels as editable mono tracks referencing the original container, retain recorder names with renaming, allow mixed mono inputs, preserve synchronization and the eight-speaker limit, and write no split WAVs on import. Ticket 21 is done, with 124 scoped checks plus the owner's complete Zoom WAV, strict Clippy and final GUI build passing. Listening and release acceptance remain separate.

The [accepted architecture](spec.md), [documentation findings](feasibility-notes.md) and [implementation sequence](implementation.md) accompany the decision resolutions. All twelve planning decisions are resolved. Build tickets 01–13 are done; 14 remains claimed, 15 remains blocked, and the single added GUI ticket 16 is claimed following owner authorization. Complete ticket 16 and owner GUI checks, then continue ticket 14's acoustic-first/quality evaluation before release qualification. Dependency completion still governs execution.
