# smart:gate Speech/Vocals packaging: retained native evidence

2026-10-31. Lightweight companion to the separate full main-VST3 Ghidra pass.
No vendor module was executed, activated or loaded. No second import was started.
No model bytes were added to our product. This report establishes a useful data
boundary and limits; it does not recover a network topology or decoding algorithm.

## Exact artifacts and method

Retained installer extraction root:
`C:/Users/User/Documents/Codex/Investigations/sonible-2026-10-24/smartgate-1.0.1/cf/sonible/smartgate/`.
The retained extraction log identifies the smart:gate 1.0.1 setup payload.
These are installer-extracted files, not a claim about currently loaded modules.

| Artifact | Bytes | SHA256 |
| --- | ---: | --- |
| Speech.nn | 1,649,152 | `4733eb7a586cfed02b0844c392db4025659d301ec95c8d5b3edf6779ecf89f16` |
| Vocals.nn | 1,649,152 | `93537dce5abf14371717b9fceabf4dabbec5eab06d80248a6844fe90e913ab50` |

REA evidence is retained in the same investigation directory:

- `speech-rea-evidence.json`, SHA256 `6d8bdfe47792aac686a96943ff2b7d4404db9f853adf5c12a2c9a3a54588f4ac`.
- `smartgate-speech-followup-evidence.json`, SHA256 `a8ef5a235d85e3e74ae21efaf57f3fb94e827518fa86efe0cf0a4c2663ab27ff`.

New ordinary Node/.NET PE parsing is explicitly **non-REA**. Reproducible script
and complete JSON are outside the repository at
`C:/Users/User/Documents/Codex/Investigations/smartgate-profile-packaging-2026-10-31/`:
`inspect-profiles.cjs` and `profile-inspection.json`. JSON SHA256 is
`259f99224f245db81336c649d94672a2bb46542df851f71068db8ebd6f809dea`;
the JSON records the script hash and Node version. This parser reads headers,
export tables, section/data bytes and strings; it does not call vendor exports.

## What the retained helper pseudocode actually proves

Speech.nn is a PE DLL with these export RVAs:

| Export | RVA | Observed retained REA behavior |
| --- | --- | --- |
| initSkynet | `0x10f5` | Returns constant `0x139498` (1,283,224). |
| sirNoTouchingTheCodePlease | `0x1500` | Returns `&DAT_180013ab0`. |
| breakItDown | `0x11ef` | Forwards to `FUN_1800037b0`; a keyed container lookup returns zero or a stored pointer-like value. Its semantic purpose is not established. |
| cantTouchThis | `0x128f` | Calls `FUN_180003690`, forwarding a global container to helper `18000a350`. Not demonstrated model inference. |
| stopHammerTime | `0x12ad` | Forwards to `FUN_1800036b0`, initializes two containers, then enters once-initialization. Not itself demonstrated graph decoding. |

Export evidence IDs in `speech-rea-evidence.json`:
`ev_4aa3ca64b008729581c7de4a90d14010eec241868061f9736e17e3c707cec133`
(length),
`ev_3da8436e4200c658d971683f49bfc0b2cfafa3168f1b17fcd9010b03166e140f`
(pointer), and
`ev_0ecf6898eaa0d493e8247d9449fd1f73987e6b0568235fe6afae41a595ac605b`
(lookup body).

The complete follow-up chain is:
`1800036b0 -> 1800062f0 -> 180006820 / 180006900 -> 1800038a0`.
`180006820` returns its argument; `180006900` calls `1800038a0`.
`1800062f0` uses Windows once-initialization. The complete `1800038a0` pseudocode
iterates a container at `DAT_180190430`, builds entries in `DAT_180190448`, then
collects values into `DAT_180190468`. Its indirect helper implementations are
not all decoded in these retained records. This is evidence of container setup,
not evidence of ONNX operators, tensor dimensions or inference.

Relevant complete follow-up REA IDs:
`ev_068c49086cfdd38a9b734694e81888093b2a540aca6a7264a151fd9ede8c1e92`
(`1800038a0`),
`ev_acc4ffb1c3b905e6ec26ae47df25f76b702685b8637d6d3bfd8e9dbcd2059af7`
(`1800062f0`).

## Exact data window and comparison

PE mapping converts returned VA `0x180013ab0` to RVA `0x13ab0`, file offset
`0x124b0` (74,928). The returned length covers `[0x124b0,0x14b948)` in the file.
It lies wholly within `.rdata` (RVA `0x13000`, raw offset `0x11a00`, raw size
1,560,064). The pointer/length pairing is an actionable candidate payload
boundary, not proof that the main plugin uses the entire region as an ONNX model.

Both profiles have identical exports and entire `.text` bytes:
SHA256 `61dc14ea0e11f3deb1b1044251a5b3533d80024fe7801777ad24d0cc690deb12`.
Vocals' corresponding window is therefore a well-grounded static comparison;
the retained REA decompilation itself targets Speech, not a fresh Vocals import.

| Window | SHA256 | Shannon entropy, bits/byte |
| --- | --- | ---: |
| Speech | `26a83cea47a3e97f26ee6e825190042b5b5e7a2938cb15306c0969d54a94342f` | 7.9998406 |
| Vocals | `1aa538e246bcbd78b595b22e4a9164f900b6015ce1436a8a930c44f7f2f80cf5` | 7.9998541 |

The first 52 bytes match; first difference is relative offset `0x34`, file
offset `0x124e4`. Exactly 202,371 of 1,283,224 bytes match at the same position.
These facts prove different payload bytes with shared structure/prefix; they
do not prove equal network architecture or different weights alone.

