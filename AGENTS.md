# CleanSlate

## Active work

Before planning or implementation, read [.scratch/offline-supervised-migration/map.md](.scratch/offline-supervised-migration/map.md) and the [accepted specification](.scratch/offline-supervised-migration/spec.md). This is the sole active roadmap. The implementation sequence and dependencies live in [.scratch/offline-supervised-migration/implementation.md](.scratch/offline-supervised-migration/implementation.md); build tickets live in `.scratch/offline-supervised-editor/issues/`.

Choose an unblocked implementation ticket, claim it before editing, and complete its scoped acceptance checks before marking it done. `ready-for-agent` describes specification readiness; every `Blocked by` dependency must be done first. Record results in the ticket. See [tracker conventions](docs/agents/issue-tracker.md) and [triage labels](docs/agents/triage-labels.md) when updating issues.

## Architecture and discovery

Read [CONTEXT.md](CONTEXT.md) and [ADR-0006](docs/adr/0006-offline-supervised-editable-envelopes.md) before changing domain behavior. The accepted architecture is the target; the retained Rust/Iced implementation has not yet acquired the new analysis and editing capabilities. Preserve source validation, cancellation, safe exports and shared envelope evaluation while migrating.

Discover code through the graph: search symbols, trace their callers/callees, then fetch their snippets. Index this checkout if coverage is absent or reflects an old branch/worktree. Use text search for literals, non-code records or graph gaps. Fetch current library/API documentation through Context7 before selecting or changing dependencies.

## Repository

Work on `codex/offline-supervised`. `main` is the clean shared starting point. Superseded branches, worktrees, roadmap folders and packaged builds have been removed from the active workspace; their recovery snapshot is outside the project. Use commit `6a60cc2` as the frozen detector comparison baseline when the acceptance tickets require it.
