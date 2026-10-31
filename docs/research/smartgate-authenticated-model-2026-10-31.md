# smart:gate authenticated profile models and native loading route

2026-10-31. Shipped-artifact investigation authorized by the owner. This tranche
uses ordinary MSVC disassembly, PE parsing, Node authenticated cryptography and a
schema-based FlatBuffer parser. **It is non-REA evidence**, complementing the
separate single-engine REA/Ghidra pass. No vendor module or model was executed.
No activation, installation, GUI/product change or external audio was involved.
Keys and decrypted model bytes remain outside Git and our product.

## Exact native route

Target: installed `smartgate.vst3/Contents/x86_64-win/smartgate.vst3`, SHA256
`edeb18dcb0b7557123cc46036fd17b2d08cbd45d3bf1cb882e4b5a8a4a35f95c`.
The [profile packaging report](smartgate-profile-packaging-2026-10-31.md) records
exact Speech.nn/Vocals.nn hashes and the retained REA helper evidence.

MSVC dumpbin 14.29.30159.0 disassembly starts at PE `.pdata` runtime-function
begins, not arbitrary byte-pattern hits. The exported-header mapping is the
retained official ONNX Runtime v1.15.1 C API header, SHA256
`cfa6e9fcf7750d386f94a0b1c3bc3ac062f9ccdff51490154bcd9243a82f1f91`.

| Native function / instruction | Validated behavior |
| --- | --- |
| `[0x180747860,0x180747880)` | Calls OrtGetApiBase through IAT `0x181b40cb0`, passes version15 to its first function pointer, stores returned API table at `0x1832b4230`. API request15 does not alone establish the complete runtime build version. |
| `[0x180cb6b10,0x180cb6b8c)` | Loads API-table slot `+0x40` at `cb6b3e`, calls it at `cb6b58`: CreateSessionFromArray. Arguments are environment, model pointer, byte length, options, output session. |
| `[0x180cbd460,0x180cbd5e5)` | Retrieves input/output counts at slots `+0xf0/+0xf8`, loads Run at `+0x48`, calls at `cbd52e` with stored session, input/output names and values. |
| `0x180cd8c90` resolver | LoadLibraryW through IAT `0x181b3e6f0`; GetProcAddress through `0x181b3e6d0`, using exact strings `sirNoTouchingTheCodePlease` (`0x182e74908`) and `initSkynet` (`0x182e748f8`). Stores resulting pointers at output offsets0 and8. |
| `[0x1818ab670,0x1818ab829)` | Calls the resolver through `cd8520`; calls returned length function at `18ab717` and data function at `18ab71e`; passes both to `cd8610`. Passes transformed output pointer/length into CreateSessionFromArray at `18ab760`. |
| `0x180cd8610` | Sets up cipher context; processes input length minus16, supplies the trailing16 bytes as authentication tag, checks finalization, writes produced byte count to output offset`+0x18`. |

The resolver-name strings, imports and instruction bytes are retained. This
connects the helper's pointer/length to real memory-backed session creation;
it is stronger than merely finding an ONNX Runtime import or neural-name string.
Other direct session-wrapper callers also exist at `0x180cc30b8`,
`0x180cc420b`, and `0x180cd889b`; this report does not assume their paths match.

## Cipher identity and authenticated decoding

Selector `0x1813520f0` passes callback address `0x181352180` to an initialization
helper and returns descriptor `0x1832a2750`. Disassembly from that explicitly
referenced callback entry through its RET at `0x1813521ff` writes descriptor
fields: NID895 (`0x37f`), block-size1, key-length16, IV-length12. This leaf
callback lacks a `.pdata` record; its entry is established by the selector's
explicit callback pointer, not inferred from a raw-byte match. Trailing bytes
after its RET are not used as proof of its body.

