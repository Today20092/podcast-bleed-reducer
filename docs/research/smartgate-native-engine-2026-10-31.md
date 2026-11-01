# sonible smart:gate: actual native engine and model research

Research completed against the installed Windows VST3 on 2026-10-31. Full Ghidra auto-analysis succeeded; focused code dossiers, authenticated model extraction and structural graph parsing establish substantially more than library names or vendor descriptions. The core is a recurrent spectral activity detector feeding independently parameterized spectral attenuation and a separate time-domain PostGate stage. This report does **not** claim every instruction, public parameter registration, runtime latency or subjective quality has been recovered.

This is research and a design recommendation, not an implementation specification or authorization to copy proprietary models. No vendor plugin, extracted model or DAW was executed during this investigation. No activation, installation or GUI changes were made by the analysis track.

## Research bundle

Read this entry report with these supporting reports, whose detailed evidence and qualifications remain applicable:

- [Authenticated profiles, model graph and Silero comparison](smartgate-authenticated-model-2026-10-31.md).
- [Profile packaging and retained export evidence](smartgate-profile-packaging-2026-10-31.md).
- [Actual PostGate DSP](sonible-postgate-dsp-2026-10-31.md).
- [Primary spectral gain equations](sonible-spectral-gain-2026-10-31.md).
- [Native parameter/controller mappings](sonible-native-parameters-2026-10-31.md).
- [Native interface/routing proof](smartgate-native-routing-followup-2026-10-31.md).
- [Official controls and operating workflow](sonible-smartgate-controls-2026-10-31.md).
- [Our engine ownership and algorithm comparison](main-engine-vs-smartgate-2026-10-31.md).
- [Our actual Silero model provenance](silero-model-provenance-2026-10-31.md).
- [Earlier native static pass](sonible-static-pass-2026-10-24.md) and [profile-wrapper follow-up](smartgate-followup-2026-10-25.md).

Some supporting reports preserve earlier discovery stages. Where they say a body was requested, the dossier index and this final synthesis identify the subsequently obtained body. An unresolved public-to-internal binding is still unresolved unless the supporting report supplies that proof.

## Artifact, tools and coverage

Target: `C:/Program Files/Common Files/VST3/smartgate.vst3/Contents/x86_64-win/smartgate.vst3`, version 1.0.1, 54,402,264 bytes, SHA256 `edeb18dcb0b7557123cc46036fd17b2d08cbd45d3bf1cb882e4b5a8a4a35f95c`.

Tools: public `rea-agents@6.7.0 mcp`, installed MCP SDK 2.3.1, Ghidra 12.1.4/Java 21, ordinary MSVC disassembly and PE/RTTI parsing, Node built-in authenticated cryptography, and version-pinned official ONNX Runtime 1.15.1 headers/FlatBuffer schema. The public tools/list and actual schemas were retained. This was an owned persistent stdio session, not a claim that the parent chat's MCP registration reconnected.

The completed full inventory has 89,365 unique entries: 88,333 local and 1,032 external, with 539 local thunks and 86,374 auto-generated FUN names. This is analysis coverage, not a count of fully understood functions. The final focused tranche exports 63 function dossiers with full returned Evidence, decompiler bodies, caller/reference metadata and assembly. No second full inventory was needed. Most names are stripped; class identity was established through validated MSVC complete-object locators, type descriptors, hierarchy records and actual vtable slots rather than adjacency guesses.

Live analysis was configured with a 32 GiB maximum heap on the approximately 40 GiB/32-logical-CPU host, Ghidra `cpu.core.override=32`, GC workers 32 and compiler workers 16. Effective JVM settings were captured with jcmd. These are capacities, not promises that every analyzer parallelizes. There was no RAM preflight, memory guard, outer runtime wrapper, global build mutex or arbitrary analysis deadline on the successful session. Provider/SDK timers used their maximum supported positive signed-32-bit value, 2,147,483,647 ms (about 24.85 days); zero was not assumed to mean unlimited. The supported provider uses an ephemeral read-only Ghidra project. Evidence/snapshots are retained, but are not misrepresented as a reusable persisted Ghidra database.

## Actual model loading and architecture

The native initializer at `0x180747860` calls OrtGetApiBase, requests API version 15 and stores the returned API table at `0x1832b4230`. The version-pinned public C header identifies the actual table slots used for CreateSessionFromArray, Run, tensor construction and shape inspection. This establishes real ONNX Runtime calls, beyond an imported DLL name.

