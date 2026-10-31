# Podcast normal native candidate handoff

Source: `b41d51e89a3bc5d413f83c0193b859e2d1a27f27`. The normal Podcast DLL and
both existing ABI hosts compile from `plugins/podcast/Cargo.toml`, with no
acceptance-driver feature. No host was executed in this handoff. The coordinator
explicitly holds execution for the next native check. Installed links and
release publication are unchanged; neither103 nor native80 is closed.

## Built artifacts

Pinned actual Cargo/Rustc1.99.0, shared locked/offline single-job settings and
selected LLD were used for `rustc --lib`, `--example podcast_host_check` and
`--example podcast_vst3_host_check`. Strict `clippy --examples -- -D warnings`
passes (2.69 compiler seconds). Receipts:

- DLL: `20261030-225939-5b226ff32d234087ba57591abfbb9f11` (43.19 compiler seconds).
- CLAP host: `20261030-230137-2f2b6e2ca0d94072b5175b2cc28b7ef7`
  (14.65 compiler seconds; 110.30 seconds including explicit helper intervention).
- VST3 host: `20261030-230328-d1cf694150324315b7b217679b005f95`.
- Strict examples Clippy: `20261030-230332-71d6748e3c7d41febde0d0e3bb990ce8`.

CLAP Cargo exited0 with sole recorded job member VCTIP29920 still alive. Its
owned diagnostic, exact path under MSVC14.44.35207 and creation time
`2026-10-30T23:01:49.5879344-04:00` were rechecked before killing that process
alone. The first path check refused an incorrect14.29 expectation without
action. The successful intervention is preserved separately; this was not
ordinary unattended completion. The first guard then finalized with an empty
job and released its mutex before VST3 started. All final receipts have root0,
no remaining recorded job PIDs and cleanup verified. Timeout/serialization only;
no RAM gate. Ticket109's descendant-accounting caveat remains.

Cargo JSON identifies each exact artifact. SHA256:

| Artifact | SHA256 |
|---|---|
| Normal DLL, both formats | `d8b796862e9c0547d1c82a568708044bc8a64f42fd42023bb59e618e07eba6e2` |
| CLAP host | `a2867e102e55cb7f1f12a8177de25366012020146a2cb73646a618733201de89` |
| VST3 host | `a1b10bef4e713f82acb54b3aed025f917d480dca63c3f6ec3896b4eba7ca6711` |

DLL exports are exactly `ExitDll`, `GetPluginFactory`, `InitDll`, `clap_entry`.
No acceptance driver export is present. Imports include Windows/OpenGL APIs,
DirectML/D3D12/DXGI/DXCore, MSVCP140/MSVCP140_1, VCRUNTIME140/VCRUNTIME140_1 and
UCRT. Full import/export dumps are preserved. This is inventory evidence, not
runtime isolation or inference evidence.

## Uninstalled candidate and runtime inventory

Fresh worktree candidate:
`dist/podcast-candidate/c2d10968a86741eea3195f11969c8153/`.
Both plugin formats match the preserved selected-LLD DLL. Example dependency
compilation subsequently replaced the shared-cache DLL with hash
`31f8cf046846faee13e77927ffabc0188f4822684f2fcfa71c3c982e82f8d34f`.
Use the staged/preserved d8b79686 candidate, not the later shared-cache DLL.

The coordinator-authorized verified Gate runtime source is
`%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/gate93/20261030-native-2e3adbf/candidates/86f0d392624b4b70b3f357e9c7d8914a/`.
DirectML1.15.4 and all four CRT14.44.35211.0 DLLs were checked against its pinned
runtime/MSVC inventories before copying beside both Podcast format binaries.
DirectML license/code-license/third-party notices and the verified runtime
license set were copied and hashed too. No download, dependency selection or
general packaging change occurred. `runtime-inventory.json` records22 matching
source/destination file checks plus Podcast source/DLL identity and runtime
provenance. The pinned runtime/MSVC receipts and a Podcast build receipt are
included. A second check verifies all22 preserved runtime/notice copies and both
preserved plugin hashes.

Complete candidate, host executables in `debug/examples`, JSON artifact
identities, build receipts, import/export dumps and helper intervention:
`%LOCALAPPDATA%/CleanSlate/Podcast/NativeCheckpoints/normal-438e85edaa8a497c8be1385296b9fe22/`.

