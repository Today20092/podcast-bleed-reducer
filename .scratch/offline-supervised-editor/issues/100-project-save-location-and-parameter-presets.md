# Let users choose project save locations and reuse parameter presets

Status: claimed
Assignee: Codex
Readiness: ready-for-agent
Type: feature
Blocked by: 02, 41, 51, 63 (all done at specification acceptance)
Related: 02, 28, 41, 51, 63

## Request and provenance — 2026-10-26

The owner requests a ticket for feedback from @BossDre05 in a YouTube reply screenshot: Save Project does not offer a choice of where to save, and built-in parameter presets plus saving custom presets would be useful. The owner replied that they would follow up when implemented.

Source: owner-attached `19030.jpg`, locally available at `.codex-remote-attachments/01a1794c-f700-7de1-bafb-7fa315e6cdc9/71ac27ad-2c04-41d7-ac7a-cbb76f6d3a77/1-19030.jpg`. This attachment may be unavailable in another checkout; this transcription preserves the actionable report. No public comment URL, application version, operating system or reproduction sequence was supplied.

The owner accepted all recommended product decisions and requested implementation on 2026-10-26. The implementation and automated acceptance are delivered below. The remaining native acceptance requires computer-use permission; the ticket stays claimed until those checks pass or the owner explicitly waives them. The reporter's original installed version remains unknown.

## Accepted scope

### Project saving

- First explicit Save Project offers a destination folder and filename. Later Save updates the selected project; Save As always offers another destination.
- Keep automatic local recovery saving. Explicit Save writes the chosen named project file; later edits update local recovery until the next explicit Save. Reopening a named file remembers it as the explicit Save destination while editing remains local. The internal working path must not silently replace that destination.
- Successful Save As makes the new named file the destination of subsequent explicit saves. Persist any required recovery/destination association in local working metadata, preserving older project compatibility; a project opened directly uses its actual opened path as its destination. A new unsaved project has no named destination even though a recovery copy exists.
- Distinguish the editable project file from automatic local working storage and exported audio. Show the current saved location and useful save success/error feedback.
- Cancellation leaves the current project, destination and dirty state intact. Failed saves preserve the previous valid file and unsaved work; overwrite handling follows the existing safe persistence contract.
- Saving elsewhere preserves source references so the project reopens correctly. It does not imply copying original media or making a self-contained media archive.

### Reusable sound presets

- Offer Default, Gentle and Strong starting points and named custom presets available across desktop projects. Presets contain only attenuation, open time, close time and minimum gap. Preserve the current mute state. Factory values below are bounded starting points, with audible acceptance still required during implementation.
- Save actual parameter values into each project so reopening remains reproducible if a preset is later changed or deleted.
- Reuse existing parameter validation, change handling, undo conventions and shared envelope evaluation. Applying sound controls must have the same audition/export and invalidation behavior as changing them individually.
- Recording-specific media, labels, trained evidence, speaker assignment, protections and manual gain edits remain project data; proposed presets do not transfer them.
- Custom presets support create, apply, update, rename and delete. Factory presets are immutable; users can save a custom copy. Trim names, reject empty names and compare names case-insensitively; factory names are reserved. Require confirmation before replacing an existing custom name or updating/deleting a custom preset. Cancelling leaves stored presets unchanged; deleting a preset leaves project values intact.
- Applying a preset is one undoable project change, including all four values. Undo restores the previous values; changing a preset definition does not apply it to projects automatically. Indicate modified preset values without silently overwriting the stored preset. Applying the same four values is a no-op.
- Store a small versioned custom-preset collection in existing local application settings storage. Validate all four fields before application, report corrupt/unsupported data, and preserve existing stored data if a write fails. Preset sharing/import/export, cloud sync and plugin preset systems are outside this scope.

## Maintainable implementation plan

Codebase longevity is an acceptance requirement, following `AGENTS.md`'s lifetime-cost rule. Optimize for understanding, testing and safely extending the behavior. Keep current functionality complete without building a generic preset framework for hypothetical controls.

### Ownership and shared behavior

