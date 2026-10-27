# Vendor GUI stacks: bounded static evidence

2026-10-27; vendor-artifact contribution to [ticket 101](../../.scratch/offline-supervised-editor/issues/101-plugin-gui-framework-and-vendor-stack-research.md). This report identifies technology present in the selected Windows binaries. Framework choice and migration cost are separate research. No product implementation, vendor runtime or performance comparison is approved by these findings.

**Result:** all seven sonible subjects contain identifiable JUCE GUI infrastructure, including Component, OpenGL-context and software-renderer type markers. Five also contain JUCE Direct2D-context markers. The three FabFilter subjects contain a shared C++ `gui` control/renderer vocabulary, including `gui::OpenGLRenderer`; retained native evidence establishes texture-upload and presentation paths. A FabFilter-specific/in-house framework is a reasonable inference, not a verified framework name or public SDK. Neither group establishes the active renderer in a particular host, or that changing CleanSlate's framework would improve its interface.

## Scope, source identity and reuse

Read first: [reference ledger](plugin-reference-ledger.md), [waveform investigation](plugin-waveform-ui-2026-10-24.md), [FabFilter performance investigation](fabfilter-performance-investigation-2026-10-21.md), [sonible extraction investigation](sonible-static-pass-2026-10-24.md) and [smart:gate follow-up](smartgate-followup-2026-10-25.md). Their earlier unknown-framework statements describe their narrower investigations; the additional marker evidence below answers only part of that gap.

Local evidence root `E` is `C:/Users/User/Documents/Codex/Investigations/`. Reused records:

- `plugin-reference-89-inventory-2026-10-26.json`: exact 29-file inventory and extraction/source provenance.
- `plugin-reference-89-pe-imports-2026-10-26.json`: ten VST3 import/section/export records, including descriptor file offsets. Imports below reuse these records; no fresh import parser or DLL loading was needed.
- `fabfilter-gui-2026-10-21/asset-and-import-inventory.json`: Q4/R2 GUI type candidates with original file offsets and resource Evidence IDs.
- `fabfilter-performance-2026-10-21/performance-evidence.json`: six selected texture/presentation procedure records reconciled to the hashes below.
- `sonible-2026-10-24/smartgate-followup-string-candidates.json`: earlier JUCE candidates, including `MouseInactivityDetector@juce` at 52,747,960 and JUCE-bridged parameter types at 52,782,192 / 52,782,800.

All ten selected files were freshly SHA256-checked against those identities. Installed FabFilter files are `C:/Program Files/Common Files/VST3/FabFilter <product>.vst3`. Sonible paths remain the exact extracted VST3 subjects recorded in the ledger/import inventory under `E/sonible-2026-10-24/`; no installer was rerun.

| Subject | Bytes | SHA256 |
| --- | ---: | --- |
| FabFilter Pro-Q 4, file 4.1.1.12 | 4,779,520 | `8f4f09d08cb2b495dadf386af90c83f47f44d36841ba66800098da809223fe15` |
| FabFilter Pro-R 2, file 2.0.5.24 | 3,914,752 | `de7ce568fb7acd12abf02cfaf73120d8bba2630f1f280cf77b87435148cf1fee` |
| FabFilter Pro-C 3, file 3.0.1.53 | 4,141,056 | `134890ede5a2f3f9786ec2009011ac02d6e3d2247a08c286692ad4157bfac42e` |
| smart:gate 1.0.1 | 54,402,264 | `edeb18dcb0b7557123cc46036fd17b2d08cbd45d3bf1cb882e4b5a8a4a35f95c` |
| smart:EQ 4 1.1.1 | 67,169,208 | `acd116e1ef8826aacd00a4381104a4c8343baa33b948b1eed87fefb279ab93ec` |
| smart:comp 3 1.0.2 | 84,871,888 | `48d2362715934552114e0db55144a658f9a53cc2068bbb1de5b16293104bbbe0` |
| smart:deess 1.0.2 | 66,057,944 | `c17c876ef56dd96ed567ca4529bc5d46319abb370305fb1883309ff4c0599cb5` |
| smart:limit 1.2.0 | 57,755,856 | `4b0c6fd46d2977f9f8599f40186120385dce9af8c02099bd2dd43a9d443fe904` |
| smart:reverb 2 1.0.3 | 78,133,696 | `2ee4aacb5d8ba53b52f68ae7ead1980e68e5ebd1e02e615e4d2bd868a90bd68c` |
| smart:chain 1.1.0 | 191,923,920 | `f9ceb8e06ac79d1a2d43d2c372a6dc0aca55abb7bc01ab9ae52816f830307ee8` |

## FabFilter: renderer identified, framework ownership inferred

Offsets in this report are decimal **original-file byte offsets**, not virtual addresses. Q4/R2 rows reuse the retained type scan; C3 received a focused streamed marker search.

