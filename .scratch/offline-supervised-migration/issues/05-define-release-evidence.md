# Define evidence required to accept supervised ownership

Type: grilling
Labels: wayfinder:grilling
Status: resolved
Assignee: Codex with repository owner
Parent: ../map.md
Blocked by: 01

## Question

What held-out and listening evidence must demonstrate that the supervised editor preserves local speech and overlap while reducing bleed sufficiently to replace the current detector?

Existing ownership evidence does not establish accepted success. Define independent labeled evaluation regions, leakage-resistant splits, speech-preservation and bleed measures, conservative uncertainty behavior, transition listening checks, long-session resource budgets and cross-platform expectations. Include source/output integrity, cancellation and save/reload acceptance. Decide dataset permissions and which existing release-gate checks remain required. Thresholds and priority tradeoffs require the human's decision.

## Comments

2026-10-08 human priorities: protect wanted speech and overlap first; preserve uncertain regions at unity for review. Release scope is Windows, eight dedicated mono tracks and up to two hours; macOS/Linux follow later. Nemotron is mandatory and its model is downloaded once on first use. Release evidence must include CPU-only inference, offline rerun after model installation and long-recording probability retention. Exact performance and bleed thresholds remain to be settled without claiming unmeasured results.

## Answer

2026-10-08: The owner accepted release gates of no detected wanted-speech/overlap loss on independent labeled material, measurable bleed improvement against the frozen current detector, and owner listening approval. Check exact effective gain on wanted samples; deliberate manual edits are separately identified. Independent labels cannot become training references or forced optimizer truth. Freeze tuning first and retain the historical failure reports as development evidence, not unseen acceptance.

Windows is the first release. Test advertised codec profiles/mixed rates, output sample/rate/metadata integrity, preview/export parity, protected edits/undo/save/reload, relinking, invalid inputs, stale/cancelled completions and required-model errors. Eight-speaker/two-hour CPU runs must retain complete Nemotron evidence, including after probability compaction and long-gap recurrence. Test clean native installation and network-disabled use after first download without Python/GPU requirements.

Measure cold analysis/model load, memory, cache growth, cancellation and warm interaction on the Ryzen 9 3950X/32 GB development machine. The owner is unsure about acceptable cold wait, so no invented hard deadline is imposed. Owner timing acceptance of measured results is a release gate. Resource exhaustion, unusable transport, wanted-speech damage or failed listening approval block release. This settles what evidence is needed; it does not claim the evidence exists. See [release acceptance in the specification](../spec.md) and [evaluation/qualification tickets](../implementation.md).
