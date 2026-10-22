# Register user actions once for buttons and customizable shortcuts

Status: ready-for-agent
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

Planning only. The existing command system already derives Settings/reference and binding conflict behavior from command definitions; the gap is that independently created UI messages/buttons can bypass that catalog. The future contract must close that gap, not merely add a reminder to ask for shortcuts.
