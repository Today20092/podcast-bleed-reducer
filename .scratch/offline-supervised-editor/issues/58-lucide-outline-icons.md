# Adopt Lucide outlines in the native GUI

Status: claimed
Type: task
Assignee: Codex
Blocked by: 57
Related: 23

## Owner request

The owner selected Lucide outlines after reviewing the GUI screenshots. Continue on `codex/fabfilter-controls`; preserve development and the prior tested builds. The owner clarified that shadcn was only a compatibility question and is not requested.

## Scope and acceptance

- Replace the custom Lock/Center/Meters artwork with bundled Lucide outline assets and add matching Previous/Play/Pause/Stop transport artwork. Keep labels, tooltips, focus/selection, meter placement feedback and all command behavior.
- Use the existing Iced native SVG widget with only the required SVG feature; pin the upstream asset revision and retain complete license/provenance in source, the application and packaged notices. No web UI, shadcn, network icon loading or processing changes.
- Verify Dark/Light and compact/wide rendering, including disabled/primary icon contrast, run appropriate standard local checks and the GUI build, and preserve a verified executable and mobile-readable screenshots.

## Results

In progress. Code graph transport is unavailable; focused source searches are the documented fallback. Context7 fetched Lucide static-asset conventions, Iced 0.14 drawing/SVG documentation and shadcn requirements. No shadcn integration is planned.