The official OpenSSL identifier table maps895 to AES-128-GCM.
[OpenSSL primary source](https://raw.githubusercontent.com/openssl/openssl/OpenSSL_1_1_1-stable/include/openssl/obj_mac.h)
This mapping is not a claim that the embedded library is exactly that OpenSSL
source version. The descriptor values, 12-byte IV configuration, 16-byte tag
configuration and successful authenticated decode independently agree.

The transform supplies key material at target VA`0x182e81d80`, file`0x2e80780`,
and IV at VA`0x182e81d30`, file`0x2e80730`. Their SHA256 values are respectively
`8c407b5efa9e470569ed4047c657b8c88eea0d55349e8f1442a0b9f563b417a3`
(16 bytes) and
`00218ed26c002eabbef6c7428bb2106f09b2241a72118dcd272daa32717cde48`
(12 bytes). Values themselves are not reproduced here.

One exact Node built-in `aes-128-gcm` decode per profile used the proven payload
window, last16 bytes as tag, these fixed key/IV bytes and no additional
authenticated data. Both `decipher.final()` calls accepted their tags; plaintext
was written only after authentication succeeded. No key search, brute force,
vendor invocation or neural inference was performed.

| Profile | Authenticated plaintext bytes | SHA256 |
| --- | ---: | --- |
| Speech | 1,283,208 | `498cf36ee6f7d73daaa673700a462d5d603cb0d0ac7fa615b82975de323dab14` |
| Vocals | 1,283,208 | `9270e7e23f9c826e01a07c3e66a45b5227e1e1f7c07a78540614ada4026e7527` |

Both contain FlatBuffer identifier **ORTM**, ORT format version5, model IR7,
producer`pytorch`, producer-version string`6f86190`. They are ORT-format models,
not raw ONNX protobuf. Producer metadata is not proof of a particular training
dataset, framework release, date, loss or source-code architecture name.

## Structurally parsed model, no inference

The ordinary bounds-checked parser follows the official
[ONNX Runtime v1.15.1 ort.fbs](https://raw.githubusercontent.com/microsoft/onnxruntime/v1.15.1/onnxruntime/core/flatbuffers/schema/ort.fbs).
Retained schema SHA256:
`fbfd25f8b94888baba7c580aab774d419e752cfa038762adb0131563cf74b87d`.
It decodes tables, dimensions, node connectivity/attributes, initializers and If
subgraphs. It does not run ONNX Runtime and is not a full FlatBuffer verifier.

All exposed inputs/outputs are FLOAT (schema data-type1):

| Input | Shape | Corresponding output |
| --- | --- | --- |
| in_mode | `[1]` | Controls a Greater/If branch; numeric mode semantics need native consumption analysis. |
| in_x_complex | `[2,1025]` | out_y `[2,1025]` |
| in_x_mel | `[2,48]` | — |
| cache_in | `[1,8,2,48]` | cache_out, same shape |
| h0_in / h1_in | each`[4,48,24]` | h0_out / h1_out, same shapes |
| h2_in | `[4,24,24]` | h2_out, same shape |
| h3_in | `[4,12,24]` | h3_out, same shape |
| h_bn_in | `[1,1,192]` | h_bn_out `[1,symbolic,192]` |
| h_emb_in | `[4,6,24]` | h_emb_out, same shape |
| h_sad_in | `[1,1,96]` | h_sad_out `[1,symbolic,96]` |
| — | — | out_activity scalar |

The symbolic dimension string on both recurrent outputs is
`GRUh_bn_out_dim_1`; it must not be silently replaced with a constant. Input
names suggest spectral/mel features, but sample rate, FFT/window/hop and native
feature normalization still need consumer evidence. In particular1025 bins do
not by themselves prove the actual host's FFT/window configuration.

Each model has88 top-level nodes including11 Conv,2 GRU,8 PRelu,1 MatMul,
1 Sigmoid and1 If, plus shape/arithmetic/assembly operators. Both GRUs explicitly
have forward direction, layout0 and linear_before_reset1; hidden sizes are192
(`GRU_349`, using h_bn_in) and96 (`GRU_383`, using h_sad_in). This establishes
convolutional/recurrent graph components; it does not establish a published
architecture family or speaker recognition.

If_432 has two five-node subgraphs: five Identity nodes in the else branch and
five Mul nodes in the then branch, returning h0–h3/h_emb state. Across these
stored graph scopes there are98 nodes and69 initializer tensors. The main graph
contains64 initializer tensors,286,581 scalar elements and1,146,480 raw bytes;
the then branch adds five scalar float initializers (20 bytes). These counts
include constants and are **not a claimed learned-parameter count**. Stored
execution-provider strings say CPUExecutionProvider; actual session placement
or user-threading behavior requires native/runtime evidence.

Speech/Vocals comparison is now stronger than same helper code: recursively
parsed node names, operators, connectivity and attributes are identical after
excluding tensor byte hashes; all value metadata and initializer dimensions,
types and lengths are identical. Of64 top-level initializer byte hashes,29
match and35 differ. The model byte hashes differ. This supports the same
serialized graph structure with different stored tensor contents for these two
specific profiles. It does not identify which tensors were trained, why they
differ, or whether either profile better preserves podcast voices.

`h_emb` is just a tensor name in this evidence. It does not prove an enrolled
speaker embedding or personalized voice identification. The model's additional
spectral output likewise does not yet prove which output drives the product's
gain, band processing or activity threshold; native caller/consumer tracing is
still required before comparing complete product algorithms.

## Evidence retained outside the product

Directory:
`C:/Users/User/Documents/Codex/Investigations/smartgate-profile-packaging-2026-10-31/`.
It contains the ordinary parser scripts, exact assembly text/bytes, authenticated
models, metadata-only graph JSON and provenance. No decrypted weights are in Git.

| Evidence file | SHA256 |
| --- | --- |
| ort-route-evidence.json | `255951d27bc45e9773fcc9d9f3a1636e42cdf87ff9114c8fa2de3404cd4c2796` |
| profile-authentication.json | `ed7e7bff893e312f2ecb4e6979b520a08b8dd124621dde45f042bda6a049ede4` |
| ort-graphs.json | `34073f38f5982a3089879e76710376074a5950aef1edaf998cd482f561b59bea` |
| ort-graph-comparison.json | `203603c26b457844dd23e0ed5406e07175d5b91bfe75d295f0933d4f9ddf46e7` |

The route manifest hashes `ort-getapi-disasm.txt`,
`ort-api-pdata-functions-disasm.txt`, `ort-session-callers-disasm.txt`,
`ort-profile-export-resolver-disasm.txt`, `ort-model-preprocess-disasm.txt` and
`ort-cipher-descriptor-disasm.txt`. Command pattern:
`dumpbin.exe /DISASM:BYTES /RANGE:0x<begin>,0x<endExclusive-1> <target>`.
The earlier broader wrapper inspection started mid-function and is retained only
as discovery material, not proof of instruction boundaries. Validated API-slot
claims above use complete `.pdata`-bounded functions.

## Follow-up: exact model mathematics and active state

The following is static graph interpretation for both authenticated profiles,
not a runtime experiment. Selected scalar constants were decoded from their
typed initializer bytes. No learned weight values are printed.

### Activity head

The active learned path reaches GRU_349 (forward, hidden192), then
Squeeze/Transpose/Reshape, Einsum_368 (`btgi,gih->btgh`), PRelu_378/Transpose,
then GRU_383 (forward, hidden96). The last head is:

`GRU_383 -> Squeeze_385 -> Transpose_386 -> MatMul_388 -> Add_390 -> Sigmoid_391 -> Squeeze_392`.

MatMul_388 uses initializer `onnx::MatMul_810`, dimensions`[96,1]`; Add_390 adds
a single float bias tensor. Let the resulting sigmoid scalar be `a`.
Mul_394, Sub_396, Div_398 and Pow_400 then compute exactly:

`out_activity = 10^((72*a - 72)/20)`.

Thus the exposed activity is an amplitude-like mapping of the sigmoid result,
with mathematical endpoints `10^-3.6` (approximately0.00025119) and1 when `a`
is0 and1. It is not simply the sigmoid probability. This proves neither a
calibrated probability nor the final product's Level Bias/threshold mapping.

### Spectral output: complete overwrite, not learned separation

out_y has exactly two producer nodes:

1. ScatterND_414 takes in_x_complex `[2,1025]`, index tensor`[[0]]` (int64),
   and a `[1,1025]` initializer containing only float1.
2. ScatterND_428 takes that result, index tensor`[[1]]` (int64), and another
   `[1,1025]` initializer containing only float0.

Each index has depth1 and replaces one complete1025-element row. The two
distinct rows cover all2050 output elements. Consequently **out_y is always
row0=ones, row1=zeros** under these graph operators; the original complex input
is fully overwritten. This is compatible with a unity complex multiplier, but
the native product's interpretation is not established. It is not evidence of
learned denoised/separated audio or a learned frequency mask in these models.
An ordinary syntactic ancestor walk still lists in_x_complex as the base input;
the complete-overwrite proof is the stronger semantic result.

### Mode and state: five exposed tensors do not affect prediction

Greater_430 evaluates `in_mode > 0.5`; If_432 uses that condition. The exact
branch outputs, mapped to their parent outputs, are:

| Parent output | False branch | True branch |
| --- | --- | --- |
| h0_out | Identity(h0_in) | h0_in × scalar initializer684=0 |
| h1_out | Identity(h1_in) | h1_in × scalar initializer686=0 |
| h2_out | Identity(h2_in) | h2_in × scalar initializer688=0 |
| h3_out | Identity(h3_in) | h3_in × scalar initializer690=0 |
| h_emb_out | Identity(h_emb_in) | h_emb_in × scalar initializer682=0 |

For finite states the true branch returns zero (possibly signed zero).
Multiplication is not a guaranteed NaN/Infinity sanitizer. The numeric host mode
values, when it sets mode, and how it initializes state are not proven here.

Graph reachability includes explicit and implicit inputs. The five h0–h3/h_emb
inputs have **no path** into out_activity, cache_out, h_bn_out or h_sad_out.
Their only use is that pass-through/zero branch. in_mode likewise does not affect
those active outputs. In these serialized graphs it does not reset the two GRU
states or the convolution cache. No personalized/learned speaker identity should
be inferred from h_emb naming; the graph gives it no role in the activity score.

| Active output | Actual external graph inputs with paths into it |
| --- | --- |
| out_activity | in_x_complex, in_x_mel, cache_in, h_bn_in, h_sad_in |
| cache_out | in_x_complex, in_x_mel, cache_in |
| h_bn_out | in_x_complex, in_x_mel, cache_in, h_bn_in |
| h_sad_out | in_x_complex, in_x_mel, cache_in, h_bn_in, h_sad_in |

This is three active history/state inputs (cache and two recurrent states),
plus five prediction-inactive exposed state inputs. It does not establish why
the interface contains the latter or prove they are unused elsewhere in native
product code.

### Feature paths and convolution attributes

in_x_complex is unsqueezed, squared and summed into a power-like feature. Gather,
ScatterND and reductions feed a clipped/logarithmic level branch, then Conv_291
and PRelu_293. in_x_mel is unsqueezed and passes through a clipped/logarithmic
branch before Conv_287/288/PRelu_290. Add_295 combines these branches, and
Concat_296 joins cache_in along axis−2. Slice_301 produces cache_out; the
convolution stack consumes the joined tensor. Thus the model uses both supplied
features and a level-derived feature; calling the entire model intrinsically
level-independent would go beyond the graph. Native feature generation and
the product's separate Level Bias remain to be traced.

All listed Conv dilations are`[1,1]`:

| Conv | Stored weight dimensions | Kernel | Stride | Groups | Pads |
| --- | --- | --- | --- | ---: | --- |
| 287 | `[8,1,1,3]` | `[1,3]` | `[1,1]` | 2 | `[0,1,0,1]` |
| 288 | `[8,8,1,1]` | `[1,1]` | `[1,1]` | 1 | zeros |
| 291 | `[8,1,1,1]` | `[1,1]` | `[1,1]` | 1 | zeros |
| 303 | `[16,1,3,3]` | `[3,3]` | `[1,1]` | 8 | `[0,1,0,1]` |
| 304 | `[16,16,1,1]` | `[1,1]` | `[1,1]` | 1 | zeros |
| 308 / 313 / 318 | each`[16,1,1,3]` | `[1,3]` | `[1,2]` | 16 | `[0,1,0,1]` |
| 309 / 314 / 319 | each`[16,16,1,1]` | `[1,1]` | `[1,1]` | 1 | zeros |

The saved tensor graph includes all attributes and exact connectivity. Stored
node ordering is not asserted to be execution order. Do not infer a temporal
hop, latency or sample rate from kernel dimensions without the native mapping
of tensor axes and feature construction.

Additional external evidence:

| File | SHA256 |
| --- | --- |
| ort-output-traces.json | `3e72b227f0ad04b5920655e250f31db69b10d7b176f63f825ef42613bfee3733` |
| ort-graph-constants.json | `36afed04c4d7674a81abb734138499f0fb2d0bf006e5ee037aa623d2382b8264` |
| ort-graph-reachability.json | `3b2442007767ea8935e3fc7339d232e3487e3546ace25eb953b6a1c4da6c7714` |

These are metadata/selected-constant analyses. The next native checks are which
outputs the product actually consumes, how it supplies/cache-resets active
states and features, and how activity reaches Level Bias/Tolerance/Impact gain.

## Optional cross-check: our bundled Silero model, structurally

Our current `assets/silero_vad.onnx` is2,327,524 bytes, SHA256
`1a153a22f4509e292a94e67d6f9b85e8deb25b4988682b7e174c65279d8788e3`,
the previously hash-verified official Silero v6.2 bytes. An ordinary bounded
protobuf reader decoded metadata with the official
[ONNX v1.17.0 schema](https://raw.githubusercontent.com/onnx/onnx/v1.17.0/onnx/onnx.proto3),
retained schema SHA256
`a287acc83cd7e9ce0db16eb2ab28348defd6832ebf8bfa3583abb6f2df59d61e`.
It did not import or execute ONNX Runtime, install packages, or alter the asset.
This schema choice describes compatible protobuf field definitions, not the
runtime/compiler version used to produce the asset.

Model IR8, default-domain opset16. Top-level inputs are FLOAT input `[?,?]`,
FLOAT state `[2,?,128]` and scalar INT64 sr. Outputs are FLOAT output `[?,1]`
and FLOAT stateN with unspecified graph dimensions. The model file does not
fix batch1 or a32ms call duration; those are the separately audited host wrapper's
choices, which supplies16kHz audio and state `[2,1,128]`.

The top-level Constant value16000 and Equal(sr,constant) drive If_0. The true
branch is therefore the16k path; the false branch is the non16k path, used for
8k by the supported-rate wrapper. This condition alone does not validate every
other sample rate.

| Stored branch feature | sr==16000 | False branch / supported8k |
| --- | --- | --- |
| STFT-labelled Conv weight shape | `[258,1,256]` | `[130,1,128]` |
| STFT-labelled Conv stride | 128 | 64 |
| Encoder Conv0 weights / stride | `[128,129,3]` /1 | `[128,65,3]` /1 |
| Encoder Conv1 weights / stride | `[64,128,3]` /2 | same |
| Encoder Conv2 weights / stride | `[64,64,3]` /2 | same |
| Encoder Conv3 weights / stride | `[128,64,3]` /1 | same |
| Final decoder Conv weights | `[1,128,1]` | same |
| Recurrent operator attribute | LSTM, hidden_size128 | same |

Each sample-rate branch has113 nodes in its outer graph and nested control-flow
graphs. In each, two LSTM nodes are serialized in alternative decoder If_1
branches. **This does not mean two sequential LSTM layers execute per call**:
the active branch selects one alternative. The condition originates from Shape
of state, Gather/Squeeze and Cast. Full node connectivity, input names and
attributes are retained; no parameter-count comparison is made by summing
duplicated control-flow tensors or interpreting file size as learned capacity.

The concrete architecture comparison is consequently: our artifact exposes a
raw-audio convolutional/STFT-plus-LSTM speech detector with128-wide recurrent
state; these sonible profiles expose supplied spectral/mel inputs, convolutions,
two sequential forward GRUs (192 then96), a scalar activity head and three active
history/state inputs. Neither difference establishes better podcast preservation,
speaker identity or audible quality. Our Desktop's supervised session/multi-mic
ownership model is an additional application layer, absent from the present Gate;
it is not a change in this embedded Silero artifact.

External metadata evidence:

| File | SHA256 |
| --- | --- |
| silero-graph.json | `be888241a2b5c255be2894e945085a42abcb46cd20c4aeabb3fd5697471122be` |
| silero-comparison-summary.json | `dc80b16b22c58e2d97034f9ad194a68fd9b90f1b2fd0dcf97b49c6108dc81be3` |

Scripts `inspect-silero-protobuf.cjs` and `summarize-silero.cjs` are retained
beside these files. Their outputs contain metadata, hashes and small control
constants, not executable inference results or copied learned weights in Git.

