# Make keyboard shortcuts adjustable and save examples with S

Status: claimed
Type: task
Assignee: Codex ticket_41
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

2026-10-20 implementation claim: clean primary checkout on development at `0d9ef4c34d18f56013855212048e6f925222b544`; dependencies 31/34/37 verified done. Read accepted design/checks, migration roadmap/spec, domain context/ADR and GUI/build guidance. Graph discovery confirmed app keyboard dispatch and appearance preference persistence; the installed Iced tooltip supports delayed hover and viewport bounds. Implementation has not started: the requested TDD skill requires owner-confirmed seams before writing tests. Proposed app update/keyboard dispatch and local preference load/save seams, plus starting HEAD as the review comparison point, are awaiting owner confirmation. No GUI control, build, source changes or acceptance checks claimed. Context-mode tools were unavailable in this session.

Historical specification note: created from the owner's 2026-10-12 request for adjustable shortcuts and S to save examples; implementation was not authorized in that original specification session.

2026-10-12 follow-up: the owner requested Center playhead, Lock playhead and Unlock playhead shortcuts, plus small tooltips that reveal shortcuts after a hover delay. Added these to this ticket's scope and acceptance checks; no application behavior changed yet.

2026-10-12 zoom follow-up: the owner requested that zoom not change playhead lock, with a possible Settings choice. Default to preserving lock; specify an opt-in local preference to unlock on user zoom. This is ticket scope only, not an implemented change.

## Implementation — 2026-10-20

The owner confirmed the app keyboard/update and local-preference load/save test seams, and `0d9ef4c34d18f56013855212048e6f925222b544` as the review baseline. Work stays in the clean primary development checkout. No owner GUI control, main promotion or release publication is authorized or performed.

- Added one context-aware command definition shared by dispatch, Settings, reference keycaps, popup hints and delayed command tooltips. S saves new/edited examples through the existing validated action; Enter remains a fixed popup confirmation and Ctrl/Command+S remains project saving. Command capture previews before applying, consumes input, names conflicts/contexts and cancels on Escape. Native text editing, dialog confirmation/cancellation, focus navigation and operating-system keys remain reserved; modifiers distinguish chords from plain keys. Individual/default restoration and unbinding preserve compatibility confirmations.
- Reused the existing local preference file, merging fields and saving atomically. Overrides and the zoom preference survive reload and New project without changing project data. Invalid/obsolete/conflicting entries recover independently with actionable feedback; malformed JSON is preserved and saving reports the required repair rather than discarding unrelated preferences.
- Center defaults to C and Toggle lock to L, using the mouse messages. Command tooltips use Iced's existing 600 ms delay and viewport bounds with semantic theme styles. Input resets hint state; click resets occur on release so the tooltip wrapper cannot discard the pending button press. Sound-value instruction tooltips remain absent. Native visual timing, edge placement and theme/width readability still require the owner check below.
- All seven available zoom/fit dispatch routes share the preserve-lock default and opt-in unlock policy. Locked zoom/fit retains the captured playhead anchor at the new scale, subject to recording bounds; it does not seek or alter selection/playback. Manual pan and non-zoom workflows are unchanged; ticket 42 retains precedence and is not implemented here.
- TDD: the initial S regression failed because the popup stayed open, then passed. The remapping slice first failed to compile for its missing interface, then passed. Persistence failed on the absent saved override, then passed. Zoom failed on the old wheel-unlocks behavior; the subsequent gain-route fixture was corrected to enable its existing gain-tool availability condition. Repeat handling first failed for the missing event seam and now passes. Focused app checks: **57 passed, 0 failed, 4 existing ignored**. Strict all-target/all-feature Clippy, formatting and diff checks passed using locked/offline, one-job shared local settings. Test compilation uses ticket 55's documented command-only `profile.test.package.rust-app.codegen-units=4` memory workaround; the repository profile is unchanged.
- Final full local checks, two-axis review, GUI build, tested executable preservation and development backup are pending. Status remains claimed; the acceptance checklist remains incomplete until required checks, including owner verification, pass.

## Required owner check

After the verified build is handed off, check the left-hand labeling loop (speaker keys, R, S, Enter, room tone and editing), shortcut capture/conflicts/remapping and restart persistence. Check shortcut hints and delayed tooltips in Dark/Light at narrow/wide widths and near window edges, including typing/click dismissal and the current Lock/Unlock name. Exercise Center/Lock plus zoom/fit while playing under both preference values. This agent has not controlled the owner's GUI or claimed these checks passed.

## Standards

Independent code-review axis compared `git diff 0d9ef4c...09f6b9d`: one P2 finding. The configurable Reference guard preceded dialog navigation, so binding it to Page Down opened the reference instead of scrolling Settings/About/Help. This violated the input-ownership requirement and AGENTS.md's instruction to update affected callers. Fixed by letting existing named-key dialog handlers own input before the reference guard. The regression failed on the checkpoint and covers all three dialogs plus subsequent workspace activation. Follow-up review: resolved; no additional actionable standards breaches or smell findings.

## Spec

Independent code-review axis compared the same checkpoint with ticket 41's accepted design: one P2 finding. The gain editor still named Arrow/Delete after remapping, contrary to accepted design item 5. Fixed by formatting the visible instruction from effective Earlier/Later/Louder/Quieter/DeleteGain bindings, including Unbound. Follow-up review confirmed the gain hint, dialog priority and unambiguous Plus key display; no new actionable Spec findings. The plus-key regression also failed before its display fix. Reviewers ran no builds, tests or GUI checks.

Review summary: initial Standards 1 (dialog ownership), Spec 1 (stale gain hints); both resolved, with 0 remaining findings on either axis after follow-up. Native owner acceptance remains pending.