The existing normal pair can use those preserved paths, after coordinator
handoff, through `scripts/test-plugin-hosts.ps1 -Product Podcast` with
`-PluginDirectory <checkpoint>/candidate -TargetDir <checkpoint>`.
That pair covers normal CLAP state round-trip and existing ABI/process checks;
it does not establish runtime isolation, full Session VST3 state, host-native
editor presentation, listening, PDC/export acceptance or native80 completion.
Full Session VST3 state requires the separate acceptance-driver workflow.

## Granted normal ABI execution

The coordinator subsequently granted the normal pair, using **only** the
preserved candidate/hosts. Exact invocation:

```powershell
& 'C:/Users/User/orca/workspaces/podcast-automix-rust-gui/podcast-seek-resume/scripts/test-plugin-hosts.ps1' -Product Podcast -PluginDirectory 'C:/Users/User/AppData/Local/CleanSlate/Podcast/NativeCheckpoints/normal-438e85edaa8a497c8be1385296b9fe22/candidate' -TargetDir 'C:/Users/User/AppData/Local/CleanSlate/Podcast/NativeCheckpoints/normal-438e85edaa8a497c8be1385296b9fe22'
```

Both actual normal ABI hosts pass. CLAP reports1,858 bit-exact mono/stereo blocks,
1,168 validated shared PCM windows, foreign Probe-state disconnection and
Podcast-state round-trip, offline activation/manual-export label and host restart
request on active render-mode change. Runtime0.72 seconds; largest sampled
process working set23.79MiB. The CLAP host's VST3 factory-symbol observation is
not itself a VST3 process test.

The separate VST3 host reports two actual instances, discovered/set controls,
mono/stereo44.1/48kHz realtime/offline processing,4,048 bit-exact blocks and3,392
shared PCM windows, in-place/separate buffers, missing transport and seeks.
Editor construction/release succeeds without opening a window. Runtime1.01
seconds; largest sampled process working set24.55MiB. Sampling is10ms and covers
these test processes only, not total product/native presentation performance.

Receipt `20261030-230706-9133d7203f934ff3a1c01a707c754a4b` finalizes root0,
no remaining recorded job PIDs and cleanup verified. The receipt/log/diagnostics
and `normal-host-pair.json` are preserved in the checkpoint. That JSON records
the exact script/arguments/hash, artifact sourceb41d51e, execution source241fdd8
(documentation-only delta), both host paths/hashes and scope exclusions. Plugin
and host hashes were rechecked after execution. Staging inventories retain their
historical pre-execution hold; this later receipt supersedes that execution
status only. No DLL or installed-link replacement occurred.

Normal pair success does not prove runtime isolation, complete Session VST3
state, native window/DPI/presentation, listening/PDC/export acceptance, native80
or whole103 completion. The compiler/heavy slot was explicitly released after
this pair for coordinator CPU benchmark work.

## Proposed Session acceptance-driver regression — unrun

Source-only inspection of `scripts/check-podcast-workflow.ps1`, both existing
format hosts, `support/podcast_acceptance.rs`, `support/vst3_state.rs` and
`product-build.rs` identifies a relevant regression after Published moved to Arc.
The graph transport was closed; pinned checkout source supplied the evidence.
No compiler, driver host, release guard or model inference was run for this plan.

The existing workflow checks two actual instances of each format. Its synthetic
48 kHz fixture has196,608 frames per microphone, complete fingerprint blocks,
room/solo/overlap labels and nonzero first/last markers. Maintained Analyze and
independently prepared Float32 final-render WAVs supply the comparison. Realtime
and host-offline Original/Processed/Removed must agree within2e-6, including
latency padding and both ends. Sign-flipping one source must invalidate analysis,
preserve Original and reject Analyze pending example review.

After that processing loop, both format adapters save their real format state,
destroy/recreate two instances, load it and capture40 further blocks. Binding/
flush must succeed, all four examples must remain and readiness must be0 until
explicit Analyze. VST3 uses real bounded IBStream getState/setState calls. This
closes the bounded machine restore regression gap if it passes; it does not
establish owner DAW, acoustic/listening or analysis-persistence acceptance.
The `PASS actual-format Session:` line is printed before the restore assertions,
so a valid receipt must require that line **and final process exit0**.

