# CleanSlate

## Active work

Before planning or implementation, read [.scratch/offline-supervised-migration/map.md](.scratch/offline-supervised-migration/map.md) and the [accepted specification](.scratch/offline-supervised-migration/spec.md). This is the sole active roadmap. The implementation sequence and dependencies live in [.scratch/offline-supervised-migration/implementation.md](.scratch/offline-supervised-migration/implementation.md); build tickets live in `.scratch/offline-supervised-editor/issues/`.

Choose an unblocked implementation ticket, claim it before editing, and complete its scoped acceptance checks before marking it done. `ready-for-agent` describes specification readiness; every `Blocked by` dependency must be done first. Record results in the ticket. See [tracker conventions](docs/agents/issue-tracker.md) and [triage labels](docs/agents/triage-labels.md) when updating issues.

For ticket inventories, use context-mode to extract only status, dependencies, and scope before returning results.

For GUI changes, read [the GUI development guide](docs/gui-development.md) and use [scripts/gui-dev.ps1](scripts/gui-dev.ps1).

## Architecture and discovery

Read [CONTEXT.md](CONTEXT.md) and [ADR-0006](docs/adr/0006-offline-supervised-editable-envelopes.md) before changing domain behavior. Consult the [active implementation tracker](.scratch/offline-supervised-migration/implementation.md) for current implementation status. Preserve source validation, cancellation, safe exports and shared envelope evaluation while migrating.

Discover code through the graph: search symbols using supported filters, trace their callers/callees, then fetch snippets by qualified symbol. Index this checkout if coverage is absent or reflects an old branch/worktree. Use text search for literals, non-code records or graph gaps.

When discovering tools, retrieve only the metadata needed for the task. Before selecting or changing dependencies, fetch current library/API documentation through Context7.

## Repository

Use `main` as the shared starting point. The offline supervised development history was merged into `main` on 2026-10-11. Create scoped `codex/` branches when needed. Use commit `6a60cc2` as the frozen detector comparison baseline when the acceptance tickets require it.