Both begin `a3 58 b6 f8 d5 77 6b 34 c3 32 46 f4 d3 ba de 5e`.
That prefix does not match a standard gzip, ZIP, zstd or LZ4-frame header.
No decompression was attempted and raw/custom compression remains possible.
Entropy does not distinguish compressed, encrypted or otherwise encoded data.

ONNX protobuf has no universal magic header. Failure to find an ONNX string or
common compressor header cannot exclude an ONNX graph. Conversely a random
`GRU` substring in high-entropy bytes is not an operator: the ordinary scan found
three such Speech matches but no decoded protobuf structure. Its two `Conv`
substring hits are outside the candidate window, in the CRT phrase “calling
convention”; they are unrelated to convolution. Vocals' isolated `BZh` substring
inside the window likewise does not establish bzip2 compression.

Retained REA defined-string searches found no `onnx` or
`sample|input|output|tensor|shape|fft|spectr|neural` matches in Speech. This is
limited defined-string evidence, not proof that such information is absent after
decoding.

## Highest-value main-program xrefs

The single full-VST3 researcher should trace:

1. String/export resolver references to `initSkynet` and
   `sirNoTouchingTheCodePlease`, including every callsite of the returned function
   pointers. Track the returned data pointer and size together.
2. Any intervening copy/transform before ONNX Runtime session creation. Identify
   the exact routine, constants, source/destination lengths and ownership; then
   inspect that routine before proposing an offline decoder.
3. The session constructor arguments (memory-backed model versus file-backed
   path), with actual argument dataflow, rather than equating embedded ONNX
   Runtime imports with this profile's model format.
4. If dataflow instead traverses `breakItDown` and the global containers, trace
   the keyed lookup's input names and stored values. The present container
   pseudocode alone cannot name their contents.

No actual ONNX graph, topology, parameter count, sample rate, receptive field,
normalization or trained-source identity is recovered by this tranche. The
precise limit is a pointer/length/data identity plus export/container behavior;
the main plugin's consumer/transform is still the missing link.

## Follow-up: shared-region layout

Ordinary non-REA `compare-layout.cjs` in the same external evidence directory
compares exact bytes, without decoding. Its `payload-layout.json` records all
58,634 matching runs, top-run hashes, all sixteen 16-byte alignments, 4 KiB window
counts, input window and script identity. JSON SHA256:
`99a71c5666ce1d288242f34a620ec8a93e5bb96ce4c5a581819f670eeacdcc54`.

The 202,371 matching bytes are highly localized. Three runs account for 136,737
of them; the largest alone accounts for 127,992. These regions are useful for
tracing segment boundaries after any decoder is actually identified.

| Matching-run length | Runs | Matching bytes |
| --- | ---: | ---: |
| 1 | 57,820 | 57,820 |
| 2–3 | 740 | 1,656 |
| 4–7 | 25 | 131 |
| 8–15 | 5 | 52 |
| 16–31 | 5 | 111 |
| 32–63 | 6 | 328 |
| 64–127 | 18 | 1,386 |
| 128–255 | 8 | 1,298 |
| 256–511 | 1 | 293 |
| 512–1,023 | 3 | 2,559 |
| 1,024–4,095 | 0 | 0 |
| 4,096+ | 3 | 136,737 |

Largest exact common regions, each start shared at the same offset in both files:

| Payload-relative start | File start | Length | SHA256 |
| --- | --- | ---: | --- |
| `0xa8c` | `0x12f3c` | 127,992 | `3dc9f0c032188ad21375e8825c0edc59e91523b104613710e4df793b0a0b49fa` |
| `0x13756c` | `0x149a1c` | 4,488 | `e5b05fe088a37614dd5725fd8c55bf215bd5f6999942fcf57ac15039a3d20b5c` |
| `0xfc1f3` | `0x10e6a3` | 4,257 | `59e2f01afbac303eb764312de74f12e096d353a459c1dc6498f4f396f083ea18` |
| `0x138dac` | `0x14b25c` | 920 | `2ef12cab289dda7ca9278cfb7f4ef2f98722e07e15b1b3abe122eef04f097044` |
| `0x138a0c` | `0x14aebc` | 896 | `9f93029372b27d684c1836bcc0fae7282b221c3ab279a12187aa7bfbb3cd4f96` |
| `0x139168` | `0x14b618` | 743 | `fcf389592526b12d6b07f36ff2de32ef9c861b015e7ef6b7f35d0b9823455ba1` |

At payload-relative alignment zero, 8,876 of 80,201 complete 16-byte blocks match
at the same position (11.067%). Across shifts 0–15 the rate remains approximately
11.065–11.089%; there is no pronounced winning alignment in this simple test.
Searching each Speech block against all Vocals blocks at relative alignment zero
finds zero blocks that match only at a different index; 71,325 have no match.
This tests a 16-byte-aligned grid, not arbitrary byte shifts or arbitrary chunks.
It neither establishes nor excludes a block cipher, common submodel or packed
tensor layout.

The shared 52-byte prefix is not yet a recognized header. Reading its thirteen
aligned 32-bit words in either endian order yields no small version/count value
and no value within this 1,283,224-byte window. None identifies the export length.
That narrow observation cannot exclude variable-length, unaligned, transformed
or external metadata. No brute-force transformations were attempted.

Actionable anchors for the main trace: inspect whether the consumer explicitly
segments near relative `0xa8c` and `0x1fe84` (the end of the largest common run),
and whether the near-tail runs correspond to decoded metadata only after actual
dataflow/format evidence exists. Do not assign those regions operator or weight
names from their lengths alone.
