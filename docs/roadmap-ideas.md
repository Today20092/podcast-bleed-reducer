# CleanSlate feature ideas

Latest owner decision, 2026-10-20: waveform-overlay gain editing (35) is deferred as unnecessary for current use because the owner is satisfied with automatic detection. Preserve its interview below as optional future reference; no specification/build handoff is active. The earlier specification-ready summary is historical. Training-panel proposal 17 also remains deferred.

2026-10-22 current status: waveform-overlay proposal [35](../.scratch/offline-supervised-editor/issues/35-waveform-automation-editor-design.md) and advanced training 17 remain deferred. The [current active queue](../.scratch/offline-supervised-migration/implementation.md#current-queue--reconciled-2026-10-22) governs release preparation, remaining verification and future menu/action-registration work. Earlier interviews and open-question lists below are historical.

Captured from owner feedback on 2026-10-11. This inventory and interview record is linked from the [public roadmap](../ROADMAP.md). I05–I14 have accepted scoped specifications and build tickets linked below; the remaining ideas are proposals. The [active map](../.scratch/offline-supervised-migration/map.md), [accepted specification](../.scratch/offline-supervised-migration/spec.md) and [implementation tracker](../.scratch/offline-supervised-migration/implementation.md) govern implementation. No delivery date has been agreed for these additions.

The two attached screenshots show a purple gain curve and a lower green/teal line that appears to follow the same attenuation changes. They are visual evidence for idea I09; the purpose of the lower line has not been verified in code.

## Synchronization decision, 2026-10-11

The owner deferred I01–I04 as a synchronization effort after reviewing the [research](research/synchronization-and-drift.md). Users should align recordings and correct drift in their audio or video editor before importing synchronized audio into CleanSlate. No synchronization, time-stretching or drift-checking build ticket is authorized. The original ideas below are retained for reference; revisit their open questions only if this scope is reopened. Existing support for importing already-synchronized sources is unchanged.

If revisited, the owner prefers natural voices evaluated through measurements and listening, and preview with explicit acceptance of every proposed synchronization. No fixed-versus-shared clock policy or processing library was selected.

## Idea table

2026-10-12 reconciliation: the deferral recorded from the research chat supersedes this chat's later invitation to grill I01–I04. That interview is closed; Q34–Q39 proposed here remain unanswered and do not reopen or authorize synchronization work.

This table preserves the original proposal inventory. For I05–I14, later confirmed interview decisions and linked specifications below supersede its open questions.

| ID | Feature or idea | Desired behavior | Decisions to settle before a build ticket |
| --- | --- | --- | --- |
| I01 | Import independently recorded sources | Combine microphone, camera and separate-recorder audio, including clips recorded outside a shared multichannel recorder. | Whether camera audio means an audio file or decoding a video container; unequal durations, partial overlap, multiple clips per speaker, and how sources map to the session timeline. |
| I02 | Automatic synchronization | Estimate starting offsets from shared audio and align long podcast recordings as reliably as possible. Offer review when alignment is uncertain. | Reference timeline, confidence/failure behavior, preview and manual correction. This extends the current requirement for already synchronized inputs. |
| I03 | Guided synchronization | Let users identify a shared short passage or click corresponding transients, such as a clap; refine alignment around those anchors. | Anchor placement and refinement, number of anchors, manual overrides, and behavior when shared audio is weak. |
| I04 | Pitch-preserving drift correction | Correct recorder clock drift over one- or two-hour sessions without changing perceived voice pitch. Coordinate four or five tracks so correction is shared rather than forcing all adjustment onto one track. | Constant versus changing drift, fixed versus shared reference clock, quality criteria and correction limits. The owner's example is end offsets of −30 ms and +30 ms: a midpoint target gives each extreme a 30 ms correction rather than giving one a 60 ms correction. Whether midpoint, minimum maximum correction or another objective is appropriate remains open. |
| I05 | Automatic loop audition while labeling | On releasing a waveform selection and opening “Who is talking?”, immediately play and loop that selected audio so the user can confirm the active speakers. | Original versus Processed audition, selected track versus all/soloed tracks, stop/pause control, and transport behavior on Save, Cancel, Escape or a new selection. |
| I06 | Clearer labeling shortcuts | Make the visible 1–8 speaker toggles, R to clear and Enter to save easier to read and understand. Keep keyboard guidance visible. | Wording, key styling, room-tone explanation and focus behavior while audio loops. |
| I07 | Custom attenuation entry and mute | Make the displayed dB value editable beyond the current user-reported −3 to −30 dB slider range; explore a maximum-reduction/silence choice. | Valid finite range, slider mapping and an explicit Mute/−∞ dB representation. “Infinity” here means zero gain, not a nonfinite ordinary numeric parameter. Check persistence, audition, envelopes and export together. |
| I08 | Custom sound timing values | Allow typed values for Fade to full volume, Fade to reduced volume and Ignore gaps shorter than, alongside convenient sliders. | Units, valid bounds, zero behavior, commit/cancel behavior, and persistence. Custom entry does not imply accepting arbitrary invalid values. |
| I09 | Remove the lower green/teal waveform line | Keep the purple attenuation curve and remove the lower line shown in the screenshots, which looks redundant and adds visual clutter. | Confirm what the line represents and whether any unique information needs another presentation before removing it. |
| I10 | Centered import dialog and focused layout | Present track/channel import options in a compact centered dialog like the Export dialog, instead of a full-screen view with controls at the upper left. Keep primary work near the center; occasional sound controls may remain at the side. | Which import stages belong in the dialog, small-window behavior, keyboard focus and validation. Whether this principle changes the pointer-adjacent labeling popup needs a separate decision. |
| I11 | Keyboard shortcut reference | Add an accessible menu/action showing all supported shortcuts and their context. | Entry point, help shortcut, platform key names and how the list stays consistent with actual bindings. |
| I12 | Zoom to fit | Add a keyboard shortcut that fits the entire recording waveform in the timeline. | Binding, definition of full session bounds and interaction with playback following. |
| I13 | Zoom to selection | Fit a selected range with left/right context margins. Consider modifier-drag as a direct way to select a zoom region. | Binding, margin policy, empty/tiny selections and modifier conflicts with labeling, panning and gain editing. Modifier-drag is an optional interaction, not yet a requirement. |
| I14 | Escape clears waveform selection | Let Escape deselect the current waveform range. | Precedence when a popup, dialog, text field or gain edit is active; distinguish dismissing a popup from clearing the underlying selection. |
| I15 | Settings, About, Help and reporting | Provide a discoverable app menu for settings, version/About, credits/references, GitHub and Report a bug. Consider refresh-rate preferences and future help documents. | Minimum initial menu, useful settings, supported refresh-rate behavior and destinations. Reporting should open GitHub Issues for the user to submit; diagnostic collection is a separate proposal. |
| I16 | Semantic color tokens and appearance themes | Define colors by roles such as background, foreground, warning and gain curve. Later support dark/light appearance, custom themes and possibly importing theme packages. | Token coverage, contrast and semantic timeline colors; default themes; theme format and validation. Separate the token foundation from customization and theme-package import. |
| I17 | Plugin and platform exploration | Investigate VST and/or CLAP integration for DAWs, potentially including DaVinci Resolve, and possible live or post-production workflows across Windows, macOS and Linux. | Host format/support must be researched rather than assumed. Choose offline versus live first; define how supervised labeling and project editing fit a host. Plugin delivery is separate from existing desktop platform packaging work. |

## Guided tutorial and usage video, 2026-10-12

The owner requested a future dedicated usage video once the basic features are settled, plus a guided in-app tutorial for first-time users. This is a roadmap proposal, not an accepted build specification or implementation ticket.

The tutorial should show a workflow chart and lead users through a practice recording with four or five synchronized microphone tracks:

1. Import the practice tracks and listen to Original audio.
2. Select waveform passages and identify everyone speaking. Practice solo speech, overlap such as speakers three and four together, and room tone with nobody speaking.
3. Supply a solo example for each speaker and a room-tone example, plus examples of varied speaking conditions. Do not prescribe a fixed sample count without evidence.
4. Analyze, then review the remaining recording using Original, Processed and Removed listening modes.
5. Find incorrect ownership decisions, select those passages, and label the actual speaker set. For example, label speakers one and three together when the model misses that overlap. Analyze again using the corrected examples.
6. Review the result again and repeat corrections where necessary. Explain that examples train the model, while manual gain edits adjust the output directly. New examples may improve other passages but do not guarantee correct predictions throughout the recording.
7. Save the editable project and export processed WAVs after listening confirms the result.

Make the tutorial optional on first launch and discoverable again from Help. The dedicated video should teach the same workflow after the basic features settle. Before a build ticket, settle the practice-audio source and redistribution rights, bundled versus separately obtained audio, exercise feedback, skip/replay and completion behavior, accessible chart/text presentation, and the dependency on the final editor controls. Coordinate the Help entry with ticket 36. No tutorial code or video production is authorized by this inventory entry.

## Suggested next steps

### Automation-editor interview, 2026-10-12

Q59–Q63 are accepted: finite view ranges 0…−30/−60/−120 dB with explicit Mute; grouped dB moves preserve finite-point differences and leave Mute muted; keyboard movement uses visible-pixel time/1 dB steps with Shift fine adjustment; background Shift/Ctrl/Alt drags select ranges/points/zoom with accessible curve/actions; manual points and boundary gains stay fixed after reanalysis. The product interview is settled and ready for specification/build-ticket synthesis. FL Studio remains the primary GUI reference. Ticket 35 remains a design record until the technical contract and build tickets are completed.

Q53–Q58 are accepted: bidirectional/retraced Pencil uses latest positions; explicit smoothing has strength/preview/Apply/Cancel with fixed endpoints; reduction previews both curves and deviation while preserving Mute boundaries; partial reset preserves outside manual portions; protection replacement is explicit and bounded; ordinary edits meet the effective curve at their edges while reset exposes automatic gain and previews discontinuities. Remaining scale/group/keyboard and reanalysis-boundary decisions live in ticket 35.

Q47–Q52 are accepted: pencil points follow visible horizontal resolution and retain actual times; pencil interpolation is initially straight; Ctrl box/toggle selection and group dragging preserve relative values; movement cannot cross neighboring points or recording bounds; bounded smooth curves retain endpoints with resettable tension; selected points support keyboard movement, exact values, Delete and Undo/Redo with Shift fine adjustment. Detailed boundary/reset/smoothing/accuracy and remaining keyboard behavior are still open in ticket 35.

Q43–Q46 are also accepted: pencil strokes replace only their drawn span while preserving boundary/outside behavior; point reduction has adjustable preview with Apply/Cancel and one undo action; overlay defaults to 0…−30 dB with expandable lower range/out-of-view markers and distinct Mute; selected-range reset removes only manual overrides and exposes automatic gain in one undo action. FL Studio remains the primary reference. Drawing resolution, interpolation details, group selection/movement and accessibility remain under interview in ticket 35.

In [design ticket 35](../.scratch/offline-supervised-editor/issues/35-waveform-automation-editor-design.md), the owner confirmed FL Studio's envelope GUI flow as the primary reference and accepted Q40–Q42: right-click creation/context actions and left-drag movement over the waveform; Points / Pencil tools with one undo step per stroke; straight transitions by default plus one smooth curve with a tension handle. Shift fine adjustment and Alt-drag zoom remain CleanSlate-specific choices. The [official reference](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/playlist_automationclip.htm) is recorded in the ticket. Remaining drawing, scale, reduction, reset and accessibility decisions must be settled before an accepted automation build specification; current manual interpolation is unchanged until implementation.

### Settings/help and appearance interview, 2026-10-11

The owner accepted these directions in Q25–Q30. Final interface details and confirmation remain pending; no new build ticket is ready from this round alone.

- I15: provide Settings, Keyboard shortcuts, Help, GitHub, Report a bug and About in an app menu. Put credits/references inside About. Help opens the existing usage documentation; Report a bug opens the GitHub issue chooser for the user to review and submit.
- Diagnostic-report generation remains separate future work; no collection or submission is included in the initial reporting action.
- The owner is unsure whether refresh-rate preferences are needed. Defer this control until measured redraw/resource behavior establishes a useful need; the initial Settings scope does not require it.
- Appearance/interface preferences apply across the app; sound settings remain project-local. Opening a project should not change preferred appearance.
- I16: initial theme work establishes semantic color tokens, preserves the current dark appearance and adds a light theme, with consistent waveform meanings, readable contrast and visible keyboard focus.
- Individual color customization and theme-package import remain separate future work after the token foundation and customization format are agreed.

On 2026-10-12 the owner accepted Q31–Q33:

- Settings offers Dark / Light, applies the choice immediately and remembers it across restarts. Dark remains the initial default. Automatic OS-theme matching is deferred.
- A clearly labeled Menu button sits in the top bar. Settings and About open centered dialogs with the waveform workspace visible behind them and existing modal focus/Escape behavior.
- About shows application version, build identifier when available, license, credits/references and Copy version info. Copied information contains only app/build/platform details, excluding audio, project names and file paths.

The owner confirmed shared understanding and authorized specs/tickets on 2026-10-12:

- [36 — App menu, Help and About](../.scratch/offline-supervised-editor/issues/36-app-menu-help-about.md), [accepted spec](../.scratch/offline-supervised-migration/specs/app-menu-help-about.md), blocked by shortcut-reference ticket 34.
- [37 — Semantic colors and Dark/Light appearance](../.scratch/offline-supervised-editor/issues/37-semantic-dark-light-themes.md), [accepted spec](../.scratch/offline-supervised-migration/specs/semantic-dark-light-themes.md), blocked by Settings entry point 36.

Both are unclaimed build specifications, not implemented features. Refresh-rate controls, diagnostic reports, OS-theme matching, individual color customization and theme-package import remain deferred. The owner requested starting synchronization I01–I04 grilling. Synchronization, plugin exploration I17 and automation design ticket 35 remain separate open discussions; no synchronization specification is accepted yet.

### Interview decisions, 2026-10-11

The owner selected I05–I14 for clarification and accepted these first-round behaviors. Later rounds and final confirmation below supersede provisional choices; the linked specifications record the accepted build scope.

- I05: opening the speaker-label popup automatically loops the selected range across all tracks in Original mode. Save or dismissal stops the loop, leaves the selection visible and does not automatically resume prior playback.
- I06: try key badges beside speakers and a visible footer: “1–8 Toggle speakers · R Clear · Enter Save · Esc Cancel”, with “None selected = room tone” beside the choices.
- I07–I08: retain sliders and make displayed values editable. Enter commits, Escape cancels and invalid input gets an explanation. Provide an explicit Mute choice separately from finite attenuation values. Numeric limits remain to be decided.
- I09–I10: remove the lower green/teal line after verifying its purpose; center the import dialog. Keep the speaker-label popup beside the selected waveform range.
- I11–I13: Help → Keyboard shortcuts, Ctrl+/ to open it, F to fit the recording and Shift+F to fit the selection, with approximately 10% of selection duration as context on each side. Bindings do not fire while typing. Alt+drag draws a zoom range without opening the speaker popup, subject to checking existing conflicts.
- I14: Escape first dismisses the speaker popup; a second Escape clears the underlying waveform selection.

The owner also accepted these second-round behaviors:

- I05: Space pauses/resumes the loop. Loop audition temporarily ignores Solo state and restores it on popup closure. If playback is preparing, show “Preparing audio…” and start only if the same popup is still open when preparation completes; dismissal prevents delayed playback.
- I07: typed attenuation accepts 0 to −120 dB, plus explicit Mute for exact silence; no amplification. Keep the slider convenient for ordinary adjustments.
- I08: typed fade and gap values accept 0–60,000 ms. Zero means instant fades or disabled gap bridging. Long values still preserve speech and manual edits. Keep sliders for common values.
- I07–I08: apply typed values on Enter or clicking elsewhere. Escape restores the previous value. Invalid text remains editable with an explanation.
- I13: selection zoom shows at least one second, centered around the selection; context margins are clipped at recording boundaries.
- I10: the centered dialog contains the entire import setup, including channel choices, names, validation and confirmation. Resize or scroll for larger imports, keep errors inside the dialog and leave the waveform workspace behind it.

Third-round decisions:

- I09: the owner prefers editing attenuation directly on the curve overlaid on the waveform, rather than retaining a separate lower gain-editor strip. The proposed hide-unless-editing strip was not accepted. Exact overlay editing interactions remain open; the owner has not yet used the existing manual editor.
- I05: restore the prior Original/Processed/Removed listening mode as well as Solo state when the label popup closes, leaving playback stopped.
- I07: Mute means exact silence where the attenuation setting applies, with fades transitioning into silence. Preserved speech and protected manual edits retain their existing behavior; this is not a master mute.

Read-only code investigation found that the lower teal curve is the effective gain-editor envelope, while the upper purple waveform curve displays the effective preview gain. The lower strip also contains manual point-editing interactions. Moving editing onto the upper curve must preserve those capabilities, validation and undo behavior. F, Shift+F, Ctrl+/ and Alt-drag have no current assigned action. Existing Escape cancellation layers must take priority over selection clearing. Typed gap values outside 10–5000 ms require a domain-validation change.

Fourth-round decisions:

- Overlay gain editing retains the explicit Edit gain points toggle. Normal mode selects waveform ranges for labeling; editing mode exposes curve controls. Alt-drag zoom remains available in both modes.
- The owner did not accept a specific add/move/delete point interaction. Automation editing needs its own design work/ticket, using FL Studio as a reference and exploring pencil drawing, point reduction and smoothing. These are investigation topics, not accepted implementation requirements. Research actual reference behavior before prescribing controls.
- The owner accepted exact typed point dB entry and Shift-drag for finer adjustment as a direction for the separate editor design, with explicit Mute. Final mechanics depend on that design.
- For reset interactions, the owner suggested a modifier-click on a parameter name/value, inspired by Melda plugins. The exact gesture and distinction between resetting a sound parameter to its default and resetting manual curve overrides to automatic gain remain open. Q21's proposed range-reset action was not accepted.

Fifth-round decisions:

- I07–I08: Ctrl+click a displayed sound parameter value restores that parameter's default. Provide a right-click Reset to default action and explain the gesture in the tooltip and shortcut reference. Resetting a sound parameter never removes manual curve edits.
- Resetting manual curve edits to automatic gain is deferred to the separate automation-editor design.
- I09 interim scope: hide the lower gain-editor strip during normal use and show it when Edit gain points is enabled. Keep the existing editing tool functional until waveform-overlay editing replaces it in separately specified work.

The owner confirmed shared understanding and authorized specifications and tickets on 2026-10-11. Four build tickets are ready-for-agent but unclaimed; the separate automation-editor ticket requires design decisions. This discussion has not implemented them.

| Record | Scope | Specification |
| --- | --- | --- |
| [31](../.scratch/offline-supervised-editor/issues/31-label-loop-audition.md) | Label audition and shortcut clarity | [Accepted spec](../.scratch/offline-supervised-migration/specs/label-loop-audition.md) |
| [32](../.scratch/offline-supervised-editor/issues/32-sound-value-entry.md) | Typed sound values, Mute and reset | [Accepted spec](../.scratch/offline-supervised-migration/specs/sound-value-entry.md) |
| [33](../.scratch/offline-supervised-editor/issues/33-focused-import-waveform.md) | Centered import and interim gain-strip cleanup | [Accepted spec](../.scratch/offline-supervised-migration/specs/focused-import-waveform.md) |
| [34](../.scratch/offline-supervised-editor/issues/34-shortcut-help-zoom.md) | Shortcut reference, fit zoom and Escape selection clearing | [Accepted spec](../.scratch/offline-supervised-migration/specs/shortcut-help-zoom.md) |
| [35](../.scratch/offline-supervised-editor/issues/35-waveform-automation-editor-design.md) | Waveform-overlay automation editor | Product interview complete; technical specification and build handoff pending |

### Proposed specification groups

| Group | Ideas | Agreed scope |
| --- | --- | --- |
| Label audition | I05–I06, I14 | Automatically loop the selection in Original across all tracks while labeling; Space pauses/resumes. Stop on Save/dismissal and restore listening mode/Solo without resuming earlier playback. Guard deferred preparation against dismissed/replaced popups. Show speaker key badges and clear shortcut/room-tone guidance. Preserve existing Escape cancellation priorities, then clear selection when no popup/edit cancellation takes precedence. |
| Sound value entry | I07–I08 | Sliders plus committed typed values: attenuation 0 to −120 dB and explicit Mute; fades/gap 0–60,000 ms. Preserve speech/manual constraints. Enter or blur commits valid values; Escape cancels. Ctrl+click and a right-click action restore individual parameter defaults without erasing manual edits. Carry changed limits and mute semantics through validation, persistence, envelopes, audition and export. |
| Focused import and waveform cleanup | I09–I10 | Center the complete import setup in a compact resizable/scrollable dialog, with channel choices, naming, validation, count/limits, errors and Cancel intact. Hide the duplicate lower gain strip normally and retain it when the existing editing toggle is enabled. |
| Shortcut help and zoom | I11–I14 | Help → Keyboard shortcuts and Ctrl+/; F fits the recording; Shift+F fits selection with roughly 10% context per side and at least one second visible. Clip to recording bounds. Alt-drag zooms without labeling. Respect text-field focus and existing Escape cancellation. |
| Separate automation-editor design | I09 follow-up | Research FL Studio automation interaction; design waveform-overlay editing while retaining an explicit mode toggle. Consider pencil drawing, point reduction/smoothing, typed point dB entry, fine Shift-drag and reset-to-automatic semantics. These require their own specification; no point gesture, drawing algorithm or smoothing behavior is settled. |

The published tickets record dependencies, checks and coordination with existing GUI ticket 23. Their numbers do not imply execution order or delivery commitments. The group summaries preserve the interview reasoning; the linked accepted specifications now govern the four build scopes.

The owner invoked [ask-matt](C:/Users/User/.agents/skills/ask-matt/SKILL.md), whose suggested repository workflow is **grill-with-docs → agreed specification → dependency-linked tickets**. That workflow produced the four accepted specs and five tickets above. The following discussion order still applies to remaining proposals; synchronization, app settings/help, themes and plugins have no accepted implementation specification yet.

1. Clarify the editor workflow ideas first: I05–I14. The centered import dialog, lower-line removal, Escape behavior and shortcut reference are possible small scopes once their behavior is settled. Check existing GUI ticket 23 before opening overlapping build tickets.
2. Discuss I07–I08 together because numeric controls must agree across validation, persistence, audition and export. Decide mute semantics separately from a finite dB range.
3. I01–I04 are deferred by the synchronization decision above. Retain the research; do not advance synchronization specifications or build tickets unless the owner reopens the scope.
4. Clarify the initial app menu (I15) and theme foundation (I16), keeping optional diagnostics, custom theme imports and refresh-rate controls as separate decisions.
5. Keep I17 as a feasibility investigation until the intended host and offline/live workflow are agreed. Desktop cross-platform packages already have a separate roadmap track.

For each discussion, record the confirmed behavior and remaining questions here or in a linked decision record. Once scope is accepted, create or amend a ticket using the repository tracker conventions, with dependencies and observable acceptance checks. Only then add it to the active implementation sequence. Do not claim a build ticket for this inventory-only change.
