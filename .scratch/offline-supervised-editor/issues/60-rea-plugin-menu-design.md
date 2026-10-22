# Research plugin menus with REA and redesign the application menu

Status: ready-for-agent
Type: task
Assignee: unassigned
Blocked by: none
Related: 36, 57, 58, 59

## Owner request

2026-10-22: the Menu panel uses inconsistent button widths and feels unfinished. Use REA to understand menu and interface design in FabFilter plugins, Melda plugins, TDR Nova and TDR Prism, then create a better CleanSlate menu. This ticket records future work; the current transport/header cleanup does not implement this menu redesign.

## Research

- Read the reverse-engineer-anything skill and prior FabFilter research before using REA. Discover locally available artifacts and record exact plugin names, versions, paths and inspection method. Report unavailable targets explicitly; do not claim runtime interaction from static evidence.
- Inspect at least one available FabFilter and Melda plugin, plus TDR Nova and TDR Prism where available. Study menu entry alignment, row size, grouping/separators, typography, shortcut hints, hover/focus/disabled states, settings submenus, help/about placement, opening, dismissal and keyboard navigation.
- Separate observed behavior, inferred implementation and CleanSlate recommendations. Record supporting evidence and identify reusable design principles rather than copying proprietary artwork or code.

## Implementation scope

- Apply the findings within the existing Rust/Iced menu and house styling. Replace the uneven stack of action-sized buttons with a coherent menu layout; select rows, groups and submenus from evidence and current needs.
- Preserve Settings, Keyboard shortcuts, Help, GitHub, Report a bug, About and all existing routes. Preserve effective shortcut hints, keyboard focus, Enter activation, Escape/outside dismissal and modal input ownership.
- Retain current project state and audio behavior. Do not migrate toolkits, add unrelated controls, acquire licenses or install software as part of this ticket.

## Acceptance

- Commit a concise REA findings/design record with evidence for each available target and explicit gaps for unavailable plugins. Explain how findings inform the chosen menu.
- Implement and inspect the menu in context at compact/wide sizes and Dark/Light appearances, including keyboard navigation and disabled states. Check long text and enlarged text where supported.
- Run relevant menu routing/focus regression tests, scripts/check.ps1 -Local and scripts/gui-dev.ps1. Record source/executable identity, checks, screenshots and remaining limitations before marking done.
- Owner review determines whether the resulting menu design is accepted; no release publication is authorized.
