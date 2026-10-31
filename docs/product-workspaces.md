# Product workspace guide

The [product overview](plugin-products.md) describes the product boundaries and qualification limits. The [active map](../.scratch/offline-supervised-migration/map.md) and product tickets record current work and acceptance; this guide is navigation, not a release receipt.

| Product | Source and role |
| --- | --- |
| Desktop | Primary checkout on `development`: standalone GUI and shared offline engine. `main` is the release branch. |
| Smart Gate | `codex/plugin-smart-gate` is the product source line; see [ticket 85](../.scratch/offline-supervised-editor/issues/85-smart-gate-plugin.md). |
| Podcast | `codex/plugin-podcast` is the product source line; see [ticket 86](../.scratch/offline-supervised-editor/issues/86-podcast-plugin.md). |

Product work may continue on scoped branches or worktrees beyond those product branches. Run `git worktree list`, `git branch -a` and `git status --short` in the relevant checkout to locate current work and inspect its state; do not assume a named temporary worktree is the latest source. The `origin/codex/plugin-smart-gate` and `origin/codex/plugin-podcast` branches are GitHub backups of product source, not proof that every local continuation is backed up. Compare commits and uncommitted files before relying on any remote backup.

Desktop and Podcast use `crates/ownership-model` and the offline engine; Smart Gate uses its own gate detector. Integrate focused shared fixes through `development`, verify desktop behavior, and bring those commits into product lines that need them. Keep product-specific host adapters, UI and builds with their product source; do not merge a whole product branch merely to obtain a shared fix. See [code and integration boundaries](plugin-products.md#code-and-integration-boundaries).

For desktop builds use [gui-dev.ps1](../scripts/gui-dev.ps1), and for local checks use [check.ps1](../scripts/check.ps1) with `-Local`. Plugin build and host-check scripts live with the relevant product source; inspect that checkout's scripts and ticket for current commands and artifacts. Record the source commit and executable or artifact path when testing. A build or prototype host check does not qualify a plugin for release.

Before integration or cleanup, inspect each checkout's status, commits and untracked files; preserve local changes and tested artifacts, and coordinate with their owner. Integrate tested, scoped commits into the appropriate branch, then test the combined result in the primary `development` checkout where applicable. Merge later revisions again before rebuilding. Push intended backup branches deliberately after verifying their contents; do not overwrite another person's work. Retire a temporary worktree only after its work and artifacts are preserved. See the [branch workflow](branch-workflow.md) for integration and release procedures; publication requires separate authorization.
