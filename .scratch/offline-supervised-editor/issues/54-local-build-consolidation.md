# Consolidate development and local build storage

Status: done
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

2026-10-20 backlog reconciliation: this ticket remains done. Its manual-cleanup exception is tracked as an explicit open item in [22](22-local-storage-cleanup.md#current-remaining-work--2026-10-20); no deletion completion is claimed.

- Committed the primary checkout's pending documentation as `96e6a54`, then merged it into development as `9a22e94`. Resolved historical documentation conflicts by retaining current release links, completed-feature records and the newer specifications. The complete original documentation snapshot remains reachable through development history. Added the previously untracked research, roadmap/design records and documentation example. Future-work tickets remain specifications rather than implemented features.
- `d9af762` implements the shared build settings, pinned Rust 1.99.0, reduced dependency debug information and local-check mode. GUI and `scripts/check.ps1 -Local` use `%LOCALAPPDATA%/CleanSlate/BuildCache`, locked/offline dependencies, one Cargo job and scripts/gui-dev.toml. Incremental compilation remains enabled; Cargo supplies target-directory locking. Hosted CI retains its original locked check behavior.
- Updated AGENTS.md, contributor/build guides and branch-workflow.md for local development integration, optional solo feature PRs, development backup pushes and one tested-batch PR to main. Temporary feature branches/worktrees are only for requested parallel work and are retired after integration.
- The primary project folder is on development. Verified every former branch tip as its ancestor, then removed all 12 feature branches and retired 15 obsolete worktrees. Only development/main remain locally and on origin. The sole secondary registered checkout is the clean detached `6a60cc2` frozen baseline. Main remains `0b9622a`; published release tags are unchanged. Development was pushed to origin.
- Before retirement, copied and SHA-256 verified 1,038 untracked files (0.591 GiB) into `%LOCALAPPDATA%/CleanSlate/Recovery/2026-10-20-worktree-consolidation`, with an untracked-manifest.json. Preserved ignored native-runtime material and the previously tested executable/receipt separately. Recordings, saved projects and the frozen baseline were preserved.
- Shared-settings checks, changed Markdown local links, formatting, workflow/packaging/download/release fixtures and strict all-target/all-feature Clippy passed. The fresh full test run passed 278 tests, 0 failures, 18 ignored across 33 suites, including compilation of the preserved documentation example. Ignored owner/device/frozen/quiet-overlap qualification checks are not counted as passing evidence.
- Fresh GUI build through scripts/gui-dev.ps1 passed in 1.24 seconds after checks populated the new cache. Explicitly awaited native self-checks of both the shared-cache binary and the saved `target/development/CleanSlate-development.exe` passed offline inference, analysis, save/reopen, render and cancellation. Saved executable SHA-256: `db660282cb5c2965079c6de9583ef18409f461e6b54a3a34872398becf1e1637`.
- Before: approximately 95.23 GiB across worktree targets, with 54.97 GiB in owner-ui including 32.59 GiB of PDB files. After the complete fresh check/build: active shared cache 4.74 GiB including 1.12 GiB PDB; C: free space rose from 72.45 to 145.50 GiB (approximately 73.05 GiB net reclaimed). Measurements are point-in-time and include other system activity.
- Cleanup exception: Git encountered a Windows long-path error in the old offline-model-runtime target. Automatic approval rejected native recursive deletion with only 'blocked by policy'. Reversibly relocated that orphaned directory into recovery as `retired-model-runtime-residue`; its unused target still occupies 12.61 GiB. The owner volunteered manual deletion. Verified the exact path and supplied an explicit bundled PowerShell 7 command after Windows PowerShell 5 reported a missing path. Latest check still found this target; no manual completion is claimed. It is not an active Cargo cache or registered worktree.
- Desktop visual/listening checks, physical SD interruption/write-lock, owner-recording speed, fresh native Mac/Linux qualification and independent stable-release quality/long-session gates remain unverified. No release was created or published.
