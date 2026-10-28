# Compare open-source plugin GUI frameworks and identify vendor UI stacks

Status: done
Assignee: Codex
Readiness: ready-for-agent
Type: research
Blocked by: none
Related: 89, 90, 94, 96, 97

## Request and scope

The owner asks on 2026-10-27 whether a better free/open-source framework could improve the VST3/CLAP and desired VST2 plugin interfaces, and requests a separate ticket. They explicitly authorize Reverse Engineer Anything inspection of their FabFilter Pro-Q 4, Pro-R 2 and Pro-C 3, plus available sonible reference artifacts, to identify the UI technology where evidence supports it.

Compare options against the existing Rust egui/nice-plug editor, accepted CleanSlate design contract and 32 GiB local memory limit. This is an evidence-backed technology decision, not authorization to replace the framework, copy vendor artwork/code, install or activate vendor products, or reopen already completed reference research without a concrete gap. Continue independent implementation; any migration needs a separately scoped, reviewable proposal and prototype.

## Acceptance

- [x] Compare the current egui/nice-plug path with a bounded shortlist of credible free/open-source audio GUI options, including audio-focused C++ choices and maintained Rust alternatives where native plugin embedding is demonstrated.
- [x] Verify current primary documentation and exact license terms, distinguishing open-source use from paid/proprietary distribution terms and the existing project's permissive licensing preference.
- [x] Compare native host embedding, VST3/CLAP integration, VST2 adapter/licensing boundaries, Windows/macOS/Linux, DPI/resizing, custom waveforms/meters/animation, accessibility/input, testing and migration/FFI/maintenance costs. Separate documented capability from measured performance; no invented RAM or speed ranking.
- [x] Reuse the exact-hash ledger and retained REA evidence first, then perform bounded static inspection for unanswered vendor framework/rendering questions. Record confirmed/inferred/unknown conclusions and evidence IDs/byte ranges; absence of framework strings is not proof of absence. No deep engine or runtime profiling overlaps a compiler job.
- [x] Deliver a cited comparison and practical recommendation: improve the existing implementation, or propose one small representative prototype with explicit acceptance criteria before any migration. Preserve current DSP, source validation, state/format identities and installed artifacts.

## Results

### Owner-requested extension — 2026-10-27

The owner added explicit research targets [Clay](https://github.com/nicbarker/clay) and [Raven](https://github.com/oscar-soirey/Raven), and points to sonible packages in Downloads for REA. The initial research below remains complete; this ticket is reopened only for the added scope. No framework migration or vendor installation/activation is implied.

- [x] Compare Clay and Raven using current source/docs/licences: distinguish layout engines, widget systems, renderers and native window/format adapters; verify Rust/C integration, threading, DPI/input/accessibility, platform and maturity/test boundaries, and resource claims versus measurements.
- [x] Reconcile the actual sonible files in Downloads with retained source/extraction identities. Reuse matching REA evidence and shipped/manual GUI references; inspect only concrete new gaps with bounded static operations. Keep native/active-renderer claims distinct and serialize any heavy work behind the compiler.
- [x] Update the recommendation and representative prototype proposal with these two candidates and the Downloads evidence; preserve the existing implementation and leave unsupported claims explicit.

Extension completed 2026-10-27. The [Clay/Raven addendum](../../../docs/research/plugin-gui-clay-raven-101.md) pins current source and licences. Clay supplies layout/render commands; Raven supplies its own layout/widgets/rendering but its reviewed window API does not establish embedding inside a DAW. Raven's global runtime, worker defaults and Rust callback lifetime require a lifecycle proof before any visual prototype. Neither displaces the existing egui-first / possible Iced-screen recommendation. No comparative RAM/FPS measurement or host compatibility is claimed.

All seven actual Downloads installers and retained extracted VST3s match their recorded hashes. The [vendor report](../../../docs/research/plugin-vendor-ui-stacks-101.md#downloads-reconciliation-and-visible-gui-references) records the complete identity chains and seven visually inspected manual UI illustrations. Reusing those exact artifacts avoided extraction, installation and deep analysis. The illustrations are manufacturer documentation, not native-host captures. Report whitespace/local links passed; no production code or dependencies changed.

Completed 2026-10-27 at the requested research boundary. The [cited framework comparison](../../../docs/research/plugin-gui-frameworks-101.md) covers egui/nice-plug, Iced/nice-plug-iced, DPF/DGL, iPlug2/IGraphics, VSTGUI, JUCE 9 and Slint. The [vendor stack report](../../../docs/research/plugin-vendor-ui-stacks-101.md) retains exact identities, offsets, REA evidence and confidence boundaries.

All seven inspected sonible binaries contain JUCE GUI infrastructure. FabFilter Pro-Q 4, Pro-R 2 and Pro-C 3 share custom C++ gui renderer/control evidence including OpenGLRenderer; an in-house framework is an inference, not an identified public SDK. Active renderer selection, comparative frame pacing and memory remain unknown. Fresh target-free REA Q4 resource inspection used an explicit 64 MiB/2048-entry limit; streamed marker checks ran sequentially without deep engines, plugin/DAW execution or an open analysis provider.

Recommendation: finish the current egui implementation fixes, then prototype one representative Iced editor only if a concrete toolkit limitation remains. Published nice-plug-iced 0.5.0 targets nice-plug-core ^0.4.2 and Iced ^0.14, matching the project's generations, but no compatibility build or host qualification was performed. DPF/DGL is the leading permissive C++ multi-platform comparison; JUCE requires a distinct AGPL/commercial licence decision. The report records limitations of the other alternatives and separates GUI licences from format adapters/SDKs.

No framework migration, new dependency, benchmark, vendor installation/activation, copied artwork/source, installed-link change or release occurred. The bounded prototype proposal includes embedding/lifecycle, DPI/input/accessibility, waveform/automation parity and measured CPU/memory/frame criteria. Existing ongoing Podcast work continues; broader migration remains a separate scoped decision.
