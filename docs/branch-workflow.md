# Local development, builds and releases

Current policy: 2026-10-20. [Ticket 54](../.scratch/offline-supervised-editor/issues/54-local-build-consolidation.md) records consolidation, storage measurements and checks. The earlier inventory remains available in Git history at commit f69b1e6.

## One place to work

The primary checkout is `C:/Users/User/Documents/Codex/2026-08-14/podcast-automix-rust-gui`, on `development`. Keep only development and main as ongoing branch names. Main remains the reviewed release branch and GitHub's default branch. The detached `ticket14-frozen-baseline` checkout is intentional comparison evidence, not another development branch.

All completed application branches and the primary checkout's pending documentation are integrated into development. Accepted future work, including adjustable shortcuts and catch-up scrolling, is retained as specifications; merging those tickets does not implement them. Generated diagnostics and experimental/private scratch material are preserved in the local recovery snapshot rather than published as source.

## Everyday work

1. For one task at a time, claim its ticket and work on development in the primary checkout. Commit useful checkpoints, build, try the application and revise locally.
2. When the owner explicitly requests parallel work, give each concurrent feature a temporary scoped `codex/` branch/worktree from development. Source editing can happen in parallel; use the shared build target for compilation.
3. Merge a runnable feature into local development and rebuild there to test the combined app. Further feature-branch revisions need another merge before they appear in that build. A local merge already updates your files; no GitHub round trip is needed.
4. Push development regularly as a backup. Solo local feature integration does not require a feature PR. External contributors use PRs targeting development.
5. Once a batch passes its checks and the required desktop/audio review, open one development → main GitHub PR. Preserve ancestry with a merge commit. Bring main-only fixes and version metadata back to development.
6. Remove integrated temporary branches/worktrees after preserving their needed files and executable. Keep the frozen baseline intact.

Pull with `git pull --ff-only` when GitHub has commits absent locally. A PR is a review/merge request, not an application download. Changing branches or merging source does not update an existing executable until you build again.

## One local build cache

On Windows, local builds share `%LOCALAPPDATA%/CleanSlate/BuildCache`, outside every worktree. The canonical GUI executable is `%LOCALAPPDATA%/CleanSlate/BuildCache/debug/rust-app.exe`.

```powershell
# Build the GUI without launching it.
pwsh -NoProfile -File scripts/gui-dev.ps1 -BuildOnly

# Build and launch a separate local copy of the default test project.
pwsh -NoProfile -File scripts/gui-dev.ps1

# Formatting, fixtures, strict Clippy and tests with the same cache/profile.
pwsh -NoProfile -File scripts/check.ps1 -Local
```

For a targeted local Cargo check, obtain the same settings rather than creating a new target/profile:

```powershell
$settings = & ./scripts/local-build-settings.ps1
$cargoArguments = $settings.CargoArguments
cargo test @cargoArguments --test export
```

`rust-toolchain.toml` pins Rust 1.99.0. Keep Cargo.lock and `scripts/gui-dev.toml` consistent. The shared settings retain locked/offline dependencies, one Cargo job and incremental compilation. Dependency debug information is disabled to reduce disk use; detailed dependency debugging needs an intentional separate configuration. Source changes still need compilation and sometimes linking, but unchanged compatible dependencies can be reused. Cargo locks the shared target while building, so coordinate compilation and close the development executable before rebuilding.

CI and release jobs retain their hosted check mode, `scripts/check.ps1`. The local cache is not a release package. Keep needed executables/receipts before cleaning caches; frozen comparison artifacts, original recordings and editable projects are not disposable build outputs.

See [Cargo's build-cache documentation](https://doc.rust-lang.org/cargo/reference/build-cache.html) and [debug-information guidance](https://doc.rust-lang.org/cargo/guide/build-performance.html) for the storage/rebuild tradeoffs.

## When GitHub makes packages

| Action | Current result |
| --- | --- |
| Edit, merge or build locally | Source changes or a local executable only. |
| Push development or merge an ordinary PR | No application release packages from that action alone. |
| Open/update a PR | Four-platform checks; no application release. |
| Manually run **Desktop draft release** on development with `build_only: true` | Checked native packages under the Actions run's Artifacts; no release is created or updated. Set release_tag to the current Cargo version with a leading v. |
| Deliberately push a `v*` version tag | Checks and packages four platforms, then prepares one draft prerelease after all package jobs succeed. |
| Publish the verified draft | Public release assets and the README-download update workflow. |

Use GitHub → Actions → Desktop draft release for build progress/artifacts and GitHub → Releases for draft/public releases. Windows packages are ZIPs, Apple Silicon and Intel Mac packages are separate DMGs, and Linux packages are AppImages; each has a checksum. The workflow is [portable-release.yml](../.github/workflows/portable-release.yml), using [GitHub's documented triggers](https://github.com/github/docs/blob/main/content/actions/reference/workflows-and-actions/events-that-trigger-workflows.md).

There is no automatic release after a particular number of features. Choose a version when the tested batch is ready, tag the reviewed source, verify every native package, then publish. Main and published alpha.6 stay unchanged during this consolidation.

## What still needs human evidence

The earlier combined Windows candidate passed 278 tests with 18 ignored, strict Clippy, GUI compilation, workflow fixtures and its offline self-check. Ticket 54 records the fresh checks with the smaller shared profile. Ignored tests include owner-recording/device measurements, external frozen-baseline evidence and the known quiet-overlap quality gate in ticket 14. Native visual/listening, physical SD interruption/write-lock and independent quality/long-session qualification remain outstanding. The application remains experimental.
