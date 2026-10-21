# FabFilter performance lessons for CleanSlate

Date: 2026-10-21. Decision status: proposals for owner selection. **No implementation tickets or product changes were made.**

The useful lesson is to give rendering, interaction and audio work separate budgets, then avoid work that cannot change the visible or audible result. CleanSlate already implements several of these ideas. The best next candidates address specific remaining gaps rather than adding another graphics layer.

## Recommended decision table

Effort is a relative estimate, not a delivery commitment. Each row is a coherent potential ticket. “Measure first” means implementation depends on a demonstrated problem.

| ID / priority | Candidate | FabFilter lesson and evidence strength | CleanSlate finding and proposed scope | Expected benefit / effort | Acceptance check if selected |
| --- | --- | --- | --- | --- | --- |
| P1 — high | **Invalidate waveform geometry only when relevant data changes** | **Native evidence:** L2 and C3 gate drawing on pending non-empty bounds and visibility; C3 only swaps buffers when the drawing routine reports work. This is evidence of selective work, not proof of an identical cache design. | Existing caches are good, but all messages except PlaybackFrame advance drawing_revision; a worker poll Tick can therefore invalidate otherwise unchanged geometry. Narrow invalidation to changes that affect cached drawing. | Less CPU contention and steadier interaction while jobs run. **Small–medium.** | Count rebuilds during unchanged worker polls; compare frame/input timings on the same project. Confirm analysis completion, selection, theme, resize, edits and viewport movement still redraw correctly. |
| P2 — high; informs other rows | **Measure complete-app frame pacing and input-to-feedback latency** | **Native evidence:** Q4/L2/R2/C3 use display timing, performance counters, waits and mode-dependent timing targets. The measured objective should be smooth delivery, not a guessed FPS from a timer constant. | Reuse existing pan, preview, editor and device probes; add only missing complete-app frame/input measurements. Cover idle, playback, locked-follow, middle-pan, zoom and knob edits with/without background work, plus display scale/refresh. | Identifies the changes that actually help; catches stutter hidden by average CPU. **Medium.** | Record reproducible scenario, source commit, build, device/adapter/display settings, frame median/p95/p99 or missed-frame counts, input-feedback latency, CPU and memory. Distinguish geometry preparation from presented frames and underruns. This is scoped engineering evidence, **not reopening the waived release benchmark gate**. |
| P3 — high audit | **Keep buffer destruction and reclamation off the audio device path where needed** | **Transferable real-time engineering candidate:** responsiveness must be assessed independently of total throughput. **FabFilter callback allocation behavior was not recovered.** This candidate comes from CleanSlate source evidence, not a claim about their implementation. | Playback already uses worker I/O, a bounded queue and nonblocking handoff. AuditionSource::next still drops/replaces Vec-owning blocks and an envelope Arc. Trace ownership and measure whether the final releases occur there; use reusable blocks or worker-side reclamation only where justified. | More predictable callback timing and reduced glitch risk under load. **Medium.** | Instrument callback allocation/deallocation and deadline tails; stress playback, seeks, mode/envelope changes, worker starvation and teardown. Retain bounded memory, silence-on-starvation behavior and existing cancellation/stale-block rules. Do not promise “zero allocations” without evidence. |
| P4 — high measure first | **Keep long-session dial and gain edits responsive** | **Documented lesson:** analyzer detail has its own cost; audio processing modes explicitly trade resolution against latency. Applying separate budgets to CleanSlate is an inference. | Inference preparation and accepted envelopes are already reused. Effective envelope rebuilding and vector cloning still happen synchronously on edits. Profile long recordings and dense edits; narrow recomputation, coalesce intermediate edits or move preparation off the UI thread only if needed. | More immediate dial/drag feedback on large projects. **Medium.** | Compare warm edit median/p95, visible feedback and time to hear the latest accepted result; preserve undo, cancellation, latest-generation acceptance and shared preview/export envelope evaluation. No silent preview-quality reduction. |
| P5 — medium measure first | **Suspend optional visual updates while minimized or otherwise explicitly inactive** | **Native evidence:** L2/C3 visibility checks gate drawing. **Documentation:** R2 can disable its analyzer; internal bypass stops that analyzer. Neither proves all DSP or all GUI work stops. | Idle Tick and playback-frame subscriptions are already conditional. The remaining question is whether playback keeps optional visual work active when the window is minimized or not shown. Gate only work with no visible consumer if profiling shows a cost. | Lower background CPU/power use while listening or processing. **Small–medium.** | Compare visible versus minimized playback; resume with the current playhead/meters. Audio, analysis progress, cancellation and export must continue correctly. Preserve measurements that must accumulate. Do not treat an unfocused visible window as invisible. |
| P6 — medium, conditional on P2 | **Reuse more waveform geometry during continuous locked-follow scrolling** | **Native evidence:** retained textures and rectangle sub-updates are present in Q4/L2/R2/C3. They show reuse, not a recovered waveform scrolling algorithm. | CleanSlate has pixel-based LOD, caches and separate overlays. A moving locked-follow viewport still invalidates geometry each frame. Consider translating retained geometry, bounded tiles or another small reuse strategy only if complete-app measurements identify it as a bottleneck. | Smoother playback scrolling on long/dense sessions. **Medium–large; more complexity risk.** | Measure both geometry and presented frames; verify zoom, resize, clipping, edits, all themes/DPI scales and middle-pan lock reanchoring. Bound cache memory. Coordinate with existing ticket42 rather than duplicating its edge-catch-up behavior. |
| P7 — medium support value | **Record useful graphics/backend diagnostics** | **Documented:** FabFilter uses GPU graphics acceleration and provides a disable path for troublesome driver/host combinations. **Native:** Windows OpenGL rendering exists. This does not establish GPU audio DSP. | Capture the actual active renderer/adapter and relevant display scale/refresh in existing diagnostic paths where the current stack exposes them. A measured rendering fault can then be reproduced and investigated. | Faster diagnosis of machine-specific stutter or rendering problems. **Small–medium.** | Verify values reflect the active backend, redact private paths, handle unavailable information gracefully and test the report on the current Windows machine. Follow existing diagnostic conventions. A new renderer or fallback switch is a separate evidence-dependent decision. |
| P8 — conditional on measured contention | **Tune bounded background concurrency for foreground playback and editing** | **Documented, historical:** FabFilter staff described single-thread Pro-L2 DSP with hosts parallelizing independent instances in 2021. Other official material describes workload-dependent CPU costs. This is not proof of today's internal worker topology. | CleanSlate already isolates work, coalesces previews and rejects stale generations. Inspect critical-path delays under large imports/analysis before changing concurrency; bound independent work to preserve responsiveness and memory. | Fewer interaction stalls or audio underruns during heavy jobs. **Medium.** | Compare representative track counts/durations/sample rates with playback and edits active; report preparation latency, throughput, memory and underruns. Preserve source validation, cancellation and export correctness. Avoid adding user-facing worker-count settings without a demonstrated need. |

