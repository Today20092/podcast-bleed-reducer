# Local development, builds and releases

Current policy: 2026-10-22. The owner authorized the release, documentation refresh and development-to-main promotion in completed [ticket 64](../.scratch/offline-supervised-editor/issues/64-alpha-release-preparation.md). Published alpha.8 supersedes alpha.7's failed dependency-license packaging attempt. Catch-up scrolling, waveform modes, dial smoothing and the latest transport/status refinements are implemented. Future releases repeat the checks below. [Ticket 54](../.scratch/offline-supervised-editor/issues/54-local-build-consolidation.md) records historical consolidation measurements.

## One place to work

The primary checkout is `C:/Users/User/Documents/Codex/2026-08-14/podcast-automix-rust-gui`, on `development`. Keep only development and main as ongoing branch names. Main remains the reviewed release branch and GitHub's default branch. The detached `ticket14-frozen-baseline` checkout is intentional comparison evidence, not another development branch.

The integrated development history includes adjustable shortcuts, the owner-approved FabFilter-inspired GUI pass (`6b99a9a`), rotary controls/Lucide/performance (`1235ca1`) and the October 22 follow-up through `6160da9`. The owner accepted the current development GUI for release preparation. Only development/main remain as ongoing local branches; release documentation agents use temporary scoped worktrees. Generated diagnostics and private scratch material remain local. Earlier receipts preserve the historical build and measurement limits.

[Alpha.8 is published](https://github.com/Today20092/podcast-bleed-reducer/releases/tag/v0.1.0-alpha.8) from source `24361a2aa201fb7e22258f44a2a1be4626d58ce1`, promoted by [PR 18](https://github.com/Today20092/podcast-bleed-reducer/pull/18) and synchronized into development. [Four-platform source CI](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39741751147) and [four native packages](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39742979614) passed. All eight downloaded assets passed checksums and identity against the tested Actions artifacts. Windows Sandbox interactive export, classification save/reopen and host audition in all three listening modes passed their functional checks. [The receipt](research/alpha8-release-receipt-2026-10-22.md) records identities and remaining qualification limits. Main promotion, draft preparation and publication remain distinct steps for future releases.

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

There is no automatic release after a particular number of features. Choose a version when the tested batch is ready, tag the reviewed source, verify every native package, then publish. The current batch is published as alpha.8 after alpha.7 stopped at the dependency-license gate. README downloads follow the actual published assets; the failed alpha.7 tag is retained unchanged.

## Release checklist

Choose the release version and status using the [versioning and stability policy](releasing.md#versioning-and-stability). The owner accepted 1.0.0 as the next stable target; the existing alpha.8 publication remains historical evidence. Complete the stable-transition steps in that policy during release preparation.

Owner accepted the current application's sound and performance on 2026-10-20 and closed 14. Ticket 15 is retired into this procedure, including package checks carried from 25. Repeat these mechanical checks when preparing an owner-authorized release; they are not an ongoing feature backlog or a request to recreate already published packages.

- [ ] Identify the intended source commit/version and tested build. Run the appropriate current source/package checks; retain checksums, dependency/model notices and truthful platform/support limits.
- [ ] Verify the actual downloaded final package in clean offline Windows: launch, embedded inference, interactive import/audition, save/reopen/relink and validated export. Exercise supported codecs/rates and advertised session limits as appropriate for the change; distinguish synthetic self-checks from physical use.
- [ ] Check real-device playback and applicable source/edit/cancellation/export safeguards. Reuse recorded GUI acceptance from 23; report unavailable checks and known limitations rather than claiming they passed.
- [ ] Verify each advertised native package on its own platform before making new platform-qualification claims. Existing experimental packages do not establish fresh cross-platform hardware acceptance.
- [ ] Get the owner's authorization for promotion/tagging/publication; keep alpha/stable wording aligned with actual evidence. Current owner acceptance does not claim frozen independent comparisons, whole-session benchmarks or absence of the known quiet-overlap limitation. A future benchmark project needs its own requested scope.

### Editor regression coverage

2026-10-22: the owner closed implemented GUI polish (23), adjustable shortcuts (41) and Catch up (42) after accepting the shipped application. The following retained checklists supply recurring, change-appropriate release coverage rather than open feature tickets. Unperformed checks are not passing evidence and do not require recreating the existing release.

- [GUI/import/playback checklist](../.scratch/offline-supervised-editor/issues/23-gui-frontend-polish.md#current-remaining-work--2026-10-20): contrast, themes, narrow/wide/enlarged text, focus and state coverage; import ordering/progress/cache/queue/cancellation/failure isolation; shortcut-reference readability; playback loops, perceived delay, clicks/dropouts and device recovery. This includes the unmet checks carried from 38/39/40.
- [Shortcut checklist](../.scratch/offline-supervised-editor/issues/41-adjustable-keyboard-shortcuts.md#acceptance-checks): labeling keys and repeat suppression, remapping/capture/conflicts, restart persistence and recovery, context/text ownership, Center/Lock, tooltip placement and the zoom/fit preference matrix.
- [Catch-up checklist](../.scratch/offline-supervised-editor/issues/42-playhead-catch-up-scroll.md#acceptance-checks): repeated catches and layer alignment, animation interruptions, middle-drag/reanchor/cancellation, mode overrides, narrow/wide/zoom/resize and recording bounds. The later persistent Catch up correction supersedes its original manual-pan-off policy.

Each original unchecked item and its recorded evidence remain in the linked closed ticket. Select relevant checks for a future change, record results in that change's ticket or release receipt, and report omissions honestly.

## Historical consolidation evidence

Consolidation checks passed with 278 tests, 0 failures and 18 ignored, plus strict Clippy, fixtures, GUI build and native self-checks. At completion the active shared cache measured 4.74 GiB, versus about 95.23 GiB across the previous worktree targets. C: free space increased from 72.45 to 145.50 GiB. These are point-in-time measurements. The recovery folder retains verified scratch material and one unused 12.61 GiB legacy target whose deletion was blocked automatically and offered to the owner manually; it is separate from the active cache.

The earlier combined Windows candidate passed 278 tests with 18 ignored, strict Clippy, GUI compilation, workflow fixtures and its offline self-check. Ticket 54 records the fresh checks with the smaller shared profile. Ignored tests include owner-recording/device measurements, external frozen-baseline evidence and the known quiet-overlap case recorded in 14. Those ignored checks have not passed through owner acceptance. Physical SD interruption/write-lock, fresh native-platform verification and formal independent/long-session measurements remain unperformed historical limitations; the owner no longer requires the formal evaluation project to keep 14 open. The application remains experimental.
