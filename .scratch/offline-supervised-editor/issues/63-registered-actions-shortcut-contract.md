# Register user actions once for buttons and customizable shortcuts

Status: done
Assignee: Codex
Type: task
Blocked by: None
Labels: ready-for-agent

## Owner request

2026-10-22: future buttons/actions must be available for customizable keyboard shortcuts without repeated requests. The owner explicitly requested one separate ticket for planning and delivering this contract. Catch up's immediate shortcut is handled in ticket 42, independently of this future infrastructure.

## What to build

Register a user action once so its button, keyboard activation, Settings entry, shortcut reference and effective-binding hint share the same identity and behavior. New actions must appear in shortcut customization even when they have no default key. A missing default key must not make an action undiscoverable or unbindable.

Extend the existing command definitions, binding persistence, context conflict checks and shared dispatch. First prove the contract with timeline follow actions; retain existing command identifiers and saved overrides. Prefer a small typed registration/declaration mechanism over a second action registry or runtime button discovery. Parameterized controls must use the same action identity with explicit context/arguments rather than inventing arbitrary global shortcuts for each transient item.

## Acceptance criteria

- [x] Adding one action declaration exposes its name/context and optional default binding in Settings/reference, uses the shared action handler for mouse and keyboard, and supplies effective-binding hints. Demonstrate this with timeline follow actions end to end.
- [x] An action with no default binding appears as Unbound and can be assigned, invoked, reassigned, cleared and restored. Overrides survive restart; old shortcut identifiers/preferences retain their behavior and unknown entries recover safely.
- [x] Define an enforceable rule for actionable controls: registered action or an explicit, documented exemption for text/value editing, transient dialog navigation or a context-dependent action that cannot be invoked safely. Audit existing controls, migrate the timeline slice and record any remaining legacy exemptions individually; do not silently exempt all future controls.
- [x] A compile-time contract where practical, plus a focused check in the normal local validation path, rejects an unregistered new command/button or an incomplete definition/handler/hint mapping. Demonstrate the guard failing for a deliberately omitted registration, then passing when registered. Avoid source-text heuristics as the sole guarantee.
- [x] Disabled/unavailable actions obey the same availability and context rules from mouse and keyboard. Typing, popups, capture mode, repeated key presses and conflicts retain their existing safeguards. Parameterized actions cannot activate stale or unrelated items.
- [x] Bound and initially unbound action tests cover registration, UI enumeration, remapping/persistence, hints and dispatch; standard GUI build/local checks passed. The owner accepted the development UI and explicitly waived further computer control in place of the remaining native Settings/button automation, as recorded below.

## Boundaries

No macros, multi-step key sequences, automatic allocation of arbitrary default chords, cloud settings or playback-engine changes. Do not perform an unrelated full UI rewrite. This ticket is one complete registration/enforcement slice; any remaining migration must be explicitly inventoried with follow-up scope rather than claimed complete.

## Closure — 2026-10-22

The owner approved the development build and explicitly requested closing this ticket and merging into development without further computer control. The [combined receipt](../../../docs/research/menu-actions-integration-2026-10-22.md) records 356 passing combined tests (20 ignored), strict Clippy, the standard GUI build and final typed-action guard. The three final timeline regressions passed, including initially-unbound discovery, remapping, persistence and shared modal availability. The review's one guard-fixture finding was fixed and re-reviewed. Native Settings remap/restart automation was not completed; the owner accepts closure using the implemented contract, software evidence and their development review. This supersedes pending integration statements below without claiming unperformed physical checks. All software acceptance criteria are complete; remaining native qualification routes to the existing recurring editor verification procedure.

## Results

Owner-approved seams: the ticket explicitly requires bound/unbound command registration, UI enumeration, remapping, persisted preferences, effective hints and shared application dispatch; the accepted specification retains the owner-confirmed application behavior boundary. Test these through the existing command/preferences/application interfaces. The typed button activation boundary is the separately required compile-time guard. Native Settings/button behavior is verified separately; these software checks do not claim physical-input acceptance.

Implemented on `codex/ticket-63-actions` from integration base `408848c`. A single typed `commands!` declaration generates the stable command enum and metadata; the exhaustive application dispatch requires every handler. `action_control` supplies shared activation, current availability and effective hints together. Center, Lock, Catch up and Meters use it; Meters is initially Unbound and appears in the existing Settings/reference enumeration. Existing identifiers, defaults, preference recovery and context conflict checks are retained.

The shared button activation methods now require `ButtonAction`, so a raw new `Message` cannot become a button command. The registered adapter is constructed by `action_control`. A closed `LegacyButton` enum individually inventories the remaining existing exceptions with argument types, mappings and reasons; there is no blanket implementation for application messages. The [developer contract and exception/migration inventory](../../../docs/agents/actions-and-shortcuts.md) is linked at the top of the required GUI guide. Existing parameterized exceptions retain their validation and input ownership; this slice does not invent global bindings for transient items or claim their complete registry migration.

TDD/software evidence: the new initially-unbound application/preferences test first failed with E0599 for the missing Meters registration/activation/hint path, then passed. Final selected checks passed 3 `timeline_` tests (including initially-unbound remap/reassign/save/reload/clear/restore and modal availability) and 7 shortcut/input tests (including bound Catch up shared button/key dispatch, old preferences and conflicts). The actual typed activation boundary's small rustc fixture rejected omitted registration with E0277, then accepted explicit registration. `scripts/test-action-contract.ps1` is in the normal check path; formatting and whitespace checks passed. No dependency, audio-engine or host configuration change was made.

Combined integration checks, standard GUI build and native Settings/button verification remain for the integration owner before marking this ticket done. No physical keyboard or native GUI check is claimed by these software results.

Merged integration tip `67bc74b` (menu 60) into this branch at `4a66578`; the expected menu-view conflict preserves the new layout and attaches its audited typed adapter. All 10 menu/application-navigation tests passed on that combined source, including the actual-widget outside-click regression. A final trivial delegation then makes keyboard menu routing reuse the same typed menu mapping as pointer activation; that final source awaits the integration owner's combined compilation/checks rather than claiming the preceding test executable covered it.

Review fix: after merging reviewed integration `2ab1c74`, removed the compile fixture's duplicate `activate` wrapper and called the production `contract::message(NewAction)` API directly. The ordinary script still passed omitted-registration E0277 and registered compilation. A separate temporary copy deliberately weakened the production helper by removing its trait bound; the revised guard correctly failed with “Unregistered action unexpectedly passed.” The unchanged production helper passed again. Temporary copied files were removed through exact file patches after automatic review rejected the shell cleanup command; empty temporary directories may remain. Application Rust and Cargo outputs were unchanged by this fix.
