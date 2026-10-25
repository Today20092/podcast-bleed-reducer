# Separate plugin products and development workspaces

Status: done
Assignee: Codex
Type: task
Blocked by: none
Related: 67, 85, 86

## Accepted scope — 2026-10-25

Owner accepts three products: the existing CleanSlate Desktop, a single-input live Smart Gate, and a prerecorded multitrack Podcast plugin. Create independent local product branches/workspaces without losing the tested 0.0.6 prototype, its shared-library changes, outstanding checks or unrelated desktop work. Update the authoritative roadmap and create one continuing product ticket per plugin. This is organization, not implementation or release of two finished plugins.

## Acceptance

- [x] Preserve the current plugin source and necessary shared-library/build changes in a committed baseline before relocating source out of the desktop checkout.
- [x] Create `codex/plugin-podcast` and `codex/plugin-smart-gate` with attached independent workspaces and clear entry points.
- [x] Record product scope, branch ownership, shared-code integration, adapter/host qualification and remaining dependencies in tickets 85/86 and the roadmap.
- [x] Retain original tracker results and installed prototype binaries; do not change plugin IDs, development links or the owner's FL projects.
- [x] Verify both workspaces contain the intended baseline and can resolve their locked local package graph. Check documentation links and record source commits/paths.

## Results

Common preserved source commit: `44cbf432c84de90168cbb6c25b386d40142d3f75`, based on `88dda154daa9f442018302e19f1c792513fd0874`. The baseline copies 62 source/planning/research/build records with SHA256 verification before staging; its commit contains 61 changed files (one selected file was unchanged). Product-specific AGENTS/PLUGIN-WORKSPACE entry points and the later overnight plan are recorded in separate branch planning commits.

- Podcast: `C:/Users/User/.codex/worktrees/plugin-podcast/podcast-automix-rust-gui`, branch `codex/plugin-podcast`.
- Smart Gate: `C:/Users/User/.codex/worktrees/plugin-smart-gate/podcast-automix-rust-gui`, branch `codex/plugin-smart-gate`.
- Canonical desktop/tracker: `C:/Users/User/Documents/podcast-automix-rust-gui`, still `development`; unrelated 70/72/74/release/research work is not staged or committed by this split.

Before relocation, verified all 23 plugin source/build-script files against the baseline Git blobs from each worktree. Both worktrees passed `cargo metadata --locked --offline --no-deps` for `cleanslate-fl-probe` 0.0.6. No recompilation was required for unchanged source. Original plugin source and its three build/link/host-test scripts are retained outside desktop at `C:/Users/User/AppData/Local/CleanSlate/Recovery/2026-10-25-plugin-product-split`; live source is in the two committed worktrees. The shared ownership-model crate, root library boundaries and common check scripts remain with desktop.

Existing `dist/fl-probe` is retained with SHA256 `94cf0dc682447677e1d942397dfc8aea929c9fe697a5a0035127968dace54aaa`. The CLAP symlink and VST3 junction still resolve to that desktop dist directory. No IDs, binaries, installed links, host projects, caches or media were changed. This does not deliver two new plugin binaries.

Owner's follow-up adds scoped tickets 88–99, the [overnight runbook](../../../docs/plugin-overnight-plan.md) and [queue](../../plugin-products/overnight-queue.json). Their guards/runner are planned, not implemented or started. Local reference inventory confirms all seven sonible folders and installed Pro-Q 4/Pro-R 2/Pro-C 3; the runbook records bounded Pro-C 3 resource evidence without runtime/performance claims.

Documentation/queue checks cover file links, unique task IDs, existing dependencies and acyclic ordering. Existing 80 and blocked 81–83 remain open; subjective listening, native host/platform qualification, private release signing and prior cleanup restrictions were not waived.
