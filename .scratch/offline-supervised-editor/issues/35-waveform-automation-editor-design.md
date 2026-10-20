# Design waveform-overlay automation editing

Status: needs-info
Type: grilling
Assignee: unassigned
Blocked by: none
Ideas: I09 follow-up
Decision record: ../../../docs/roadmap-ideas.md

## Design question

How should users edit the effective attenuation curve directly on the waveform, preserving the existing source-bound manual-edit contract while making automation controls easier to use?

The owner requested separate design work informed by FL Studio. This is a design ticket, not permission to implement a new editor. The old lower editor remains usable under ticket 33's hide-unless-editing behavior while this question is resolved.

## Settled direction

- Keep the explicit Edit gain points toggle; normal waveform use remains selection/labeling.
- Ultimately edit the purple effective curve on the waveform and replace the lower strip.
- Include exact typed point dB entry, explicit Mute and fine Shift-drag as design directions; final mechanics depend on the chosen controls.
- Alt-drag zoom remains available in either mode.
- Sound-parameter reset never removes manual curve edits; reset-to-automatic needs a separately discoverable operation.

## Investigation and interview scope

1. Inspect the existing point/region editor, undo, manual constraints and effective-envelope evaluator before designing replacements. Use graph-first discovery and the GUI guide.
2. Research FL Studio automation interactions through current official documentation. Compare point creation/movement/deletion, pencil drawing, point reduction and smoothing; cite evidence and distinguish reference features from CleanSlate choices. Fetch current library/API docs before proposing dependencies.
3. Present an interaction design for overlaid points, hit targets, scaling, exact-value entry, zero gain, keyboard/accessibility, focus and reference/selection/zoom conflicts. Settle existing constraints, point density and reset-to-automatic semantics with the owner.
4. Decide whether drawing/reduction/smoothing belongs in the initial scope or later work. Preserve manual-edit authority across reanalysis, finite/bounded validated values, source binding, atomic saves and shared display/audition/export evaluation. Do not silently change manual interpolation or fade shapes.
5. If an interaction cannot be settled on paper, propose a bounded prototype and capture its findings before specification.

## Completion criteria

- [ ] Referenced behavior and current-editor capabilities are documented with primary sources/code evidence.
- [ ] Owner confirms the complete initial interaction contract, including accessibility, reset and mode/gesture conflicts.
- [ ] An accepted specification and scoped dependency-linked build tickets are linked here, with observable GUI/edit/persistence/envelope parity checks.
- [ ] Optional drawing, reduction and smoothing are explicitly accepted or deferred; no unsettled feature is marked build-ready.

## Results

Pending owner design decisions. Resolve this design ticket only after the completion criteria are met; do not mark a replacement editor implemented.

### Reference review, 2026-10-12

Owner supplied the [official Automation Clips manual](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/playlist_automationclip.htm) and clarified that the reference is envelope GUI interaction, not FL Studio's full workflow.

The documented editor supports right-click creation of points, left-drag movement, point context actions for deletion/exact values, segment tension handles, and grid-spaced freehand drawing in Step mode. Point decimation and envelope smoothing/filtering are separate tools. These reference capabilities inform the interview; they are not automatically requirements for CleanSlate.

CleanSlate-specific decisions remain open: initial point gestures, inclusion of pencil drawing, manual curve shapes/tension, drawing resolution, reduction/smoothing scope, selection/reset behavior and vertical gain scaling. Existing Shift fine-adjustment and Alt-drag zoom decisions must be reconciled deliberately with reference modifiers. No curve interpolation or manual-edit contract has changed.

### Owner decisions, 2026-10-12 (Q40–Q42)

The owner accepted all three recommendations and explicitly requested FL Studio's envelope-editing flow as the primary interaction reference. Preserve that reference in the eventual specification and acceptance review; do not expand scope to FL Studio's full application or unrelated automation features.

- In Edit gain points mode, right-click empty waveform space adds a point; left-drag moves it; right-click a point opens exact time/dB entry, Mute and Delete. Retain Shift fine adjustment and Alt-drag zoom as CleanSlate-specific modifiers.
- Provide Points / Pencil tools. Pencil drawing creates a custom manual envelope, with each stroke grouped into one undoable edit. Drawing resolution and simplification remain open.
- Straight-line interpolation is the default. Include one smooth-curve option with adjustable tension through a handle between points. This explicitly extends the previous manual-linear-only contract when implemented. Exact curve definition, boundaries, persistence and shared evaluation need specification and checks; automatic fade behavior remains unchanged.

Unresolved: stroke overwrite/edge behavior, drawing direction/resolution, reduction and smoothing scope, vertical dB scale and exact zero display, point selection/group operations, reset-to-automatic, and complete keyboard/accessibility behavior. These accepted directions do not make this ticket build-ready.

### Owner decisions, 2026-10-12 (Q43–Q46)

- A pencil stroke replaces manual points only within its drawn time span. Boundary points preserve the surrounding envelope; conflicting protections require explicit replacement. Each stroke remains one undoable action.
- Reduce points is explicit, with a preview and adjustable strength, Apply/Cancel and one undo step on Apply. Keep the original until accepted; do not silently smooth or simplify completed edits.
- Default the waveform-overlay editing scale to 0 through −30 dB, with an expandable lower range, clear out-of-view point indicators and exact-value entry. Mute uses a distinct bottom marker rather than a finite dB value. Expansion/zero representation details remain to be specified.
- A selected time range offers Reset range to automatic, removing manual overrides in that range while exposing generated gain. It is one undoable operation, distinct from resetting a sound parameter. Preserve outside-range edits; intersecting manual-constraint behavior must be explicit in the final contract.