The profile loader at `0x1818ab670` resolves the profile's data and length exports through `0x180cd8520`, transforms the returned bytes through `0x180cd8610`, then passes the resulting buffer to the in-memory session wrapper `0x180cb6b10`. The Run wrapper `0x180cbd460` supplies named input/output OrtValues and obtains the actual session input/output counts. The reviewed path is inference/session construction, not a model-training API.

Speech and Vocals .nn files contain native wrappers around 1,283,224-byte payloads. Code establishes AES-128-GCM, with a fixed model-packaging key/nonce in the VST3, ciphertext length minus 16 and a trailing 16-byte authentication tag. Offline decoding authenticated **both** complete payloads. There was no key guessing, vendor execution or activation manipulation. These constants package model assets; they are not personal credentials. Decoded files remain outside the product repository.

Each plaintext is a 1,283,208-byte ORTM FlatBuffer, not raw ONNX protobuf. SHA256:

- Speech: `498cf36ee6f7d73daaa673700a462d5d603cb0d0ac7fa615b82975de323dab14`.
- Vocals: `9270e7e23f9c826e01a07c3e66a45b5227e1e1f7c07a78540614ada4026e7527`.

Parsing against the official schema recovers format 5/IR 7 and raw producer metadata `pytorch` / `6f86190`. The two profiles have identical recursive connectivity, attributes, tensor metadata and initializer shapes/types, but different values: 29 of 64 top-level initializer hashes match and 35 differ. They are therefore distinct parameterizations of the same recovered graph; this says nothing about relative quality.

The main graph has 88 nodes, 11 Conv nodes, two forward GRUs with hidden sizes 192 and 96, a Sigmoid head and one If. Including the two If branches gives 98 stored nodes. The main 64 initializers contain 286,581 stored values; this is not asserted to be the learned parameter count. The active detector uses complex spectral input `[2,1025]`, two 48-band feature rows `[2,48]`, a convolutional cache and active GRU states. Exact tensor shapes/operators are in the authenticated-model report and JSON.

The scalar head is:

`out_activity = 10^((72 * sigmoid(logit) - 72) / 20)`

with logit from GRU-96 and a `[96,1]` MatMul plus bias. Its approximately 0.000251..1 range is amplitude-like; it must not be treated as a raw calibrated class probability. The training objective, dataset, labels and calibration are not recovered.

Two traps matter. First, `out_y` is **not** a learned separated spectrum: two ScatterND operations overwrite all 1025 real values with ones and all 1025 imaginary values with zeros. Second, the apparently suggestive h0..h3 and h_emb inputs have no graph path to activity, active recurrent/cache outputs or the learned score. The mode-controlled branch passes these five input states through or multiplies them by zero. It is not evidence of speaker identification, model-weight adaptation or an active-GRU reset.

## Native input, adaptation and recurrent feedback

Validated model secondary-interface slots resolve to actual providers `0x181898190` and `0x181898090`. Reading only the base template vtable would incorrectly yield no-op methods. The RTTI-based secondary table at object offset 8 removes that ambiguity.

The native processing body `0x1818986b0` forms each positive-bin power from real squared plus imaginary squared, then sums through 48 weighted frequency bands. For current band powers E and running reference R, the reviewed scalar/SSE implementation computes:

`R_i <- alpha * R_i + (1-alpha) * E_i`

The two Mel-feature rows are the current E values and `E_i / (sqrt(R_i) + 1e-13)`. Thus the second row is an adaptive feature normalization/reference path, not a duplicate channel and not evidence of a second neural model. Reset initializes the 48 reference values along a 0.001-to-0.0001 ramp. Setup selects a 44.1 kHz grid for integer rates divisible by 44100, otherwise a 48 kHz grid, with alpha around 0.988/0.989. The bank geometry is a 2048-point positive-frequency grid with 1025 bins and 48 weighted bands. This does **not** prove actual runtime sample-rate conversion, hop duration or total plugin latency. Alternate dispatched CPU kernels were exported; exact bitwise equivalence across all kernels is not claimed.

The input provider deinterleaves the spectral data into planar real and imaginary 1025-element rows and copies the two 48-value rows at the correct tensor offsets. The mode scalar is genuinely copied into its tensor. The output provider copies scalar output zero into the returned activity value; under an instance flag it also copies planar out_y back to a spectral block. Because the recovered out_y is constant complex unity, its name alone must not be used to claim neural source separation or direct replacement of audible audio. How every public mode selects that native flag remains unproved.

