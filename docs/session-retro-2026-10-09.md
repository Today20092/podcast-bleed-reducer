# Session retrospective — 2026-10-09

Historical record of that session's checkout and candidates. Its CI, GUI and tooling observations are not current architecture requirements. Use the [developer guide](development.md) and [current architecture decision](adr/0006-offline-supervised-editable-envelopes.md) for current behavior.

Scope: ticket inventory, faster GUI iteration, and uniform red attenuation shading. These are environment improvement candidates, not additional implementation tickets or changes to the accepted roadmap.

| Priority | Finding and evidence | Smallest useful improvement |
| --- | --- | --- |
| High | Existing checks have no automatic gate. README documents tests, formatting, and strict Clippy, but this checkout has no CI workflow or active Git hook. Clippy caught the test module placed before later items during this session. | Wire the existing commands into CI. Use a quick formatting hook if local enforcement is wanted. Keep the Clippy rule; do not duplicate it in prose or invent another lint. |
| Medium | The fast GUI build settings were difficult to discover. Two build attempts used the wrong target/profile before the retained cache and package-specific settings were found. AGENTS has no pointer to the new GUI development guide. | Add one conditional pointer: “For GUI changes, read docs/gui-development.md and use scripts/gui-dev.ps1.” Keep commands and cache details in that guide as the single source. |
| Medium | AGENTS describes the implementation as not yet having the new analysis/editing capabilities, although implementation tickets 01–13 are done. That can misdirect future planning. | Replace the stale status sentence with a pointer to the active implementation tracker. Keep the architecture constraints; avoid copying changing ticket status into AGENTS. |
| Medium | Native automation initially selected a 16×16 icon capture instead of the main window. Some observations immediately after clicks were stale, and an automated drag produced seeking rather than a selection. | Select the main window capture using its returned dimensions, then refresh and observe the resulting state after input. Keep actual mouse selection as an explicit owner acceptance check until it is observed successfully. |
| Low | Early discovery surfaced a large ticket batch and tool descriptions when only a small inventory was needed. An unsuitable graph file pattern also caused an empty search. | Derive status, dependencies, and scope inside context-mode before returning output. Discover only the needed tool metadata; use graph-supported filters and qualified symbols. |

## What worked

The GUI development script now preserves the dependency cache, enables incremental compilation for local code, opens a separate saved development project, and refuses to overwrite a running executable. Measured iteration fell to 9.82 seconds for a small GUI edit and 0.80 seconds for a no-change build. A first incremental build took 31.59 seconds; these are observed timings, not guarantees.

The design questions settled the important distinction: red shading represents actual reduced gain, including fades and manual edits, rather than a speech classification. Uniform shading follows the shared effective envelope and stays beneath the waveform and overlays. Native observations confirmed shading across audition modes. The change passed 84 tests with six documented opt-ins ignored, strict Clippy, formatting, and diff checks.

The saved-project workflow and running-executable guard protected edits. The broader GUI ticket stayed claimed because owner acceptance remains incomplete.

## Keep the limits visible

Do not describe this workflow as hot reload. It is a cached incremental build and restart; the current shared-library GUI structure makes the experimental binary-root hot reload approach a poor immediate fit.

Successful controller tests and drawing order do not establish physical mouse selection behavior. Actual mouse selection, listening approval, narrow-window/enlarged-text checks, and continuous interaction latency remain pending. Background removal and subsequent red-shading feedback were recorded as a specification refinement rather than silently restoring earlier uncertainty decorations.

No hooks, CI configuration, dependencies, or agent instructions were changed by this retrospective. The existing AGENTS file is short; it needs accurate navigation, not more standing rules.
