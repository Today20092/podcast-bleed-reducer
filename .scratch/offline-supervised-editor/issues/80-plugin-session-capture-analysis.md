# Persistent plugin Session capture and offline analysis

Status: claimed
Assignee: Codex
Type: task
Blocked by: none
Related: 67, 78, 79

## Accepted scope

Implement the confirmed Session-first contract in `plugin-modes-interview.md`: source-time capture/cache independent of transport resets, exact source validation, room/solo/overlap labels, explicit maintained desktop analysis, original fallback and shared control for up to eight microphone inserts. The old paired streaming classifier is not desktop parity. Preserve existing binary identities and tested output until a new candidate passes checks. No vendor runtime or DAW automation required.

## Acceptance

- [x] Capture native PCM off callbacks into local project-specific disk caches, with bounded queues/RAM and explicit missing/error states.
- [x] Loops/seeks preserve timeline; changed PCM invalidates processing; unchanged PCM does not churn analysis. Missing/stale/unverified audio stays original.
- [x] Cache references/labels persist in host state; conflicting saved sessions cannot silently join by numeric group; missing cache requests recapture. Native host save/reopen remains below.
- [x] One to eight synchronized tracks can label and call the maintained offline analysis on verified common contiguous coverage; gaps are never synthesized as trusted silence. Coupled two-source sparse-slot analysis and eight-lane UI pass; native eight-track setup remains unqualified.
- [x] Session UI shows persistent waveforms, time navigation, labels, analysis/readiness and shared members; actual CPU renders inspected.
- [x] Appropriate regressions, strict checks, actual CLAP/VST3 hosts and candidate preservation/publication pass; identity and limitations recorded.
- [ ] Owner accepts native FL UI, loop/source behavior, delay compensation, real sound, save/reopen and export. Keep claimed until tested; software checks are not this acceptance.

Live Mic, manual gain point editing, VST2 adapters and non-Windows qualification remain separate later work. Native host testing will qualify the actual editor, synchronization and sound after a concrete checked build exists.

## Checked candidate

[0.0.5 delivery receipt](../../../docs/research/plugin-session-delivery-2026-10-25.md): 50 plugin tests, 371 app/shared/example checks, strict Clippy, desktop BuildOnly/self-check and both actual-format hosts pass. Linked CLAP/VST3 files match SHA256 `bcdd98812733ceb440ab61a899935631fd47d3f5ff053ca240e80848a99f1548`; prior 0.0.4 remains preserved. New Session delay is fixed while bypassing/enabling processing, avoiding unnecessary reload/loss of analysis. Source capture and full desktop analysis work without vendor code or DAW automation. Owner instructions are in the plugin README. Remaining shared sound automation/overrides, metadata and analysis persistence are tracked separately; no release or native acceptance is claimed.
