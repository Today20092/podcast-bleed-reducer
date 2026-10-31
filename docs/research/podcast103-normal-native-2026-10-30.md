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
