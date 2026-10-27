# CleanSlate visual and motion study

2026-10-27 owner direction: FabFilter/sonible-level visual quality, animation and performance. [Ticket 102](../../../.scratch/offline-supervised-editor/issues/102-plugin-visual-motion-study.md) supplies an original, concrete refinement of the [existing contract](../plugin-ui-contract.md). It does not replace the native GUI stack or prove comparative performance.

Open [the self-contained interactive study](visual-study.html) in a browser. Switch products, move Threshold/Reduction, Release/Fade and Preview position, then select Animate preview. Pause, reduced motion and the operating-system reduced-motion preference are supported. It starts paused and needs no server, package install, external font, network or audio device. The controls below the illustration operate the study; drawn product buttons/knobs illustrate future native controls, not implemented DAW actions. Phase, confidence, reduction, waveforms, names, readiness and example counts are generated design fixtures.

## Visual direction

Large stable plots carry the task. Subtle grid lines provide a real signal/time scale; teal source, purple gain and selected blue actions retain CleanSlate's roles. Numeric values, units and hierarchy stay readable; controls use restrained geometry rather than oversized shadows or decorative blur. Gate separates confidence, threshold, input/output and applied-reduction state. Podcast retains source-time examples, explicit Analyze and all three audition choices. The illustrated Alex-only range holds Mic 1's gain at unity and the other tracks at explicit absence throughout the exact selected span, including boundary vertices. No vendor logo, texture, artwork, font or proprietary code was reused.

| Product | Compact frozen frame | Full frozen frame |
| --- | --- | --- |
| Smart Gate | [640×440 PNG](study-renders/gate-compact.png) · [SVG](study-renders/gate-compact.svg) | [1000×620 PNG](study-renders/gate-full.png) · [SVG](study-renders/gate-full.svg) |
| Podcast | [720×560 PNG](study-renders/podcast-compact.png) · [SVG](study-renders/podcast-compact.svg) | [1000×680 PNG](study-renders/podcast-full.png) · [SVG](study-renders/podcast-full.svg) |

All four final PNGs were visually inspected. They render the same `renderPanel(mode, 5, {width, amount:50, timing:180})` output as the HTML, using installed ImageMagick and the installed Segoe UI regular face. Text is regular sans-serif, with no 600-weight fallback involved. These observations do not establish the exact native egui font or host DPI behavior. Full study sizes are design canvases; actual editor negotiation and 100/150/200% DPI remain implementation checks. SVG/PNG/source hashes and dimensions are in [the manifest](study-renders/manifest.json).

## Motion and resource discipline

The demo uses one scheduled animation callback, a selectable 30/60 Hz drawing cap, bounded visible signal geometry and at most three illustrated lanes. Refresh choices are scheduling targets, not measured presented frame rates. It starts paused; Pause, hidden-document state and reduced motion cancel pending animation. Visibility restoration resumes existing intent without a time jump. Readouts and controls still update immediately with reduced motion. No background interval, network, audio processing, dependency or framework is added.

The study rebuilds a small SVG for clarity. Do not port this allocation pattern into the native plugin or infer native memory/FPS from it. The real product must reuse bounded snapshots/peak caches, cull invisible geometry, preserve exact source-time edges and keep GUI work off the audio callback. Control tweening/Catch up interruption, live DSP meters, state transitions and native lifecycle remain 94/103 acceptance.

## Checks and limits

`node docs/design/plugin-ui/visual-study-check.mjs` validates deterministic compact/full states, bounds and fewer than 1,600 path commands per frame, finite geometry, exact human-label gain rules, generated motion/control changes, no remote resources/audio/timers, and a small DOM/scheduler double. The double verifies product switching, one scheduled frame, pause/hidden cancellation, visibility resumption, reduced motion/OS preference changes and immediate value feedback. It is not a browser layout, assistive-technology or presentation test. One initial geometry-budget failure was fixed by bounding source/gain samples; the final check passes.

The Chrome connector could not find an installed Chrome executable. CUA reported no in-app browser and an empty browser-provider inventory. No browser was installed/launched and no native DAW was opened. Consequently actual HTML layout, native pointer/keyboard forwarding, animation feel, browser presentation/CPU, GPU allocation and host performance are unverified. Four code-native SVG/PNG renderings plus logic checks establish this study's design boundary only.

All final checks/rendering ran sequentially under the explicit lightweight guard (2 GiB start, 1 GiB reserve, 0.5 GiB owned cap); heavy 14/8/6 GiB compiler defaults were unchanged. The final self-check receipt records 31,293,440 bytes peak owned job commit (29.84 MiB); the four final renderer jobs record 17,092,608–31,379,456 bytes (16.30–29.93 MiB), exit zero and cleanup verified. These are local artifact-generation observations, not plugin/browser benchmarks. Narrow LF attributes preserve manifest hashes across Windows checkouts. Final receipt files under `%LOCALAPPDATA%/CleanSlate/BuildReceipts/`:

- `20261027-010256-633ef37e0ec9425ca7930ab7e91e86ac.json` — final source/logic/frozen-frame check.
- `20261027-010015-25d96965c8b14bd0a848cbfc1e9124f6.json` — Gate compact.
- `20261027-010016-5c498f379b8844f1a5cefa33a8387eaa.json` — Gate full.
- `20261027-010017-2f59cd516e314204b0baed506c269d4e.json` — Podcast compact.
- `20261027-010018-189d3f0e17684def8b655ce5d0efc7db.json` — Podcast full.

The Impeccable mechanical HTML check returned `[]`; inline craft review also corrected small labels, plot-label overlap, compact-height fit and source-time label/gain semantics. No production source, plugin state/parameter identity, installed link, dependency or release changed. Gate implementation stays in 94 after 93; Podcast refinement is 103 after final 96 verification/integration. Native animation, measured low idle cost and full performance acceptance are still required before claiming this quality in the plugin.