State initialization at `0x180cbfbd0` builds input and output tensors over the **same backing arrays** for the persistent recurrent/cache states. This establishes in-place state feedback between Run calls. Native reset at `0x180cb9840` clears external tensor buffers and cached state under a critical section, separately from the graph's inactive-placeholder mode branch. Snapshot/restore code includes the feature reference and cached states. The inspected adaptation is feature/reference and recurrent-state adaptation, not an observed neural-weight update. This is a bounded positive finding, not a proof that no other personalization mechanism exists anywhere.

The provider allocates/copies temporary feature containers, and Run is entered under a critical section. The actual execution thread and worker scheduling are not established by these bodies alone. Do not extrapolate them into a realtime callback allocation/blocking verdict or a timing guarantee.

## Detector evidence becomes spectral attenuation

The traced chain separates frame/window processing, spectral processing and PostGate. The block wrapper `0x180a64c50` accumulates frames, applies pre/post coefficients and calls the transform wrapper `0x180a6daf0`. Ordinary assembly bridges false-noreturn decompiler annotations around the forward/inverse transform calls. Its middle route reaches `0x180a532e0`, which invokes the optional spectral-input callback, selects fresh or cached scalar/result fields and continues through `0x180a572b0` into the actual bin operation `0x180a58e10`.

The scalar-processing stage `0x181794670` has three internal modes. Mode 2 converts an amplitude argument with `clamp(20*log10(a+FLT_MIN), -72, 0)/72+1`; this mathematically inverts the model's amplitude encoding **if that argument is the model output**. Other modes combine two callback scalars through running means, clamped ratios and a blend term, with an additional level-floor route in one mode. The callback/result framework and cached-result route are real; the exact public-profile/No Target selection and every scalar lane's semantic binding are not all proved. The helper `0x180a5af20` is a static 2048-value normalization factor, not by itself a dynamic audio RMS measurement.

After scalar conditioning, the native curve/AHR stage yields g. The gain builder `0x180a60150` computes, for each of 1025 bins:

`G_k = (1-I) + I * (A_k * g^(B_k))`

The complex processing body multiplies both real and imaginary components by the product of G_k, an overall scalar and another frequency-dependent array. It is actual attenuation arithmetic, not merely a meter. The control-array update `0x181796410` assigns bins to three frequency regions, forms per-region factors/exponents/enable-solo masks and smooths the arrays across frequency with forward/backward recurrences (`0x180afb520`). The exponent helper `0x181796170` uses a clamped -1..1 input, with `1+5*x*x` for nonnegative x and `0.3+0.7*(x+1)^2` for negative x when enabled. Exact queue callback/public-field bindings not established by the final constructor tranche remain marked as such in the DSP/parameter reports.

The final requested typed constructor `0x180a53d00` is exported. It initializes the embedded spectral processor through `0x18179afa0`, associated buffers and callback/result storage, then calls `0x180a54050`. This confirms construction flow but does not alone establish every dynamic callback binding. No further framework-constructor chase is required to explain the recovered gain law.

## The separate PostGate cleanup stage

The PostGate state is an independently proved object P embedded at main-model offset 0x360, initialized by `0x181795160` and consumed by `0x181794b00`. Its detector reads a sample-stream descriptor separate from the descriptor it multiplies. Its detector is **not a direct load of the ONNX activity score**. The precise upstream alias relationship between original/delayed/remixed/processed audio is not fully established. In the reviewed scalar/SSE route, repeated-first-stream summation and channel-count division reduce to the first stream; calling it stereo averaging would be inaccurate.

Its local envelope uses absolute value or squared input with attack/release-style exponential smoothing; the squared mode takes the square root. With its level influence enabled, curve input is `a=e/(e+b)`. The named Level Bias setter computes `b=2^((x/100)*6.228818-9.965784)`. This gives a concrete normalization mechanism rather than an assumed linear ML/level blend.

For threshold T, tolerance v, lower edge L=(1-v)T and floor m, `0x181795da0` returns 0 dB above T, m below L (or for width below 0.001), and between them a linear-amplitude interpolation expressed in dB. This is a soft transition with relative width, **not Schmitt hysteresis**. The constructor's floor amplitude is 1e-5, approximately -100 dB. A local inverted branch mirrors dB around the floor; the public Duck selector's exact binding to this member is not proved.

After clamping and amplitude conversion, AHR processing at `0x1817aa280` precedes the final mix `1-I+I*g`. The native timing recurrence and counter saturation are documented in the DSP report; it must not be described as an indefinitely growing elapsed-time ramp. The PostGate controller additionally scales Impact down as Attack moves from 10 to 50 ms. Public Attack/Release/Hold enter in milliseconds-to-seconds conversions, but this cleanup stage's queued timings include controller offsets/couplings. They are not necessarily the public values applied verbatim to every stage. Spectral Impact also exists independently, so no universal 6 dB maximum-attenuation claim follows from the cleanup mix.

