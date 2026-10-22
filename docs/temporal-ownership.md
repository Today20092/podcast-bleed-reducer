# Ownership decisions over time

This stage follows the independent [supervised ownership models](supervised-ownership-models.md). It turns per-frame estimates into track states, then applies every current human reference over its exact time range. Gain settings and manual edits are later inputs to the [effective envelope](effective-envelopes.md). Start with the [reader explanation](ownership-explained.md#5-ownership-becomes-a-gain-curve).

## Current acoustic editor policy

The current editor supplies acoustic ownership estimates and raw Silero evidence, with no assigned Nemotron activity. `TemporalEvidenceGrid::acoustic_only` selects the acoustic branch. Each track begins absent and uses hysteresis:

| Evidence | Decision |
| --- | --- |
| Ownership estimate at least 0.8 | Enter `Owned` |
| Ownership estimate at most 0.2 | Enter `Absent` |
| Estimate strictly between 0.2 and 0.8 | Keep the prior owned/absent state |
| Missing estimate, missing raw speech or explicit preservation reason | Preserve as `UncertainUnity` |

Intermediate estimates alone do not become uncertainty. Raw Silero is a model input and must be present, but the acoustic temporal branch does not additionally require its score to exceed 0.5. There is no winner selection or minimum speech duration. Different tracks may own the same frame. An empty owner set in this acoustic branch does not trigger the old global-speech preservation rule.

The general temporal defaults include a two-frame gap guard, but `EditorAnalysisWorker` explicitly sets `preserved_gap_frames` to zero. The editor's **Ignore gaps shorter than** control instead bridges bounded automatic gaps during gain generation, with a 768 ms default. Gain changes reuse the ownership evidence; explicit human absence is not bridged. This separates detection from the user's sound settings.

## Exact human constraints

All current reference intervals apply after statistical decisions, including frames omitted by training sampling. Their exact half-open source-time boundaries split the timeline. Solo and overlap labels mark the selected speakers `Owned` and the other tracks `Absent`; room-tone truth makes every track absent. Consistent overlapping labels are united, contradictory labels block installation, and statistical warnings are clipped out of current human-truth intervals.

These labels are both training examples and direct constraints. A mislabeled section can therefore force a wrong decision inside its own range. Manual gain edits remain a separate, intentional correction layer.

## Identity and failure boundaries

`TemporalOwnershipCache::optimize` consumes the source-bound supervised result. Project source order, duration, synchronization, reference bindings, evidence identity, settings and the temporal recipe must match. Source fingerprints are checked before computation and again before installation. Changed labels cannot be applied to an old supervised result. Cancellation or validation failure leaves the previous completed result intact.

The result retains probabilities, per-track states, exact regions and grouped preservation warnings. The current GUI lets users inspect waveforms and audition directly; it has no uncertainty review queue. Internal uncertainty protection remains. Owned and uncertain regions stay at unity in automatic gain, while absent regions are eligible for attenuation. Manual constraints are composed afterward.

## Historical policy and validation

The retained non-acoustic branch supports controlled development fixtures from the retired pipeline. It uses assigned speaker activity, raw speech, global activity and conservative intermediate uncertainty. Those conditions and its two-frame guard do not describe current editor Analyze behavior. The recipe name `independent-bands-consistent-activity-gap-preservation-v1` is shared, so the acoustic flag and settings also participate in identity.

Tests cover acoustic hysteresis, optional bounded gap preservation, exact off-grid human boundaries, room truth, contradictory references, source identity, cache invalidation and cancellation. They establish mechanics, not independent wanted-speech/overlap quality. The owner accepted current sound/performance and closed 14 on 2026-10-20; unperformed formal measurements are optional future work. Ordinary package checks from retired 15 continue in the [per-release checklist](branch-workflow.md#release-checklist).

Source: [temporal decisions and human constraints](../src/temporal_ownership.rs), [editor settings](../src/editor_workflow.rs), [automatic gain generation](../src/effective_envelope.rs).
