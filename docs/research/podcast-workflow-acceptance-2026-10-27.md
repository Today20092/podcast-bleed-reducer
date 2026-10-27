# Podcast workflow machine acceptance

Ticket [96](../../.scratch/offline-supervised-editor/issues/96-podcast-workflow-and-acceptance.md) remains **claimed: machine checks complete, ready for integration**. After the owner-authorized reboot, clean source `6070569` passed all 70 tests, strict normal all-target Clippy, the normal build and both basic actual-format hosts. Both independently reviewed UI fixes are verified. Native DAW/listening ticket 80 remains open and does not unlock 81–83. No release, installation, owner recording/project write or framework migration occurred.

The historical artifacts/captures below preserve the **pre-review** 4558474 delivery. Source fix `47e83bb9a9ddee719c8c8564a51f4c77081ef6f9` discards a replacement when its selection changes and culls offscreen lane geometry. Both independent review axes cleared those fixes read-only. The initial geometry assertion included a four-vertex scrollbar quad; the corrected exact-waveform counter now passes. The earlier 13.82 GiB guard refusal is retained as historical evidence. Final post-reboot normal artifacts and all six refreshed captures are recorded below; no threshold was relaxed.

## Final post-review verification

Clean tested source: `6070569f47d78b7ab2430d325bf73ede60fe2cf7`; production UI remains `47e83bb`. Final documentation changes do not change the tested source. Permanent integration base and merge-base: `da9b297b814802985b35a8704da12dffe36475a0`.

- All 70 unit/event tests pass, including changed-selection replacement and top/bottom scroll geometry/history/selection checks. Six compact/full native-DPI geometry combinations pass. Strict normal all-target Clippy and formatting pass.
- Final normal basic actual-format hosts pass: CLAP 1,858 and VST3 4,048 bit-exact blocks; normal driver symbol absence remains checked. Earlier 4558474 actual-format processing/state/independent-render acceptance remains the processing qualification; the later fixes affect UI only, so that full driver workflow was not repeated.
- Normal candidate: `dist/podcast-candidate/f0be4aab16654eea97364eb8517b56b4`. CLAP and VST3 SHA256: `cf1ddab61ac90879ca49e9c0b4d43d33e327ad94bfd0fc5ae0c670de4cd89485`.
- Outside-worktree preservation: `C:/Users/User/AppData/Local/CleanSlate/VerifiedBuilds/plugin-products/Podcast/6070569-normal-cf1ddab61ac9`. All eight copied file hashes match; `preservation-manifest.json` records every file. Prior normal and acceptance-only artifacts remain intact.
- Final six PNGs, corresponding PPMs, capture script and `source-receipt.json`: `C:/Users/User/AppData/Local/CleanSlate/Podcast/Workflow96Captures/final-6070569-5d990b1e8a904b40b045517fa27e7a0d/`. Filenames and physical sizes match the six-entry table below. All six PNGs were visually inspected: label/actions/Analyze/readiness remain reachable. These are actual egui CPU meshes, not native DAW captures.
- Final build receipt: `C:/Users/User/AppData/Local/CleanSlate/BuildReceipts/20261027-100020-4c428ef490b64973902956e5d27f05d0.json` and `.log`: done, exit 0, cleanup verified, no competing heavy processes; 137.921 seconds including owned helper cleanup, peak owned commit 1,860,972,544 bytes, aggregate peak working set 1,753,747,456 bytes, minimum physical availability 12,670,218,240 bytes (above the 8 GiB running reserve).
- Final capture receipt: `C:/Users/User/AppData/Local/CleanSlate/BuildReceipts/20261027-100247-53eb8b43156143f18ffe091163dfafbd.json` and `.log`: done, exit 0, cleanup verified; 3.200 seconds, peak owned commit 100,577,280 bytes, minimum physical availability 18,313,367,552 bytes.

Both jobs used the unchanged 14 GiB start / 8 GiB reserve / 6 GiB owned cap. The sole lingering VCTIP helper was stopped by the coordinator only after proving named-job ownership and compiler exit. The heavy slot was released after capture completion. Ticket 96 awaits coordinator integration; 80 and dependencies 81–83 retain their native acceptance limits.

## Source and behavior

Branch `codex/podcast-workflow-96`, fixed review base `da9b297b814802985b35a8704da12dffe36475a0`.

- Production and actual-format qualification: `4558474645b746af5776bf2811d660664e6abef2`.
- Additional navigation regressions: `22b2d9fc6f9d178409c113b22b38aebe3b8ddb07`. Only the `#[cfg(test)]` module changes after 4558474; the production UI prefix was compared with that commit and is identical.
- Current review-fix source: `47e83bb9a9ddee719c8c8564a51f4c77081ef6f9`. It changes production UI behavior and is represented by the final 6070569 normal binaries and PNGs, not the preserved pre-review artifacts.
- The permanent Podcast tip was merged/checked before handoff: already up to date at `da9b297`.

