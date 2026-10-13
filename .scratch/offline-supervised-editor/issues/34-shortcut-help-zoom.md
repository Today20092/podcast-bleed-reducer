# Add shortcut help, fit zoom and Escape deselection

Status: done
Type: task
Assignee: Codex shortcut_34
Blocked by: 31
Spec: ../../offline-supervised-migration/specs/shortcut-help-zoom.md

## Scope

Implement the [accepted navigation specification](../../offline-supervised-migration/specs/shortcut-help-zoom.md): Help → Keyboard shortcuts / Ctrl+/, F full-session fit, Shift+F selection fit, Alt-drag zoom and Escape deselection after existing modal/edit cancellation. Reuse the viewport/timebase and preserve source, labels, gain and playhead.

Ticket 31 gates this ticket's popup/selection Escape acceptance. Tickets 32/33 do not gate viewport behavior; integrate reference entries for their controls when available and coordinate shared GUI edits. Ticket 23's ongoing appearance review is not a functional blocker.

## Acceptance checks

- [x] Keyboard-accessible Help/reference dialog lists all implemented bindings and contexts consistently; Ctrl+/ opens it without interfering with text/modal input.
- [x] F fits valid session bounds; Shift+F uses approximately 10% context per side and at least one second visible, clipped/repositioned at recording bounds. Empty/no-selection cases are safe no-ops.
- [x] Alt-drag in normal and gain-edit modes fits a temporary range without labeling, moving gain points or replacing an existing selection/label. Tiny/cancelled drags are safe.
- [x] Explicit fit/zoom stops viewport following while retaining playback, playhead and data; existing Ctrl-wheel/middle-pan actions still work.
- [x] Escape first cancels popup/dialog/text edit/drag, then clears selection when no cancellation owns it. First Escape dismisses the label popup; second clears its retained range. No saved label/manual edit/protection is deleted.
- [x] Keyboard/viewport tests cover bounds, short sessions, margins, unchanged data/playhead, modifier precedence, focus suppression and the ticket-31 popup lifecycle.
- [x] Use scripts/gui-dev.ps1, appropriate Rust checks, formatting and strict Clippy; visually review keyboard reference, Alt-drag and Escape in both editing modes. Record unavailable checks.

## Results

Implemented and integrated as `56009c2c2e465bd64f0e7c3e946c2895a35b2fdc`. F/Shift+F fit session/selection bounds, Alt-drag fits a temporary range with modifier precedence, explicit zoom stops following, and Escape respects the popup/dialog/edit/drag cancellation order. Help and Ctrl+/ share a scrolling reference; Alt+H, Tab/arrows and Enter provide keyboard access.

Validation: 93 library tests and 8 timeline tests passed (8 existing opt-in tests ignored); strict all-target/all-feature Clippy, formatting, diff checks and `scripts/gui-dev.ps1 -BuildOnly` passed. Root GUI review confirmed Ctrl+/, End scrolling, Enter close, Alt+H and keyboard Help routing, selection fit with unchanged playhead, full-session fit and popup-dismiss/selection-clear Escape sequence. It also confirmed ticket 31's popup Stop and Paused · Resume wording. Native Alt-held drag and Ctrl-wheel gestures, narrow-window reference and the full gain-mode keyboard pass were unavailable; regression tests cover those behaviors.

## Comments

2026-10-12 follow-up: Ticket 37 GUI review verified the narrow reference and corrected scrollbar gutter, plus popup dismissal followed by selection clearing while gain editing is enabled. Native Alt-held drag and Ctrl-wheel gestures remain covered by automated checks.

2026-10-11: Owner accepted shortcuts, selection margins/minimum zoom and Escape precedence, and authorized specification/ticket creation.
