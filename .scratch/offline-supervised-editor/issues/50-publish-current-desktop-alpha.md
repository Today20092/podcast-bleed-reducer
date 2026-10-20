# Publish the current desktop application

Status: claimed
Type: task
Assignee: Codex
Blocked by: none

Owner requested on 2026-10-20 that the public GitHub download include the approved logo and the newer keyboard-shortcut/editor workflow. Published alpha.2 predates both. Draft alpha.5 includes the editor workflow but predates the application icons merged in PR 12.

## Scope

- Prepare alpha.6 from current main, retaining the implemented editor workflow and application icons.
- Build and check all four native packages through the existing release workflow.
- Verify the downloaded Windows executable's embedded icon, package integrity and offline self-check.
- Publish the experimental prerelease and update README downloads through the protected PR workflow.
- Identify the existing local development executable for the owner; preserve unrelated checkout changes.

## Acceptance

- Cargo manifest/lock version and release notes identify alpha.6.
- Four platform checks and package jobs pass; all four downloads have checksums.
- Downloaded Windows executable has the approved icon and passes the package self-check.
- Published release and README Windows link resolve to alpha.6.
- Record unchecked native desktop/audio/listening qualification honestly.

## Results

- Existing local GUI executable: `C:/Users/User/.codex/worktrees/offline-timebase/podcast-automix-rust-gui/target/owner-ui/debug/rust-app.exe`, built 2026-10-17. Windows extracted its 32px icon with 274 mint pixels.
- Current main contains the editor workflow through alpha.5 and PR 12 application branding. Configurable shortcut remapping remains planned in ticket 41.
- Local native-workflow, protected-branch download automation, package integrity/download, and release-status fixture checks passed; Cargo formatting and diff whitespace checks passed.
