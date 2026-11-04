# Actual control bodies: established ingress, consumer analysis in progress

Subject SHA256 `edeb18dcb0b7557123cc46036fd17b2d08cbd45d3bf1cb882e4b5a8a4a35f95c`.
Read exported Ghidra dossiers offline; no live provider queries or execution.
The earlier dumpbin findings independently confirm the important arithmetic.
This is analysis of observed proprietary artifact behavior, not a proposal for
our plugin and not a claim to have recovered the complete audio algorithm.

## Evidence map

All dossier files are under `run-02/dossiers/ADDRESS.{c,metadata.json,assembly.json}`.

| Function | Evidence ID | Established role |
| --- | --- | --- |
| `0x1818459e0` | `ev_935e9f6886d38b1cd7c3ad371435e1514bfd5f9a3ba56266d7fb8de8c9f41a05` | PostGateController named-parameter update |
| `0x1818460a0` | `ev_9d1dd1a1e80263a11ca847a44f99e0a6d10d480871453e82747a93cc38c30472` | PostGateController initialization |
| `0x18178eee0` | `ev_e7c3c53e05d632b69ef6f61e52a4bab605482a262e069fd1ea914855764473ee` | MainProcessingController and children initialization |
| `0x1809d1d60` | `ev_42f3c7e6c657df3a08ca2e26df32199ef9669754bf38abd2d12d5b94961e24b0` | AudioProcessorWrapper construction; owns processing/controller graph |
| `0x181779db0` | `ev_4bccb2b2408aefd9c5eb572c418db7cec691e683f0b0120a3b901bfdc1bd8988` | Main model initialization and embedded-state pointer wiring |
| `0x180a578f0` | `ev_c881aec9cfcfae177b32786a5e5f45bc26ab3b30c03f29292c015709568ee0c4` | Processing-chain member construction |
| `0x1817d75a0` | `ev_caeaab764e9eb506d2fc5c6dd44d108cbc23edc3c7e0c8e9f9b40a15b3a54458` | SpectralProcessingController/SpectralParameterController construction |

Metadata provides actual caller/body ranges, including non-contiguous ranges;
nearest-entry brackets from the original inventory are no longer a substitute.
Decompiler types such as `float*` can obscure tagged-union enum/bool payloads.
Assembly confirms the payload/type comparisons and atomics. The actor also
identified erroneous no-return/security-cookie treatment in some other bodies;
do not infer endless audio loops or omitted execution from that artifact.

## Precisely recovered conversions

Use S for PostGateController; P is the object at `*(S+0x130)`.

- Attack x: `S+0x120 = S+0xf8 = x/1000`.
- Release x: `S+0x104 = x/1000`.
- Hold x: `S+0x108 = x/1000`.
- Threshold x: `S+0x118 = x/100`.
- Tolerance x: atomic `P+0x14 = x/100`.
- Level Bias x: atomic `P+0x08 = powf(2,(x/100)*6.228818f-9.965784f)`.
  This yields approximately0.001/0.008660/0.075 for x=0/50/100. Whether it is
  applied to amplitude, power, a target score or another quantity needs readers.
- Impact x: `S+0x124 = x/100`.

Threshold/Tolerance/Level Bias changes also publish
`P+0x10 = clamp(S+0x118 + S+0x11c/100,0,1)`.
This proves a threshold-related derived value; it does not prove a Schmitt
hysteresis equation, a partial-closure knee or the consumer's comparison units.

Attack or Impact changes also publish
`P+0x04 = (1-(clamp(S+0x120,0.01,0.05)-0.01)/0.04)*(S+0x124)`.
Thus this particular Impact-derived value depends on the Attack mirror, not
solely the visible Impact percentage. It is not yet the complete gain law.

AHR changes enqueue six floats into a16-slot,24-byte-record publication path:
`[max(S+0x110,0), S+0xfc, S+0x100, max(S+0x104+S+0x114,0), S+0x108, S+0x10c]`.
The constructor initializes `S+0x110=0`, `S+0x114=0.5`, the visible timing
mirrors to0, and marks P+1. This is **internal construction state**, not a
verified plugin factory preset after parameter restoration. In particular the
published first value is not simply the user's Attack, and the fourth includes
an additional0.5 initial offset. Actual timing recurrence/units require the
queue consumer. Do not describe this one post-stage as the whole audible AHR.

Operation-mode payload0 and solo-mode payload0 become boolean flags. Low/mid/
high enable payloads are stored separately. A published P+0 flag is the
conjunction of a controller enable flag, those two zero-payload flags and all
three band-enable flags. Its exact audio-path eligibility/bypass meaning and
the labels corresponding to enum0 remain to be established by consumers.

## Typed ownership and the remaining gate

RTTI independently identifies PostGateController (see
`offline-dispatch-class-rtti.json`). Constructor wiring proves P is embedded
at model D+0x360; hence these fields correspond to D+0x364/368/370/374 along
the proved paths. The model also routes D+0x6c0 to the separate
SpectralProcessingController and D+0x6da0 to RealtimeInferenceController.
The spectral constructor's actual RTTI names `ComplexSpectralGate` and a
callback returning `optional<SpectralGateInput<complex<float>,2,1025>>`.
That names a typed spectral interface; it does not alone establish FFT setup,
band curves, gain multiplication or the inference model architecture.

Four observed std::function RTTI classes include the complete
BusAwareRemixingChain with ComplexSpectralGate and MainBusOnly<PostGate>.
COL/vtable linkage and actual invocation jumps were saved in
`offline-postgate-chain-rtti.json` and
`offline-typed-postgate-chain-functors.asm.txt`. Their mapped bodies
`0x180a57610`, `0x180a546f0` are copy/remixing preparation hooks;
`0x180a56440` is generic scalar block multiplication. These are **not** yet
PostGate control-field readers. A broad type name containing PostGate does not
turn every associated closure into the gain algorithm.

The focused next body is `0x180a6fb60`, first member initializer called by
chain constructor `0x180a578f0`. Its typed construction/callers may expose the
actual PostGate stage/process interface. The independent disassembly actor is
checking clustered embedded-state reads; this analysis does not duplicate that
broad scan. No full gate/hysteresis/knee/hold or band DSP formula is claimed yet.