**Suggested first batch: P2 + P1**, with **P3 as a separate audio-path audit**. P4 is the next useful measurement on a long project. Select P5–P8 when the relevant scenario shows a problem. These are proposed priorities, not approved work.

After owner selection, selected rows can become individual scoped tickets carrying their evidence, dependencies and acceptance checks. P2 should inform performance-dependent implementation decisions; it need not block a small correctness-preserving invalidation fix whose behavior can be demonstrated directly. Related work can share an umbrella record without making one large implementation ticket.

## What was inspected

The current source review was on `codex/fabfilter-controls`, HEAD `7be3fe3`. Only research artifacts were added. Development, release code, plugins and the open review GUI were not changed.

### Installed coverage

- **51 Windows x64 files** inventoried and hashed: **15 VST3, 15 CLAP and 21 DLL files**, including mono variants.
- Every one of the 15 products has a DLL whose SHA-256 equals its VST3 file. Those duplicates do not require independent decompilation; this says nothing about host-specific runtime performance.
- Fresh **REA PE-resource inspection covered all 30 VST3/CLAP files**. PNG signatures were checked at recorded payload offsets; these are RCDATA resources, not a declared PNG resource type.
- Across the 15 products, the VST3 files contain **2,758 PNG resources**. Every product's named PNG payloads match its CLAP counterpart byte-for-byte. This supports shared presentation assets across formats, not a speed ranking between formats.
- Existing Q4/L2/R2 Ghidra evidence was imported and reused. **Pro-C3 received fresh native decompilation** to cross-check the shared performance-related GUI mechanisms.
- The other 11 products received file/resource inspection, **not full native DSP decompilation or runtime profiling**.

