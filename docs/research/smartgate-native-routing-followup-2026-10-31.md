# Native routing follow-up: secondary model interface and PostGate reader

2026-10-31. Ordinary, non-REA static PE/RTTI parsing and MSVC disassembly of
smartgate.vst3 SHA256
`edeb18dcb0b7557123cc46036fd17b2d08cbd45d3bf1cb882e4b5a8a4a35f95c`.
No provider queries, second Ghidra import, vendor execution or product changes.
The existing single-engine researcher owns subsequent native dossiers.

## Secondary interface resolved by inheritance metadata

`find-derived-model-rtti.cjs` scans file-backed non-executable sections for x64
Complete Object Locators with signature1 and a self-RVA resolving exactly to
the candidate. It validates TypeDescriptor names and the Class Hierarchy
Descriptor/base-class-array layout, then checks whether any base descriptor
contains the exact OnnxInferenceModel or OnnxInferenceInterface TypeDescriptor.
It follows actual COL pointers to code-pointer vtables; no adjacency assumption.

The scan accepts5,055 COLs overall. Three contain one of the target bases:

| Complete type | COL | Object offset | Vtable |
| --- | --- | ---: | --- |
| OnnxInferenceModel, TD`0x18328b878` | `0x182f14d00` | 0 | `0x182e7e9f8` |
| OnnxInferenceModel, same TD | `0x182f14cb0` | 8 | **`0x182e7eb30`** |
| OnnxInferenceInterface, TD`0x18328b6e0` | `0x182f14ec0` | 0 | `0x182e7ebb0` |

No further complete derived type containing these base descriptors was found
in this bounded file-backed RTTI scan. This does not exclude composition,
RTTI-free code or an external runtime-derived type.

The model's secondary object-offset8 table supplies:

| Virtual byte offset | Actual function |
| --- | --- |
| +0x10 | **`0x181898190`** |
| +0x20 | **`0x181898090`** |
| +0x28 | `0x180cb8340` (shared base implementation) |

These are the correct callbacks to investigate when the native model uses its
`this+8` interface. Reading the base-template table would instead yield no-op
functions. The primary model's neighboring lambda table is a different RTTI
type and is not used to infer these slots. Exact mangled names and all base
member displacements/slots are retained in `model-derived-rtti.json`, SHA256
`7242c7858c196b868ba96cbf7efc417c065298e7b462f51c7a4898c46015827e`.
The interface tuple's literal mangling begins `M` (float value), not `AEAM`
(float reference); prefer the retained literal to an informal type paraphrase.

The main native researcher received both callback addresses for actual
preprocessing/output-binding dossiers. This static RTTI result identifies the
dispatch targets, not the actual audio consumed or returned by those functions.

## PostGate parameter-reader candidate

Prior controller/constructor evidence establishes parameter block
`P=D+0x360`, stored through `D+0x15de8`. The independent typed-constructor actor
connected chain construction to `0x181795160(P)`. Ordinary disassembly of that
constructor agrees on the layout: initial flags atP0/P1, scalar fieldsP4/P8/P10,
configuration storage atP20, flagP200, coefficient/state fieldsP258/P25c/P268.
Offsets here are hexadecimal, with P10 meaningP+0x10.

An initial broad displacement scan for D+0x360..0x37c did not establish any
typed DSP reader. Most clustered matches were unrelated initialization or stack
storage. An SSE-memory-read filter likewise missed the eventual reader because
it first copies atomic float bit patterns with integer MOV instructions and
because the consumer receives an adjusted P pointer. These negative searches
are not global absence proofs and do not cover every alias or instruction form.

A targeted follow-up found known inventory entry **`0x181794b00`**. Its ordinary
instruction stream saves incoming RCX into RDI at`0x181794b28`. The same base then:

- exchanges the flag at`+0x200` and reads queued configuration via`+0x20`;
- reads P+0x10 at`0x181794d14`, P+0x14 at`d23`, P+4 at`d2b`, P+8 at`d32`,
  and P+0x0c at`d36`, before moving their stored bits into scalar float registers;
- runs a sample loop at`0x181794de2` onward using coefficients`+0x258/+0x25c`,
  flagsP0/P1, state/buffer storage and an envelope-like object at`+0x268`.

This correlated layout and clustered same-base reads make it a strong
PostGate consumer candidate. A typed caller/constructor connection and full
native dossier, owned by the other agents, are still required before treating
offset meaning or an envelope equation as established.

Two direct call instructions are independently validated:

| Caller | Callsite | Pointer route |
| --- | --- | --- |
| `0x180a5ecd0` | `0x180a5f39c` | Calls181794b00 with a pointer originally supplied as a stack argument and retained in a local; typed chain owner is being traced separately. |
| `0x180a5fb00` | `0x180a5fb2c` | Wrapper moves its incoming RDX into RCX and calls181794b00 with a copied process-context structure. |

Important coverage trap: the first PE RUNTIME_FUNCTION record for181794b00
covers only its prologue, ending at181794b16. Disassembling only that record
incorrectly suggests an empty method. The retained source inventory names the
function, and sequential disassembly from that entry follows actual branches
through the clustered reads and sample loop. The next inventory entry is not
treated as proof of the prior function's complete extent; trailing neighboring
code is excluded from semantic claims. The REA native dossier should resolve
its complete, possibly non-contiguous body ranges.

External evidence directory:
`C:/Users/User/Documents/Codex/Investigations/smartgate-profile-packaging-2026-10-31/`.

| File | SHA256 |
| --- | --- |
| postgate-neighbor-0x181794b00-sequential.txt | `d528e002f78c6041b0a52e60b313a542fff5dabd5d55e586dbcd2475d4d43178` |
| postgate-chain-process-caller.txt | `baec2851a308644045b0417e5c470db105397df3cd75196b302b30fb24347b05` |
| postgate-typed-process-wrapper.txt | `880e5057321de33a87d80d44516277a0c7f8ffa457d2e51c6dea4c1376292fa0` |
| postgate-cluster-candidates.json | `9dd03efaf122738c0c5a7d3fda613c6992ade0e8d8230bffd5d8475713e906dd` |
| postgate-fp-candidates.json | `5301379236bd6a7a91ace2604860a090fa4c61274b995831b54059597900aa29` |

Scripts and full candidate evidence remain external. No Tolerance/Impact audio
equation is asserted by this routing note; the concrete consumer address and
dispatch targets were handed to the native/DSP owners for that work.
