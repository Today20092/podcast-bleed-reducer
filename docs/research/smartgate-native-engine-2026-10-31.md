# smart:gate 1.0.1 native engine investigation

2026-10-31 continuation: the owner resumed unrestricted research. A subsequent
Ghidra import completed successfully, and a corrected persistent client is now
running the focused code investigation. Independent ordinary disassembly and
authenticated decoding have recovered both profile model graphs; see the
[authenticated-model report](smartgate-authenticated-model-2026-10-31.md).
The bounded-failure record below is historical, not the current coverage limit.

Status: saved and stopped. **The bounded main-VST3 analysis timed out before Ghidra completed auto-analysis. No main-VST3 pseudocode, xref chain, model topology, spectral preprocessing or gate equation was recovered in this pass.** The confirmed profile-wrapper code below comes from retained complete REA records; it is not a substitute for that missing main-engine trace.

## Target and reproducibility

The installed target is `C:/Program Files/Common Files/VST3/smartgate.vst3/Contents/x86_64-win/smartgate.vst3`, 54,402,264 bytes, version 1.0.1. Its independently rechecked SHA-256 is `edeb18dcb0b7557123cc46036fd17b2d08cbd45d3bf1cb882e4b5a8a4a35f95c`, matching the retained installer-derived artifact.

New complete evidence is retained outside the repository at `C:/Users/User/Documents/Codex/Investigations/smartgate-native-2026-10-31/`. It includes `installed-target-identity.json`, `providers.json`, the complete `tools-list.json`, and a reviewable `rea-session.cjs` driver. The driver uses public MCP SDK 2.3.1 exports and the public `npx --no-install rea-agents@6.7.0 mcp` launcher. The schema-only server identified itself as REA 6.7.0 and closed cleanly. No Codex registration or engine configuration was changed.

The installed provider is Ghidra 12.1.4 PUBLIC with Eclipse Adoptium Java 21.0.12. The supported `open_binary` schema exposes no CPU-count parameter. A single persistent server is required to retain one analyzed target across focused queries; CLI snapshots alone are query caches, not persistent open databases.

The first timeout-only guarded target launch was denied by the existing `Global\\CleanSlate.HeavyTask.v1` mutex. `native-session-01/owned-session.json` records `heavy-task-busy`; the target was never opened in that attempt. No competing process was stopped. The proposed open request has an 1100-second client timeout within the existing 1200-second owned-task timeout, leaving time for evidence export and orderly close. No RAM threshold or memory cap is applied.

After the GUI native check released the mutex, the owned persistent session started at 2026-10-31 05:11:23 UTC (`native-session-02/session.json`, driver PID 31376, server PID 16800). `open_binary` verified and bound the target immediately; the first native query triggers lazy import/analysis. An incorrectly short 60-second timeout on that first query cancelled its import. The already queued next query began a fresh import at 05:12:30 UTC with the corrected 1100-second request timeout, within the original outer 1200-second bound. The failed query is retained as `response-2.json`; it is not native evidence. No further query is queued until the corrected import's outcome is known. REA's observed engine command selects a default Java `-Xmx2G` heap and two GC threads; that is an engine default, separate from the removed workspace memory guards.

That import reached REA's independent 330,000 ms provider startup deadline and returned `provider_timeout` at 05:18:00 UTC (`response-3.json`, including complete retained startup diagnostics). Its owned Java/project were removed. The session then exported Evidence, closed and released the mutex normally; wrapper completion does not mean native analysis passed.

The owner authorized one corrected run using the error's supported process-local `REA_GHIDRA_STARTUP_TIMEOUT_MS=1000000` setting. `native-session-03/session.json` records its start at 05:18:55 UTC, driver PID 20808/server PID 6364, with 1100-second native requests and a 1200-second outer timeout. It applied no analyzer exclusions or persistent configuration changes. Any tool error exits through export/close rather than queuing another import.

That corrected run also returned `provider_timeout` after 1,000,000 ms, at approximately 05:35:42 UTC. The same Ghidra process passed the previous 330-second cutoff, so the override was effective. Progress diagnostics show constant propagation, MSVC RTTI/vftable recovery and application of `windows_vs12_64` data types. Warnings include invalid decompiler references and C++ namespaces exceeding Ghidra's name-length limit. None of those partial phases establishes a completed call graph. The error response retains the startup diagnostics; the separately copied progress logs and read-only thread snapshots describe the owned Ghidra JVM, not vendor runtime behavior.

Cleanup completed normally: the final owned-task receipt has `cleanup_verified=true`, `remaining_process_ids=[]`, exit 0 and 1011.49 seconds elapsed. Exact owned driver/server/Java processes were absent and the shared mutex was available at 05:35:55 UTC. Exit 0 here means the driver preserved the error and cleaned up successfully; **native analysis failed**. The close snapshot explicitly contains zero primitive entries, zero workflow entries and zero Evidence records. No partial Ghidra database is offered as a reusable completed analysis. No additional retry, implementation ticket, backend change or GUI change followed, as the owner requested.

## Retained observations and their limits

Read [the earlier follow-up](smartgate-followup-2026-10-25.md) and [the static pass](sonible-static-pass-2026-10-24.md) for the original complete records.

- The main VST3 imports `OrtGetApiBase` from `sonible_onnxruntime_v1-15-1.dll`. This is a native dependency observation; it does not identify invoked inference API slots, tensor dimensions, or scheduling.
- Ten profile `.nn` files are native x64 PE DLLs. Speech and Vocals have identical `.text` bytes and different `.rdata` bytes. Shared loader code with differing data is consistent with this observation; identical networks or behavior are not established.
- Retained Speech export dossiers expose a data pointer and a 1,283,224-byte constant. They do not establish the pointed data's decoded format or a model graph.
- Main-VST3 RTTI contains a spectral `OnnxInferenceInterface` specialization and `OnnxInferenceModel`/`OnnxSessionManager` types. Strings are search anchors, not proof of the live preprocessing or detector path.

