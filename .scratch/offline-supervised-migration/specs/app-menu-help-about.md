# App menu, Help and About

Status: accepted specification, 2026-10-12
Idea: I15 initial scope
Build ticket: [36](../../offline-supervised-editor/issues/36-app-menu-help-about.md)
Decision record: [settings/help interview](../../../docs/roadmap-ideas.md#settingshelp-and-appearance-interview-2026-10-11)

## Problem Statement

Users need a discoverable place for application settings, help, shortcuts, project links, bug reporting and version information.

## Solution

A clearly labeled Menu button in the top bar offers Settings, Keyboard shortcuts, Help, GitHub, Report a bug and About. Settings and About use centered in-app dialogs with the waveform workspace visible behind them.

## User Stories

1. As an editor, I want one discoverable app menu so I can find supporting actions.
2. As an editor, I want to open the existing keyboard reference from this menu.
3. As an editor, I want Help to open the current usage documentation.
4. As an editor, I want GitHub and Report a bug to open the repository and issue chooser.
5. As an editor, I want About to show version, license and credits/references.
6. As an editor, I want to copy version information without including private project details.
7. As an editor, I want dialogs accessible by keyboard and dismissible with Escape.

## Implementation Decisions

- Reuse current GUI menu/dialog and external-link mechanisms. The top-bar Menu replaces any redundant entry for the same shortcut reference while preserving the Help → Keyboard shortcuts route and Ctrl+/ behavior from ticket 34. Keep names and destinations centralized through existing patterns.
- Settings provides the entry point used by appearance ticket 37. Before that ticket lands, show a concise unavailable/pending appearance state rather than nonfunctional toggles. Do not invent settings just to populate the dialog.
- About displays application version, build identifier when actually available, license and credits/references derived from the repository and packaged software. Missing build identity is omitted or identified as unavailable; never invent it. Keep longer credits scrollable and readable.
- Copy version info uses an explicit allowlist of app version, available build identifier and platform details. Exclude audio, transcripts, source paths, project names, usernames, environment-variable dumps and diagnostic logs. Show clipboard success or actionable failure without collecting extra information.
- Help opens the repository's existing usage documentation. GitHub opens the project repository; Report a bug opens its existing issue chooser. Use verified repository destinations and explain link-opening failures. Opening a link is user initiated; no background network request or issue submission is added.
- Center Settings/About using existing modal sizing, focus, scrolling and Escape conventions. Retain workspace context, suppress background edits/shortcuts during modal interaction and restore focus on dismissal. Preserve unsaved projects, existing transport and validation behavior.
- Interface preferences are app-wide; sound settings remain project-local. Preference persistence and actual appearance controls belong to ticket 37.

## Testing Decisions

Use existing app-event tests for menu routing, reference reuse, focus/Escape and modal suppression. Check URL destinations without requiring network access, allowlisted copied text with and without build identity, clipboard/link failures and long credits. Verify version/license/reference information against the repository. Review menu and centered dialogs in narrow/wide development GUI views using the GUI development workflow.

## Out of Scope

Diagnostic reports, automatic issue submission, refresh-rate controls, new help-writing work, updater behavior and sound-setting changes.

## Further Notes

Owner confirmed this scope on 2026-10-12. Report-a-bug remains a browser link the user reviews and submits themselves. No implementation or release qualification is implied by this specification.
