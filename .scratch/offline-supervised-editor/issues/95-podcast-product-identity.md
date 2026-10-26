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

### Isolated source candidate — 2026-10-26

Branch `codex/podcast-identity-95` starts at product integration revision
`cf1cc5293c5b65379a5c75db72928eabc16db26d`. Workspace:
`C:/Users/User/.codex/worktrees/podcast-identity-95/podcast-automix-rust-gui`.
Focused canonical 87/88/90/95/97/98 prerequisite/design/host/resource records
were synchronized from desktop `ed6393e`; the entire desktop branch and its
plugin-source removal were not merged. This candidate stays off the permanent
product integration branch until qualified.

[`plugins/podcast/README.md`](../../../plugins/podcast/README.md) records the
separate package/name/CLAP/VST3 identities, state rejection/recovery, cache/report
namespaces, output/link boundaries and remaining root-library UI dependencies.
The package compiles existing maintained adapter/Session sources directly rather
than copying detector/trainer code. Its custom cfg comes from the Cargo package
name, so feature combinations cannot switch product identity. The product editor
entry uses the maintained Session workspace; activation and callbacks force
Session mode even when host state/automation tries the old mode parameter.

Tests were added before implementation at the accepted plugin identity/state
seams. They cover probe/Gate/Podcast identity separation, legacy/future state
disconnection and visible rejection, new state binding retention and forced
Session mode. The probe has its own legacy identity/state regression. Existing
Session unit/event/waveform/label/history tests are compiled as part of the new
package, and its differently named actual-format hosts check the new artifact.
The CLAP oracle accounts for mandatory Session latency with an independent FIFO;
VST3 checks retain their existing latency-aware oracle and assert the new class
name/ID. Red/green execution is pending; source ordering is not a passing TDD run.

`scripts/build-podcast.ps1` uses standard guarded Cargo commands, then only fresh
GUID-named `dist/podcast-candidate/` children, rejects linked output parents,
checks both bundle hashes against the source DLL and runs the product-selected
host runner. It neither publishes probe/Gate output nor activates Common Files
links. Existing probe IDs/state/default host-runner behavior remain in source.
Manual callers must honor 88's inherited outer-budget contract; ticket 99 owns
dispatch classification. This product change does not relax shared guard policy.

No detector/DSP algorithm, 81–83/96 feature, native DAW, SDK installation,
release, push/tag, link activation, cleanup or user recording/project change is
part of this source checkpoint. All acceptance boxes remain unchecked until
their actual required evidence exists.

### Check receipts — source preparation, not acceptance

Receipts/logs are retained under
`C:/Users/User/AppData/Local/CleanSlate/BuildReceipts/`:

| Receipt | Result | Measured job peak / elapsed |
| --- | --- | --- |
| `podcast95-format-7ea201cbfcab482192645af6aaa42652.json` | Guarded `cargo fmt` completed | 24.93 MiB / 1.25 s |
| `podcast95-static-feb4a8019d924304965a1b19b30e43b1.json` | Full locked/offline metadata failed: uncached `nix 0.31.3`; offline mode prevented downloading it | 107.12 MiB / 7.40 s |
| `podcast95-nodeps-static-cadaca27ead34a67b79e80632dde412e.json` | First scoped harness assertion failed because Cargo preserved `../` target paths; corrected the external harness to normalize paths | 52.87 MiB / 1.16 s |
| `podcast95-nodeps-static-719aea5d657d4052b1e541a0935891ad.json` | Both manifests pass format checks, locked/offline **no-deps** metadata for default/no-default/all-feature combinations, shared adapter/mandatory builder path assertions; both PowerShell scripts parse; diff whitespace passes | 62.83 MiB / 2.17 s |
| `podcast95-test-2c963d3d7bef45bf84654286e68816aa.json` | Resource-blocked before Cargo: 5.24 GiB available, below the unchanged 14 GiB start requirement; no child launched | 0 MiB / 0.076 s |

Noncompiling tasks used explicit 2 GiB start / 1 GiB reserve / 0.5 GiB job cap,
90 s timeout. The test attempt retained 14/8/6 and the standard 30-minute timeout.
All receipts verify owned-process cleanup. Final scoped static checks observed a
5.19 GiB minimum available physical memory; these are bounded process measurements,
not a whole-machine guarantee. The external compact check harness is retained as
`podcast95-noncompiling-20261026.ps1` beside those receipts.

No full dependency-graph success, compilation, Clippy, unit/event/waveform/history
green, actual CLAP/VST3 artifact check, native editor render, listening, PDC,
save/reopen/export, new artifact path/hash or product integration is claimed.
The UI launcher/render was not run under the resource/download restrictions;
existing project/accepted design context and pinned API source were read directly.
Next: with at least 14 GiB available, rerun the standard guarded Podcast build
and preserved-probe regression checks; resolve genuine offline dependency needs
without unapproved downloads. Ticket 95 remains **claimed**, and 96 remains
blocked by its missing acceptance.

