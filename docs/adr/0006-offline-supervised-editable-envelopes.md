# Use supervised offline ownership and protected editable envelopes

Status: accepted 2026-10-08, amended by owner decisions on 2026-10-09 and 2026-10-20. The acoustic editor pipeline is implemented; the owner accepts current sound and performance. Formal independent/long-session measurements remain unperformed rather than an active qualification ticket.

## Current decision

CleanSlate retains its Rust/Iced editor and uses human-reference session calibration, embedded Silero speech detection and independent per-track logistic regression ownership models. Original native-rate sources remain separate from 16 kHz analysis copies. Speech presence is one input to local ownership, not proof that a microphone's assigned person is speaking. Independent estimates allow overlap.

Each track uses 0.8/0.2 ownership hysteresis. Intermediate estimates retain the previous decision; missing or invalid evidence receives unity protection. Exact human labels constrain every marked range after statistical decisions. The current editor has no uncertainty review queue. Users inspect waveforms and compare Original, Processed and Removed, then add labels or manual corrections.

Manual gain edits remain authoritative above regenerated automatic envelopes. Automatic fades use finite quintic S-shaped curves with exact endpoints and zero endpoint slope and curvature. Manual points retain linear interpolation. The live minimum-gap control operates during gain generation and cannot erase explicit human absence. The shared effective-envelope evaluator supplies display, audition and final native-rate rendering.

A source-aware versioned project binds references, manual edits and results to the recording. Analyze is explicit after label changes; gain changes update immediately without retraining. Stale analysis blocks final export. Source validation, cancellation, safe output publication and source preservation remain required.

Nemotron is removed from the application, GUI, dependencies and package. No diarization download, transcript or voice-identity embedding is required. Acoustic analysis runs locally with the included speech detector.

## Reasons and remaining gates

Dedicated microphones and human examples let the application learn this recording's direct-speech and bleed relationships. The removed Nemotron experiment did not establish a benefit that justified retaining its integration. These decisions address known bleed failures while keeping import, transport, caching and safe outputs.

Model estimates and numerical tests do not establish recording quality. On 2026-10-20 the owner accepted current sound/performance and explicitly closed 14 without requiring its remaining formal independent comparisons/benchmarks. Ticket 15 is retired into the [per-release checklist](../branch-workflow.md#release-checklist). Existing evidence limits and the quiet-overlap case remain recorded; owner acceptance is not a claim that unperformed tests passed. Public alpha packages remain experimental.

## Decision history

The initial 2026-10-08 target required Nemotron. The first 2026-10-09 amendment made it optional; the later owner override removed it entirely. Those earlier modes are superseded and recoverable in Git history and the [research note](../research/nemotron.md).

This decision supersedes earlier read-only envelopes, asymptotic-only automatic fades and optional-reference categorical inference. Later owner refinements replace the review queue with direct waveform/audition review and move gap handling into live gain generation.

See the [accepted specification and its amendments](../../.scratch/offline-supervised-migration/spec.md), [decision map](../../.scratch/offline-supervised-migration/map.md), [reader explanation](../ownership-explained.md), [model recipe](../supervised-ownership-models.md) and [temporal policy](../temporal-ownership.md).
