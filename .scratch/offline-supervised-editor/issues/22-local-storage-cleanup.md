# Remove disposable local storage

Type: task
Status: claimed
Assignee: Codex
Blocked by: none

## Current remaining work — 2026-10-20

Retain this as the single residual cleanup ticket. Ticket 54 completed the shared-build/worktree consolidation; its exception belongs here rather than reopening that completed work.

- [x] Remove the reconciled abandoned Git temporary objects. After the owner's manual Command Prompt cleanup, a fresh scan found zero `tmp_obj_*` files and `git fsck --full --no-dangling` passed. The prior inventory was 26 files totaling 1,526,473,928 bytes (1.42 GiB); actual volume free-space change was not measured.
- [x] Reconcile the historical compiled-probe inventory against the current filesystem and recovery manifests. The four original pan-drawing probe EXE/PDB paths are absent from the checkout. Their recovery copies match the recorded SHA-256 manifest and remain preserved; no duplicate active-checkout probe cleanup remains.
- [ ] Remove only `%LOCALAPPDATA%/CleanSlate/Recovery/2026-10-20-worktree-consolidation/retired-model-runtime-residue/target` when deletion is permitted. Fresh measurement is 13,541,383,452 bytes (12.61 GiB). Preserve the surrounding residue root's source/native-runtime/snapshot material. Automatic approval rejected the authorized deletion as recorded below.
- [ ] Preserve recovery snapshots/manifests, recordings, projects, the tested executable and active shared build cache. Record reclaimed bytes and exact checks before marking done.

No storage was removed during tracker consolidation. The original in-checkout target-bound requirement below applies to those original targets; the separately named recovery residue is the explicit exception inherited from 54, requiring its exact resolved path to be checked before any future deletion.

## Owner-authorized cleanup attempt — 2026-10-20

Later owner-manual cleanup verification: zero Git temporary objects remain; a fresh full Git integrity check passed. Development matches origin/development after fetch (0 ahead, 0 behind). The separate retired recovery target still exists, so this ticket remains open only for that remaining storage scope and final preservation checks. Earlier automatic deletion rejections below are historical, not the current Git-temp state.

### Renewed Git-temporary-only request

The owner subsequently explicitly requested removing the abandoned Git temporary files and finishing 22. On that fresh authorization, Codex repeated the same guarded native PowerShell attempt for **Git temporaries only**, after a new full `git fsck --full --no-dangling` passed, the baseline returned `commit`, and no Git writer/lock was present. Inventory remained 26 files / 1,526,473,928 bytes; bounded compressed-header inspection identified 24 blob temporaries and two empty files, newest write 2026-10-10 UTC. Automatic approval again rejected the command before execution with **`blocked by policy`**, giving no further reason. Codex stopped without retrying or changing mechanism. Post-rejection inventory and HEAD were unchanged; connectivity check passed. Reclaimed bytes remain zero. The recovery root/target were not targeted in this renewed request. This owner-requested Git cleanup remains blocked by automatic approval, so the ticket cannot be marked done.

Owner explicitly requested completing this ticket after the consolidation audit. Codex rechecked the exact scopes and attempted only the abandoned Git temporary files and the retired recovery **target**. The residue root also contains source, native-runtime and snapshot material; preserve that root and delete only its obsolete `target` when deletion is permitted.

