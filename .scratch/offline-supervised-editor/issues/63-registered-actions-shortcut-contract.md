# Register user actions once for buttons and customizable shortcuts

Status: claimed
Assignee: actions_63
Type: task
Blocked by: None
Labels: ready-for-agent

## Owner request

2026-10-22: future buttons/actions must be available for customizable keyboard shortcuts without repeated requests. The owner explicitly requested one separate ticket for planning and delivering this contract. Catch up's immediate shortcut is handled in ticket 42, independently of this future infrastructure.

## What to build

Register a user action once so its button, keyboard activation, Settings entry, shortcut reference and effective-binding hint share the same identity and behavior. New actions must appear in shortcut customization even when they have no default key. A missing default key must not make an action undiscoverable or unbindable.

Extend the existing command definitions, binding persistence, context conflict checks and shared dispatch. First prove the contract with timeline follow actions; retain existing command identifiers and saved overrides. Prefer a small typed registration/declaration mechanism over a second action registry or runtime button discovery. Parameterized controls must use the same action identity with explicit context/arguments rather than inventing arbitrary global shortcuts for each transient item.

## Acceptance criteria

- [ ] Adding one action declaration exposes its name/context and optional default binding in Settings/reference, uses the shared action handler for mouse and keyboard, and supplies effective-binding hints. Demonstrate this with timeline follow actions end to end.
- [ ] An action with no default binding appears as Unbound and can be assigned, invoked, reassigned, cleared and restored. Overrides survive restart; old shortcut identifiers/preferences retain their behavior and unknown entries recover safely.
- [ ] Define an enforceable rule for actionable controls: registered action or an explicit, documented exemption for text/value editing, transient dialog navigation or a context-dependent action that cannot be invoked safely. Audit existing controls, migrate the timeline slice and record any remaining legacy exemptions individually; do not silently exempt all future controls.
- [ ] A compile-time contract where practical, plus a focused check in the normal local validation path, rejects an unregistered new command/button or an incomplete definition/handler/hint mapping. Demonstrate the guard failing for a deliberately omitted registration, then passing when registered. Avoid source-text heuristics as the sole guarantee.
- [ ] Disabled/unavailable actions obey the same availability and context rules from mouse and keyboard. Typing, popups, capture mode, repeated key presses and conflicts retain their existing safeguards. Parameterized actions cannot activate stale or unrelated items.
- [ ] Test one bound action and one initially unbound action through registration, UI enumeration, remapping/persistence, hints and dispatch. Use the standard GUI build/local checks and record a native Settings/button verification before completion.

## Boundaries

No macros, multi-step key sequences, automatic allocation of arbitrary default chords, cloud settings or playback-engine changes. Do not perform an unrelated full UI rewrite. This ticket is one complete registration/enforcement slice; any remaining migration must be explicitly inventoried with follow-up scope rather than claimed complete.

## Results

Owner-approved seams: the ticket explicitly requires bound/unbound command registration, UI enumeration, remapping, persisted preferences, effective hints and shared application dispatch; the accepted specification retains the owner-confirmed application behavior boundary. Test these through the existing command/preferences/application interfaces. The typed button activation boundary is the separately required compile-time guard. Native Settings/button behavior is verified separately; these software checks do not claim physical-input acceptance.

Implemented on `codex/ticket-63-actions` from integration base `408848c`. A single typed `commands!` declaration generates the stable command enum and metadata; the exhaustive application dispatch requires every handler. `action_control` supplies shared activation, current availability and effective hints together. Center, Lock, Catch up and Meters use it; Meters is initially Unbound and appears in the existing Settings/reference enumeration. Existing identifiers, defaults, preference recovery and context conflict checks are retained.

The shared button activation methods now require `ButtonAction`, so a raw new `Message` cannot become a button command. The registered adapter is constructed by `action_control`. A closed `LegacyButton` enum individually inventories the remaining existing exceptions with argument types, mappings and reasons; there is no blanket implementation for application messages. The [developer contract and exception/migration inventory](../../../docs/agents/actions-and-shortcuts.md) is linked at the top of the required GUI guide. Existing parameterized exceptions retain their validation and input ownership; this slice does not invent global bindings for transient items or claim their complete registry migration.

TDD/software evidence: the new initially-unbound application/preferences test first failed with E0599 for the missing Meters registration/activation/hint path, then passed. Final selected checks passed 3 `timeline_` tests (including initially-unbound remap/reassign/save/reload/clear/restore and modal availability) and 7 shortcut/input tests (including bound Catch up shared button/key dispatch, old preferences and conflicts). The actual typed activation boundary's small rustc fixture rejected omitted registration with E0277, then accepted explicit registration. `scripts/test-action-contract.ps1` is in the normal check path; formatting and whitespace checks passed. No dependency, audio-engine or host configuration change was made.

Combined integration checks, standard GUI build and native Settings/button verification remain for the integration owner before marking this ticket done. No physical keyboard or native GUI check is claimed by these software results.
