# Consolidate development and local build storage

Status: claimed
Type: task
Assignee: Codex
Blocked by: none

Owner approved the shared-cache, consistent-toolchain, incremental-build, serialized-compilation and reduced-debug-information plan on 2026-10-20, and requested that all work live in development with no remaining feature branches. Retain main as the established release branch and preserve the frozen detector baseline.

## Scope

- Reconcile the original checkout's uncommitted documentation into development without losing newer source, release links or specifications.
- Preserve recovery snapshots and the tested development executable before retiring worktrees or obsolete build caches.
- Use one persistent local Cargo target outside worktrees, pinned Rust and one shared development profile for GUI and local checks. Keep incremental compilation and Cargo's cache locking; reduce dependency debug information.
- Update agent/contributor guidance for local integration, development backups and batch PRs to main.
- Make the primary checkout the development checkout. Remove integrated feature branches and retire obsolete worktrees after verifying their content is preserved. Preserve main, tags, source recordings, editable projects and frozen comparison material.

## Acceptance

- No feature branch has commits outside development; only development/main remain as branch names locally and on origin.
- Meaningful pending documentation is retained; other working-copy records and the tested executable have a recoverable local location.
- Shared-target/settings checks, GUI compilation, strict Clippy and relevant application/workflow tests pass with the new settings.
- Record before/after disk usage and remaining unchecked desktop/audio qualification.

## Results

Claimed before edits. Consolidation and validation in progress.
