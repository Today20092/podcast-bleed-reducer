# Use supervised offline ownership and protected editable envelopes

Status: accepted target architecture, 2026-10-08; implementation pending

2026-10-09 owner override: acoustic-first supervised analysis is now the default; Nemotron is an explicit optional mode, default off. Dedicated track assignments and human labels train every track and predict over the whole recording, independent of anonymous native speaker mapping. Aggressive acoustic decisions preserve explicit human speech/overlap and manual edits. Source validation, mode-bound result identity and independent listening/preservation/bleed release gates remain required. This supersedes this ADR's required-Nemotron default and blanket conservative acoustic uncertainty policy; see the specification's owner feedback addendum.

CleanSlate will retain its Rust/Iced editor and reuse the ownership development baseline, while replacing categorical-only inference with required local Nemotron evidence plus human-reference session ownership models. Original native-rate sources remain separate from 16 kHz analysis copies; independent speaker estimates allow overlap, and uncertain evidence preserves audio for review. This targets the known bleed failures while retaining useful import, transport, cache and safe-output infrastructure.

Manual gain edits remain authoritative above regenerated automatic envelopes, with exponential-shaped finite fades that reach exact endpoints. A fresh source-aware versioned project binds references, overrides and result identities to the recording; no legacy saved projects need conversion. Explicit Analyze follows label changes, while gain changes update immediately and stale analysis blocks final export.

This supersedes ADR-0002's read-only-envelope portion and ADR-0004's asymptotic-only transition contract for the migration target, and supersedes optional-reference/global-first categorical inference and old-project opt-in from ADR-0005 in the selected ownership baseline. Single-workspace design, live audition separate from final rendering, shared effective-envelope evaluation, source preservation and conservative uncertainty remain. The code can still implement the historical decisions until migration tickets complete.

The owner requires Nemotron for the Windows-first release, with a verified first-use model download and later offline CPU operation. The native runtime and model have different licenses. Two-hour/eight-speaker tests, independent wanted-speech/overlap preservation, measured bleed improvement and owner listening/timing approval gate release; model support documentation does not establish success.

See the [accepted specification](../../.scratch/offline-supervised-migration/spec.md) and [resolved decision map](../../.scratch/offline-supervised-migration/map.md).