Proposed execution preserves those exact existing fixtures/assertions while
using the established pinned, serialized per-stage commands:

1. Recheck the preserved d8b79686 normal candidate and normal-host hashes. Copy
   its DLL as a new acceptance directory's normal reference; never use the
   overwritten shared-cache DLL as that reference. Keep the existing normal
   checkpoint/candidate immutable. Mark the fresh directory DO-NOT-INSTALL.
2. Pinned Cargo `clippy --manifest-path plugins/podcast/Cargo.toml --features
   acceptance-driver --all-targets` plus shared locked/offline/single-job args
   and `-- -D warnings`. Build the driver DLL with pinned `cargo rustc --lib
   --features acceptance-driver`, then each named host using `--example
   podcast_host_check` / `podcast_vst3_host_check` with that feature. Each selected
   target gets the same explicit pinned LLD arguments used above. Capture exact
   JSON artifacts, feature sets, source/hash and receipts before later builds
   can overwrite outputs. Preserve all acceptance-only artifacts separately.
3. Stage the already verified DirectML/four CRTs and notices next to the driver
   DLL and both driver hosts, hash checked against the preserved runtime
   inventory. Driver hosts themselves call the maintained analysis oracle and
   therefore need those dependencies. No download or new runtime selection.
4. Compile the exact existing `product-build.rs` with pinned actual Rustc and
   LLD to a small guard-check executable. Run with child-only package
   `cleanslate-podcast`, feature env1 and PROFILE=release. Require nonzero exit
   **and** `acceptance driver must never ship` in stderr; no release dependency
   graph or installable release artifact is built.
5. Run each driver host against the copied normal reference. Each must reject
   it with nonzero exit **and** `Acceptance mode requires the debug-only driver`.
   An unrelated loader error/panic is not a passing negative. Normal export
   inventory/normal hosts must also still show driver absence. Then run each
   driver host against the acceptance DLL, requiring its actual-format PASS
   line and final exit0 after state restoration. Use fresh child-only
   LOCALAPPDATA caches and preserve stdout/stderr/exit status. Existing oracle
   paths under fresh TEMP directories are retained and recorded separately.
6. A finally-style restoration phase, after verified job cleanup, rebuilds the
   ordinary no-feature DLL and both no-feature hosts with the pinned commands.
   Recheck driver absence, ordinary features/JSON identities and preserve the
   restored outputs. Never repair Cargo's shared target by merely copying DLLs
   over feature-built outputs: its artifact fingerprints must reflect the normal
   builds. Record restoration as its own outcome even if acceptance fails. The
   untouched d8 checkpoint remains usable if restoration itself fails.

The stock top-level script invokes unpinned PATH cargo/rustc, repeats the normal
build/check package at both ends and restores normal outputs only on its success
path. Its acceptance children have no individual external timeout. Therefore the
proposed staged invocation reuses its existing checks rather than running it
unchanged or adding a new framework/test seam. Source scripts need no edit.
Each compiler stage uses the existing timeout-only wrapper; positive driver
hosts can be bounded at120 seconds each, negatives/guard at30 seconds. No RAM
preflight/reserve/cap. All jobs remain serialized under the coordinator's slot.

Planning estimate: roughly3–6 minutes warm, excluding a lingering helper; allow
up to10 minutes for feature relinking, two oracle/Analyze runs and normal restore.
Historical full workflow197.99 seconds included helper waiting and is not a
current timing guarantee. Cold dependency invalidation can add minutes. Actual
Analyze already has its own30-second poll deadline and restored-view polling a
3-second deadline. Do not extend those assertions to obtain a pass. If VCTIP
again holds a finished compiler's job, apply only the recorded exact-owned
identity/diagnostic cleanup and preserve the intervention separately.

Risks before execution: driver feature builds overwrite shared DLL/host outputs;
normal restoration must finish or be explicitly reported incomplete. Both hosts
perform genuine model inference and filesystem-backed oracle/export work on
synthetic fresh paths. Negative controls must match their intended reason,
including cases that print a PASS before later assertions. Runtime isolation,
native editor presentation, owner listening and DAW PDC/export remain separate.
This is an unexecuted plan pending the coordinator's compiler/inference slot.
