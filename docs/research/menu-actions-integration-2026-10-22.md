# Menu, registered actions and toolbar integration

The owner approved the development appearance on 2026-10-22 and explicitly requested closing tickets 60/63 and merging completed work into development. They confirmed that further computer control was unnecessary. Ticket 69's icon-only Catch up and removal of the Waveform group label are included in that accepted build.

## Delivery

- Ticket 60: equal-width grouped menu rows, separate effective shortcut hints, bounded content sizing and outside-click dismissal that does not reach the waveform workspace. Existing seven routes and keyboard/modal ownership remain intact. [REA findings](plugin-menu-design-2026-10-22.md) distinguish installed/static evidence and unavailable targets.
- Ticket 63: one typed command declaration feeds Settings, reference and effective hints. The shared action adapter binds availability and activation for timeline controls; Meters demonstrates an initially unbound customizable action. Existing stable identifiers and saved preferences remain compatible. A typed button boundary and normal validation guard reject missing registration; individually documented legacy exceptions do not exempt future controls. See the [developer contract](../agents/actions-and-shortcuts.md).
- Ticket 69: waveform choices retain their three icons without the group label; Catch up uses a fixed 34 × 34 arrow-to-edge icon through the same registered action helper.

## Verification

The combined pre-toolbar source passed `scripts/check.ps1 -Local`: 356 tests passed, zero failed, 20 ignored across 38 targets, formatting, strict all-target/all-feature Clippy and script fixtures. Menu/modal TDD and initially-unbound action TDD are recorded in their tickets. The final toolbar source passed formatting, strict all-target/all-feature Clippy, the omitted-registration E0277/registered-success guard, and three actual library `timeline_` regressions. An earlier binary-only timeline test invocation matched zero tests and is not counted as coverage.

Two-axis review against base `408848c1cfe68358ded807aeb9dea6c76582008c`: Standards had no findings. Spec found one P2 guard-fixture gap; commit `725ac13` removed a duplicate bound so the fixture exercises the production helper directly. A deliberate weakened-helper mutation was rejected. Follow-up Spec review confirmed no remaining finding at integration `670bb00`.

`scripts/gui-dev.ps1` built and opened the final source. The first attempt failed from Windows memory allocation exhaustion; an unchanged retry succeeded in 6.93 seconds. The combined earlier executable also passed its package self-check: offline model inference, analysis, project save/reopen, render and cancellation. No audio engine changes followed that check.

## Source and executable identity

Final application source: `eab2e720869a9049ce80ee8e89b8dcf231e30b0b`, descended from reviewed integration `670bb00`. Subsequent closure/roadmap commits change documentation only.

Preserved final executable: `C:/Users/User/Documents/Codex/Builds/CleanSlate-menu-actions-20261022/rust-app-toolbar-final.exe`.

SHA-256: `ECF59CED6E74AD75F538718D117DB085EE5D45909C2922F65492F7FF0A5EBD14`.

The same artifact directory retains `combined-check.log`, `gui-build.log`, `self-check.txt`, `toolbar-build-retry.log`, `final-timeline-tests.log` and `final-clippy.log`. The preview used an isolated preferences/development-project copy and left original source audio/project files untouched.

## Acceptance limits

The final native toolbar was inspected and the owner accepted the development appearance. Computer-use interruptions prevented completing the automated compact/wide Light/Dark menu matrix, native Settings remap/restart sequence and new screenshot captures. The owner's explicit closure instruction supersedes those pending automation steps; none are claimed passed. Software persistence, routing and availability checks passed. Future native qualification remains in the [recurring editor verification procedure](../branch-workflow.md#editor-regression-coverage).

This is development integration. It does not bump the version, promote main, tag, package or publish a release. Other ongoing discovery and version-policy edits are outside this batch.

## Owner-authorized documentation follow-up

After the menu/action merge `a9846b6`, the owner explicitly requested including the existing discovery and release-policy work in development. The follow-up includes discovery ticket 67 and its product-opportunities research, completed documentation ticket 68, the authoritative versioning/stability policy, AGENTS/branch-workflow pointers and matching roadmap/spec decisions. Ticket 67 stays claimed and needs information; documenting it does not implement a plugin. Ticket 68 remains done. The next stable target is 1.0.0, while current manifests and published alpha.8 packages retain their existing identity. Separate Auto-Align Post research ticket 70 remains ongoing and outside this commit. Documentation link/whitespace and release-status fixture checks cover this follow-up; application behavior is unchanged.
