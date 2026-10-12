# Semantic colors and dark/light appearance

Status: accepted specification, 2026-10-12
Idea: I16 initial scope
Build ticket: [37](../../offline-supervised-editor/issues/37-semantic-dark-light-themes.md)
Decision record: [appearance interview](../../../docs/roadmap-ideas.md#settingshelp-and-appearance-interview-2026-10-11)

## Problem Statement

Colors need consistent semantic roles before appearance customization can be reliable. Users also need a readable light appearance and a persistent application-wide choice.

## Solution

Establish semantic color tokens across the application, preserve the current dark appearance and add Light. Settings offers Dark / Light, applies changes immediately and remembers them across restarts.

## User Stories

1. As an editor, I want Dark or Light selectable from Settings.
2. As an editor, I want the choice applied immediately without reopening my project.
3. As an editor, I want appearance remembered across restarts and project changes.
4. As an editor, I want the existing dark appearance retained as the initial default.
5. As an editor, I want waveforms, gain, selections and warnings to keep consistent meaning in either theme.
6. As a keyboard user, I want visible focus, readable text and usable control states in both themes.
7. As a contributor, I want colors expressed by their purpose so future themes do not require editing scattered constants.

## Implementation Decisions

- Use the existing Iced styling/palette structure where possible. Define semantic roles for surfaces, text, borders, actions, focus, selected/disabled states, errors/warnings and timeline waveform/gain/selection/attenuation/protection roles. Replace application-owned literal UI colors throughout widgets and custom drawing with these roles; opaque assets/platform-owned native dialogs are not recolored.
- Preserve current dark visual intent. Light must retain semantic distinctions and legibility rather than mechanically inverting colors. Keep gain/selection/attenuation meanings intact; text, focus and control states remain distinguishable without relying solely on hue.
- Settings offers only Dark and Light. Dark is the initial default. Apply selection immediately across workspace, menus, dialogs, tooltips, loading/error states and timeline drawing without changing audio, project data, selection or playback. Invalidate drawing caches only as required for colors.
- Persist appearance using the existing application preference mechanism or the smallest app-local validated preference record if none exists. Project opening/new-project/save/reload does not replace it. Missing preference uses Dark; malformed/unsupported preference safely falls back to Dark with existing appropriate error reporting. Save failure keeps the app usable and exposes that persistence failed; no silent claim of a saved choice.
- Sound parameters, labels, gain edits, source identities and export settings keep their project scope. Theme changes never trigger analysis, source decoding or render/audio invalidation.
- Keep token definitions internal for this scope. Do not add a theme file format, arbitrary color editor, import/export mechanism or dependency solely to support future customization.

## Testing Decisions

Use app-preference and theme-selection behavioral tests: default, reload, invalid/missing data, persistence failure, project switching and immediate application. Review token coverage in widgets/custom timeline drawing and verify unchanged audio/project state after switching. Measure text/focus/control contrast and visually review both themes in narrow/wide, selected/disabled, stale/error/loading, import/export and manual-gain-edit states. Use the existing GUI development workflow and record unavailable states instead of claiming them checked.

## Out of Scope

OS-theme matching, custom colors, theme-package import, refresh-rate controls, a whole new visual style or audio/analysis changes.

## Further Notes

Owner confirmed this scope on 2026-10-12. Coordinate with ongoing GUI polish and the waveform visibility/shortcut work; this ticket changes colors and appearance preference, not those interactions.
