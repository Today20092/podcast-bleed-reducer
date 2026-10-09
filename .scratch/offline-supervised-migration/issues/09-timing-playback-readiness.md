# Make timing and playback readiness clear

Type: grilling
Labels: wayfinder:grilling
Status: resolved
Parent: ../map.md
Blocked by: none

## Answer

2026-10-09: Owner confirmed the [consolidated design](../gui-design-summary.md). Earlier questions and recommendations below are interview history; the consolidated design is the final decision. Implementation is consolidated into [build ticket 16](../../offline-supervised-editor/issues/16-gui-editing-workflow.md); do not create a separate build ticket for this decision. No implementation has been performed for this decision.

## Question

Should Original audition be available when durations match but start alignment is unverified, while analysis/export still require a clear timing decision?

## Owner feedback

Play/Stop were gray and Space did not play until the owner found I supplied synchronized tracks deep in the inspector. The owner expects equal durations to establish synchronization and asks why first playback needs Preparing audio.

## Recommendation and pending decisions

2026-10-09 owner answer: allow Original playback before synchronization confirmation. This does not establish alignment for analysis/export. Preparation timing and visible progress remain to be settled.

Equal physical duration is necessary compatibility evidence but cannot prove common start timing. Recommend an immediate, visible import/timing status, blocking actual timing conflicts, and allowing Original audition for compatible unverified tracks with an explicit status. This changes the current gate and requires owner confirmation. Analysis/export policy is separate; automatic alignment remains outside the accepted scope unless explicitly reopened.

After this decision, define early playback preparation, reusable device-rate data, cancellation/progress, disabled-button reasons and Space behavior. Preserve source validation and source-time conversions. Do not silently stretch, trim or pad sources.

## Proposed acceptance questions

2026-10-09 second round: automatically prepare playback when tracks open. Owner expects DAW-like playback, citing FL Studio and REAPER. Do not gate Original audition on model/reference readiness. Define progress, cancellation, reuse and measured readiness without promising instant cold playback.

Can the owner understand the unavailable action without scrolling? Are matching metadata, missing metadata and conflicting metadata distinct? Does Original audition remain useful before references/model setup? What cold/warm playback wait is acceptable on the real recording, and can preparation start earlier without waste or excess memory?
