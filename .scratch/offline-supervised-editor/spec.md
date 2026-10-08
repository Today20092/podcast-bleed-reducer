# Offline supervised editor implementation

Follow the canonical [architecture specification](../offline-supervised-migration/spec.md) and [implementation sequence](../offline-supervised-migration/implementation.md). Decision resolutions live in the [wayfinder map](../offline-supervised-migration/map.md).

This effort implements the plan. It does not create a new repository, preserve old saved-project compatibility, or treat the existing detector as quality-accepted. Start from ownership `6a60cc2`; keep all original sources and repository records intact.

## Ticket convention

Each issue below contains its scope, affected modules, dependencies and acceptance checks. Before implementation, confirm dependency issues are complete and claim the issue. The owner confirmed the architecture and implementation plan on 2026-10-08. Status `ready-for-agent` indicates specification readiness; dependencies still govern when work can start. Use the repository's canonical triage labels.