- Give the four preset-eligible values one explicit typed model, separate from the complete `AutomixParameters` and from preset name/storage metadata. Reuse a suitable existing sound-settings type if its semantics match; otherwise introduce the smallest focused type. Avoid arbitrary string-to-value maps, duplicated parameter lists in GUI handlers, or serializing unrelated engine fields.
- Centralize conversion from current project values, validation, equality/modified-state comparison and application of those values. Factory and custom presets use that same model and application path. Keep labels, units and bounds with existing sound-control definitions where practical, rather than creating a competing source of truth.
- Individual sound edits and whole-preset application converge on the same project-owned parameter update logic, so envelope refresh, dirty state, invalidation and undo cannot drift between entry points. Preserve the accepted difference that a preset leaves mute unchanged while the existing individual attenuation gesture clears it. Express that policy explicitly at the caller; do not change established gesture behavior accidentally.
- Validate a complete candidate before committing any project mutation. One preset application creates one undo record and one logical envelope update, with no intermediate partially applied state. GUI handlers present controls and dispatch actions; parameter semantics, preset persistence and migration must be testable without constructing the GUI.
- Keep the preset collection's storage/version/name rules together in a focused module using existing safe settings-write patterns. Keep project save-destination/recovery ownership in project/storage code, separate from preset storage. Reuse existing modules and registered actions; add files only when they establish a clear responsibility, not to satisfy a prescribed file count.

### Adding a future control

1. Decide explicitly whether the control is reusable sound configuration or recording-specific data. New engine fields do not automatically become preset fields.
2. For an eligible control, extend the typed sound model and its shared validation/application/conversion together. Update factory definitions, the existing control's UI metadata and focused behavior tests. The compiler should help identify incomplete typed construction; menu, preset CRUD and storage orchestration should not need another parameter-specific branch.
3. Define compatibility before writing the new field: loading a supported older preset applies only the fields that version specified and leaves a newly introduced control at the project's current value. Current factory presets explicitly specify all eligible controls. Keep version decoding/normalization at the storage seam; callers should not know about historical formats. Reject unsupported newer versions with actionable feedback and preserve their stored bytes.
4. Keep project snapshots independent of preset identity. New sound fields need an explicit project-load default or migration under the existing project contract; changing factory defaults must not alter previously saved project sound.
5. Add a fixture from the previous preset version when the schema actually evolves, covering unchanged newer controls, complete validation, undo and save/reopen. Do not introduce fake future controls, extensibility plugins or a dynamic registry now.

### Execution order and review

1. Trace existing parameter updates, undo, settings persistence and project autosave/open paths before editing. Record the concrete callers and compatibility assumptions in this ticket; inspect current library documentation only when library-specific changes require it.
2. Establish or reuse the typed sound model and shared project update path; verify current individual control behavior, mute and envelope parity remain intact.
3. Implement versioned factory/custom presets and focused persistence/application tests, then wire the GUI to that behavior.
4. Implement named Save / Save As through existing project/storage persistence, retaining independent local recovery and safe source references. Verify its lifecycle and failure cases separately from presets.
5. Run the scoped integration/GUI checks below and review the diff for duplicate validation, parameter mappings and business logic in `src/app.rs`. Record a short extension guide with the implemented model/module names and files to change when adding an eligible control. Refactor within scope where needed; record unrelated cleanup separately.

## Factory starting points

| Preset | Attenuation | Open | Close | Minimum gap | Description |
| --- | --- | --- | --- | --- | --- |
| Default | -18 dB | 50 ms | 500 ms | 768 ms | Current application parameter defaults |
| Gentle | -12 dB | 50 ms | 500 ms | 768 ms | Less reduction in automatically attenuated regions |
| Strong | -24 dB | 50 ms | 500 ms | 768 ms | More reduction in automatically attenuated regions |

Default mirrors `AutomixParameters::default()` in `src/engine.rs`; Gentle and Strong vary only attenuation to keep the distinction understandable. These are proposed numerical starting points adopted under the owner's instruction to follow recommendations, not measured quality claims. Check bounds and shared-envelope behavior in implementation; audition representative transitions before claiming sound acceptance. Preserving mute means attenuation differences are inaudible while mute is enabled; explain that state in the UI without changing it.

## Current-source findings

- `src/app.rs:9821`: Save Project directly invokes `project.save()` and reports success without a destination chooser. `src/project.rs:305` validates and atomically saves the current project path.
- `src/storage.rs:33`: imports create local application-storage projects under `Projects/session-…/CleanSlate.cleanslate`. `src/app.rs:11162` exposes the autosave location.
- `src/project.rs:324` and `src/app.rs:5566`: opening an external project creates a local editing copy. This implements ticket 51's deliberate storage separation; destination selection must account for it.
- No desktop Save As command or parameter preset implementation was found. Current Sound controls in `src/app.rs:5330` are attenuation, open time, close time and minimum gap; mute also affects attenuation. `AutomixParameters` includes additional older detector/padding fields, so persisting the entire structure as a preset would broaden scope.
- The current individual attenuation setter clears mute. Preset application must explicitly preserve mute, as accepted by the owner, while reusing the existing envelope/invalidation behavior; calling that setter unchanged would violate this ticket.
- These are source observations on 2026-10-26, not native reproduction or confirmation of the reporter's installed version. Graph discovery encountered a closed transport; bounded text discovery was used for the coverage gap.

