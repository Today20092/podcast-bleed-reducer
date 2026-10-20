# Plan the offline supervised CleanSlate migration

Labels: wayfinder:map
Status: complete

2026-10-20 owner-requested GUI follow-up: [42 — Animated playhead catch-up scrolling](../offline-supervised-editor/issues/42-playhead-catch-up-scroll.md#accepted-design) refines Page-at-edge navigation into a stationary viewport followed by a quick leftward slide when the playhead reaches the right edge, stopping near the left edge before repeating. Specification-ready and unclaimed; dependency 34 is done. Preserve existing lock/zoom behavior and coordinate with 41. This is a ticket only; no GUI implementation or verification is claimed.

2026-10-12 owner-requested roadmap proposal: a [guided tutorial and dedicated usage video](../../docs/roadmap-ideas.md#guided-tutorial-and-usage-video-2026-10-12) should teach the settled editor workflow, including a workflow chart, four- or five-track practice audio, solo/overlap/room-tone examples, analysis, listening review, corrective examples and reanalysis before export. Optional first-launch guidance and replay from Help need specification, practice-audio rights and exercise/completion decisions before a build ticket. This records future work only; no tutorial implementation or video production is claimed.

2026-10-12 owner-requested playback follow-up: [40 — Reusable selection playback](../offline-supervised-editor/issues/40-reusable-selection-playback.md#accepted-design) specifies measured warm-start improvement, prepared source/output-stream reuse, truthful status and preserved selection/monitoring/loop safety. Implementation waits for 38. Rust FOSS/installed-stack research is linked from the ticket; no playback implementation or latency acceptance is claimed.

2026-10-12 owner-approved UI follow-up: [39 — Tooltip removal and shortcut readability](../offline-supervised-editor/issues/39-shortcut-reference-readability.md#accepted-design) records removal of redundant sound-value hover instructions and an action-first reference with text keycaps, separate action rows, quieter conditions and clearer spacing/separators. Tooltip removal is already a working-tree edit; redesign and owner verification remain pending. No playback-preparation change or merge is claimed.

2026-10-12 owner-authorized branch refinement: [38 — Progressive import flow](../offline-supervised-editor/issues/38-progressive-import-flow.md#accepted-design) is one ready-for-agent implementation ticket on `codex/editor-workflow-31-37`. It covers early navigable timeline skeletons, whole-recording coarse previews, truthful progress, shared source work, queued imports, independent recording readiness, retry/cancellation and stable active playback. It supersedes ticket 33's blocking modal requirement after channel confirmation while preserving validation, synchronized inputs and safe persistence/export. The owner wants this branch refined before deciding to merge into `main`; no merge or implementation is claimed by this record.

2026-10-12 synchronization reconciliation: in [Research audio synchronization](codex://threads/01a12e35-39b2-7c62-8b6d-380b599fa5d8), the owner explicitly selected “Defer synchronization; users align in their editor”. This supersedes the invitation to start I01–I04 grilling below. Automatic/guided synchronization, drift correction, time stretching and drift checking are deferred; already-synchronized input remains required. The interview is closed, no synchronization build ticket is authorized, and [research](../../docs/research/synchronization-and-drift.md) is retained for a future concrete need.

2026-10-12 owner confirmed the settings/help and appearance interview and authorized [ticket 36](../offline-supervised-editor/issues/36-app-menu-help-about.md) and [ticket 37](../offline-supervised-editor/issues/37-semantic-dark-light-themes.md), with scoped specifications linked in the implementation tracker. Menu/help/About precedes persistent Dark/Light appearance; neither is implemented or claimed by this discussion. Refresh-rate controls, diagnostic reports and theme customization remain deferred. The owner requested starting synchronization I01–I04 grilling; that discussion does not yet amend the synchronized-input contract.

2026-10-11 owner-confirmed workflow extension: the [I05–I14 interview](../../docs/roadmap-ideas.md#interview-decisions-2026-10-11) is accepted for four scopes, and the owner authorized specs and tickets. [31](../offline-supervised-editor/issues/31-label-loop-audition.md) covers popup-bound loop audition and labeling guidance; [32](../offline-supervised-editor/issues/32-sound-value-entry.md) covers typed sound values, explicit mute and parameter reset; [33](../offline-supervised-editor/issues/33-focused-import-waveform.md) covers centered import and conditional gain-strip visibility; [34](../offline-supervised-editor/issues/34-shortcut-help-zoom.md) covers shortcut help, zoom and Escape deselection after 31. These are unclaimed build specifications, not implemented behavior. [35](../offline-supervised-editor/issues/35-waveform-automation-editor-design.md) is a separate open design ticket for waveform-overlay editing. Other inventory ideas remain exploratory. Release gates are unchanged.

2026-10-11 owner confirmed [export settings and optional multichannel WAV delivery](../offline-supervised-editor/issues/28-export-settings-multichannel-wav.md). One Export dialog selects destination, separate mono WAVs or one WAV with separate channels, and preserve-source/PCM24/float32 encoding. Remember encoding per project; preserve native grids, require an explicit common encoding for mixed-encoding multichannel delivery, disable that layout for mixed rates, show size estimates, apply final integer dither while preserving exact silence, and block integer overloads. Ticket 28 records the full agreed scope and checks; implementation remains pending.

2026-10-10 owner authorized future macOS and Linux release packages alongside
Windows and direct platform downloads in the README. Claimed build
[ticket 26](../offline-supervised-editor/issues/26-cross-platform-release-packages.md)
tracks this extension; stable qualification gates 14/15 remain unchanged.

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

2026-10-12 owner shortcut request: [ticket 41](../offline-supervised-editor/issues/41-adjustable-keyboard-shortcuts.md) specifies adjustable application shortcuts and S as the primary default for saving speaker examples, preserving popup ownership and project-save chords. It also includes Center playhead, Lock / Unlock keyboard access and small delayed-hover tooltips showing effective shortcuts. Continue on `codex/editor-workflow-31-37`; this request creates a ticket and does not authorize a main merge.

The [accepted architecture](spec.md), [documentation findings](feasibility-notes.md) and [implementation sequence](implementation.md) accompany the decision resolutions. All twelve planning decisions are resolved. Build tickets 01–13 are done; 14 remains claimed, 15 remains blocked, and the single added GUI ticket 16 is claimed following owner authorization. Complete ticket 16 and owner GUI checks, then continue ticket 14's acoustic-first/quality evaluation before release qualification. Dependency completion still governs execution.
