# Delegated design decisions

The user authorized automatic answers to all grilling and workflow questions on September 5, 2026. The existing tracker/domain setup satisfies ask-matt prerequisites. Route: grill-with-docs → to-spec → to-tickets → implement/TDD → standards/spec review → integration/package.

## Round 1: Scope and output

- Implement all recommendations? Yes, including correctness/UI/playback and measured detector-quality evaluation, not just the two biggest timings.
- Rewrite Iced or replace Silero? No evidence supports that cost. Retain both.
- One deliverable? One integration branch, codex/cleanslate-responsive, based on c268882, plus one identified Windows EXE. Keep main and old builds available.
- Separate tasks? Six implementation tasks, each its own branch/worktree; parent task owns integration and final verification.

## Round 2: Semantics and performance

- Preserve audio behavior? Exact or documented floating-point equivalent gain behavior is mandatory for performance changes.
- Source analysis lifetime? Cache compact immutable facts per validated ordered sources, not duplicate multi-gigabyte PCM.
- Target response? Measure warm median/p95 on actual full session, aim below 100 ms first; do not claim an unmeasured target.
- Slider cadence? Coalesce during dragging, 30–50 ms initial cadence, current-generation installation and one active worker.
- Sensitivity migration? Keep stored threshold, invert presentation at UI seam. Old projects retain their sound.
- Padding? Display effective 32 ms granularity without silently changing legacy stored values.
- Display accuracy? Use conservative gain-aware summaries and explain approximation; actual audition/render retain shared gain behavior.

## Round 3: Evaluation and acceptance

- Detector preprocessing? Implement/test anti-alias path; compare human-labeled preservation/overlap/bleed before default promotion. Insufficient evidence means explicit experiment, legacy default.
- Cross-mic competition? Evaluate actual evidence; no unvalidated winner-takes-all default.
- Test seams and breakdown? Approved as spec and seven issue files. First 01/02/03 independent; then 04/05/06 from integrated interfaces; 07 validates all.
- Human listening? Agent cannot answer a listening check as passed. Automated acceptance plus runnable user test build, with listening limitation explicit.
- Source fixtures? Use full session and existing labeled excerpts read-only, never copy WAVs into evidence.

## Available quality references

Human labels are preserved as evidence/reference-labels.csv. Actual excerpt sources exist at C:/Users/User/Downloads/00005603_A01.wav, 00005603_A02.wav, 00005603_A03.wav. Existing evaluation-contract note says these are byte-identical full-track excerpts from sample 8973765 through 23524301; evaluate labels against excerpts directly, not mismatched full-session offsets. Use the actual WAV sample count and metadata as authoritative.

The original unoptimized full session at stored sensitivity threshold 0.20 yielded 2673 regions in 91.713816 seconds. Loading ~3.739 s; analysis ~70.878 s; gain construction ~16.991 s. Full original timings remain in C:/Users/User/AppData/Local/Temp/cleanslate-real-preview-timing.json.