## Acceptance checks

- [x] Product decisions are accepted and implementation dependencies 02/41/51/63 are done. Source evidence and unverified reporter-version/native-reproduction limits are recorded.
- [ ] Reproduce the save workflow in the current GUI and record the tested source/build; the reporter's unknown installed version does not block the accepted improvement.
- [ ] First-save, repeated-save and Save As workflows match the agreed behavior through menu/toolbar/shortcut entry points. Cancellation, overwrite rejection and permission/disk failures do not lose work or report success.
- [x] Reopen the project from its chosen destination and from Save As with its media references, labels, edits and processing settings intact. Preserve existing source validation and atomic persistence.
- [x] Forged recovery metadata in an external project cannot redirect explicit Save to another project. Retain recovery associations only for matching explicit markers within private local project storage.
- [x] Verify unsaved local recovery, first named Save, later local autosave, explicit named Save, Save As and reopening retain the correct distinct destinations. Failure does not switch the named destination, clear unsaved state or damage the last valid named file. Missing/read-only destinations give actionable errors while local recovery and in-memory work remain available. Native chooser cancellation and physical media remain in the unchecked matrix above/below.
- [x] Built-in presets expose the agreed validated values; custom collection reload and cross-project/reset persistence pass automated checks. Native restart interaction remains in the pending GUI matrix.
- [x] Applying a preset updates precisely the agreed fields, preserves recording-specific data, and follows existing undo and analysis-staleness rules. Invalid/corrupt custom preset data cannot partially change a project.
- [ ] Cover create/update/rename/delete, reserved and duplicate names, cancelled confirmation, restart persistence, immutable factory presets, undo/redo, no-op application and both mute states. Parameter application preserves mute even though the individual attenuation control currently clears it.
- [x] Save/reopen retains applied values independently of later preset changes/deletion. Audition and export use the unchanged shared envelope evaluator; automated parity/self-check evidence passes, without a listening claim.
- [x] Factory/custom application and individual sound edits share validation and project update behavior, with the explicit mute policy preserved. Focused tests exercise the same interface used by GUI callers; no duplicate preset-specific envelope or invalidation implementation is introduced.
- [x] Supported-version decoding, unsupported-version rejection, failed-write preservation and all-or-nothing application are tested. Document the future-field compatibility policy and implemented extension points; an added control does not require editing preset CRUD/menu/storage orchestration.
- [x] Review confirms clear ownership of sound values, preset storage and project save/recovery, with no serialization of unrelated engine state or speculative generic framework. The short extension guide names the actual types/modules rather than restating this plan.
- [ ] Complete focused regression tests and GUI checks for keyboard use, supported themes, compact layout and meaningful errors. Use `scripts/gui-dev.ps1` and applicable `scripts/check.ps1 -Local` checks when implementing; record source commit, tested executable and unperformed checks.

## Implementation notes

