# Simplify waveform and Catch up toolbar controls

Status: done
Type: task
Assignee: Codex
Blocked by: none
Related: 60, 63

## Owner request — 2026-10-22

Remove the visible Waveform group label and replace Catch up's visible text with an icon. Preserve the three waveform choices, Catch up selection, its registered command, configurable shortcut and descriptive tooltip. Match the existing icon size and house styling; do not change playback or project behavior.

## Acceptance

- [x] Waveform choices have no redundant group text; Catch up is a fixed-size icon button using the existing registered-action helper and selected-state styling.
- [x] Standard GUI build and native toolbar inspection completed; the owner approved the appearance. Effective hints remain supplied by the shared action helper and software regressions; no additional hover automation is claimed. Source/executable identity is in the combined receipt.

## Results

Claimed on codex/menu-actions-integration before editing. This presentation-only follow-up accompanies the owner's review of tickets 60 and 63.

Completed in `eab2e720869a9049ce80ee8e89b8dcf231e30b0b`. The owner approved the development build and requested closure/integration. Final formatting, strict Clippy, typed-action guard and three timeline regressions passed. [Combined receipt](../../../docs/research/menu-actions-integration-2026-10-22.md) records the preserved executable and limits. No dependencies or playback behavior changed.
