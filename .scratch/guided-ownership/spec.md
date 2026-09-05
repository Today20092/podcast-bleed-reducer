# Guided cross-microphone ownership

Status: ready-for-agent

## Problem Statement

Speech bleed triggers independent voice detection and keeps microphones open. The user cannot remove obvious copied speech through the current controls. They want better joint decisions with useful human calibration while retaining responsive previews and a single runnable application.

## Solution

Compare synchronized microphones using calibrated speech-band levels and delay-aware copy evidence, preserve independent overlap, and expose optional room-tone and solo-speech examples in the editor. Apply identical decisions to preview, audition and final render. The user remains in control of calibration and preservation.

## User Stories

1. As an editor, I want obvious quieter copies of another microphone's speech attenuated.
2. As an editor, I want quiet direct speech preserved even when another participant is louder.
3. As an editor, I want interruptions, laughter and simultaneous speakers preserved.
4. As an editor, I want initial automatic estimates without a mandatory calibration wizard.
5. As an editor, I want to select room tone on a particular track to teach its background level.
6. As an editor, I want to select a solo speaker interval to teach direct-to-bleed relationships across microphones.
7. As an editor, I want understandable instructions distinguishing those two examples.
8. As an editor, I want to see and remove saved examples and replace mistaken examples.
9. As an editor, I want examples and detector mode retained across save/reopen.
10. As an editor, I want invalid or stale calibration rejected when source tracks change.
11. As an editor, I want joint ownership enabled on new projects and an explicit opt-in for legacy projects.
12. As an editor, I want calibration and parameter edits to update preview using cached evidence.
13. As an editor, I want the last completed preview retained during preparation and obsolete work prevented from installing.
14. As an editor, I want protections honored and final render to agree with preview and audition.
15. As an editor, I want controls to state what they actually change.
16. As a maintainer, I want reproducible quality tests covering delayed bleed, gain mismatch, polarity, genuine overlap and source changes.
17. As a maintainer, I want measurements on the supplied recordings, with calibration and scored spans kept separate.
18. As the user, I want one identified Windows EXE and instructions, ready for me to open and listen.

## Implementation Decisions

- One concrete ownership settings record stores enabled mode, typed room-tone/solo-speech ranges, and ordered source stamps. Calibration ranges are metadata, separate from protections.
- New projects enable joint mode; absent settings deserialize to legacy. Keep existing public cold/legacy entry points compatible. Add cache-aware and render entry points accepting ownership settings.
- Room tone measures the selected microphone's background; solo speech learns its direct level and the simultaneous leakage into other microphones. Recording gain differences and ambiguity are explicit.
- Prepare compact speech-band and pairwise delay/similarity evidence once alongside current source facts. New calibration or thresholds reuse it; source/model/preprocessing changes invalidate it.
- Joint ownership combines calibrated relative level and credible copy evidence. Independent residual speech/energy and ambiguous evidence preserve additional owners. Use conservative temporal behavior and the shared existing gain envelopes.
- No source subtraction, source resynthesis, full Dugan gain sharing, forced single winner, or automatic speech-level normalization.
- Preview cache keys include settings/calibration as appropriate; immutable preparation is distinct from derived ownership. Playback continues matching the completed preview's source identity.
- Editor selections can create examples without silently creating protections. Provide an explicit calibration selection mode if necessary; retain existing Shift-drag protection workflow and keyboard accessibility.
- Calibration changes mark renders stale, refresh preview, and survive reopening; examples bound to changed sources are invalidated with an understandable message.
- Reports identify the actual detector mode/settings and truthful evidence reasons. Preserve the categorical ownership vocabulary rather than a user-facing fabricated confidence percentage.
- Preserve Rust/Iced and existing dependencies. Numerical classifier thresholds are settled by runnable evidence and documented, not guessed as production facts.

## Testing Decisions

Use TDD at existing project/editor/preview/render seams. Tests demonstrate changed behavior rather than mirror helper internals. Cover serialized legacy compatibility, invalid ranges and sources, delayed and polarity-inverted copied speech, microphone gain mismatch, quiet overlap and laughter, cache reuse/cancellation, and final output integrity. Independent Standards and Spec reviews must be clean after integration.

Use the three original full stems read-only, plus the existing36 labels on the original excerpts. Treat screenshot-inferred ranges as provisional labels and state their provenance. Measure bleed RMS attenuation and wanted-speech gain changes separately, including exact-unity counts and meaningful gain-depth duration relative to legacy. Keep calibration/training intervals separate from evaluation. A good outcome needs actual quality improvement without sacrificing overlap or warm responsiveness; if a particular aggressive rule fails, revise it rather than merely hiding behind uncertain labels.

## Out of Scope

Neural source separation, transcription, automatic loudness leveling, DAW plugin packaging, cloud service deployment, destructive cleanup of old branches, and claiming human listening acceptance.

## Further Notes

The user explicitly delegates all grilling answers, test seam choices, ticket decomposition and integration decisions. The decisions record captures those answers and exact references. Deliver a test build for the user to open; do not launch Computer Use.
