# Actual spectral multiplication: incremental evidence

Same target SHA256 `edeb18dcb0b7557123cc46036fd17b2d08cbd45d3bf1cb882e4b5a8a4a35f95c`.
Companion to `ghidra-postgate-core-analysis.md`; retained bodies in run-02/dossiers.
This is read-only analysis. No whole-plugin equation is inferred from wrappers.

## Whole audited pipeline and limits

The proved processing path combines framed/transformed audio, a fresh-or-cached
ML-route callback payload, two-scalar activity preparation, a scalar soft-knee
gate with directional hold/smoothing, frequency-dependent attenuation exponents
and band weighting/selection, inverse transformation, and a separate time-domain
PostGate cleanup stage. It is substantially more than a threshold on a generic
speech probability. This does not establish recognition quality, target identity,
exact latency, every preset, or a complete neural model architecture.

The native math establishes that Tolerance controls a relative amplitude knee
in both recovered gate helpers, not simply an open/close threshold difference.
Impact has independent spectral and cleanup-stage ingress paths. The frequency
arrays and blend produce a composed gain law; their still-unproved public field
bindings must remain distinct from manual descriptions. The exact public Duck
enum-to-primary-gate-mode binding is not established by the local inversion
branch. No copied implementation or new parameter defaults are proposed here.

## Actual call/data path

`180a5ecd0 → 180a64c50 → 180a6daf0 → 180a532e0 → 180a572b0 → 180a58e10`.

The block wrapper and transform wrapper are described in the companion report.
`180a532e0` invokes the optional SpectralGateInput callback at state+2298.
Fresh callback scalars or cached state+63c0/+63c4 feed the same descriptor.
A third scalar is computed separately by averaging a per-channel audio statistic
from helpers180a5af20/180a5e730. Do not conflate this statistic with either callback
scalar or label the callback scalars probabilities without their producer.

`180a572b0` handles profile invalidation/reset and optional capture. Its capture
record contains two arrays copied as0x2008 bytes each, consistent with1025
complex<float> entries, plus three tail scalars at40a0/40a4/40a8 from descriptor
48/4c/50. It then always delegates to180a58e10.

## Actual per-bin DSP, not just parameter ingress

`180a58e10` obtains a processed version of descriptor+48 through
`181794670(state+2160, scalar)`, stores it at state+2208, and passes it to
`181795e80(state+1ee0, scalar)`. That is a proved callback-scalar consumer route.
Precise scalar meaning, transformations and side effects require those bodies.

It calls `180a60150(state, result)` to build1025 per-bin values. Their arithmetic
mean is stored atomically at state+220c. It then walks channels and complex bins
and multiplies both real and imaginary components by the same real factor:

`G[k] = state[2210] * generated_gain[k] * pointed_array_at_state_48[k]`.

The same generated per-bin gain applies to each processed channel in this body.
The meanings of state2210 and the state48 array remain unresolved; do not call
them input gain, output gain, frequency weighting or neural mask without writers.
The actual gain generation body180a60150 is the next requested target. This
already establishes frequency-dependent complex-bin processing before the
inverse transform, separate from the subsequent time-domain PostGate.

## Recovered generated-gain equation

Actual builder180a60150 now proves, per k=0..1024:

`generated_gain[k] = (1-I) + I * A[k] * pow(g, B[k])`

where I=state+80, A is the array pointed to by state+08 and B the array pointed
to by state+28. This scalar g is **not an unbound decompiler argument**: caller
assembly180a58e8e calls181795e80, then180a58e93 copies XMM0 to XMM2 before
the180a60150 call at180a58ea0. It is precisely that AHR gate's returned gain.

