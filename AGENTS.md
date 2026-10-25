# CleanSlate

## Product workspace scope

This is the `codex/plugin-smart-gate` worktree. Read [PLUGIN-WORKSPACE.md](PLUGIN-WORKSPACE.md) first. Select only unblocked Smart Gate leaf tickets (91–94 subject to actual dependencies); do not claim unrelated desktop work from this checkout. Shared guard/research/design prerequisites are coordinated from development. Product umbrella 85 is not one executable unlimited task. The global resource policy in [the overnight plan](docs/plugin-overnight-plan.md) applies across all worktrees; independent folders do not authorize parallel heavy builds.

## Active work

Before planning or implementation, read [.scratch/offline-supervised-migration/map.md](.scratch/offline-supervised-migration/map.md) and the [accepted specification](.scratch/offline-supervised-migration/spec.md). This is the sole active roadmap. The implementation sequence and dependencies live in [.scratch/offline-supervised-migration/implementation.md](.scratch/offline-supervised-migration/implementation.md); build tickets live in `.scratch/offline-supervised-editor/issues/`.

Choose an unblocked implementation ticket, claim it before editing, and complete its scoped acceptance checks before marking it done. `ready-for-agent` describes specification readiness; every `Blocked by` dependency must be done first. Record results in the ticket. See [tracker conventions](docs/agents/issue-tracker.md) and [triage labels](docs/agents/triage-labels.md) when updating issues.

For ticket inventories, use context-mode to extract only status, dependencies, and scope before returning results.

For GUI changes, read [the GUI development guide](docs/gui-development.md) and use [scripts/gui-dev.ps1](scripts/gui-dev.ps1).

## Maintainability

For every project change, optimize for the lifetime cost of understanding, testing, and safely changing the code. Choose the smallest coherent solution; file count and diff size are not quality targets.

- Fix root causes in the layer that owns the behavior. Update all affected callers, interfaces, tests, fixtures, and documentation, even when the solution spans multiple files.
- Keep responsibilities clear and reuse established project patterns. Size the design to current requirements and demonstrated extension needs.
- Perform refactoring needed to make the requested change maintainable within its accepted scope. Record unrelated cleanup separately.
- Verify changed behavior and relevant failure cases with appropriate checks. Explain consequential design tradeoffs and remaining limitations when handing off the change.
- For a temporary workaround, document its constraint, consequences, and the concrete condition for replacing it. Use a `shortcut:` comment beside the affected code and record any deferred work in the relevant ticket.

## Architecture and discovery

Read [CONTEXT.md](CONTEXT.md) and [ADR-0006](docs/adr/0006-offline-supervised-editable-envelopes.md) before changing domain behavior. Consult the [active implementation tracker](.scratch/offline-supervised-migration/implementation.md) for current implementation status. Preserve source validation, cancellation, safe exports and shared envelope evaluation while migrating.

Discover code through the graph: search symbols using supported filters, trace their callers/callees, then fetch snippets by qualified symbol. Index this checkout if coverage is absent or reflects an old branch/worktree. Use text search for literals, non-code records or graph gaps.

When discovering tools, retrieve only the metadata needed for the task. Before selecting or changing dependencies, fetch current library/API documentation through Context7.

## Repository

For version selection, release-status wording or release preparation, read the [versioning and stability policy](docs/releasing.md#versioning-and-stability). It defines the compatibility promise and stable-release transition.

Use `development` as the ordinary local working branch and retain `main` as the release branch. For owner-requested parallel work, use temporary scoped `codex/` branches/worktrees from development; integrate their tested commits locally and remove them after preserving their files. Sequential work can stay on development. Push development as a backup; feature PRs are optional for solo local integration. External contributions still use PRs targeting development.

Test combined features in the primary checkout on development. A local feature merge already updates its files; merge subsequent feature revisions again before rebuilding. Pull with `git pull --ff-only` when GitHub has commits absent locally. Use `scripts/gui-dev.ps1` for GUI builds and `scripts/check.ps1 -Local` for local checks; both share the application-data Cargo target, pinned toolchain and development profile. Keep incremental compilation and coordinate builds through Cargo's target lock. Record the source commit and executable path; preserve the tested executable before retiring a cache or worktree.

Promote a tested batch through a `development` → `main` PR, preserving ancestry with a merge commit. Keep the GitHub default branch as `main`. Merge main-only fixes and release metadata back into development before subsequent feature work.

Release publication is a separate owner-authorized step. A deliberate pushed `v*` tag triggers the four-platform draft-package workflow; a feature/main merge alone does not. For cross-platform development packages, manually dispatch that workflow on `development` with `build_only: true` and a release_tag matching Cargo's current version; inspect its checked Actions artifacts. Verify all native packages and release acceptance before publishing a draft. See [branch and release workflow](docs/branch-workflow.md) for the inventory, package triggers and promotion checks.

Use commit `6a60cc2` as the frozen detector comparison baseline when the acceptance tickets require it.
