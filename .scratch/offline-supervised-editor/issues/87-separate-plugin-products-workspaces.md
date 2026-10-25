# Separate plugin products and development workspaces

Status: claimed
Assignee: Codex
Type: task
Blocked by: none
Related: 67, 85, 86

## Accepted scope — 2026-10-25

Owner accepts three products: the existing CleanSlate Desktop, a single-input live Smart Gate, and a prerecorded multitrack Podcast plugin. Create independent local product branches/workspaces without losing the tested 0.0.6 prototype, its shared-library changes, outstanding checks or unrelated desktop work. Update the authoritative roadmap and create one continuing product ticket per plugin. This is organization, not implementation or release of two finished plugins.

## Acceptance

- [ ] Preserve the current plugin source and necessary shared-library/build changes in a committed baseline before relocating source out of the desktop checkout.
- [ ] Create `codex/plugin-podcast` and `codex/plugin-smart-gate` with attached independent workspaces and clear entry points.
- [ ] Record product scope, branch ownership, shared-code integration, adapter/host qualification and remaining dependencies in tickets 85/86 and the roadmap.
- [ ] Retain original tracker results and installed prototype binaries; do not change plugin IDs, development links or the owner's FL projects.
- [ ] Verify both workspaces contain the intended baseline and can resolve their locked local package graph. Check documentation links and record source commits/paths.

## Results

Pending.