| Product | Confirmed bytes / static path | Inferred | Still unknown |
| --- | --- | --- | --- |
| Pro-Q 4 | `.?AVOpenGLRenderer@gui@@` at 2,799,128; `.?AVAcceleratedRenderer@gui@@` at 2,791,056; `.?AVControl@gui@@` at 2,789,936. OPENGL32/DWM/GDI import descriptors at 2,747,580 / 2,747,600 / 2,747,700. | Shared custom C++ GUI layer with an OpenGL presentation implementation. | Original framework name/ownership, third-party constituents and active backend in the user's host. |
| Pro-R 2 | Same renderer/control type vocabulary: OpenGLRenderer 2,518,232; AcceleratedRenderer 2,510,432; Control 2,509,424. OPENGL32/DWM/GDI descriptors 2,466,652 / 2,466,672 / 2,466,772. | Same framework family as Q4, based on repeated type names and retained implementation shape. | Same boundaries as Q4; not proof every view uses the same paint path. |
| Pro-C 3 | OpenGLRenderer marker 2,639,544. OPENGL32/DWM/GDI descriptors 2,587,276 / 2,587,296 / 2,587,396. Retained native texture/presentation procedures below. | Same renderer family, with custom controls likely. | Complete widget/type architecture and active host renderer. |

Q4/R2 retained type-scan records are respectively `ev_ffb4025dc39cc9993f4bf7bad2a79fb628b1279735866983118f8b519930d3d2` and `ev_691dbbda2c6ac6ffd23e1a2f986a0821796acd0ed8249a143fb065b4f0d8f53a` in the asset/import inventory. The native records are stronger evidence of an implemented graphics path than imports or class names alone:

| Subject / retained virtual address | Evidence ID | Scope |
| --- | --- | --- |
| Q4 texture `0x18008a760` | `ev_b70e52b8b3c9cf723bec1ba8dca6d3dccede2c97a4bc711535fea60d8bb12eaa` | Texture creation/full upload and rectangle sub-updates. |
| R2 texture `0x1800815d0` | `ev_083db736b65780cf4c4f0c49d231e31867d48c38e06ab666f01f6ad6863f0f1b` | Same bounded texture-update interpretation. |
| C3 texture `0x180082890` | `ev_39d5b16ffacc1a599661ac5a784a68872d81a5752451f6499cf3f406137555d4` | Same bounded texture-update interpretation. |
| Q4 timing `0x1801497e0` | `ev_add5d7a198dd9e3b3ee4deed60495b4c1145b45896f076df590128cff2cee096` | Display timing and waits. |
| R2 timing `0x18013bd00` | `ev_831edfdb2e1a572d2bf761e306de3f15fe908c48831cf7e765550802c4d6271c` | Display timing and waits. |
| C3 timing `0x1801454d0` | `ev_0da00248f17acb1cb2d71ae593619dfb289f33a53c02c1b6bad69b01bbbe5be6` | Timing/waits and conditional buffer swap. |

These are previously retained procedure observations, not fresh decompilation or an observed call trace. They do not establish rendering completeness, FPS, GPU utilization, audio DSP acceleration or speed relative to egui.

Fresh target-free REA answered the narrower resource question: does Q4's Windows manifest declare a named GUI framework dependency? `inspect_pe_resources` used `max_file_bytes=67108864`, `max_entries=2048`; Evidence `ev_13a4fb38d3bd51e2b3c0a80b0ab235c72ad56650284662cb441184b0235f8dfa` reports complete coverage, 726 directory entries / 361 leaves. Resource types are 5, 10, 16 and 24. Type 24/name 2 occupies `[4,752,864,4,753,245)`, SHA256 `4bb79dcea0a901f7d9eac5aa05728ae92acb42e0cb22e5dd14134f4421a3d8df`. A separate exact 381-byte UTF-8 read shows only an `asInvoker`, `uiAccess=false` execution-level declaration, with no framework dependency declaration. This is a negative result for that manifest only; resources are not a framework bill of materials. No asset was decoded or copied.

