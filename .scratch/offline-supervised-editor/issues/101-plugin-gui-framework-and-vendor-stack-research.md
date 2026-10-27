# Compare open-source plugin GUI frameworks and identify vendor UI stacks

Status: claimed
Assignee: Codex
Readiness: ready-for-agent
Type: research
Blocked by: none
Related: 89, 90, 94, 96, 97

## Request and scope

The owner asks on 2026-10-27 whether a better free/open-source framework could improve the VST3/CLAP and desired VST2 plugin interfaces, and requests a separate ticket. They explicitly authorize Reverse Engineer Anything inspection of their FabFilter Pro-Q 4, Pro-R 2 and Pro-C 3, plus available sonible reference artifacts, to identify the UI technology where evidence supports it.

Compare options against the existing Rust egui/nice-plug editor, accepted CleanSlate design contract and 32 GiB local memory limit. This is an evidence-backed technology decision, not authorization to replace the framework, copy vendor artwork/code, install or activate vendor products, or reopen already completed reference research without a concrete gap. Continue independent implementation; any migration needs a separately scoped, reviewable proposal and prototype.

## Acceptance

- [ ] Compare the current egui/nice-plug path with a bounded shortlist of credible free/open-source audio GUI options, including audio-focused C++ choices and maintained Rust alternatives where native plugin embedding is demonstrated.
- [ ] Verify current primary documentation and exact license terms, distinguishing open-source use from paid/proprietary distribution terms and the existing project's permissive licensing preference.
- [ ] Compare native host embedding, VST3/CLAP integration, VST2 adapter/licensing boundaries, Windows/macOS/Linux, DPI/resizing, custom waveforms/meters/animation, accessibility/input, testing and migration/FFI/maintenance costs. Separate documented capability from measured performance; no invented RAM or speed ranking.
- [ ] Reuse the exact-hash ledger and retained REA evidence first, then perform bounded static inspection for unanswered vendor framework/rendering questions. Record confirmed/inferred/unknown conclusions and evidence IDs/byte ranges; absence of framework strings is not proof of absence. No deep engine or runtime profiling overlaps a compiler job.
- [ ] Deliver a cited comparison and practical recommendation: improve the existing implementation, or propose one small representative prototype with explicit acceptance criteria before any migration. Preserve current DSP, source validation, state/format identities and installed artifacts.

## Results

Claimed from the owner's request. Framework documentation and bounded vendor technology identification will be researched separately from the ongoing Podcast implementation. The existing reference ledger and UI contract are inputs, not proof of a particular vendor GUI framework or rendering performance.