## Recommendations for our podcast/vocal gate

Keep the actual, provenance-verified Silero v6.2 detector as the starting point. Its 16 kHz branch has an STFT Conv, four encoder Conv stages, a decoder Conv and an LSTM with hidden size 128. Alternative serialized LSTM branches are not stacked layers. Sonible's recovered Conv/GRU-192/GRU-96 spectral detector is meaningfully different, but topology alone establishes no quality winner. Do not copy extracted proprietary weights or invent separate Speech/Vocals models when our implementation has one Silero model.

Our product should own the transformation from detector evidence to attenuation. In particular, a VAD output must not silently inherit the amplitude units, threshold calibration or inverse-log mapping recovered from sonible. Research recommendations for each relevant family are:

| Control family | Recommended behavior in our design |
| --- | --- |
| Threshold | A documented detector-evidence threshold with explicit units/meaning and a visible relationship to the resulting attenuation. Calibrate on podcast/vocal examples rather than copying a vendor number. |
| Tolerance | A clearly defined soft transition or independently named hysteresis policy. Do not call sonible's recovered local soft knee hysteresis. |
| Level influence | Optional, independently controlled audio-level evidence alongside VAD, with explicit normalization, silence handling and reset policy. Keep confidence and amplitude distinct. |
| Depth / Impact | Separate maximum attenuation in dB from wet/strength mixing when both are useful. Define the fully closed floor explicitly. Do not hardcode an unmeasured universal 6 dB or default depth. |
| Attack, Hold, Release | One documented state machine with time-based behavior across block sizes, deterministic transitions and a minimum-gain policy. If a secondary cleanup envelope exists, expose its purpose rather than silently coupling unrelated controls. |
| Three bands / crossovers | Optional band-specific attenuation sensitivity/depth, ordered crossovers, smooth frequency transitions and a well-defined overall gain composition. Start with the smallest useful implementation rather than cloning vendor array math. |
| Band enable / solo | Define enable as processing participation and solo as audition routing; ensure neither accidentally changes detector calibration or stored presets. |
| Gate / Duck | Share the evidence path but explicitly invert the desired attenuation policy. Test transfer endpoints and transitions through real parameters; do not rely on an unproved vendor enum binding. |
| Sidechain | Explicit internal/external detector routing with correct unavailable-bus behavior, channel mapping and state/reset policy. Never silently substitute an unintended source. |
| Mono Side / stereo link | Define detector reduction and linked gain application separately. Preserve image/phase unless the user explicitly selects a spatial operation. Validate actual channel routing rather than copying a decompiler's pointer loop. |
| Diff audition | A named residual/listen mode with precise subtraction/alignment and safe level behavior. It is monitoring, not a detector model or quality claim. |
| Presets and A/B | Serialize all public processing/routing values consistently, with versioned state and clear treatment of transient recurrent/envelope state. Use genuine host parameter/state APIs. |
| Lookahead and PDC | Choose and measure a deliberate latency policy. Report actual latency to the host and align dry/bypass/sidechain/diff paths. Do not freeze a guessed 64 or 80 ms from frame geometry or vendor marketing. |

Preserve the bounded worker, thread-safe parameter publication and fault-to-unity protections already designed for our runtime. Realtime ownership, stale result policy and consent/readiness must remain genuine; a displayed model name or mocked ABI arm is not proof of processing readiness. Main Desktop's supervised multitrack workflow and user labels offer a different capability from a single gate. That ownership should remain distinct from generic VAD and should not be presented as an automatic quality advantage.

These are research recommendations only. They do not create an implementation ticket, change the accepted product specification, or authorize backend/GUI work after this research handoff.

## Evidence and reproducibility

All large raw evidence is retained outside Git under:

`C:/Users/User/Documents/Codex/Investigations/smartgate-native-completion-2026-10-31/`

The successful persistent query session is `run-02/`. `tools-list.json` contains actual schemas; `request-N.json` and complete `response-N.json` preserve every request/returned record. `dossier-index.json` maps 63 entry addresses to complete Evidence IDs, raw responses, pseudocode, metadata and assembly. `key-evidence-index.json` is a smaller selected map. Exported checkpoints are full public Evidence bundles, not rewritten summaries. `ghidra-successful-autoanalysis.log`, effective JVM flags/properties and owned-process metadata preserve the successful run. Final manifests contain lengths and SHA256 hashes; export/cleanup status is recorded separately so this report need not pretend a live session is already closed.