| Installed product | File version | PNGs per VST3 / CLAP | Named PNG payload matches |
| --- | --- | --- | --- |
| Micro | 1.3.1.30 | 56 / 56 | 56/56 |
| One | 3.5.1.48 | 54 / 54 | 54/54 |
| Pro-C 2 | 2.2.0.45 | 122 / 122 | 122/122 |
| Pro-C 3 | 3.0.1.53 | 276 / 276 | 276/276 |
| Pro-DS | 1.3.1.34 | 199 / 199 | 199/199 |
| Pro-G | 1.4.1.38 | 170 / 170 | 170/170 |
| Pro-L 2 | 2.2.5.41 | 124 / 124 | 124/124 |
| Pro-MB | 1.3.2.28 | 128 / 128 | 128/128 |
| Pro-Q 4 | 4.1.1.12 | 354 / 354 | 354/354 |
| Pro-R 2 | 2.0.5.24 | 171 / 171 | 171/171 |
| Saturn 2 | 2.1.2.14 | 224 / 224 | 224/224 |
| Simplon | 1.4.1.37 | 76 / 76 | 76/76 |
| Timeless 3 | 3.0.9.50 | 244 / 244 | 244/244 |
| Twin 3 | 3.0.6.65 | 315 / 315 | 315/315 |
| Volcano 3 | 3.0.8.59 | 245 / 245 | 245/245 |

File paths, lengths, versions and SHA-256 values are in [the installed inventory](fabfilter-installed-inventory-2026-10-21.json). Resource counts and REA evidence identifiers are in [the resource summary](fabfilter-resource-summary-2026-10-21.json).

### Native findings and limits

REA 6.7.0 used the configured Ghidra 12.1.4 provider. No new decompiler installation was needed. The source files were inspected read-only; the native session was closed and its snapshot preserved.

| Finding | Local artifact evidence | Meaning and limit |
| --- | --- | --- |
| Display-aware scheduling | Q4 `0x1801497e0`; L2 `0x180117220`; R2 `0x18013bd00`; new C3 `0x1801454d0` | DWM timing, performance counters, event waits and conditional SwapBuffers. Constants 16.666668, 33.333336 and 50 ms suggest approximately 60/30/20 Hz targets. Display synchronization and mode selection affect actual delivery; **these are not measured FPS**. Preference names/mapping remain unknown. |
| Pending/visible draw gating | L2 `0x180116b30`; new C3 `0x180144d80` | Non-empty bounds and IsWindowVisible guard drawing; bounds are cleared afterwards. The frame loop only swaps when the routine reports drawing work. “Damage/invalidation bounds” is an interpretation of reads/resets, not a recovered original field name. The routine performs other work before this guard, so zero background CPU is not established. |
| Texture reuse and partial updates | L2 `0x18005f510`; Q4 `0x18008a760`; R2 `0x1800815d0`; new C3 `0x180082890` | Texture creation/binding, full glTexImage2D upload and a rectangle loop using glTexSubImage2D. This supports retained texture objects and selective updates. It does not establish how every graph is rasterized or that audio uses GPU compute. |
| High-DPI reusable presentation assets | Fresh resource inventories of all 15 products in both VST3 and CLAP | Most PNGs have _2X names; the three original plugins also expose shared GUI control/renderer RTTI. Resource payload size is not loaded GPU memory, decode cost or per-frame cost. No recommendation to copy their artwork. |

