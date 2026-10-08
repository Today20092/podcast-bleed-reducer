# Decide whether to retain Iced or migrate to egui

Type: grilling
Labels: wayfinder:grilling
Status: resolved
Assignee: Codex with repository owner
Parent: ../map.md
Blocked by: 01

## Question

Does replacing the implemented Iced editor with egui/eframe justify its migration cost for waveform selection, protected envelope editing, uncertainty review and cross-platform operation?

The source chat recommends egui; the existing developed UI uses Iced 0.14. Fetch current documentation before comparing capabilities. Prefer retaining usable code unless a concrete workflow or measured constraint requires replacement. Specify which UI code survives and whether a prototype is needed before choosing.

## Comments

2026-10-08: The owner accepts keeping Iced if it supports the required editor and asks whether egui would be easier to code. Context7 fetched Iced 0.14 documentation and egui custom-widget documentation. Iced Canvas supports custom graphics, geometry caching and message-producing interaction. Its [examples include an interactive curve tool](https://github.com/iced-rs/iced/blob/0.14.0/examples/README.md). [egui also supports custom widgets and painting](https://github.com/emilk/egui/blob/main/crates/egui/src/widgets/mod.rs). Both are plausible; documentation does not prove one is faster to implement or that the planned editor meets its performance budgets.

Recommendation: retain Iced and the existing timeline/waveform UI, introduce the new labels and effective-envelope interactions there, and reopen toolkit choice only if an interaction prototype or measured constraint fails. No requirement identified so far justifies replacing the working UI. Acceptance and exact interaction design belong in the downstream architecture and implementation plan.

## Answer

2026-10-08: The owner confirmed keeping Iced in the recommended starting point. Retain the current Rust/Iced editor and custom timeline/waveform canvas. Extend its existing selection, caching and message flow for reference labels, probability/uncertainty overlays and manual gain editing. UI and audio analysis stay separate; a toolkit choice does not determine the detector.

No egui rewrite or custom GPU renderer is planned. Verify drag/zoom/keyboard behavior and long-session responsiveness in the new editor implementation. Reopen the choice only if those checks expose a concrete blocker. This is a reuse decision supported by current capability documentation, not a claim that the new interactions have been benchmarked.
