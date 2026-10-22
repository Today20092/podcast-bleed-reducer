# Refresh the release README and user guide

Status: claimed
Type: task
Assignee: Codex user-docs
Blocked by: None
Labels: ready-for-agent

## Scope

2026-10-22 owner authorized release preparation and parallel documentation work. Refresh the README and user-facing editor/help guide for the completed development interface and upcoming alpha.7 release. Explain the timeline follow modes, customizable shortcuts, waveform view choices, dial gestures, concise status/details and current performance behavior. Preserve source/export safety, privacy and experimental limitations. Keep public download links on the published version until publication.

## Acceptance

- [x] Current controls and default bindings match implementation/documented checks.
- [ ] Replace branch-only wording and stale screenshot descriptions with current behavior and new native captures supplied by the release coordinator.
- [x] README retains installation, usage, disclosure, support and limitations without duplicating the detailed guide.
- [ ] Validate edited Markdown links/anchors/images, wording and whitespace. Record unavailable rendered/physical checks.

## Results

Claimed before edits on codex/release-user-docs from development 6160da9. Documentation-only work; builds and screenshots are coordinated by the parent release task.

Updated README and editor guide for the integrated alpha.7 candidate, replacing branch-only descriptions. Added Catch up/Lock/Meters behavior, independent waveform visibility, rotary gestures/indicator smoothing, Ready/Details, shortcut remapping and common defaults. Corrected the export-folder description to match the current parent-location workflow. Preserved the published alpha.6 download table until publication, AI-development disclosure, read-only originals, export safeguards and experimental/qualification limits. README links to the authoritative release notes rather than repeating benchmark numbers. The ownership explanation uses diagrams rather than stale native screenshots, so it needs no screenshot replacement.

Checks: 56 local Markdown links/image references and heading anchors checked in the two edited documents. All existing paths/anchors pass; five agreed alpha7 image assets remain for coordinator integration. Five existing Shields badge endpoints returned HTTP 200 with SVG content. Whitespace check passed and the unslop wording pass is complete. No application code or Cargo build was changed/run. Parent owns native capture/render inspection and final asset-link verification; keep status claimed until those checks are recorded.