New C3 native evidence identifiers:

- Frame timing: `ev_0da00248f17acb1cb2d71ae593619dfb289f33a53c02c1b6bad69b01bbbe5be6`.
- Visibility/pending draw: `ev_731dfd5f8ce44aab899461779e8298da8748a55c4252ce2eb219721062885dfe`.
- Texture upload: `ev_39d5b16ffacc1a599661ac5a784a68872d81a5752451f6499cf3f406137555d4`.
- C3 target SHA-256: `134890ede5a2f3f9786ec2009011ac02d6e3d2247a08c286692ad4157bfac42e`.

Full preserved REA evidence is at [performance-evidence.json](<C:/Users/User/Documents/Codex/Investigations/fabfilter-performance-2026-10-21/performance-evidence.json>), with a [C3 analysis snapshot](<C:/Users/User/Documents/Codex/Investigations/fabfilter-performance-2026-10-21/pro-c3-analysis-snapshot.json>) and a separately preserved canonical C3 evidence bundle in that folder. Earlier Q4/L2/R2 evidence and [GUI report](<C:/Users/User/Documents/Codex/Investigations/fabfilter-gui-2026-10-21/report.html>) remain preserved.

## What CleanSlate already does

These should not become duplicate “new optimization” tickets:

| Existing mechanism | Source evidence |
| --- | --- |
| Active-only worker polling and playback frame subscription | `src/app.rs:8476`, particularly8484–8500 |
| Static waveform geometry cache; separate playhead/overlays | `src/waveform.rs:1006`, `1271`; equivalent gain-editor separation at `src/gain_editor.rs:203`, `331` |
| Pixel/zoom-dependent waveform LOD, clipped ranges, disk peak caches and progressive refinement | `src/waveform.rs:102`, `201`, `258`, `278`, `299` |
| Background preview work, coalescing, cancellation and stale-result rejection | `src/app.rs:6059`, `6130`, `6616`; `src/audition.rs:364` |
| Bounded four-block playback queue, reader worker, nonblocking callback handoff and underrun accounting | `src/audition.rs:601`, `704`, `792` |
| Reused inference/effective-envelope preparation | `src/effective_envelope.rs:523`, `615`, `660` |
| Finite control transitions and bundled Lucide outline SVGs | `src/gui_controls.rs:463`; `src/parameter_dial.rs:288`; delivered GUI tickets56–58 |

The two most concrete gaps are the unconditional revision increment at `src/app.rs:7440` after the PlaybackFrame early return at7435, and heap-owning playback block/Arc replacements at `src/audition.rs:725`, `743`, `751`, `769`. Their actual cost remains to be measured. Synchronous effective rebuild/cloning is at `src/app.rs:6162`; continuous-follow viewport invalidation is at6794–6796.

Reuse `src/bin/preview_benchmark.rs`, `src/editor_benchmark.rs`, `scripts/probe-pan.ps1` and the opt-in device probes in `src/audition.rs:1158`. The earlier geometry probe's roughly15.63ms→7.30ms median comparison is **historical synthetic CPU geometry preparation**, not a new measurement, GPU frame time or presented FPS.

Existing ticket23 already retains optional end-to-end measurement notes; ticket42 covers animated edge catch-up. Ticket14's formal evaluation gate was explicitly accepted/closed by the owner. This report does not reopen it.

## Published controls worth learning from

These are documentation observations; the CleanSlate applications are design inferences.