### What the retained Speech code actually establishes

The complete `speech-rea-evidence.json` was reviewed again during the current import, without starting a second native analysis. Its Speech subject is SHA-256 `4733eb7a586cfed02b0844c392db4025659d301ec95c8d5b3edf6779ecf89f16`.

| Export | Observed decompiled path | Limit |
| --- | --- | --- |
| `initSkynet` | Returns constant `0x139498` (1,283,224). | The name does not prove initialization or inference. |
| `sirNoTouchingTheCodePlease` | Returns `&DAT_180013ab0`. | A returned data address does not identify its decoded model format. |
| `breakItDown` | Forwards its argument to `FUN_1800037b0`; that helper accesses global `DAT_180190430`, compares helper-returned positions and returns zero or a value derived from the selected element. | Container-style lookup is an inference; this is not a recovered neural graph. Decompiled wrapper return types are not reliable enough to declare a public ABI signature. |
| `cantTouchThis` | Calls `FUN_180003690`, which forwards global `DAT_180190430` to another helper. | Cleanup/reset semantics require the callee's implementation; the export name alone is insufficient. |
| `stopHammerTime` | Calls `FUN_1800036b0`, which uses MSVC thread-initialization guards, registers exit callbacks, invokes once-initialization helper `FUN_1800062f0` and then forwards another global object. | One-time/container initialization is visible; no model layer or feature transform is established. |

Relevant complete Evidence IDs in that retained bundle:

- Size return: `ev_4aa3ca64b008729581c7de4a90d14010eec241868061f9736e17e3c707cec133`.
- Data pointer: `ev_3da8436e4200c658d971683f49bfc0b2cfafa3168f1b17fcd9010b03166e140f`.
- Lookup helper: `ev_0ecf6898eaa0d493e8247d9449fd1f73987e6b0568235fe6afae41a595ac605b`.
- Guarded initializer: `ev_d3896e481bbc25a8c2c0ee60ab2984d757e57adf592bd80406e4da2394530cdd`.

The follow-up bundle `smartgate-speech-followup-evidence.json` contains the deeper helper dossiers. In particular, `ev_acc4ffb1c3b905e6ec26ae47df25f76b702685b8637d6d3bfd8e9dbcd2059af7` covers `0x1800062f0`, and `ev_068c49086cfdd38a9b734694e81888093b2a540aca6a7264a151fd9ede8c1e92` covers container iteration at `0x1800038a0`. These are retained code observations, not a new completed main-VST3 trace.

## Related research bundle

- [Complete vendor control outline](sonible-smartgate-controls-2026-10-31.md).
- [Our engine ownership versus smart:gate](main-engine-vs-smartgate-2026-10-31.md).
- [Current Silero model provenance](silero-model-provenance-2026-10-31.md), including the v6.2 correction. The model and the Desktop engine's queue, fallback and gate policy are separate responsibilities.

## Unresolved proof and comparison limits

The intended main-code queries targeted native references to the ONNX binding, profile export-name resolution, product session/input-dimension diagnostics and `gate_tolerance`/`level_bias` parameter paths. The first native query never reached readiness, and the queued procedure-name search was not executed after the error. This is a coverage failure, not evidence that those functions, references or features are absent.

Unknown: model layers and tensor dimensions; input sample/window/hop handling; profile selection's actual data route; detector score calibration; gain/timing formulas; callback versus worker scheduling. In particular, spectral RTTI does not establish an FFT configuration, learned feature representation, temporal context, causal/recurrent architecture or active inference route. The profile data bytes were not decoded or identified as a model graph. A missing familiar model-file header does not rule out formats without a universal magic header.

For the podcast comparison, the defensible boundary is a native ONNX dependency plus profile-specific data/wrappers and spectral-interface type candidates, versus our independently documented model and engine policy. This cannot establish model-quality equivalence, prove that one algorithm is better, or justify reproducing sonible's layers or transfer equations. Vendor control behavior is documented separately in the linked control outline. No proprietary model payload or implementation was copied into CleanSlate.

The official [ONNX Runtime v1.15.1 C API header](https://github.com/microsoft/onnxruntime/blob/v1.15.1/include/onnxruntime/core/session/onnxruntime_c_api.h) was retained with its source URL/hash as interpretation support for a prospective API trace. It establishes that header's API contract; it does not prove which API calls this plugin executes. Context7 returned a monthly-quota error, so this primary source was used directly. REA's timeout remediation agrees with its [upstream installation documentation](https://github.com/morluto/rea/blob/main/docs/installation.md); the actual installed-version error and process observations are the authority for this run.

No vendor binary, model, DAW, activation path, installed link, GUI source or saved tool configuration was executed or altered by this investigation. Only REA/Ghidra and diagnostics of the owned analysis JVM ran.

## Complete evidence inventory

The new investigation root is `C:/Users/User/Documents/Codex/Investigations/smartgate-native-2026-10-31/`. `evidence-manifest.json` lists the exact byte count and SHA-256 of every retained new evidence file, plus the referenced older bundles. It covers the complete MCP responses rather than shortened console previews, the driver and request files, the failed attempts, exact session identity, progress diagnostics, export/close results and cleanup receipts. The final failed native query is `native-session-03/response-2.json`; it has no successful native Evidence ID. Do not mistake the zero-record exported bundle for missing saved diagnostics: the complete failure response and progress files are retained separately.

The final manifest contains 61 new evidence files and eight retained-reference files. Its SHA-256 is `d4822819b12d5c86ace649a61f4ced3d7ab74b82a0ef7e7cd9880a9df35a36c9`.
