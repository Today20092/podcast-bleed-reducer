# 10: Remove the redundant listening selector label

Status: done
Assignee: Codex
Type: task
Blocked by: 09 (done)

## Owner request

Remove the visible “Listen” label beside Original / Processed / Removed on development. Retain the options, states, shortcuts and monitor controls.

## Acceptance criteria

- [x] Remove only the redundant label from the shared compact/wide transport layout.
- [x] Check formatting and build using scripts/gui-dev.ps1; record the result.

## Comments

2026-10-21: Claimed before editing. This is presentation-only; no audio, persistence or selector behavior changes are required.

Completed on development: removed the single text node from src/app.rs. Formatting and diff checks passed; scripts/gui-dev.ps1 -BuildOnly passed (54.35 seconds), and the executable's --package-self-check passed. Executable: C:/Users/User/AppData/Local/CleanSlate/BuildCache/debug/rust-app.exe. SHA256: F67E7FBBB1D4E6355E6BD6E9149BCE66C1D81C185E2FD96C917F5081FBDCF9D0. Native appearance was not rechecked; the already open preserved GUI remains the earlier build. No new tests were added for this one-line presentation change.
