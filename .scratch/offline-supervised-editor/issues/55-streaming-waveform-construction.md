# 55: Share streaming waveform-cache construction

Status: claimed
Type: task
Assignee: Codex
Blocked by: none

## What to build

Progressive import and waveform-cache rebuilding produce identical detailed waveforms for identical source samples, regardless of decoder chunk boundaries. The waveform module owns streaming bucket accounting, completed-prefix refinement and final cache construction. Preserve early full-recording previews, per-channel identity, validation before acceptance, cancellation and persisted-cache compatibility.

## Acceptance criteria

- [ ] The waveform module owns streaming bucket size, extrema, sample count and final partial-bucket handling; import and rebuilding no longer duplicate these rules.
- [ ] Chunked construction matches independently known peak values for empty input, exact bucket boundaries, irregular chunks, multiple buckets and a final partial bucket.
- [ ] Progressive prefix refinement includes completed detail without presenting unfinished source validation as accepted audio.
- [ ] Multichannel source separation and existing cache identity/version behavior are retained.
- [ ] Existing import cancellation, corrupt/truncated/changed-source rejection and persisted-cache regressions pass.
- [ ] Shared local checks and GUI build pass; source commit and executable path are recorded.
- [ ] Two-axis code review is complete; the tested batch is merged into development and temporary worktrees are retired safely.

## Accepted testing seams

1. Waveform construction: samples/chunks to resulting cache and observable peaks. Catches bucket/tail/chunk discrepancies; does not prove decoder correctness.
2. Existing import events and waveform loading: source fixtures to previews, accepted cache and persisted reload. Catches wiring, per-channel separation and validation regressions; does not establish listening quality or physical removable-media behavior.

## Comments

2026-10-20: owner approved the single-ticket breakdown, no blocking edges and both testing seams after the architecture report. This is the only suggestion selected from that review and the only implementation ticket authorized in this batch. It follows the accepted migration architecture and ADR-0006. No new dependency, source mutation, cache-format change, GUI interaction or audio-policy change. Existing unrelated tickets remain outside this batch; main/release promotion remains separate.

Review report: `C:/Users/User/AppData/Local/Temp/architecture-review-20261020-cleanslate.html`. Exploration notes: `C:/Users/User/AppData/Local/Temp/cleanslate-architecture-scan-20261020.md`.