These decisions supersede the corresponding unresolved items above. Remaining interview frontier: drawing resolution/direction, interpolation of pencil strokes, point selection/group movement and ordering, smooth-curve bounds/tension behavior, and keyboard/accessibility. FL Studio remains the primary reference.

### Owner decisions, 2026-10-12 (Q47–Q52)

- Pencil sampling follows visible timeline resolution: approximately one point per horizontal screen pixel crossed, including endpoints. Store source-aware actual times; zooming later never changes the saved curve. Resolve native-time quantization collisions safely rather than storing duplicate timestamps. Automatic simplification is not implied.
- Pencil strokes initially use straight interpolation. Smoothing is explicit; drawing should follow the user's hand without invisible smoothing.
- Ctrl+drag box-selects points; Ctrl+click toggles point membership. Plain click selects one point, empty-space click clears point selection, and dragging a selected point moves the group while preserving relative timing and gain.
- Moves stop before crossing neighboring points or leaving recording bounds; do not silently reorder/merge points. Group moves remain one undo action and retain their relative arrangement.
- Smooth transitions retain fixed endpoints and do not overshoot. A straight segment has a centered tension handle; dragging bends it. Right-click on the handle offers Reset to straight. Curve behavior stays attenuation-only. The precise bounded curve family remains a technical specification task, not an arbitrary FL curve catalog.
- Selected points support Delete, arrow movement, exact-value entry and standard Undo/Redo, with visible focused-point values and keyboard-accessible actions. Shift gives finer movement; text entry owns typing. Keyboard step sizes and tension/drawing accessibility still need a complete interaction contract.

Remaining: backward/retraced drawing, explicit smoothing/reduction accuracy and scope, partial-range reset and protection semantics, automatic/manual boundary behavior, scale expansion and grouped gain movement at Mute, keyboard movement increments and accessibility of remaining controls. The reference remains FL Studio's envelope-editing flow; these decisions do not authorize implementation.

### Owner decisions, 2026-10-12 (Q53–Q58)

- Pencil works in either horizontal direction; the latest position wins when a stroke retraces a time. Release finishes one undoable edit. Native-time ordering and repeated sample positions need deterministic handling.
- Smooth selected points is explicit with strength, preview and Apply/Cancel. Preserve span endpoints, valid gain bounds and surrounding edits. The smoothing algorithm and guarantees require specification before build readiness.
- Reduce points previews original and proposed curves, reports maximum gain-deviation estimate, preserves endpoints and Mute boundaries, and applies in one undo step. Error units, numerical verification and near-silence reporting must be defined accurately in the final spec.
- Reset range to automatic splits intersecting manual edits as necessary: reset only the selected portion and preserve manual behavior outside. Preview resulting boundaries and group the complete operation into one undo step.
- Editing across protection requires explicit Replace protection in this range confirmation. Cancel leaves data unchanged; replacement affects only the edited span and joins the same undo action. Preserve the existing validation/conflict behavior for other manual constraints.
- Ordinary bounded edits meet the current effective curve at their edges. Explicit reset instead exposes automatic gain inside the selected range. Preview resulting discontinuities without silently extending edits or changing neighboring audio. Automatic/manual reconnection behavior on subsequent reanalysis still needs a precise persistence contract.

Remaining interview choices: vertical range selector/Mute interaction, grouped gain movement at Mute and bounds, exact keyboard increments, access to drawing/tension/range actions and behavior of manual boundaries after reanalysis. FL Studio remains the primary GUI reference. No implementation authorized.

### Owner decisions, 2026-10-12 (Q59–Q63)

- Offer overlay view ranges 0…−30, 0…−60 and 0…−120 dB. View changes never alter gain. Bottom-edge dragging produces the finite displayed minimum; Mute is an explicit point-menu/exact-editor action, displayed distinctly.
- Vertical group moves apply one dB offset to finite points while Mute points remain muted. Stop before any finite point would exceed 0 or −120 dB; do not independently clamp points and distort relative gain. Horizontal moves include Mute points, preserve relative time and obey ordering/bounds.
- Left/Right move approximately one visible horizontal pixel in time; Up/Down moves 1 dB. Shift uses one tenth of either increment. Quantize time to valid source-time positions, retaining no-crossing/bounds rules. Mute does not acquire an arbitrary finite value through vertical nudging.
- In editing mode, Shift+drag on waveform background selects a time range; Ctrl+drag selects points; Alt+drag zooms. Shift on a point drag retains fine adjustment. Tension, Smooth, Reduce and Reset have keyboard-accessible menu actions. Typed tension entry complements handle dragging; keyboard Points editing is the Pencil alternative.
- Persisted manual points and boundary gains remain fixed across reanalysis. Newly generated automatic gain can change the junction outside the manual span, but never reshapes accepted manual edits. Display the junction; Reset range to automatic deliberately adopts generated behavior.

The product interview frontier is now settled. Ready to synthesize an automation-editor specification and scoped build tickets, retaining the official FL Studio reference and explicit CleanSlate modifier choices. Technical work remains: inspect current edit/evaluation/persistence seams; define bounded curve/tension and smoothing math, reduction error metrics, deterministic pencil time handling and backward-compatible saved curves; attach behavior/parity checks. Do not mark this design ticket resolved until its accepted specification and build tickets are linked and its completion criteria are met.

## Comments

2026-10-11: Owner requested a separate automation-editor design ticket and confirmed the interim display cleanup in ticket 33. Its design can proceed independently; future build tickets must reconcile navigation and visibility changes in 33/34 and any mute representation in 32.
