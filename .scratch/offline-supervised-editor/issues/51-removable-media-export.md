# Read original media directly and keep working files local

Status: done
Type: task
Assignee: Codex
Blocked by: 28
Spec: #accepted-design

## Accepted design

2026-10-20: Owner authorized implementation after an SD-card export failed with Windows error 1006. Read source recordings in place, including write-protected media. Never modify originals or automatically create files beside imported media. New projects and disposable analysis/render files use local application storage independently of the source and chosen export destination. Existing projects remain openable without changing source identities. No full original-media copy is required.

Exports use the user-selected destination, including writable removable media, and preserve collision, native-grid, provenance, cancellation and staged-publication checks. Explain failures using operation, path and recoverable cause. Retry only recognized transient I/O failures, with fresh handles, bounded attempts and cancellation; preserve identity validation and never skip samples or accept changed sources. A persistent failure keeps edits/results available for explicit retry or a new destination.

## Acceptance checks

- [x] Read-only source import, analysis and export use local project/cache/render storage; source bytes and directory contents remain unchanged.
- [x] Existing projects and saved source-bound decisions remain compatible; original paths continue referencing the card.
- [x] I/O errors identify operation, path and underlying OS cause, with useful reconnect/permission/space guidance.
- [x] Deterministic injected transient failures recover safely; permanent failures, changed sources and cancellation do not retry indefinitely or publish partial output.
- [x] Destination collisions, unavailable media and output errors preserve originals and previous outputs; another destination can be chosen.
- [x] Relevant library/integration/controller tests, formatting, strict Clippy and scripts/gui-dev.ps1 BuildOnly pass. Record hardware-only checks separately.

## Results

Claimed before implementation on codex/removable-media-export from main 35fab8d. No hardware fault reproduction or release qualification claimed.

Completed 2026-10-20. New projects and GUI editing copies live in local application Projects storage; derived analysis and private renders live in local Work storage, keyed by the saved local project path. External project files remain unchanged, including when stored in a different folder from source audio. Reopening the displayed local project retains its location and edits. Sources shows the autosave path and offers Copy project path. The development launcher also keeps its persistent editing copy local. No full copy of original media or dependency change was added.

The destination picker chooses a fresh CleanSlate export child folder, with a numeric suffix for collisions, inside the selected parent. This supports populated Downloads folders while keeping staged directory publication. Read-only source handles retry recognized transient open/read/seek failures at most twice, with 50 ms backoff and fresh handles at the last confirmed byte position. Reconnect metadata must match; existing full source fingerprints and final provenance checks remain enforced. Exhausted Interrupted errors cannot trigger an infinite read_exact retry. WAV readers preserve the library's buffering. Slow output flushing occurs before the final cancellation/generation check and publication lock, followed by the existing final source checks and rename. Writes are not automatically replayed.

Verification:

- Red regression: imported_media_directory_is_untouched_and_project_is_local failed in 0.06 seconds before the storage fix because the project parent equaled the source parent.
- Full cargo test --all-targets --all-features run with locked/offline dependencies, one job and scripts/gui-dev.toml: 277 passed across 32 suites, 18 opt-in checks ignored.
- After separating flush from the final publication gate, all library tests plus export, export_settings, render, supervised_delivery and native_delivery passed: 160 checks across 6 suites, 10 opt-in checks ignored.
- The production workflow test runs actual analysis with a local analysis-cache/audio directory, prepares local renders from read-only originals, rejects an unusable destination, retries another destination and reopens the saved local project. Source bytes, original project bytes and source-directory entries stay unchanged.
- Failure tests cover injected Windows 1006/timeouts, bounded retries, seek-position recovery without missing/duplicate bytes, changed reconnect media, permanent permission errors, exhausted Interrupted errors, read-only/full-disk guidance, disk-full flush cleanup, disappeared/changed sources before publication and previous-output preservation. A three-channel PCM test guards against a source read per sample. Existing cancellation, generation, provenance, encoding and native-grid regressions remain passing.
- Strict all-target/all-feature Clippy, formatting, diff and PowerShell parser checks passed. scripts/gui-dev.ps1 -BuildOnly passed, and the built executable's --package-self-check exited successfully.
- Updated executable: C:/Users/User/.codex/worktrees/offline-timebase/podcast-automix-rust-gui/target/owner-ui/debug/rust-app.exe. Its dependency manifest points at this checkout, despite the retained build-cache folder name.

Limits: actual SD hardware write-lock/unplug/reconnect and physical listening were not exercised. Read-only file permissions, directory snapshots and deterministic I/O fault injection provide automated evidence. Filesystem calls still depend on OS/device response; automatic recovery cannot repair hardware or finish without readable originals. Derived caches/renders use local disk space; automatic eviction and space forecasting were not added (storage cleanup remains a separate effort). Existing release and listening qualification remains unchanged.
