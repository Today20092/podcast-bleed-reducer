# Reuse waveform caches across projects

Type: grilling
Labels: wayfinder:grilling
Status: resolved
Parent: ../map.md
Blocked by: none

## Answer

2026-10-09: Owner confirmed the [consolidated design](../gui-design-summary.md). Earlier questions and recommendations below are interview history; the consolidated design is the final decision. Implementation is consolidated into [build ticket 16](../../offline-supervised-editor/issues/16-gui-editing-workflow.md); do not create a separate build ticket for this decision. No implementation has been performed for this decision.

## Question

Should each audio file have an adjacent waveform sidecar, or should a shared application cache reuse waveforms by validated source identity?

## Owner feedback

Opening the same tracks in another project should reuse their waveform data. The owner observed `261005-supervised.waveforms.json` and `CleanSlate.waveforms.json` and prefers caches attached to audio. See [feedback](../gui-feedback.md).

## Recommendation and pending decisions

2026-10-09 owner answer: requests a recommendation rather than selecting a location. Recommend a shared application cache keyed by audio identity, so projects reuse waveforms and source folders stay uncluttered. Confirm this recommendation before deciding identity, movement and invalidation details.

Prefer reuse by source identity. An adjacent sidecar fits the owner's preference for visible portable data; a shared cache avoids writes to source folders. Confirm storage preference first, then decide changed-source detection, moved-file reuse, read-only directories, stale versions, atomic writes and handling existing project caches. Never transfer project labels or edits merely because a waveform cache matches.

## Proposed acceptance questions

2026-10-09 second round: shared application cache accepted. Audio identity and reuse/invalidation behavior still need a concrete specification.

Can two projects reuse one valid waveform without repeating waveform decoding? Does changed audio invalidate it? Can missing/corrupt cache rebuild safely? Are existing projects still editable? What happens when the audio directory is read-only? Do not delete existing caches before deciding migration and retention.