Selected code evidence:

| Address / role | REA Evidence ID |
| --- | --- |
| `180747860`, real API v15 initialization | `ev_7311b387c199e8ca54a052825b34a94c87027ffbca0be99a77202c53914be308` |
| `180cb6b10`, in-memory session creation | `ev_5d03fc96883b292e3a2f92ae20970a6199fb4fe957581ec3bcb8dee62e126f94` |
| `180cbd460`, real Run wrapper | `ev_f17717037d07e28b3135e2dc7e1d89c6aad3547a8a80099cb46a0de1936e4554` |
| `180cd8610`, authenticated decoder | `ev_5d4cf163b8bc6da53cc41d3c25448f3369608c0f91ca2d68cacbcfbf4b69d3d0` |
| `1818ab670`, profile load to session | `ev_28bfef6ee75e1fd00623ed6370935d6c00c019937c12859b5d455f749b635617` |
| `1818986b0`, spectral features to inference | `ev_795636abe7a7921feacc17cd4702cbb1ada0a71716bab87c582115a872ede2de` |
| `181898190`, actual model input provider | `ev_7f52f337b096c039290153257d13175b2458a74a83dd336455d0a706cd1a98a5` |
| `1818459e0`, named PostGate controls | `ev_935e9f6886d38b1cd7c3ad371435e1514bfd5f9a3ba56266d7fb8de8c9f41a05` |

The model/profile evidence is in sibling directory `smartgate-profile-packaging-2026-10-31/`, including authentication scripts/receipts, decoded assets, exact official schema, structural JSON, reachability proofs and ordinary assembly/RTTI exports. The authenticated-model report supplies their hashes and original profile provenance. Decoded vendor assets are research evidence, not redistributable product dependencies.

## Historical failures and remaining limits

Earlier attempts are preserved rather than erased: a first SDK-default 60-second cancellation; REA's default 330-second startup timeout; a 1000-second import timeout; and a first unrestricted completed index whose >10 MiB inventory response hit the server's response budget. The old driver then incorrectly treated that ordinary error as a reason to close the completed engine. That was an orchestration mistake, not evidence that Ghidra could not analyze the target. The corrected driver increased supported response budgets, passed mocked error/malformed-JSON continuation checks and completed the successful persistent analysis. Failed receipts and the first complete inventory remain retained.

Decompiler limitations mattered: a shared indirect-call/security thunk was incorrectly marked no-return, hiding valid fallthrough; some .pdata entries cover only a prologue fragment; inferred types hide tagged values and atomics. Important math/call flow was cross-checked with ordinary sequential assembly. A missing symbol-regex match, no direct caller or neighboring vtable is never treated as proof of absence or ownership.

The recovered scope establishes architecture, authenticated graph structure, feature adaptation, in-place state feedback and substantial actual spectral/PostGate arithmetic. It does not establish training data/objective, calibrated class meaning, full public registry ranges/defaults, every callback's field binding, all mode/routing semantics, actual worker scheduling, cross-ISA numerical equivalence, measured host PDC/latency, subjective sound or quality superiority. Runtime audio and listening validation were outside this static investigation. Those limits do not invalidate the recovered code/model facts, and they should remain explicit when choosing our own design.

## Final saved state and stop

The owner-requested research is saved and all analysis workers are stopped.
Explicit close completed normally with exit0; the owned driver, REA server,
Ghidra JVM and known analysis children were verified absent, with no forced
termination. No implementation ticket or plugin/GUI change followed this work.

The final public Evidence bundle contains78 records (36,123,969 bytes); the close
snapshot contains63 primitive entries and78 records (72,028,787 bytes). The
after-close manifest inventories489 files and376,727,224 bytes. Under the
completion evidence directory:

- `final-cleanup-receipt.json`, SHA256 `3a3e65a9909d52dab016f7b361b879cd32c5f971beeac783d960bb21b89fc16e`.
- `final-research-file-manifest.json`, SHA256 `8972fcc92136bac2239d4962d513e606fef60b2a04b641ba69e7ce22736ce166`.

The supported Ghidra CPU performance settings remain available, with the original
`launch.properties` preserved as `launch.properties.before-cpu32` in the earlier
`smartgate-native-2026-10-31` evidence directory. Its SHA256 is
`5a0fa4849ee760fa36fda8183be56c008bfb73199b9dc21359125e47abd6ae83`.
The saved completion driver retains the reviewed error-continuation behavior,
32GiB process-local heap and maximum supported timers; the short analysis
deadlines and close-on-ordinary-tool-error policy were not restored.
