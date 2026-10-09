# Make playback and reference labeling flow directly from the tracks

Status: claimed
Assignee: Codex
Labels: ready-for-agent
Blocked by: 12, 13
Spec: ../../offline-supervised-migration/gui-design-summary.md

## Scope

Consolidate GUI decisions 07–11 into one owner-testable Rust/Iced workflow. Keep the finite DAW lane layout and existing transport/envelope architecture. Implement in sequence within this ticket: direct selection/labeling, playback readiness, concise inspector/readiness, and source-bound waveform reuse. Build a verified candidate and physically test the flow with the owner before completion. No analysis algorithm changes belong here.

## Reach and reuse

Inspect graph symbols/callers in src/app.rs, src/waveform.rs, src/audition.rs, src/editor_workflow.rs and source validation/project identity helpers. Preserve and reuse the existing uncommitted app.rs/waveform.rs changes in the offline-timebase worktree. They implement right-click-primary behavior and therefore need adjustment to the accepted immediate-after-drag behavior; they are not verified. Reuse current labels, validation, conflict replacement, reference undo/persistence, stale-analysis signaling and cancellation. Use existing controller/canvas/cache/playback tests where applicable.

## Required behavior

- Plain click seeks. Drag creates a track-bound range; release after a nonzero drag shows pointer-adjacent label actions automatically. An ordinary click does not open label actions. Offer any configured solo speaker, arbitrary active-speaker overlap and room tone. Labels describe the whole session interval. Keep accessible keyboard entry, Escape/outside dismissal, viewport clamping and captured input; retain no selection mode requiring Finish selecting to seek.
- Original audition does not require model setup, references or supplied-alignment confirmation for otherwise valid, compatible tracks with missing timing evidence. Keep actual timing conflicts and invalid sources blocked, and show their reason visibly. Analyze/export still require the current timing contract; equal duration is not verified synchronization.
- Opening/installing validated tracks starts cancellable, bounded playback preparation. Reuse compatible prepared audio; source/job changes cannot install obsolete preparation. Show preparation/readiness and make Space follow the same gate as Play. Keep Stop available during preparation/playback. Avoid retaining unnecessary full-session copies.
- Use concise readiness and a next useful action in one workspace. Collapse technical/setup information. Installed verified model shows compact ready status; missing/corrupt states show actionable recovery. Keep notices/provenance accessible and all tracks/transport visible. Exact-times labeling remains a keyboard alternative.
- Store reusable per-source waveform data in a shared application cache, independent of project path and track ordering. Bind to validated source identity, decoded rate/length and waveform recipe/version using existing identity machinery. Changed audio must invalidate its waveform. A missing/corrupt entry rebuilds without losing project data. Cache writes are safe; sources may be read-only. Keep existing project waveform files intact, with compatible old-cache import/reuse if practical and safe rebuild otherwise. Do not delete caches or modify user audio/projects as cleanup.

## Acceptance

1. Meaningful canvas/controller checks cover click versus drag/release, immediate popup, track binding, input capture, label choices, validation/conflict replacement, undo, persistence and stale analysis. Keep explicitly labeled speech protected under the current engine.
2. Playback checks distinguish missing timing evidence from conflicts, exercise preparation reuse/cancellation/obsolete jobs, and verify Play/Space behavior without model/reference setup. Gain parity and source-time conversion checks remain green.
3. Cache checks demonstrate reuse across two projects and track order changes, changed-source invalidation, corrupt/missing recovery, and no required source-folder writes. Do not advertise waveform reuse as instantaneous source validation or zero file reads.
4. Run applicable tests, strict all-target Clippy, formatting and diff checks. Use one Cargo lane, j1, and the retained owner-ui target/cache with the known low-memory package configuration. Preserve the last verified GUI/helper/runtime until a replacement is verified; record actual build settings.
5. Owner tests a fresh project and the ready three-track recording without overwriting its saved labels/audio. Confirm tracks/transport visibility, seeking/labeling, concise readiness, first and repeated playback, saved-project reopening and menu placement at viewport edges. Record measured preparation wait and physical UI results; no fabricated listening/performance approval.
6. Integrate a clean reviewed checkpoint into codex/offline-supervised and record results here. Mark done only after scoped checks and owner workflow verification. Ticket 14 audio-quality and ticket 15 release gates remain separate.

## Planning record

2026-10-09 owner authorized implementation through implement-spec and requested three synchronized 10-minute clips for fast feedback. Preserve the existing offline-timebase worktree changes and owner-ui build cache; do not reset the worktree. The acceptance seams above are the authorized controller/canvas, cache, source-validation and playback checks. Prepare a verified candidate for owner feedback before claiming physical usability or audio-quality acceptance.

2026-10-09: Owner confirmed the simplified diagrams/design and requested minimal ticket count. This is the only new build ticket. Decisions 07–11 are resolved planning records; acoustic-first evaluation from decision 12 extends existing ticket 14. Documentation only in the current session; this ticket is unclaimed.
