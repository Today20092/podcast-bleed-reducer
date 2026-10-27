# Gate 93 runtime packaging — bounded offline findings

## Primary-source licence follow-up after reboot

2026-10-27: Context7 resolution reached quota. The official [NuGet 1.15.4 page](https://www.nuget.org/packages/Microsoft.AI.DirectML/1.15.4) links a version-specific [Microsoft DirectML software licence](https://www.nuget.org/packages/Microsoft.AI.DirectML/1.15.4/License). It is a separate Microsoft binary licence, not the MIT licence of repository examples. Section 1(a) permits Windows/Xbox use and distribution within the described machine-learning applications; 1(c) addresses separate third-party notices; 3(c) preserves Microsoft/supplier notices; 3(e) restricts standalone redistribution outside the stated grant. Read the complete terms for scope and other conditions. This identifies a primary version-specific licence, not legal sign-off, verified compliance, a user agreement acceptance or publication authorization.

The cached DLL version is 1.15.4, but its exact bytes have not been matched against the official NuGet binary. The 192.92 MB package was not downloaded for this light lookup. Keep its current Pyke-cache provenance/hash explicit; verify package provenance and applicable accompanying notices before claiming distributable packaging. No runtime/provider execution was inferred from this lookup. The original offline observations below retain their historical unknowns; this follow-up supersedes only the absence of an identified version licence.

2026-10-27. Read-only investigation of the retained Gate worktree and installed
registry/build artifacts. No compiler, index, native host/runtime, download,
network request, resource poll or source edit was performed. Primary graph has
no Smart Gate coverage; manifests and narrowly named local source files supplied
the evidence. Gate remains at the doc-only 93 checkpoint `d734bbd`, based on
accepted product source `c7513c8`.

## Verified

- `plugins/gate-detector/Cargo.toml` uses `ort = "=2.0.0-rc.10"` with defaults;
  its lock pins `ort` and `ort-sys` to rc.10. The installed `ort` manifest enables
  `std`, `ndarray`, `tracing`, `download-binaries`, `copy-dylibs` by default, and
  passes features to `ort-sys` with that dependency's defaults disabled.
  `load-dynamic` is separate and is **not** in the retained lib-ort fingerprint
  feature sets. The default features are not evidence of loading onnxruntime.dll.
- Installed source root:
  `C:/Users/User/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`.
  `ort-sys-2.0.0-rc.10/build.rs:8` pins ONNX Runtime **1.22.0**.
  `real_main` prefers an available static archive unless dynamic linking is
  explicitly preferred. Existing build outputs
  `%LOCALAPPDATA%/CleanSlate/BuildCache/debug/build/ort-sys-9b13179d112275ed/output`
  and `ort-sys-f012cd542851e130/output` explicitly say
  `cargo:rustc-link-lib=static=onnxruntime` and link the retained Pyke cache.
  `ort-2.0.0-rc.10/src/lib.rs` calls `ort_sys::OrtGetApiBase()` in its non-dynamic
  path; the DLL loader/ORT_DYLIB_PATH branch is conditional on load-dynamic.
- The retained cache is
  `%LOCALAPPDATA%/ort.pyke.io/dfbin/x86_64-pc-windows-msvc/540D19B3379FDA6FB8F7280D8C15EFDE20ED225A67A357A6DAE38C4300FE190D/onnxruntime/lib/`.
  It contains `onnxruntime.lib` (303,265,000 bytes) and **DirectML.dll**
  (18,527,776 bytes), file version `1.15.4+241025-1615.1.dml-1.15.fac7597`,
  SHA256 `9c9e6d822561c6c41b90e6994b3e8857cf1d66dbfb1e0c4c799c7c89b4e92da1`.
- Bounded PE-header/import-table reads of the preserved category harness
  `%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/gate-detector/categories-9cbfc2ab/feasibility.exe`
  (21,849,600 bytes; rehashed
  `9cbfc2ab82c10cdd7a957a5df4cd2414833992ab326a80ddc8657137489f3541`)
  find ordinary imports of **DirectML.dll**, `dxcore.dll`, `dxgi.dll`,
  `d3d12.dll`, `MSVCP140.dll`, `MSVCP140_1.dll`, `VCRUNTIME140.dll` and
  `VCRUNTIME140_1.dll`, plus Windows/API-set/UCRT libraries. No onnxruntime.dll
  import or delay-import directory was present. This corroborates static ORT
  **with additional native DLL prerequisites**, not a fully standalone binary.
- `copy_libraries` in ort-sys creates native-library symlinks/copies beside build
  outputs (`debug`, `examples`, `deps`). The present `debug/DirectML.dll` and
  `debug/examples/DirectML.dll` are symlinks into the Pyke cache above. The
  preserved category executable directory has **no DirectML.dll**. Its prior
  hash-preservation protects executable bytes but is not a self-contained package.
- `plugins/gate-detector/src/lib.rs` embeds `assets/silero_vad.onnx` using
  `include_bytes!` and prepares it with `commit_from_memory`. The model is
  Silero v5, 2,327,524 bytes, SHA256
  `1a153a22f4509e292a94e67d6f9b85e8deb25b4988682b7e174c65279d8788e3`.
  This implementation requires **no model sidecar or runtime model download**.
- Current `plugins/smart-gate/Cargo.toml` does not depend on gate-detector/ort.
  Static PE reads of preserved 91 `CleanSlate Smart Gate.clap` (9,240,576 bytes)
  find neither ORT nor DirectML imports. Thus current pass-through qualification
  does not establish deployment of a future inference-linked Gate.

## Concrete packaging gap and limits

`scripts/build-smart-gate.ps1` currently stages the plugin DLL as CLAP/VST3,
README and root LICENSE only. Once 93 links the detector, that is insufficient
evidence of an independently loadable package. Its native imports must resolve
from the actual DAW loader environment, not accidentally from Cargo cache links,
developer PATH, working directory or already-loaded host modules. An ordinary
DirectML import must resolve before plugin code can provide a bypass fallback.

The existing `docs/licenses/embedded-model-runtime.txt` contains Silero MIT,
ONNX Runtime 1.22.0 MIT and upstream third-party notices;
`bundled-dependencies.txt` and `msvc-runtime.txt` also exist. The Gate script does
not currently stage them. No DirectML-specific redistribution notice/license was
found in the inspected cache root or these notice files; applicable DirectML
redistribution terms have **not been established** by this offline pass.

Inference: the retained 92 path is static ORT but depends on DirectML and MSVC
runtime resolution. This does **not** establish that inference uses a GPU or
DirectML execution provider: Detector configures one intra-op/inter-op thread
and no explicit GPU provider. Execution-provider use was not measured.
Future feature unions, linker flags/environment and the new Gate artifact need
fresh verification; neither future DLL imports nor clean-machine/DAW loading,
minimum Windows compatibility or redistributable licensing passed here.

## Smallest coherent next check, after memory headroom returns

Implement 93 with its default bypass/explicit arm contract, then inspect the
**new** plugin's imports and stage verified prerequisites plus applicable notices
in the product's own candidate paths. Verify the actual host loader search-path
strategy; do not assume a DLL beside a plugin is automatically discoverable.
Run an isolated guarded CLAP/VST3 load/state/inference check with candidate-local
runtime resolution and no developer cache/PATH assistance. Alternatively, a
verified CPU-only ORT link strategy could avoid DirectML, but the current defaults
already omit the directml Cargo feature: merely disabling that flag is not proof
of removing the imported DLL. No dependency/link change is selected in this note.
