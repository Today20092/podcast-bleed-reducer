# FabFilter Pro-L 2 edge-scroll reference

2026-10-22, scoped reference research for [ticket 42](../../.scratch/offline-supervised-editor/issues/42-playhead-catch-up-scroll.md). The owner requested REA inspection of Pro-L 2's quick return at the right edge. This record establishes the shipped feature description; it does not measure a running plugin.

## Artifact identity and evidence

- Installed target: `C:/Program Files/Common Files/VST3/FabFilter Pro-L 2.vst3`, a 3,015,168-byte x86-64 PE shared library. Product version **2.25**, file version **2.2.5.41**. Its installed Readme identifies a March 16, 2026 build.
- Binary SHA-256: `033bc8bce956634336eefc5f57a2ec368e6c5d9cd2c753e450bb071740c1bf0f`. This matches the [earlier installed inventory](fabfilter-installed-inventory-2026-10-21.json).
- REA `open_binary` bound that exact artifact. A case-insensitive literal `search_strings("Slow Down")` through **Ghidra 12.1.4** returned three defined strings. Evidence ID: `ev_e120a5f7dd187bbfa439fd3b846d833723570fede39d7263bbf6db81b9997ff6`; analysis-profile digest: `189821c22f9a3c65d594fc3eda7b5a03806941e5d6504c95c1b268b890a7c7ff`.
- At `0x1801bb1d0`, embedded UI help describes slow display movement followed by a quick scroll left at the right edge. At `0x1801bb260`, the Display Mode help explains Slow Down and distinguishes Fast, Slow and Infinite. At `0x1801be470`, the available-mode string enumerates Slow Down, Fast, Slow, Infinite and Off. These are observed shipped strings, not proof of the rendering code's execution.
- Independently, the installed help archive `C:/Program Files/FabFilter/Pro-L 2/FabFilter Pro-L 2.chm`, member `Html/using/metering.html`, describes incoming audio reaching the rightmost display edge and a quick return that creates room for new audio. CHM SHA-256: `F98324C91FE93FBA016A8885EA45A5344BFC7EBB8FCEDF1E6A168A7ED5CA0C0E`. The member was read to stdout with the existing 7-Zip installation; source artifacts were unchanged.

## Implications for CleanSlate

The reference supports the owner's intended two-part interaction: let progress approach the right edge, then make one quick leftward catch. It does not support treating Fast's continuous right-to-left movement, Infinite's history compression, or extra metering modes as requested features.

Ticket 42 deliberately adapts this to a finite, source-bound editor: keep the viewport stationary between catches, animate all synchronized timeline layers left together, and finish with the real advancing playhead near the left edge. FabFilter's help describes *slow movement* between catches; it does not establish that its viewport is strictly stationary. The owner's accepted CleanSlate specification governs that difference.

The proposed **250 ms** duration and **5%** left inset are CleanSlate's accepted initial design constants. They were not recovered or measured from FabFilter. Keep cancellation, bounds, zoom, lock, middle-pan and real playback-time behavior in the existing editor's owning layers. The animation must move the viewport without seeking audio or changing source/sample timing, labels, gain or saved-project state.

## Coverage and limits

The target was inspected statically and then closed with REA `close_binary`. No plugin host, DAW project, audio playback, installed plugin state or source recording was modified. No proprietary implementation was copied.

No runtime capture, frame timing, easing curve, catch threshold, pixel inset or animation duration was measured. No drawing-handler decompilation or string cross-reference was needed to establish the documented feature; its exact implementation remains unknown. REA string coverage applies to Ghidra-defined string Data, not every possible byte sequence. Ghidra reports a PDB reference but no established loaded-debug-symbol state. These limitations do not turn the embedded description into runtime evidence.

Owner review of CleanSlate's motion remains necessary to judge the feel. The reference is answered at the shipped UI-help level; exact Pro-L 2 motion matching remains unmeasured.
