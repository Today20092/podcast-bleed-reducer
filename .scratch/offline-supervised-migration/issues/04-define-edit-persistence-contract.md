# Define protected edits and reproducible project persistence

Type: grilling
Labels: wayfinder:grilling
Status: resolved
Assignee: Codex with repository owner
Parent: ../map.md
Blocked by: 01, 03

## Question

How do human training labels, existing unity protections, generated envelopes and protected manual gain edits coexist across reanalysis, parameter changes, saving and relinking?

The user requires training labels and protected gain edits. Define precedence and what happens when an edit contradicts a new label. Specify undo, provenance, source identity checks, stale results and versioned project compatibility. Reopen the read-only gain-envelope preview contract explicitly. Preview, audition and individual WAV rendering must use the same effective envelope, while original files remain unchanged.

## Answer

2026-10-08: The owner confirmed source-bound manual gain edits remain authoritative until removed, including when later labels or analysis disagree. Labels teach detection; manual edits control sound. Keep automatic envelopes separate from manual points/segments and unity protections. Explicitly replace conflicting manual constraints; do not introduce invisible precedence. Provide draggable points/fades and quick Preserve/Attenuate actions, with grouped undo and atomic save/reload. The first release remains attenuation-only.

Keep exponential-shaped automatic fades with finite editable durations and exact endpoints; reach unity before preserved speech and keep insufficient short gaps preserved. A later fade-shape selector is deferred. This reopens read-only envelopes in ADR-0002 and asymptotic time-constant-only behavior in ADR-0004; preview/render separation and shared effective gain remain intact.

Use a fresh versioned project format with source IDs, labels, settings, overrides, review state, recipes and result/render identities. No old saved projects require conversion. Missing caches are rebuildable. Moving identical files permits relinking; changed audio requires revalidation/explicit review before bound decisions apply.

Label/source changes keep the last completed preview visibly stale and require explicit Analyze; final export is blocked until current. Gain/fade/manual edits regenerate effective gain immediately without model inference. Failed/cancelled/obsolete work preserves completed results, sources and prior valid deliverables. One evaluator drives waveform preview, audition, removed signal and native-rate render. Detailed rules are in [the accepted specification](../spec.md).
