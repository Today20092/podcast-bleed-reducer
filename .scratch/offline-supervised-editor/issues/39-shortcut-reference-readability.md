# Remove sound-value hover help and improve shortcut readability

Status: claimed
Type: task
Assignee: Codex implement_39
Blocked by: 32, 34, 37
Target branch: codex/editor-workflow-31-37
Spec: #accepted-design

## Scope

Apply the owner's accepted tooltip and Keyboard shortcuts presentation changes on the current branch. Dependencies 32, 34 and 37 are done. Preserve ticket 38's in-progress changes and unrelated workspace edits. This ticket does not authorize merging into main.

## Problem

Sound-value hover text overlaps controls and becomes unreadable. The owner selected removal because the value is already clickable and Reset is available in its context menu and documented shortcuts. The shortcut reference is difficult to scan because combined bindings and descriptions contain several actions, with little separation between rows.

## Accepted design

1. Remove the sound-value tooltip for attenuation and fade/gap parameters. Keep click-to-type, Ctrl+click reset, the right-click Reset to default action, validation and keyboard behavior. Keep these gestures documented in the reference. Do not replace the removed tooltip with another hover instruction.
2. Use small text keycap badges with a subtle themed background and border. Show chords as separate keys joined by a plus sign, such as Ctrl + Z. Use readable text rather than images or a complete keyboard illustration. Mouse gestures stay plain text.
3. Put the action first and the binding in a consistent right-hand column. Give each action its own row, including Original/Processed/Removed and Fit recording/Fit selection. Present alternative bindings clearly under the same action, rather than slash-separated actions requiring readers to match positions.
4. Place necessary conditions or explanations on a quieter second line below the action. Keep them readable in both themes and preserve distinctions between workspace, popup, text-entry and gain-editing shortcuts.
5. Increase section spacing and add subtle row separators. Start without alternating row colors. Keep existing theme tokens, a restrained type scale and a visible Close control.
6. Allow long descriptions and bindings to wrap without overlapping or clipping at narrow window widths. Preserve scrolling, keyboard navigation and current platform-specific behavior. Describe actual bindings without changing them.

## Acceptance checks

- [ ] Sound-value hover instructions no longer appear; typing, reset, mute, validation and context menus still work.
- [ ] Every documented action has an unambiguous binding; grouped action/binding pairs have been split and no existing shortcut or condition is lost.
- [ ] Action-first rows, keycap badges, secondary explanations, section spacing and separators are readable in Dark and Light, at narrow/wide widths and with long content.
- [ ] Close, Escape, Enter and reference scrolling retain their current behavior; text/dialog shortcut ownership remains unchanged.
- [ ] Run formatting, applicable existing GUI tests and strict all-target/all-feature Clippy. Build with scripts/gui-dev.ps1 when the owner has closed the running development executable.
- [ ] Ask the owner to verify tooltip absence and shortcut readability in both themes and a narrow window. The owner is using the computer; do not use Computer Use unless they subsequently request it. Record unavailable checks and do not mark done until scoped checks pass.

## Out of scope

Playback preparation/cache changes, new shortcut bindings, global tooltip styling/delays, alternating row colors, keyboard illustrations, custom themes and ticket 38's import work. Useful tooltips elsewhere can receive a delayed-hover treatment in a separately authorized change.

## Results

Implemented on scoped branch `codex/shortcut-readability-39`, based on integration checkpoint `df3ad23`. The checkpoint already contains the accepted sound-value tooltip removal. The reference now uses one action per row, a consistent binding column, themed text keycaps with wrapping chords, explicit alternative bindings, muted condition lines, section spacing and separators. Under 560px of available content width, bindings move below the action. Responsive rows explicitly shrink vertically so scrolling remains bounded. No shortcut dispatch, sound-value validation, theme tokens or dependencies changed.

Formatting and diff checks pass. Existing app behavioral tests, strict all-target/all-feature Clippy and the GUI development build remain pending compilation. Owner verification remains pending for tooltip absence, value gestures, narrow/wide readability in both themes, reference scrolling and Close/Enter/Escape. The ticket's instruction prohibits Computer Use unless subsequently requested; no GUI was controlled or closed. Keep claimed until these acceptance checks pass.

## Comments

2026-10-12: Owner accepted the proposed keycap badges, one-action rows, action-first layout, secondary conditions, section spacing and subtle separators, and explicitly requested a new ticket covering tooltip removal plus shortcut readability. Playback preparation was explained separately; no playback change is authorized by this ticket.