- **Visual detail costs work:** Pro-Q4 analyzer resolution changes FFT window/detail and spectrum update behavior. Its **Speed control means spectrum release**, not render FPS. Keep visual precision and audio correctness distinct. [Official analyzer manual](https://www.fabfilter.com/help/pro-q/using/analyzer)
- **Optional displays can stop:** Pro-R2 exposes analyzer Off, and internal bypass stops its frequency analyzer. Preserve useful audio behavior while skipping unnecessary display work. [Analyzer manual](https://prod.fabfilter.com/help/pro-r/using/spectrumanalyzer), [I/O options](https://www.fabfilter.com/help/pro-r/using/outputoptions)
- **Long history can become denser:** Pro-L2 offers history modes including Infinite and Off, plus Compact layout. Its manual describes display behavior, not a proven bounded storage implementation. CleanSlate's existing LOD already captures much of this lesson. [Metering manual](https://www.fabfilter.com/help/pro-l/using/metering)
- **Graphics compatibility needs diagnostics:** FabFilter documents GPU graphics acceleration and disabling it for problematic host/driver combinations. A renderer change requires evidence from the current app and hardware. [Official support FAQ](https://www.fabfilter.com/support/faq#how-can-i-disable-graphics-acceleration-on-my-computer)
- **Quality has a purpose and cost:** Pro-L2 oversampling can reduce nonlinear aliasing while increasing CPU and potentially pre-ringing; Pro-Q4 linear-phase resolution trades against latency. These algorithms differ from CleanSlate's gain-envelope workflow. [Oversampling](https://www.fabfilter.com/help/pro-l/using/oversampling), [Processing modes](https://prod.fabfilter.com/help/pro-q/using/processingmode)
- **Parallelism follows the workload:** A dated2021 FabFilter staff response describes single-thread Pro-L2 DSP with hosts distributing separate instances. It supports measuring independent work, not assuming all cores should process one dependent chain. [Staff response](https://prod.fabfilter.com/forum/topic/6783/pro-l-x32-upsampling)

The fuller [primary-source note](fabfilter-performance-primary-sources-2026-10-21.md) contains evidence, dates, distinctions and additional official references.

## Changes not justified by this investigation

- **Switch Rust to C++:** no evidence of a language-related bottleneck. These findings concern when and how much work is done.
- **Replace the GUI renderer with FabFilter-style OpenGL:** selective invalidation/reuse can be applied within the existing architecture; active backend and bottlenecks must be established first.
- **Add oversampling, preview/export quality modes, FFT knobs or plugin formats:** no demonstrated need in the current gain-envelope workflow; preview/export consistency matters.
- **Add SIMD or more threads speculatively:** compiled instructions/imports alone do not identify the end-to-end bottleneck. Only optimize a measured hotspot with equivalent outputs.
- **Rank CLAP, DLL and VST3 as universally faster:** identical assets and some identical files do not establish host/format performance.
- **Optimize the small dials/icons before measuring them:** there is no demonstrated cost problem.

## Validation and remaining uncertainty

Inventory consistency checks cover 51 unique paths, 15 VST3/CLAP product pairs, unchanged target hashes and matching named PNG payload counts. The original three products' resource counts match the earlier report; C3 provides an independent new native cross-check. No product code changed, so no application build or Cargo test run was required.

**No controlled runtime CPU/GPU/FPS comparison or speedup was measured.** Plugin audio callback deadlines, active SIMD dispatch, DSP allocation policy, current internal thread topology and GUI-open/closed CPU differences remain unknown. REA's native live-observation facilities available here do not provide a Windows plugin performance profiler. A fair future plugin comparison would need a named Windows host, fixed audio/preset/sample rate/buffer size, matching format versions, editor-open/closed and analyzer/quality settings, repeated runs and separately reported CPU/GPU/audio deadline metrics. Different reverb/EQ/limiter workloads cannot supply a meaningful direct speed ratio for CleanSlate.

