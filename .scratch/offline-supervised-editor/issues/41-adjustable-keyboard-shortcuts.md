# Make keyboard shortcuts adjustable and save examples with S

Status: ready-for-agent
Type: task
Assignee: unassigned
Blocked by: 31, 34, 37
Target branch: development
Spec: #accepted-design

## Dependency reconciliation — 2026-10-20

Unimplemented and unclaimed; all remaining implementation dependencies are done. Ticket 39's reference layout/tooltip removal is already integrated into development, so it no longer blocks this build. Its manual acceptance is retained in 23; do not recreate or discard that work. Use development under the current repository policy. Ticket 42 remains unchanged and retains precedence for its locked middle-pan behavior. This reconciliation authorizes no implementation in this audit.

## Scope

Add adjustable application keyboard shortcuts to Settings on the existing integration branch. Make unmodified `S` the primary default for Save example / Save changes in the Who is talking? popup, so speaker toggles, Clear and Save can be performed with the left hand. The owner's screenshot shows the current `Enter Save` instruction; Enter currently dispatches the reference-save action in `CleanSlateApp::handle_edit_key`.

Include keyboard access to Center playhead and Lock / Unlock playhead. Add small delayed-hover shortcut tooltips to application command controls so users can discover the current bindings without opening the reference.

Zooming must preserve the current playhead lock by default. Offer a local preference for users who want zooming to unlock it.

This is a build specification, not a request to implement or merge into main now. Specification readiness does not clear the listed dependencies.

## Accepted design

1. **Left-hand example saving.** While the speaker-label popup owns input, `S` saves the selected speaker set or room-tone example through the existing validated Save action. It also saves edits to an existing example. Keep Enter as a compatibility confirmation option. The visible primary hint is `S Save`; describe Enter as an alternative where useful. Preserve the existing button, R Clear, speaker toggles, Space pause/resume, Stop and Escape behavior.
2. **Context ownership.** A popup save must not also save the project or invoke a workspace action. Keep Ctrl+S / platform Command+S for project saving. Text entry, gain-value editing, dialogs and shortcut capture own their input before application shortcuts. Preserve existing repeat suppression and shortcut availability conditions. Saving once must create only one example; validation failures must keep the popup and explain the problem.
3. **Settings editor.** Provide a Keyboard shortcuts section listing current application command names, bindings and activation contexts. Allow reassignment of existing application keyboard commands, including label-popup commands, speaker toggles, transport, zoom, editing and existing app commands. Capture a key or modifier chord instead of requiring users to type a configuration string. Show the proposed binding before applying it; Escape cancels capture without changing the previous binding. Capture must not trigger the underlying command.
4. **Conflicts and reserved keys.** Reject duplicate bindings when their commands can be active together, with the conflicting command and context named. Allow reuse across mutually exclusive contexts. Preserve standard widget text editing, focus navigation, dialog confirmation/cancellation and operating-system/native-dialog behavior; explain reserved bindings in the editor. Do not silently swap, overwrite or disable another command. Modifier chords must remain distinct from plain keys.
5. **One binding definition.** Reuse one command/binding definition for dispatch, Settings, shortcut reference, popup hints and other existing displayed keyboard hints. Show effective bindings immediately after changes, retaining ticket 39's readable themed keycaps and narrow-window layout. Avoid separate hard-coded shortcut tables that can disagree with actual behavior.
6. **Local preferences.** Persist accepted overrides as application preferences across restart, independently of projects and recordings. Provide Restore default for one command and Restore all defaults. Defaults include S for example saving. Missing, invalid or obsolete preference entries fall back safely to documented defaults with actionable feedback, without corrupting the file or preventing startup. Preserve valid unrelated settings.
7. **Playhead commands.** Expose Center playhead and Toggle playhead lock as configurable commands with conflict-free documented defaults chosen against the existing binding inventory. The toggle handles both Lock and Unlock, matching the current state-dependent button label; show the applicable action in hints and the shortcut reference. Center changes the viewport only, without seeking, changing the selection, restarting playback or enabling/disabling lock. Lock and Unlock retain their existing follow behavior. Preserve text/dialog/popup ownership and suppress repeated toggles while a key is held. The mouse controls and keyboard commands must dispatch the same actions.
8. **Delayed shortcut tooltips.** Show a compact tooltip containing the action name and effective binding after approximately 600 ms of continuous hover over an application command control. Include Save example / Save changes, Center playhead, Lock / Unlock playhead and other visible command controls with assigned shortcuts. Use the shared binding definition, including state-dependent action names and user overrides; never show an obsolete default after remapping. If an action is unbound, show its name without inventing a shortcut. Dismiss promptly on pointer exit, click, typing or popup dismissal, and reset the delay for another control. Tooltips must not capture input or obscure immediate interaction. Use existing semantic theme styles and bounded placement at window edges. This authorizes concise shortcut hints on command controls; do not restore ticket 39's redundant sound-value hover instructions or add tooltips to every value/text field.
9. **Zoom and lock preference.** Default all user zoom operations to preserve the current playhead lock state, including wheel/button/keyboard zoom, Fit recording and Fit selection. Zoom must neither silently unlock a locked playhead nor enable lock when it was off. Preserve the locked follow behavior at the new scale without seeking, restarting playback or changing the selection. Add an application preference labeled `Unlock playhead when zooming`, off by default; when enabled, a user zoom action releases an active lock using the existing Unlock behavior. Persist this preference locally across restart independently of projects. Apply the same policy at every user zoom entry point. Manual panning, explicit Lock/Unlock and non-zoom workflows retain their existing behavior; do not treat a background viewport update as a user zoom command.

