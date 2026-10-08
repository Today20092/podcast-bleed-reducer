# Introduce a source-aware timebase and versioned project contract

Status: ready-for-agent
Blocked by: 01
Spec: ../spec.md

## Scope

Make source sample rate/frame count explicit and use a checked common integer time coordinate for labels, regions, protections and editable envelopes. Define stable track/speaker IDs and one dedicated track per speaker, with up to eight speakers. Convert native sample indices through their own rates; remove implicit 48 kHz coordinate contracts. Preserve half-open intervals and rounding rules that cannot shorten preserved speech.

Create the fresh versioned project contract with source bindings, speaker map, active-speaker-set reference labels, settings, manual overrides/protections, review/result/render identity and recipe provenance. No old-project conversion is required. Validate on load/save, save atomically, and distinguish a moved identical source from changed audio. Bound labels/edits on changed sources remain recorded but cannot silently apply until explicitly reviewed/rebound. Cache loss must not make a project uneditable.

## Reach

src/audio_io.rs Stem metadata, src/calibration.rs CalibrationExample/OwnershipSettings, src/project.rs Project/open/save/source invalidation, src/engine.rs Region/Protection/GainEnvelope, src/timeline.rs and app/waveform/quality coordinate consumers. Update existing src/bin/{ownership_diagnose,ownership_evaluate,preview_benchmark,release_gate}.rs callers, exports and fixtures using fixed 48,000 conversions or old sample-index assumptions.

## Acceptance

- Equal physical timestamps at 44.1/48/96 kHz map correctly, including first/last samples and large two-hour indices.
- Malformed versions/IDs, contradictory labels, invalid bounds/nonfinite values and checked-arithmetic overflow fail safely.
- Track reordering and relinking cannot transfer a speaker, label or override to different audio.
- Atomic save failure retains prior project data; save/reload retains confirmed human decisions.
- Current callers compile against one consistent contract rather than parallel incompatible timebases.