181795e80 applies the same181795da0 relative soft transition to its input scalar,
using this gate object's threshold, tolerance and floor, optionally inverts the
dB attenuation for local mode1, clamps and converts to amplitude, then invokes
1817aa280 directional hold/smoothing and returns its gain. Thus the primary
spectral gain includes a scalar gate/AHR path as well as frequency-dependent
arrays and blending; it is not established as a direct per-bin neural mask.
Binding array A/B, state80 and the final state48 array to band controls requires
the actual parameter update/consumer181796410 and its queue callbacks, discussed below.

181794670 demonstrably receives **both** callback scalars: assembly180a58e6d
loads descriptor+4c into XMM2 and180a58e72 loads+48 into XMM1 before its call.
The earlier two-argument decompiler view in180a58e10 loses the second float.
Its body has modes0/1/2, asymmetric level smoothing, bounded ratios and a final
smoother; mode2 includes a [-72,0]dB to[0,1] mapping. Helper180a5af20 is needed
to interpret its repeated helper-derived factor, and state writers are needed
to bind mode enums and level-bias controls. No public label is guessed.

## Band-array writer and scalar preparation

181796410 consumes a dirty flag at+84, reads per-bin frequency values through
state+68, and selects low/mid/high using ordered splits state+a0/+a4. The three
band values are state+88/+90/+98, enable flags+8c/+94/+9c, and separate selection
flags+8d/+95/+9d. These field roles are established by reads, but their public
queue bindings still need180a60d80 (180a5acf0 delegates to that routine).

For each enabled band value v, A[k]=1 unless v>0.7, when A[k]=(1-v)/0.3.
The inline high-band B[k] clips v to[-1,1] and gives0.3+0.7*(v+1)^2 for v<0,
otherwise1+5*v*v; a disabled high band gives0. Low/mid call181796170 and must
not be assumed identical until that helper is read. C[k] is0/1 from the
separate selection flags. A/B/C are each postprocessed by180afb520 with
arguments0.28/0.9/0.28 respectively; its numerical meaning is still pending.

Ordinary disassembly repairs truncated helper180a5af20: after its call to
180001040 with a global float buffer182e58e40, length2048 and output pointer,
it computes `k=1/sqrt(result*result/2048)` and actually returns. The called
reduction operation's exact identity is not yet resolved. The constant at
182e82048 is exactly1/2048. Saved evidence is
`offline-factor-helper-180a5af20.asm.txt`, SHA256
`3f7d4e67b6f11ec372d2d5794741e23499027f812bea9cde8e0344fbaf57c1c0`.
This is a helper-derived normalization factor, not an assumed neural weight.

Writing callback scalars as p,q, 181794670 modes0/1 compute
`m=clamp(k*p,0,q)` and `n=clamp(p,0,q/k)`. A direction-dependent smoother tracks
z from m. The blended activity is
`(1-w)*clamp(m/(epsilon+z),0,1) + w*clamp(m/(epsilon+q),0,1)`.
Mode1 additionally multiplies a smoothed floor/unity selector according to
whether n is below an internal dB-derived amplitude threshold. Mode2 instead
maps p in[-72,0]dB to[0,1]. All modes pass a final directional smoother.
The state writers and producer must establish p/q physical roles and public
LevelBias/mode correspondence; no probability or speaker-identity claim follows.

Follow-up bodies close two details:181796170 is exactly the same enabled-band
piecewise B law for low/mid as the inline high-band code.180afb520 is a spatial
forward/backward one-pole pass across frequency bins, not a temporal smoother:
forward `y[k]=alpha*y[k-1]+(1-alpha)*x[k]`, then the corresponding backward
recurrence using the already updated following bin. The three alpha values
are0.28/0.9/0.28 for A/B/C respectively.

180a60d80 only updates each generic queue's current cell, clears its dirty flag
and invokes a stored std::function callback. It does not directly write the
band fields. Public control-to-band-field identities therefore still require
the typed callback construction. A bounded ordinary disassembly establishes
constructor route180a578f0(D+40)→180a6f550(D+640)→180a53d00(D+680), the last call
at180a6f5b3 after LEA+40. This is the typed transform-wrapper constructor;
its embedded spectral constructor is confirmed in the final tranche below.
Saved `offline-spectral-wrapper-ctor-180a6f550.asm.txt` SHA256
`4bcf25d6d8adfabc8b84123e28c9699642c45771e73d3095990a0712c8e0a2e2`.