The lane viewport has readable 64-point rows and its own scrollbar, leaving selection, examples, shared sound, Analyze/cancel and readiness reachable at compact/full sizes. Current source skips nonintersecting lanes before names, peak-scale scans and mesh construction while retaining full scroll content and selection bounds. Label commands distinguish a single speaking microphone from overlap checkboxes. Conflicting examples require explicit replacement; coverage and member checks still apply after confirmation. The pending proposal is bound to its exact range, discarded when selection changes, and identifies the proposed speakers/range. Label actions use the current rendered view, including fast frames that temporarily take the cached view. Escape retains a completed selection and cancels a live draft.

Explicit cancellation records user intent separately from source/settings invalidation. It signals atomically before attempting a status lock; snapshot copies observe cancellation between chunks and retain the existing private-output cleanup. Accepted PCM, examples and source validation remain authoritative. Starting analysis immediately marks the local view busy/stale instead of briefly leaving a second Analyze action available.

Sound remains shared and requires Analyze. Per-track overrides, reusable analysis, host metadata and native recovery remain their existing later tickets. The maintained detector/trainer/envelope evaluator is unchanged. This work retains egui 0.37.0 / nice-plug-egui 0.6.0 / nice-plug 0.4.2.

## Checks and qualification boundaries

| Check | Evidence |
| --- | --- |
| Unit/event suite | Final clean 6070569: 70 passed, including both review regressions; earlier counter failure and guard refusal retained below |
| Normal actual formats | CLAP 1,858 / VST3 4,048 bit-exact baseline blocks; mono/stereo, 44.1/48 kHz, realtime/offline, latency/state/transport checks and acceptance-driver symbol absent |
| Debug driver isolation | Both driver runners reject the normal artifact with an explicit missing-driver error; the exact product build guard rejects the feature with `PROFILE=release` |
| Actual CLAP and VST3 processing | Two actual instances capture synthetic native PCM through their format ABIs, explicitly invoke the same Session label/Analyze API, then compare realtime and host-offline output with independently generated maintained Float32 final-render WAVs |
| Processing truth | Known labeled speaker unity and other-microphone non-unity; Original/Processed/Removed, reported delay, nonzero first/last sample markers and changed-PCM Original fallback |
| Native-format state | Save, destroy, recreate and load using CLAP streams / bounded VST3 IBStream; four shared examples retained, binding/PCM flush verified and original fallback retained until fresh analysis |
| GUI interaction | Existing quick selection, wheel/zoom, middle pan and Alt-range checks retained; Center, Catch up, Lock, Off, Fit and navigator pointer checks preserve selection and host transport; popup replacement and visible gap recovery checked |
| GUI geometry | Compact 720×560 and full 1000×680, each at native 100/150/200% DPI; Analyze text must exist and lie inside its clip; six actual-mesh CPU renders |
| Static checks | Final normal all-target Clippy, formatting and diff checks pass; pre-review driver all-target Clippy passed |

The nondefault `acceptance-driver` builds only in the Podcast debug package. Its fixed scalar commands reach the existing Session API; PCM, parameters, transport, state and output continue through real format interfaces. It has no alternate detector or playback path. It cannot qualify the native editor's Analyze gesture, a DAW's PDC/export implementation or listening quality. The normal build is restored after the test workflow; test-driver files must never be installed or distributed.

The processing fixture is generated noise with trusted room/solo/overlap examples, 196,608 frames at 48 kHz (96 complete fingerprint blocks). Its nonzero boundary markers catch missing/shifted edges. Partial-fingerprint conservative fallback remains separately covered by the Session tests; this fixture does not claim native partial-edge or acoustic-quality qualification. Reopening is checked after changed-source invalidation, not as an analysis-persistence feature.

The initial restored-state view could be unavailable while the capture actor held its lock. The final checks first require a successful binding/PCM flush, then permit at most three seconds for the existing nonblocking view to become available. No unbounded retry, implicit analysis or persistence workaround was added.

## Preserved binaries

Paths below are relative to this scoped worktree, not the desktop checkout.

| Artifact | Location | SHA256 |
| --- | --- | --- |
| Driver-free normal CLAP/VST3 candidate | `dist/podcast-candidate/ddcab0ca2e1e41f0aa700cdb32af0d2a` | `b2c3a23329763b2080b7fb6af845dfa04c58827c7861926bab63fa1a00132d5c` |
| Acceptance-only DLL and separately copied runners/logs/private caches | `dist/podcast-acceptance-driver/b994dd605fdd4e0bb1ee74895bbfbe13` | DLL `d7c4fd823fbb28a3ed1df1354726b24d7e808bacd395c9bcf88a3ab75f5bbe98` |

