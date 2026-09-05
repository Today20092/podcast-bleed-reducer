# Responsive preview and audio review improvements

Status: ready-for-agent

## Problem Statement

The 83-minute three-stem reference session takes 91.71 seconds to preview after a parameter adjustment: 3.74 seconds loading, 70.88 seconds analysis, 16.99 seconds gain construction. Slider changes only commit on release. Obsolete jobs continue consuming resources. The user wants every recommendation from the architecture review addressed, separate tasks/worktrees, automatic design decisions, and one integrated branch with a runnable Windows EXE.

## Solution

Prepare source analysis once, derive warm previews from retained facts, construct exact gain curves at target changes, and coalesce live adjustments. Correct control semantics and waveform representation, improve playback preparation, and evaluate detector preprocessing and bleed evidence without silently weakening preservation.

## User Stories

1. As an editor, I can adjust attenuation and Open/Close without rerunning speech detection.
2. As an editor, I can adjust sensitivity, bleed discrimination and padding using existing detector evidence.
3. As an editor, I see changes during dragging rather than waiting for release.
4. As an editor, my newest settings win even while older work completes.
5. As an editor, I keep hearing and seeing the last completed preview during source preparation.
6. As an editor, protections remain effective and preserve the established half-open interval and one-pole behavior.
7. As an editor, changing or relinking a source invalidates obsolete analysis.
8. As an editor, higher displayed sensitivity admits weaker speech while old projects retain their sound.
9. As an editor, padding steps reflect the actual 32 ms resolution.
10. As an editor, bleed controls explain their per-track noise-floor meaning and do not imply unsupported inter-microphone ranking.
11. As an editor, processed waveform summaries represent changing gain within their display buckets honestly.
12. As an editor, starting audition avoids blocking the editor on multi-gigabyte reads.
13. As an editor, playback uses efficient shared envelope evaluation without changing monitoring or source-minus-processed behavior.
14. As an editor, initial preparation reports meaningful stages, and warm edits avoid a full analysis progress cycle.
15. As a maintainer, I have reproducible benchmarks, equivalence tests, and a detection-quality evaluation instead of unverified claims.
16. As the user, I receive one branch and one identified EXE containing all accepted changes.

## Implementation Decisions

User explicitly delegates all grilling answers, testing seam choices, ticket breakdown, branch creation, commits and integration. No additional interview is required.

- Base: accepted corrections c268882. Integrate into codex/cleanslate-responsive, keep main untouched until the user tries this branch.
- Preserve the shared GainEnvelope representation and one-pole semantics; optimize construction from sorted ownership/protection edges. Keep an old-algorithm oracle in tests where useful.
- Retain probabilities before thresholding, energies, noise floors, sample counts and ordered source identities in one concrete prepared-analysis module. Memory cache first. Avoid retaining multi-gigabyte PCM solely for analysis.
- Reuse source-stamp concepts; same-path changes must invalidate. Detector/preprocessing changes require versioned identity. Final render still performs trust-boundary validation.
- Maintain compatibility entry points for current render/preview callers. New cache-aware entry point may sit beside a cold one until callers migrate.
- Scheduling owns one active job, newest pending request, cancellation and generation validation. Coalesce changed values and install current results promptly. No job per drag event.
- Keep stored speech_sensitivity as its legacy probability threshold. Display sensitivity as its inverse and convert at the UI seam, avoiding silent stored-schema changes. Report output continues identifying the actual threshold.
- Processed display should sample actual source/gain at an appropriate cache resolution or provide a conservative gain-aware bucket summary, clearly documenting approximation; no false exactness.
- Audition preparation moves off the UI thread. Reuse appropriately bounded or shared audio resources; stream if that is simpler than retaining duplicate PCM. Remove per-output-sample synchronization overhead and use a sequential envelope cursor where verified equivalent.
- Detection-quality recommendations are evaluations plus implementable improvements: add/test proper anti-alias preprocessing and compare against current detector on labeled evidence. Do not silently replace defaults without preservation/overlap evidence. If labels/listening are insufficient, keep legacy default, expose an explicit persisted experimental opt-in if needed, and document exact evidence and unresolved tradeoff. Cross-microphone discrimination likewise requires measured examples; an evaluation finding with a runnable comparison satisfies the original recommendation to evaluate before changing competition.
- Keep Iced and existing dependencies. Avoid generic cache frameworks, speculative adapter hierarchies, detector/GPU replacement, or unrelated refactors.

## Testing Decisions

Approved seams: gain evaluation via the shared envelope; cold/warm preview derivation with source identity; editor request lifecycle; saved parameter round-trip; waveform bucket representation; playback preparation and sample output; end-to-end render/release gate. Test behavior at those seams, including stale generations, superseded jobs, errors, and cancellation.

Each implementation task uses red-green slices, relevant formatting/Clippy/tests, and standards/spec review before committing. Integration runs complete checks and benchmarks the actual three sources read-only. Warm target: below 100 ms initially, report actual median/p95 and do not fake a target pass. Human listening cannot be claimed by an agent.

## Out of Scope

Framework replacement, DAW plug-in format, GPU detector replacement, remote deployment, destructive branch cleanup, duplicating supplied audio, and claiming a new human listening sign-off.

## Further Notes

Actual test inputs, in order: C:/Users/User/Downloads/chopshow podcast 155_A03.wav; C:/Users/User/Downloads/chopshow podcast 155_A02.wav; C:/Users/User/Downloads/chopshow podcast 155_A01.wav. Each is 239660221 samples, mono 48000 Hz, 32-bit.

Architecture review: C:/Users/User/AppData/Local/Temp/architecture-review-cleanslate-20260905.html. Timing evidence: C:/Users/User/AppData/Local/Temp/cleanslate-real-preview-timing.json. Original timing harness: C:/Users/User/AppData/Local/Temp/cleanslate-preview-profile. Treat these as evidence; this spec controls implementation.