[FabFilter's official FAQ](https://www.fabfilter.com/support/faq#how-can-i-disable-graphics-acceleration-on-my-computer), checked 2026-10-27, documents GPU-accelerated interface graphics and a global disable option. It corroborates a graphics path and compatibility fallback, not the internal framework's name, current runtime selection or comparative performance. No registry/preference setting was changed.

## sonible: JUCE GUI infrastructure present in every selected product

Each row below contains the exact ASCII marker `.?AVComponent@juce@@`, plus `OpenGLContext@juce@@` and `LowLevelGraphicsSoftwareRenderer@juce@@`. The latter two are **substrings of decorated type names**; their offsets identify the marker start, not necessarily the entire enclosing type. This establishes compiled GUI/rendering infrastructure beyond a VST3 wrapper alone. It still does not identify every visible control's implementation or prove execution of a renderer.

| Product | Component / OpenGL-context / software-renderer marker offsets | Additional Direct2D-context marker | Version evidence |
| --- | --- | --- | --- |
| smart:gate | 52,572,984 / 52,641,001 / 52,719,540 | Not found by this exact ASCII-marker search | Literal `JUCE v6.1.6` at 41,077,168. |
| smart:deess | 64,298,536 / 64,360,681 / 64,428,260 | Not found by this exact search | Literal `JUCE v6.1.6` at 51,572,912. |
| smart:EQ 4 | 63,800,064 / 64,017,121 / 63,968,324 | 63,895,124 | Unknown: `JUCE v8.` fragments occur within instruction bytes; no complete contiguous version string established. |
| smart:comp 3 | 80,746,240 / 82,152,161 / 81,645,716 | 81,572,516 | Same version-fragment limitation as EQ4. |
| smart:limit | 55,794,408 / 55,832,169 / 55,882,212 | 55,873,316 | Literal `JUCE v8.0.11` at 43,232,672. |
| smart:reverb 2 | 75,601,976 / 75,695,353 / 75,860,724 | 75,853,948 | Literal `JUCE v8.0.4` at 58,325,936. |
| smart:chain | 188,022,528 / 188,325,137 / 188,276,596 | 188,203,268 | Same version-fragment limitation as EQ4. |

For each product, **confirmed** means those source bytes exist in the hash-bound artifact. **Inferred** means a JUCE-based GUI with additional sonible-specific controls/rendering is strongly supported. **Unknown** includes the actual selected backend, framework patches, exact configured JUCE module/version set, third-party/custom drawing layers, cross-platform implementation and licensing arrangement. A version string identifies embedded metadata, not an independently authenticated complete dependency manifest. No absence conclusion follows from the two missing exact Direct2D markers.

Retained imports corroborate available graphics paths: all seven import OPENGL32/GDI32; chain/comp/EQ/limit/reverb also import D3D11 and DWM. Example descriptor pairs (OPENGL32 / D3D11) are gate 52,178,564 / not recorded; deess 63,931,128 / not recorded; EQ 66,698,256 / 66,698,216; comp 84,361,388 / 84,361,348; limit 55,179,968 / 55,179,868; reverb 75,185,316 / 75,185,216; chain 191,108,416 / 191,108,376. In the latter five, `D3D11CreateDevice` is imported. The combination does not prove that Direct2D is selected, or that D3D11 is the main plugin renderer.

The [existing ledger](plugin-reference-ledger.md#performance-evidence-and-experiments) records each supplied manual's OpenGL-disable setting. A first-party [sonible C++ UI/Frontend Engineer description](https://www.sonible.com/wp-content/uploads/2022/10/Stellenanzeige_CUIFrontEndEngineer_2022_compressed.pdf), checked 2026-10-27, describes C++ UI development and requests familiarity with JUCE/Qt and OpenGL, with Metal knowledge beneficial. This corroborates the development context; a job advertisement does not establish that any selected binary uses Qt or Metal. The local JUCE GUI markers are the product-specific evidence.

## Bounded method, limitations and decision relevance

Fresh marker checks used read-only FileStreams, one file at a time, a 1,048,576-byte buffer with 4,096-byte overlap, an admission ceiling of 200 MiB and original-file offsets. The first pass searched five literal ASCII markers (`.?AVOpenGLRenderer@gui@@`, `@juce@@`, `JUCE v`, `VSTGUI`, `QWidget`), retaining only counts and at most three examples per marker. The second pass addressed the specific wrapper-versus-GUI ambiguity using the four exact GUI marker strings in the table, including `Direct2DGraphicsContext@juce@@`. SHA256 checks streamed file contents. Negative results cover only these exact case-sensitive ASCII markers, not UTF-16, stripped/obfuscated symbols, dynamically loaded components or all possible frameworks. The failed initial second-pass shell invocation produced no findings; the corrected pass supplied the table.

No whole sonible DLL was materialized into a managed byte array, and no file contents were loaded as executable code. Streamed passes finished in approximately 4.85 s and 2.74 s including their shell work. These are tool wall-clock observations, not profiling results or a process-memory guarantee. Buffer size does not bound total PowerShell/REA RAM; peak memory was not measured. The portable REA parser was restricted to the 4.78 MB Q4 file. `binary_session` reported `open=false`, `cleanup_pending=false` before and after; no native provider target was opened and none required closing.

Podcast 96 retained the sole heavy-job lane. There was no IDA/Ghidra/Kuna opening, indexing, export, runtime profiling, vendor plugin/DAW launch, install, activation, private-setting access, build, source/ref change or installed-link operation. Only this Markdown report was authored; no vendor artwork, fonts, code, models or assets were added.

The evidence supports taking sonible's JUCE path seriously in the separate framework comparison, while treating FabFilter's renderer as a design/performance reference rather than an available framework to adopt. It does **not** justify a migration, a RAM/speed ranking or copying vendor UI. Remaining useful work is CleanSlate-specific: measure a representative waveform/control/resize interaction using the current editor, then compare one alternate prototype only if a concrete limitation remains. Exact vendor active-backend selection and frame/input timing stay unknown until separately scoped, resource-controlled runtime evidence is available; broad decompilation is unnecessary for this ticket's technology decision.
