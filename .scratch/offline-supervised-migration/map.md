# Plan the offline supervised CleanSlate migration

Labels: wayfinder:map
Status: complete

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
- [Define protected edits and reproducible project persistence](issues/04-define-edit-persistence-contract.md): edits persist above generated gain; exponential-shaped finite fades and explicit Analyze use a fresh project format.
- [Define evidence required to accept supervised ownership](issues/05-define-release-evidence.md): independent preservation/bleed checks, Windows long-session evidence and owner listening/timing approval gate release.
- [Turn the settled architecture into migration tickets](issues/06-plan-implementation-sequence.md): fifteen scoped build specifications with dependencies and acceptance checks are ready.

## Not yet specified

None blocking implementation planning. Numerical tuning, acoustic estimator/downmix qualification, measured speed and final audio quality have explicit development/release checks in the build tickets; they have not been measured or accepted by this map.

## Out of scope

- Live processing and DAW plugins in the first release.
- Automatic alignment in this migration.
- Mixed-master and DAW-automation exports in the first release.
- Production code changes during this planning effort.
- Fade-shape selector, other classifier families, blind source separation and neural fine-tuning in the first release.
- Shared microphones, speaker track switching, more than eight speakers and macOS/Linux release qualification in the first release.

## Handoff

The [accepted architecture](spec.md), [documentation findings](feasibility-notes.md) and [implementation sequence](implementation.md) accompany the decision resolutions. All six planning tickets are resolved. All fifteen build specifications are ready; dependency completion still governs execution. Start with [Establish the accepted development baseline](../offline-supervised-editor/issues/01-establish-baseline.md).
