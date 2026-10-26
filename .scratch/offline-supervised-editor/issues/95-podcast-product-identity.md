# Separate Podcast package and probe-compatible baseline

Status: claimed
Assignee: Codex
Readiness: ready-for-agent
Type: task
Blocked by: 87, 88
Related: 85, 86

## Scope and acceptance

In codex/plugin-podcast, introduce the CleanSlate Podcast package, product-specific editor entry point, IDs/state and build/test/dist/link destinations around the existing Session implementation. Preserve maintained offline analysis and persistent source-time caches. Do not add the live gate as a product mode.

Acceptance:
- [ ] Unique package/name/CLAP/VST3 identities and documented state/cache namespace; probe identities unchanged so old FL projects still open.
- [ ] Old state is not silently treated as the new product's state; any import/migration is explicit, source-bound and reversible.
- [ ] Existing Session unit/event tests and actual-format host checks run against the new artifact; baseline waveform/label/history behavior survives.
- [ ] No detector copy or dependency on a running desktop GUI. Document remaining UI-bearing root library dependencies for later focused extraction.
- [ ] Product-specific build scripts cannot overwrite Gate/probe output or change the existing Common Files links. Check with 88; installer/development-link activation is a separate deliberate step.

## Results

Claimed 2026-10-26 after 87/88 completion. Prepare the separate Podcast identity in its isolated product lane while Gate source preparation proceeds; local process jobs remain serialized under the shared guard. Preserve the legacy probe and maintained Session engine. Build/event/format acceptance remains pending whenever the default 14 GiB heavy-work start requirement refuses execution; static checks alone cannot close this ticket.

### Isolated source checkpoint — 2026-10-26

Retained candidate: `codex/podcast-identity-95` at `70f33401e54cf785169dfe5795b6252b0b9b0f1b`, based on permanent Podcast revision `cf1cc52`, in `C:/Users/User/.codex/worktrees/podcast-identity-95/podcast-automix-rust-gui`. The candidate's `plugins/podcast/README.md` records identity, state/storage/output boundaries and remaining root-library UI dependencies. Product source remains in that isolated branch: it has not been merged into Desktop or the permanent Podcast branch.

The source candidate binds Podcast identity to the Cargo package rather than optional features, shares maintained adapter/Session sources without a detector copy, preserves the probe's identity/state contract, rejects foreign/future Podcast state into visibly disconnected defaults, and forces the Podcast editor/activation into Session mode. Separate state/cache/report namespaces and fresh GUID-named `dist/podcast-candidate/` children keep probe/Gate output and installed links separate. These are source changes, not qualified runtime behavior.

Bounded checks used explicit lightweight budgets for noncompiling work only. Receipts under `%LOCALAPPDATA%/CleanSlate/BuildReceipts/`:

| Receipt | Result |
| --- | --- |
| `podcast95-format-7ea201cbfcab482192645af6aaa42652.json` | Formatting passed; kernel peak owned commit 24.93 MiB, 1.25 s. |
| `podcast95-nodeps-static-719aea5d657d4052b1e541a0935891ad.json` | Both package manifests pass formatting, locked/offline **no-deps** metadata for default/no-default/all-feature combinations, shared-source/mandatory-identity assertions, PowerShell parsing and diff checks; kernel peak owned commit 62.83 MiB, 2.17 s. |
| `podcast95-static-feb4a8019d924304965a1b19b30e43b1.json` | Full offline dependency metadata blocked by uncached `nix 0.31.3`; no dependency download or full-graph success claimed. |
| `podcast95-test-2c963d3d7bef45bf84654286e68816aa.json` | Default heavy test preflight refused at 5.24 GiB available, below the unchanged 14 GiB requirement; no child launched. |

Both static review axes found no high-confidence source defect; that is not compilation or runtime acceptance. No compiled unit/event/waveform/history green, actual CLAP/VST3 ABI/artifact check, native GUI, listening, PDC, save/reopen/export or new product binary is claimed. All acceptance boxes remain unchecked and this ticket stays **claimed**; ticket 96 remains blocked by 95. Next: once the normal heavy-work memory requirement is met, resolve genuine offline dependency needs explicitly and run the guarded Podcast build plus preserved-probe regression checks before product integration.

