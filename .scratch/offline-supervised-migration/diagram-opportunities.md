# Where more documentation visuals would help

Review date: 2026-10-11
Status: owner approved the recommended first three static figures on 2026-10-11

## Recommendation

Keep the README's existing workflow and real editor screenshot. Add four explanatory figures in the guides, starting with the first three below. Readers need to see what the sound, labels and gain mean. Another processing overview would repeat the seven diagrams already present.

The audience remains podcast editors, consistent with [ticket 20](../offline-supervised-editor/issues/20-editor-first-documentation-visuals.md). Continue that ticket after the interview; this review does not create a second implementation effort.

| Priority | Figure and Diagram Design treatment | Placement | What the reader learns |
| --- | --- | --- | --- |
| 1 | Gain against time, using a line chart with aligned speech intervals. Show a normal gap and a shorter gap in separate panels. | [Effective envelopes](../../docs/effective-envelopes.md), after the paragraph beginning “Automatic opening and closing”. Link from “Ownership becomes a gain curve” in the ownership guide. | What attenuation and the two fade controls change; why a short gap compresses the transitions; why preserved speech stays at full volume. |
| 2 | Solo, overlap and room tone, using three aligned timeline examples with matching speaker selections. | [Ownership guide](../../docs/ownership-explained.md), “You supply examples”, after the selection rules. Link from the labeling instructions in the editor workflow. | A label describes everyone speaking in the interval. Selecting a microphone does not mean only that microphone's assigned person belongs in the label. |
| 3 | One speaker reaching two microphones, using a small data-flow illustration plus aligned waveform strips. | [Acoustic evidence](../../docs/acoustic-evidence.md), “A simple example”, after the Speaker 3 explanation. Link from “What does ownership mean?” in the ownership guide. | Both microphones can contain the same speech, but only one belongs to the speaker. Loudness or speech activity alone cannot establish ownership. |
| 4 | Automatic, manual, protected and effective gain, using aligned line-chart panels on one time axis. | [Effective envelopes](../../docs/effective-envelopes.md), after “Manual points use…”. Link from the manual editing instructions in [Editor workflow](../../docs/editor-workflow.md). | Which curve reaches playback/export, why manual work survives Analyze, and what removing an override reveals. |

Each figure should answer one question, with a short caption and a text equivalent. Reuse a single asset through links rather than repeating a large diagram in several guides. Keep the approved CleanSlate palette and typography.

## What the diagrams must not imply

- Illustrative waveforms are teaching examples, not measured results or evidence of audio quality. Label them explicitly. A real example needs verified speaker annotations and permission-safe source material.
- Plot gain curves from the current shared evaluator. Include exact speech boundaries, attenuation floor and fade maxima. Do not invent an attractive S-curve or imply that smooth geometry guarantees inaudible transitions.
- Show gap handling separately from fade fitting. Ignoring a brief automatic gap must not appear to erase an explicit room-tone label.
- Manual points use linear interpolation. Protection and manual edits have conflict rules; do not depict them as a generic precedence stack that permits combinations the editor rejects.
- Reduced-volume shading does not mean confirmed silence. An ownership percentage is not a volume meter or measured accuracy.
- Verify current evaluator and invalidation behavior before drawing. This review checked documentation, not implementation behavior.

## Delivery

Keep a static SVG and text equivalent in Markdown for every figure. The gain-control explanation is the strongest candidate for a later interactive companion: changing attenuation, gap length and fade duration directly teaches the controls. The other three figures do not need motion to explain their content.

The previously stated preference for useful interactivity remains relevant, but HTML files linked on GitHub are source files, not a hosted interactive documentation site. Decide whether a downloadable local companion is sufficient or whether hosting is wanted before promising an interactive experience. Do not add a site solely to display these figures.

## Lower-value additions to skip

- Another README pipeline, technology diagram or architecture overview: those relationships are already shown or listed clearly.
- A release swimlane: the release guide's ordered checklist and platform table are sufficient for now.
- A multichannel import tree: the current text covers the simple file-to-selected-tracks relationship. Add it only if users confuse recorder channels with speakers or exported files.
- A full analysis state machine: first extend the existing “Run Analyze again?” table to distinguish reanalysis, rerendering and monitoring-only controls. Source changes also require validation; they are not just another Analyze trigger.
- Historical research and verification pages: label or reconcile stale prose before making it more visually prominent.

## Documentation to reconcile first

[Supervised ownership models](../../docs/supervised-ownership-models.md) and [Temporal ownership](../../docs/temporal-ownership.md) still describe earlier Nemotron/conservative processing. The accepted ADR's later override removes Nemotron. Do not base new architecture diagrams on those earlier descriptions without checking and updating them against current behavior. Historical checkpoint records can retain history, but must remain visibly historical.

## Interview frontier

1. First-pass scope: the first three figures, or all four? Recommendation: first three; the gain-layer comparison is useful but requires more explanation and validation.
2. Example material: clearly marked illustrative signals, or verified real recording excerpts? Recommendation: illustrative signals for the teaching figures, with evaluator-derived gain curves. Real audio examples can follow when listening evidence is the purpose.
3. Interactive delivery: static figures first with a later gain-control companion, or include that companion now? Recommendation: static first, unless the owner wants to settle hosting/local-file delivery during this pass.

The owner accepted all recommendations on 2026-10-11. The first three static figures are implemented under ticket 20, with illustrative signals and evaluator-sampled gain curves. The existing README visuals remain unchanged. Gain-layer composition and an interactive companion remain deferred. See [the diagram inventory](../../docs/diagrams/README.md#audio-behavior-teaching-figures).