180001040 itself is an indexed dispatch thunk through a global function table;
its name does not resolve the reduction operation. The ordinary post-call
arithmetic in180a5af20 remains valid independently of that dispatch identity.

## Final typed-constructor tranche and stop boundary

The exported180a53d00 confirms the exact nested ownership: it zeros state+40
for0x6400 bytes, then calls **18179afa0(state+40)**. On the independently
validated route, that address is D+6c0, the same object provided to the
SpectralProcessingController. It then initializes transform buffers and calls
180a54050. This closes the typed construction address, not the callback field
bindings:18179afa0 and its stored callbacks were not analyzed in this tranche.

Final unresolved items are therefore explicit: exact public control→internal
band-field callbacks; public operation enum→primary inverted-gate branch;
physical meaning/producer of the two callback scalars (owned by the separate
ML-source investigation); source identity/aliasing of PostGate's detector block;
dispatch-selected reduction identity; complete runtime initialization/defaults,
model architecture, measured latency and independent audio quality. The
verified local gain equations and observed routing do not depend on guessing
those answers. No further callbacks or framework construction are followed.

## Evidence anchors

The complete14-body derived manifest, including exported helper/writer/constructor
bodies, is `ghidra-spectral-gain-artifact-hashes.json` beside this report.
It records actual EvidenceIDs, exact retained paths and .c SHA256 hashes.

| Function | Evidence ID | .c SHA256 |
|---|---|---|
|180a532e0|ev_879c6dcc83c95dbac5e89b64e7c6c222a085245e17ca09dba0d85f53ae613c58|8c04c665ef6ab5d0eb1cd2b14630066b838be85808282616f0d0a69d32b18a83|
|180a572b0|ev_1fd40fba41205cf73916dc13c313d30ca8e67620601f1e127cc441abb3a9c602|dc3cb991757d3e1e6a472408b7f73f337d61bde06977b4578abf7ded9c948e13|
|180a58e10|ev_ace8501a3e4d352653ebc31fd0ce8be54d56ab0f119840df9f6949c908f72515|d2856bb4338e330f76931992674e6e94666bfc4e25a410b09ae2fae8c680198c|
|180a60150|ev_70cfb3b46224f45c6e1927dc88e486aa96a97d53669ff562deec81d35ebb7c30|0e8bd267c0aa4eb699023c5e2b96080a141bd08fb7ebbec2ec4f4aca2d7d9fee|
|181794670|ev_85c9b3d1f433759f3a334a17a66eacbd899109384c65bd89b06446f1f6823098|6a2bea21660f9a6060f0641c61121b402013c4d9fe7b5c3ebae37f4364b2b6c0|
|181795e80|ev_73ed7db3fdea9089f68e4c49171e8dbae8f367deb29649481b282f8d3106ba8b|7eaba43fc8c3533fb047b43b9eb7df82f4b6cf8c6adc996fa0ace679618362f3|
|181796410|ev_9334328330905049a226155c17fa4cd0033b391a227ce2a212cb60a1513db8d6|90a767fa85447e35212b06a9e25be14d3a7b537e3f9d83f95b84ca7bdecd2d89|
|180a5acf0|ev_1b9a480f06b6d38a1619b71f25f93697d2c6eacb11eade5051fe3787b280759d|227eaa1ea2aec8c3a51c8cece574667b9e2a4b4e90d1faaee7e6c59cae6630a0|
|180a5af20|ev_0764f90c5733ea08332b926eaea7de723c739fdb260367925e597708d171d5ad|779cb90557d9dea7640cf46ef22544dc136590277b420d5f309855e26871e5f0|
