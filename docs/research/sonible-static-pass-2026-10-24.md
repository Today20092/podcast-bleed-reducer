# sonible supplied-package static investigation

Bounded first pass for [ticket 79](../../.scratch/offline-supervised-editor/issues/79-sonible-static-reference-investigation.md), following the accepted [Session/Live Mic contract](../../.scratch/offline-supervised-migration/plugin-modes-interview.md). The owner supplied seven extracted download folders. Selected payloads were materialized without installing or executing any sonible program. This establishes artifact structure and limited native evidence, not complete algorithm recovery or measured processing behavior.

## Acquisition and reproducibility

Sources are under `C:/Users/User/Downloads/To Review/`, in the seven owner-named folders. Each contains an Inno Setup EXE and PDF manual. Their exact paths, lengths and SHA256 values are retained in `C:/Users/User/Documents/Codex/Investigations/sonible-2026-10-24/source-inventory.json` (14 source files). The gate installer SHA256 `05eb409cd5f3ecbfc0ffe0041e775a522986bc6aa4a92725e922d7bb1fac1210` still matches its original REA open identity.

The existing 7-Zip/NanaZip command identified the setup wrapper. No local Inno extractor was found through the Everything index. Portable innoextract 1.9 was downloaded from its [official distribution](https://constexpr.org/innoextract/) into `%LOCALAPPDATA%/CodexTools/innoextract-1.9`; archive SHA256 `6989342c9b026a00a72a38f23b62a8e6a22cc5de69805cf47d68ac2fec993065`. No PATH or global application registration was changed. All seven installer listings identify Inno setup data 6.1.0 (unicode), and listing/extraction completed successfully with this tool.

Selected output is in `C:/Users/User/Documents/Codex/Investigations/sonible-2026-10-24/`. Per-product extraction logs retain exact operations. Filters materialize VST3 files, manuals and shipped runtime DLLs; smart:gate additionally includes its ten profile libraries. Other formats, temporary prerequisite installers and the large EQ/chain profile collections were not materialized. On this Windows build, include-directory filters required backslashes; an initial forward-slash pass only selected top-level sys/userdocs paths, then the corrected filters added VST3/profile files. No setup EXE, VST3 or profile library was loaded as a running target. Trusted extraction, PDF/PE parsing and Ghidra processes were the analysis tools.

## Actual VST3 payload inventory

All seven payloads were identified by the REA portable PE resource parser, with complete coverage of each admitted resource table. Full hashes, byte locations and records are retained in `pe-resource-evidence.json` in the investigation directory. The table gives exact subject SHA prefixes for navigation; the complete record remains authoritative.

| Product/version | VST3 bytes | SHA256 prefix | REA Evidence ID |
|---|---:|---|---|
| smart:gate 1.0.1 | 54,402,264 | `edeb18dcb0b7` | `ev_deb358ed66572fa68789acbfb6829ea93af4b33d5e711c15007fd7140b64bd2a` |
| smart:comp 3 1.0.2 | 84,871,888 | `48d236271593` | `ev_afe78aaa2f5b044120fb5380aafd72dac400df6912f29e0cb097db52a7481bdb` |
| smart:EQ 4 1.1.1 | 67,169,208 | `acd116e1ef88` | `ev_301d29e22a5a0fbb52c6c62a268e0131db18b23ab807e7217cec4a19127571c3` |
| smart:chain 1.1.0 | 191,923,920 | `f9ceb8e06ac7` | `ev_75b419daf41c2689942a6d57aa75df18dc31a209dccbda2a6ffe40a0927ec85d` |
| smart:reverb 2 1.0.3 | 78,133,696 | `2ee4aacb5d8b` | `ev_782afeb1b4240b3035619c5f7253120baa292ba07bef90215c60e046735185c9` |
| smart:deess 1.0.2 | 66,057,944 | `c17c876ef56d` | `ev_0c81b9f55d1c603bea88634512963d573572cefda96229449757d897d14a5f4c` |
| smart:limit 1.2.0 | 57,755,856 | `4b0c6fd46d29` | `ev_4f8a554277d969d07e2e88318f06818442f3aedc22206e13e8326bc7d9326f65` |

These resource results establish identity and resource structure only. Neither the sparse Windows resource tables nor DLL sizes establish the implementation/framework, quality or runtime cost. The outer smart:gate installer is x86; the actual plugin and Speech.nn payload are x86-64. An installer architecture is not the plugin architecture.

## smart:gate model/inference findings

**Observed source bytes:** the installer includes `sonible_onnxruntime_v1-15-1.dll` and ten equally sized (1,649,152 bytes each) profile payloads: Agit, Bass, Egit, Hihat, Keys, Kick, Snare, Speech, Tom and Vocals, all with `.nn` suffixes. Speech and Vocals are different files: Speech SHA256 `4733eb7a586cfed02b0844c392db4025659d301ec95c8d5b3edf6779ecf89f16`; Vocals SHA256 `93537dce5abf14371717b9fceabf4dabbec5eab06d80248a6844fe90e913ab50`.

**Observed PE linkage:** a separate read-only source PE parser found a smart:gate VST3 import descriptor for `sonible_onnxruntime_v1-15-1.dll` at file offset 52,178,204, importing `OrtGetApiBase`. This is stronger than a filename-only ONNX inference, but does not observe an inference session running or prove how frequently any path executes. The parser and complete imports/exports/sections are retained as `inspect-pe.ps1` and `pe-import-export-evidence.json`. This evidence is distinct from REA decompilation.

**Observed native payload structure:** despite its suffix, Speech.nn is a PE x64 DLL, not a plain standalone ONNX file. Its .text section is 71,168 source bytes; .rdata is 1,560,064 bytes. Five named exports were observed. REA/Ghidra 12.1.4 decompiled the exports and three focused helpers. One export returns constant `0x139498`; another returns a pointer to `DAT_180013ab0`, mapped to source offset `0x124b0`. The latter data does not begin with an established plain-model header. Other exports reach native container helpers. These observations support a data-bearing native wrapper interpretation; the export names and size/pointer alone do not establish its network topology, tensor layout, preprocessing or encoding.

Key REA IDs: `ev_4aa3ca64b008729581c7de4a90d14010eec241868061f9736e17e3c707cec133` (constant-returning export), `ev_3da8436e4200c658d971683f49bfc0b2cfafa3168f1b17fcd9010b03166e140f` (data-pointer export), and `ev_0ecf6898eaa0d493e8247d9449fd1f73987e6b0568235fe6afae41a595ac605b` (focused container helper). `speech-rea-evidence.json` was exported **before** closing the native session: 12 complete records, 58,311 bytes. Ghidra's profile digest is `189821c22f9a3c65d594fc3eda7b5a03806941e5d6504c95c1b268b890a7c7ff`.

Bounded Ghidra-defined-string searches for ONNX and model/input/output/tensor/shape/spectral terms returned no matches (`ev_2141b17aaba16cff37d7f09db32046fc76f02013d45480e1fd3d68a9287c9a1f`, `ev_33a005989ef6c6c8f788461ed7412e4eb64672ccb3e1177ced365060411fd997`). This does not establish that those concepts/bytes are absent. A broad raw-string scan was stopped and supplies no findings. The full large plugin binaries were not auto-analyzed in Ghidra; the small Speech.nn target was, and the native session was closed afterward.

**Inference:** the combination of named profile libraries, documented source-category detection and static ONNX linkage is consistent with learned target-source detection feeding gate control. Exact runtime selection, graph/operator architecture, feature normalization, training method/data, state/window dimensions and confidence calibration remain unknown. No model was invoked, decoded into a claimed network or incorporated into CleanSlate.

## Manual-level findings and implications

The supplied smart:gate manual describes source-profile activity separately from the gate envelope, with level bias, threshold/tolerance, timing and impact controls. This supports the accepted Live Mic separation: an estimated speech/activity signal drives a conservative temporal gain decision. It does not establish personal voice recognition or that an activity percentage is a calibrated probability of correctness.

The supplied smart:comp 3 manual explicitly describes grouped remote control, parameter linking and batch learning while each instance processes/analyzes independently. That is a useful control-view reference, not evidence of joint cross-microphone speaker ownership. CleanSlate's Session mode still needs the richer multitrack evidence and persistent source-time identity from the accepted application approach.

Practical recommendation: retain the accepted Session/Live Mic distinction. Build Live Mic around a properly benchmarked speech detector and a separately tested gate/fade layer; the existing embedded Silero path is an implementation candidate, not currently callback-ready or proven equivalent to sonible. A vendor's ONNX dependency does not require copying its payloads or adopting an unmeasured model. Keep the persistent timeline and shared instance control work separate from detector choice.

## Checklist and remaining unknowns

- **Answered:** selected VST3/manual/runtime files can be extracted from all seven supplied installers without installation. Static inspection does not require activating the products.
- **Answered:** smart:gate's shipped speech/vocal payload identity, native wrapper structure and static ONNX import are recorded with source evidence.
- **Partially answered:** broad detection/control architecture is supported by binary linkage plus supplied manuals; actual inference call chain and complete model architecture are not recovered.
- **Unresolved:** quiet-speech, keyboard/noise and competing-voice behavior; onset loss, gate pumping, measured latency, CPU/working set, learning persistence and actual remote host-automation semantics. These require focused additional analysis and/or a runnable activated/trial setup with a declared test lifecycle.
- **Not claimed:** equivalent signal behavior, a working cloned detector, recovered training data, network weights/topology, or exhaustive investigation of all seven DSP engines.

No sonible installer or audio plugin, prerequisite installer, DAW, trial/activation or registration was executed. Source folders were left in place; extracted files and analysis tools/evidence stay outside the CleanSlate repository. Product code, build artifacts and development links are unchanged by this pass.