- `src/sound.rs` owns the typed four-value model, shared validation, factory definitions and version-1 custom collection. `Project::apply_sound` uses the same transaction as individual sound edits and records one undo entry. `src/app/presets.rs` owns dialog presentation and registered commands; project-file presentation is separate in `src/app/project_files.rs`.
- `Project::save` retains local autosaving; `save_named`/`save_as` publish independent snapshots. A matching recovery marker is trusted only within local application project storage. Named snapshots reopen at their actual selected path, including inside the storage tree; choosing the current working path rehomes recovery first. Forgeable external metadata cannot redirect saves.
- Named-file publication is the save commit. Failure before that publication preserves the previous file and destination. If only the subsequent local association write fails, the named save succeeds with an explicit recovery warning; a two-destination filesystem transaction cannot be atomic. Regression coverage includes this case.
- [Extension guide](../../../docs/agents/sound-presets.md) names the implemented interfaces and explains adding a field and its real schema migration. Updated editor workflow and action-contract documentation include Save As, preset input ownership and mute semantics.
- Both code-review axes used pre-implementation baseline `77213378ff08cf05759483093d33dd6f7709a84c`; all reported application, persistence and supporting-wrapper findings were fixed. Source re-reviews report zero remaining Standards/Spec findings.
- TDD started with a failing project sound-transaction/undo test before adding the interface. Final full `scripts/check.ps1 -Local` passed 378 tests with 20 existing ignored tests, strict all-target/all-feature Clippy, formatting and all workflow/packaging/action fixtures (`20261026-144011-ab5ead9e01874d3e9c9c9844ad31aab7.json`, peak owned job commit 2.34 GiB, verified cleanup). The subsequent changes add test assertions only: custom update/rename replacement and both mute states. Focused verification then passed 14 project tests, two preset/application tests and the private-storage/recovery test (`20261026-145933-8f2ab135475b485fb5913b1a13983c49.json`).
- Ticket-specific [verification wrapper](../ticket100-verify.ps1) runs existing scripts inside the unchanged resource guard. This MSVC installation leaves VCTIP telemetry and its console host in the job after Cargo exits; the wrapper refuses uncontained execution, verifies helper image allowlists and captured-handle job membership, and stops only those owned completed-task helpers. This temporary limitation and removal condition are documented beside the cleanup. Earlier interrupted compilation/test attempts are not passing acceptance evidence.

## Comments

2026-10-26 — Final assertion additions passed the focused suite (14 project + two preset/application + one private-storage test); final strict Clippy passed in `20261026-150519-91dc8021530d4ea1805f7c9f1ee3bad6.json`. Final executable self-check passed in `20261026-150539-2c626b4fdf44423e9aa52f82abbbd914.json`. Executable SHA256: `459a1b1cf63b5a0880d3b256731ae9630256589be5e50613db3c5366c450a89a`. Retention directory: `%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/ticket-100-459a1b1cf63b/`. Code and automated evidence are committed; native permission/acceptance remains outstanding, so this is not marked done.

2026-10-26 — Automated implementation acceptance passed. Production GUI build through `scripts/gui-dev.ps1` passed (`20261026-150252-8eec0459870b48cabaffd880ecd10d2f.json`); executable self-check passed offline inference, analysis, project save/reopen, rendering and cancellation (`20261026-144952-e5b83a3857b34135b6fa45e15126dbf6.json`). Tested source is development baseline `77213378ff08cf05759483093d33dd6f7709a84c` plus this ticket's committed patch. Executable: `%LOCALAPPDATA%/CleanSlate/BuildCache/debug/rust-app.exe` (the Codex packaged host may expose its virtualized physical path). Native launch used the script's separate quick-test development copy. Compact Dark workspace and Save As control were visible, but an unexpected “Let ChatGPT enumerate your windows” permission prompt prevented completing native chooser/preset/theme/restart checks. Owner approval/waiver was requested; neither is assumed. The owned QA process was closed with local recovery retained. No native completion, listening or physical removable-media acceptance is claimed; the unchecked requirements remain open.

2026-10-26 — Claimed for owner-requested implementation on development. Pre-agreed test seams from this specification: project parameter transactions/undo and save/reopen lifecycle; preset collection load/save/name/version/application behavior; registered GUI actions and dialog integration. Tests at these interfaces cover state/data safety, while physical input and listening require native checks. Graph transport remains unavailable; use bounded text discovery for this coverage gap.

2026-10-26 — Codex captured the screenshot request and proposed scope. Owner clarification is pending. No application code changed and no runtime acceptance is claimed.

2026-10-26 — Owner answered yes to standard Save / Save As and desktop sound presets. Follow-up questions cover local recovery versus named-save persistence, the four-control/mute boundary, and factory/custom management scope. Those remaining choices are pending; no implementation claim is made.

2026-10-26 — Owner: “Follow all recommendations.” This resolves the pending questions: local recovery plus named explicit Save; four Sound controls with mute preserved; Default/Gentle/Strong plus custom create/update/rename/delete, replacement confirmation and one-step undo. The specification above is current and supersedes earlier pending-decision comments. Codex supplied concrete bounded starting values and operational edge cases under that direction. Ticket is ready-for-agent, unassigned and unimplemented. Documentation checks only; no listening or runtime acceptance claimed.

2026-10-26 — Owner explicitly prioritizes codebase cleanliness and longevity, including adding future controls. Added an implementation sequence, typed sound-model ownership, shared validation/application, a version-compatibility policy, focused tests and an extension-guide deliverable. This applies the existing `AGENTS.md` maintainability policy; it does not expand the current four-control feature scope or implement application code.