## Implementation pointers

- `src/app.rs`: `CleanSlateApp::handle_edit_key`, `Message::KeyPressed`, `ContextReference`, speaker-label popup construction, `shortcut_reference`, Settings dialog and existing key-ownership tests.
- Tickets 31 and 34 define label-loop and shortcut ownership behavior. Ticket 39 defines reference presentation. Reuse existing local settings persistence and theme components if available.
- Existing `Message::CenterPlayhead`, `Message::LockPlayhead` and `PlaybackScroll` implement the mouse controls. The latter message currently toggles lock; reuse its semantics for the state-dependent keyboard action. Inspect the GUI framework's supported tooltip delay before adding timer machinery.
- Existing zoom and fit handlers explicitly set `PlaybackScroll::Off`; replace those unconditional user-zoom resets with the accepted preference policy. Inspect wheel zoom, fit commands and their keyboard routes together; leave unrelated pan and workflow resets unchanged.
- Graph discovery was attempted during ticket creation but its transport was closed; scoped source inspection confirmed the current Enter-only popup binding. Implementation must refresh graph coverage and inspect all callers before changing dispatch.

## Acceptance checks

- [ ] With defaults, select speakers using 1–8, clear using R and save using S without moving the hand across the keyboard. S also saves room tone and updates an existing example. Enter and mouse confirmation still work.
- [ ] One S press produces one save. A held key does not create duplicate examples. Popup shortcuts do not leak to workspace commands; Ctrl+S retains project-save behavior and typing s in a text field never saves an example.
- [ ] Remap Save example to another available binding: the new binding saves, the old S binding no longer saves, and hints/reference/Settings show the effective binding. Verify other command categories through meaningful dispatch tests.
- [ ] Capture consumes command keys; Escape cancels capture. Same-context conflicts and reserved bindings are rejected clearly, while mutually exclusive contexts can reuse a binding. Plain keys and modifier chords remain distinct.
- [ ] Overrides survive restart without changing project data. Individual/all-default restoration works. Invalid/obsolete configuration preserves valid preferences and recovers safely.
- [ ] Defaults preserve existing transport, label-loop, shortcut ownership, gain-edit and help/dialog behavior. Regression tests cover save validation, repeat suppression, text entry, conflicts, persistence and displayed-binding consistency.
- [ ] Center, Lock and Unlock are available from the keyboard, listed in Settings/reference, remappable and consistent with mouse controls. Center does not seek or change lock/selection/playback. Holding the lock shortcut cannot oscillate the state; typing in fields and active popups cannot trigger playhead actions.
- [ ] Shortcut tooltips appear only after the hover delay, show the effective remapped binding and the current Lock/Unlock action, and disappear/reset correctly. Verify Dark/Light, narrow widths and edge placement. Sound-value instruction tooltips remain absent; tooltips never steal clicks or keyboard input.
- [ ] With defaults, every available user zoom/fit path preserves both Locked and unlocked states, while active locked playback continues following at the new scale. Position, selection and playback are unchanged. With `Unlock playhead when zooming` enabled, the same paths release an active lock but never enable one. The preference survives restart; manual pan and background viewport updates retain their documented behavior. Add regression coverage for each distinct zoom entry point and both preference values.
- [ ] Run formatting, applicable tests and strict all-target/all-feature Clippy. Build through `scripts/gui-dev.ps1` using a safe target that does not overwrite the owner's running executable.
- [ ] Ask the owner to check the left-hand labeling loop, remapping, restart persistence and shortcut readability in both themes at narrow/wide widths. Do not control their GUI unless subsequently requested. Record unavailable checks and leave the ticket incomplete until scoped checks pass.

## Out of scope

Macros, multi-step key sequences, downloadable presets, cloud sync, per-project bindings, mouse-gesture remapping, editor commands beyond the specified keyboard access to existing playhead controls, playback-engine changes, custom keyboard diagrams, general instructional/value-field tooltips and native operating-system shortcut customization.

## Results

Not implemented. Created from the owner's 2026-10-12 request for adjustable shortcuts and S to save examples. No source changes, GUI actions or builds were performed for this ticket.

2026-10-12 follow-up: the owner requested Center playhead, Lock playhead and Unlock playhead shortcuts, plus small tooltips that reveal shortcuts after a hover delay. Added these to this ticket's scope and acceptance checks; no application behavior changed yet.

2026-10-12 zoom follow-up: the owner requested that zoom not change playhead lock, with a possible Settings choice. Default to preserving lock; specify an opt-in local preference to unlock on user zoom. This is ticket scope only, not an implemented change.