The driver directory retains each format's normal-artifact rejection log, passing processing/state log, the expected release-guard rejection and its explicitly named `DO-NOT-INSTALL.txt`. The independently rendered oracle WAVs remain under the exact fresh temporary paths printed in the two passing host logs. Earlier candidates remain preserved. Common Files links and all `dist/fl-probe` outputs were untouched.

Both complete pre-review directories are also preserved outside the worktree under `C:/Users/User/AppData/Local/CleanSlate/VerifiedBuilds/plugin-products/Podcast/`:

- `4558474-normal-b2c3a2332976`: all 8 copied files (78,620,971 bytes) match their source SHA256.
- `4558474-acceptance-only-d7c4fd823fbb`: all 208 copied files (146,707,682 bytes) match their source SHA256.

Each destination has a `preservation-manifest.json` with every relative path, byte count and verified hash. These are pre-review artifacts; the acceptance-only directory must never be installed.

## Resource and source receipts

All paths in this section begin at `C:/Users/User/AppData/Local/CleanSlate/`.

- Full workflow: `BuildReceipts/20261027-001951-346ac428bb304491be80f3e4e25b3f47.json` and `.log`. Done, exit 0, cleanup verified, no competing heavy process. Elapsed 197.99 s includes waiting for an owned MSVC helper to exit; it is not an isolated compile or processing benchmark. Peak owned job commit 1,784,451,072 bytes (1.662 GiB), sampled aggregate working set 1,718,648,832 bytes, minimum available physical RAM 16,358,998,016 bytes (15.236 GiB).
- Capture/export: `BuildReceipts/20261027-002514-bc3b790c5c7549a3a93b86a36726b425.json` and `.log`. Done, source commit 4558474, exit 0, cleanup verified; 3.18 s, peak owned job commit 103,010,304 bytes (98.24 MiB).
- Additional 69-test run: `BuildReceipts/20261027-003111-124ae6ef3e5a44abb9fb64f5854dcc5b.json` and `.log`; 69 passed in 2.79 s after compilation.
- Final test-only Clippy: `BuildReceipts/20261027-003331-add1a98395394c37833780ffc748d7aa.json` and `.log`; done, exit 0.
- Historical review-fix test attempt: `BuildReceipts/20261027-003947-9feeba7466ad4ce8a9df978ee48e76de.json` and `.log`; 69 passed, culling assertion saw 12,004 vertices versus its 12,000 peak-only limit. The corrected exact-waveform-mesh assertion excludes scrollbar/selection chrome and passes in the final run above.
- Final focused verification denial: `BuildReceipts/20261027-004212-5e39d1247dc343ac804ddb95ca04accd.json` and `.log`; `resource-blocked`, `low-memory-before-start`, no process launched, cleanup verified. Available physical RAM 14,839,242,752 bytes (13.82 GiB), system commit headroom 13,988,331,520 bytes. Tests, Clippy and final UI build did not start.

Every job retained the 14 GiB start / 8 GiB reserve / 6 GiB owned cap, single Cargo job, pinned toolchain and locked/offline shared development profile. No resource limit was relaxed. Lingering VCTIP helpers were stopped only after job ownership was proven and compilation had finished; no user services were stopped.

`Podcast/Workflow96Captures/final-4558474-d8f666d03c4f49879bc66f7490da9fed/source-receipt.json` records the clean production checkpoint, source trees, original workflow receipt hash, binary hashes and six PNG hashes. The full workflow originally recorded parent 13bcc2c while building the final 96 diff. That diff was committed unchanged as 4558474; the original receipt was retained rather than rewritten. Later changes are the explicitly identified test-only follow-up and receipt documentation.

## Actual CPU captures

Directory: `C:/Users/User/AppData/Local/CleanSlate/Podcast/Workflow96Captures/final-4558474-d8f666d03c4f49879bc66f7490da9fed/`.

| PNG | Physical size |
| --- | --- |
| `session-workspace-compact-100.png` | 720×560 |
| `session-workspace-compact-150.png` | 1080×840 |
| `session-workspace-compact-200.png` | 1440×1120 |
| `session-workspace-100.png` | 1000×680 |
| `session-workspace-150.png` | 1500×1020 |
| `session-workspace-200.png` | 2000×1360 |

These rasterize the actual first-frame egui meshes using generated captured PCM, not the design mockups or a native DAW screenshot. Native DPI is separate from UI zoom; CPU raster coordinates and clipping scale into physical pixels. Compact/full 100% and compact 200% were visually inspected, with all six geometry assertions passing. Native pointer/keyboard forwarding, compositor/presentation performance, font matching and subjective speech/transition quality remain unqualified.

No automatic completion of 80 or unblocking of 81–83 is implied. Post-review UI checks, Clippy, normal build/source/hash preservation and refreshed six PNGs are complete as recorded above. Keep 96 claimed until coordinator integration is complete.
