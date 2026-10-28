# Separate Podcast package and probe-compatible baseline

Status: done
Assignee: Codex
Readiness: ready-for-agent
Type: task
Blocked by: 87, 88
Related: 85, 86

## Scope and acceptance

In codex/plugin-podcast, introduce the CleanSlate Podcast package, product-specific editor entry point, IDs/state and build/test/dist/link destinations around the existing Session implementation. Preserve maintained offline analysis and persistent source-time caches. Do not add the live gate as a product mode.

Acceptance:
- [x] Unique package/name/CLAP/VST3 identities and documented state/cache namespace; probe identities unchanged so old FL projects still open.
- [x] Old state is not silently treated as the new product's state; any import/migration is explicit, source-bound and reversible.
- [x] Existing Session unit/event tests and actual-format host checks run against the new artifact; baseline waveform/label/history behavior survives.
- [x] No detector copy or dependency on a running desktop GUI. Document remaining UI-bearing root library dependencies for later focused extraction.
- [x] Product-specific build scripts cannot overwrite Gate/probe output or change the existing Common Files links. Check with 88; installer/development-link activation is a separate deliberate step.

## Results

### Post-reboot qualification — 2026-10-26

Ticket 95 is **done** for the independent Podcast package/state identity and maintained Session baseline. Earlier unrun/memory-blocked descriptions below are historical. Clean tested source: `70f33401e54cf785169dfe5795b6252b0b9b0f1b`. Standard `scripts/build-podcast.ps1` passed formatting, all 62 unit/Session/egui event tests, strict all-target Clippy, library/examples build and both actual-format hosts. Windows compilation succeeded locked/offline; the earlier full all-platform metadata failure for nix 0.31.3 did not block this Windows build, and other platforms remain unqualified.

CLAP passed 1,858 bit-exact mono/stereo blocks, 1,168 shared PCM windows, foreign probe-state disconnection and Podcast state round-trip, offline activation/export labeling and restart signaling. VST3 passed two actual instances, controls, mono/stereo at 44.1/48 kHz, realtime/offline, 4,048 bit-exact blocks, 3,392 shared windows, in-place/separate buffers, missing transport/seeks and editor construction/release without opening a window. Host times were 0.57/0.93s and sampled process peak working sets 23.87/24.82 MiB. Preserved probe actual-format regressions also passed in the immediately preceding Gate qualification; installed probe binaries and links were unchanged.

All compiling stages used default 14 GiB start / 8 GiB reserve / 6 GiB owned cap, locked/offline shared profile/cache and Cargo -j1. Peak kernel job commit was 1,742.98 MiB, with minimum observed physical availability 11.69 GiB and verified cleanup. Two lingering VCTIP processes were confirmed in their named build Job Objects after compiler exit and ended individually; elapsed guard times include their wait. No unrelated process or service was stopped; no policy was weakened.

Preserved candidate: `C:/Users/User/.codex/worktrees/podcast-identity-95/podcast-automix-rust-gui/dist/podcast-candidate/8929f581a41f41ee864f98aec8e6f1ef/`. CLAP and VST3 SHA256 `c30df7f54f9f97040c74cbd63423a41d80c2f7270ee41e9b76379342ea4bdb55`. Source/build receipts under `%LOCALAPPDATA%/CleanSlate/BuildReceipts/`: fmt `20261026-112426-8445418fee874032b2d2b5bc4e8d8a24.json`; tests `20261026-112427-c545c17c4473431ea79c9953845c037f.json`; Clippy `20261026-112813-ffbcf31fd9c4425b9a996203d6a90e57.json`; library `20261026-112837-3245728c481b45b182f50f2d8c4c89e2.json`; examples `20261026-112839-909a6e24a699490eb35354fe0db9d146.json`; hosts `20261026-112842-49dd93d067ee4fd78a8d266cc30ce94f.json`.

This unblocks 96's workflow work. It does not close 80's native sound/export/PDC/save/listening acceptance, qualify process-isolated hosts or other platforms, activate installation links, or authorize signing/publication.

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