- Before attempted cleanup, `git fsck --full --no-dangling` passed; `git cat-file -t 6a60cc2` returned `commit`. No Git, Cargo or Rust compiler process and no Git lock files were present. The owner application was running from the preserved `target/development/CleanSlate-development.exe`; neither that folder nor the active shared cache was targeted.
- Reinspected 26 abandoned `.git/objects/<prefix>/tmp_obj_*` files, totaling **1,526,473,928 bytes**. Their latest recorded write was 2026-10-10 UTC, and all were inside this checkout's object directory. A guarded native PowerShell deletion was rejected before execution by automatic approval review: **`blocked by policy`**, with no further reason. No alternative deletion mechanism was attempted.
- Separately measured the explicitly named recovery residue's `target`: **9,549 files, 13,541,383,452 bytes (12.61 GiB)**. Verified the resolved absolute path, no reparse points, no executable running from it, and no build/Git process. Automatic approval review also rejected this distinct native PowerShell recursive deletion before execution: **`blocked by policy`**, with no further reason. No workaround or repeated deletion attempt followed.
- Reconciled all four historical pan-drawing probe EXE/PDB files (528,950,784 bytes): original checkout paths are absent; copies in `primary/untracked/.scratch/offline-supervised-editor/` match the recovery manifest hashes. Preserve those recovery copies. A complete checkout EXE scan found only intended release packages outside `target`; no other disposable probe EXEs remain there.
- After the rejected attempts, inventories remained exactly 26 Git temporary files and 9,549 retired-target files. `git fsck --connectivity-only --no-dangling` passed; the frozen baseline remained a commit and HEAD stayed `002dc9db8875ad5267867168e7295d6a7efc5222`. The recovery manifest hash remained `97108e98ea528b8cafc3959ba1b2ec1b809ece6a835be809914630b61722174a`; the tested executable hash remained `db660282cb5c2965079c6de9583ef18409f461e6b54a3a34872398becf1e1637`. Active shared build cache, recovery source/native-runtime/snapshots, recordings and projects were untouched.
- **Reclaimed this attempt: 0 bytes. Remaining disposable scope: 15,067,857,380 bytes (14.03 GiB).** Ticket remains claimed because automatic approval review blocked both deletions; the scoped cleanup cannot honestly be marked done. No application build/tests were run, since there was no code change and they would recreate build output. Only this ticket was edited by the cleanup agent.

## Cleanup resumed, 2026-10-12

Claimed again for the owner's renewed disk-space cleanup request. Inspect and remove disposable build output, compiled drawing probes and verified abandoned Git temporary objects. Preserve current working changes and all source and project data.

## Results, 2026-10-12

- Confirmed `target` contains no tracked files and is inside this checkout. No Cargo, Rust compiler or application process was active during inspection.
- `git fsck --full --no-dangling` passed before cleanup; `git cat-file -t 6a60cc2` returned `commit`.
- Automatic approval review rejected the combined build/probe/temporary-object deletion command with `blocked by policy`, without a specific reason. No files were removed by that command.
- Standard `cargo clean` succeeded, removing 11,190 files and reporting 11.9 GiB reclaimed. Compiled drawing probes and Git temporary objects remain; this ticket stays claimed because those acceptance targets are unfinished.
- Preserved existing working changes, recordings, saved projects, caches and release evidence. Only this cleanup ticket was edited by this cleanup. No rebuild or tests were run because they would recreate the removed build outputs.

## Scope

Owner requested disk-space cleanup on 2026-10-09. Remove generated Rust build output and compiled drawing probes. Remove redundant packed Git loose objects and abandoned temporary object files without changing refs, reflogs or history. Preserve recordings, saved projects, analysis caches, source, active roadmap and release evidence.

## Acceptance checks

- Confirm deletion targets are inside this checkout and contain no tracked files.
- Confirm no build or Git process is active before removing its generated files.
- Verify Git object integrity and the frozen comparison baseline after cleanup.
- Record reclaimed bytes and final tracked changes.

## Results, 2026-10-09

- `cargo clean` succeeded, removing 3,770 build entries and about 4.2 GiB.
- `git prune-packed` succeeded. No redundant packed loose objects remain; loose storage fell from 1.83 GiB to 310.45 MiB.
- Project file sizes fell from about 14.3 GiB to 8.6 GiB, reclaiming about 5.7 GiB. Measurements count file lengths, not allocated disk blocks.
- `git fsck --full --no-dangling` passed. `git cat-file -t 6a60cc2` still returns `commit`. No tracked source files changed. Recordings, saved projects, caches and roadmap remain.
- Automatic approval review rejected the combined deletion command and the subsequent probe-only deletion command with `blocked by policy`, without a specific reason. Four compiled probes totaling 528,950,784 bytes and 25 abandoned Git temporary objects totaling about 1.42 GiB remain. No attempt was made to bypass the rejection. Ticket remains claimed because that scope is unfinished.
- No application rebuild or tests were run. The next build must recreate Cargo output.
