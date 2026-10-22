# Research plugin menus with REA and redesign the application menu

Status: claimed
Type: task
Assignee: menu_60
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

## Implementation record — 2026-10-22

Claimed on `codex/ticket-60-menu`, verified base `408848c1cfe68358ded807aeb9dea6c76582008c` from `codex/menu-actions-integration`. [REA findings and design record](../../../docs/research/plugin-menu-design-2026-10-22.md) records exact installed identities, evidence IDs, manufacturer menu illustrations and limitations. Pro-L 2 and MAGC wrapper string inspection completed; the Melda kernel and Prism have static resource inventories. Prism's deep string analysis hit its 330-second provider startup deadline; Nova was unavailable in the bounded installed-directory search. No plugin runtime behavior is claimed.

The menu uses equal-width quiet rows, grouped separators and a distinct effective-binding column. The panel fits its contents with bounded scrolling. All seven ordered routes remain, with unchanged Tab/Up/Down/Enter/Escape ownership. An outside click now dismisses the menu without reaching the waveform workspace; Settings/About retain their existing dialog behavior. User documentation describes the interaction.

TDD uses the owner-confirmed app command/modal boundary. An actual Iced view mouse event at the scrim first failed because no dismissal message was emitted; the implementation supplies only CloseHelp and preserves selection. Existing routing/focus regression tests remain authoritative. Combined checks, executable identity and compact/wide Dark/Light native inspection are owned by integration after both tickets merge. Owner design acceptance remains pending; this ticket is not marked done by the implementer.

Focused checks: genuine RED (1 failed, 0 passed), then GREEN (1 passed), all 8 `app_menu_tests`, plus the existing Help keyboard-ownership and Tab/menu-ownership tests passed through the shared local-build configuration. Formatting and `git diff --check` passed; Impeccable's mechanical detector returned no findings for `src/app.rs` (not a substitute for native visual inspection). No full Cargo checks or GUI rebuild were duplicated in this worktree; integration coordinates those checks. The shared-target fresh artifact initially yielded zero matching tests after another worktree's build; refreshing this worktree's changed source timestamp forced its actual compilation before the recorded RED/GREEN runs.
