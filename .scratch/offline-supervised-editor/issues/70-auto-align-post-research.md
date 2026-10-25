# Investigate installed Auto-Align Post for CleanSlate

Status: done
Type: research
Assignee: Codex
Blocked by: none

## Owner request — 2026-10-22

Use the reverse-engineer-anything skill on the installed Auto-Align Post VST3 and supplied 2.3.4 installer to understand its algorithms and identify useful ideas for CleanSlate. Research only; no audio behavior changes or roadmap implementation are authorized by this investigation.

## Acceptance

- [x] Identify the installed artifact and retain reproducible static-analysis evidence.
- [x] Inspect documented behavior and relevant native code where available; distinguish observations, inferences and unknowns. Native function recovery failed; the report explicitly retains that unresolved question.
- [x] Compare findings with existing CleanSlate delay/correlation/transfer code and accepted preservation contracts.
- [x] Record independent implementation candidates, limitations and unanswered questions in a linked research report.
- [x] Preserve the complete returned REA Evidence record and close the native analysis session.

## Results

Claimed on the existing checkout before writing research artifacts. Existing unrelated working-tree changes remain outside this task. No installer execution is required because the installed VST3 is available.

Completed the bounded research in the [report](../../../docs/research/auto-align-post-2.3.4-2026-10-22.md). The installed DLL is version 2.3.4, SHA-256 `1b3c52650e4e307f5c9b61a2d75c9bcee464d8d292f45553504b3774ee621c5f`. Retained PE-resource evidence and an explicit native-failure/entropy ledger live in `.scratch/auto-align-post-research/`.

Default Ghidra analysis timed out after 330 seconds. A distinct temporary CLI profile with a longer deadline and reduced analyzers failed with Windows resource error 1450. No native function chain or exact proprietary estimator/filter equations were recovered. No engine or client configuration was changed permanently, and no additional software was installed. Sessions/processes were cleaned up.

Embedded all-pass/mode help and vendor documentation identify intended behavior. Independent CleanSlate opportunities are analysis-only changing-delay estimates, unreliable-match rejection and regional acoustic calibration. These are untested hypotheses; timing/phase-changing exports conflict with the accepted current contract and are not implemented here.

Checks: six exact embedded-text offset assertions passed; report links resolved; independent read-only review verified existing CleanSlate timing/lag/validation claims. No audio, runtime-quality, build or performance acceptance is claimed. Done means this bounded investigation is recorded, not that the proprietary DSP has been fully reverse engineered.
